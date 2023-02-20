use std::{
    ops::{Deref, DerefMut},
    process::{Command, Stdio},
};

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    id: i32,
    name: String,

    pub volume: i32,
    pub mute: bool,
    pub locked: bool,
}

impl Source {
    pub fn flush(&self) -> Result<(), std::io::Error> {
        let status = self.flush_volume()?.code().unwrap_or(0);
        if status != 0 {
            return Err(std::io::Error::from_raw_os_error(status));
        }

        let status = self.flush_mute()?.code().unwrap_or(0);
        if status != 0 {
            return Err(std::io::Error::from_raw_os_error(status));
        }

        Ok(())
    }

    pub fn flush_volume(&self) -> Result<std::process::ExitStatus, std::io::Error> {
        Command::new("pactl")
            .arg("set-sink-input-volume")
            .arg(self.id.to_string())
            .arg(format!("{}%", self.volume))
            .output()
            .map(|out| out.status)
    }

    pub fn flush_mute(&self) -> Result<std::process::ExitStatus, std::io::Error> {
        Command::new("pactl")
            .arg("set-sink-input-mute")
            .arg(self.id.to_string())
            .arg(if self.mute { "1" } else { "0" })
            .output()
            .map(|out| out.status)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Sources(Vec<Source>);

impl Deref for Sources {
    type Target = Vec<Source>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Sources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Sources> for Vec<Source> {
    fn from(sources: Sources) -> Self {
        sources.0
    }
}

impl Sources {
    pub fn new() -> Self {
        let mut sources = Sources(Vec::with_capacity(4));
        sources.update();

        sources
    }

    pub fn flush(&self) -> Result<(), std::io::Error> {
        for source in self.iter() {
            source.flush()?;
        }

        Ok(())
    }

    pub fn by_id(&mut self, id: i32) -> Option<&mut Source> {
        self.0.iter_mut().find(|source| source.id == id)
    }

    pub fn by_name(&mut self, name: &str) -> Option<&mut Source> {
        self.0.iter_mut().find(|source| source.name.contains(name))
    }

    pub fn update(&mut self) {
        self.0.retain(|source| source.locked);
        self.0.iter_mut().for_each(|source| {
            source.id = -1;
        });

        let raw = String::from_utf8(
            Command::new("pactl")
                .args(["list", "sink-inputs"])
                .stdout(Stdio::piped())
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .replace('\t', "");

        if raw.is_empty() {
            return;
        }

        let raw_sources: Vec<Vec<String>> = raw
            .split("\n\n")
            .map(|line| line.split('\n').map(String::from).collect::<Vec<String>>())
            .collect();

        for raw_source in raw_sources {
            let id: i32 = raw_source[0][12..].parse().unwrap();
            let mut volume: i32 = 0;
            let mut mute = false;
            let mut name = String::new();

            for line in raw_source {
                if line.starts_with("Mute: ") {
                    if line.contains("yes") {
                        mute = true;
                    }
                } else if line.starts_with("Volume: ") {
                    let re = Regex::new(r"Volume: front-left: \d+ / +(\d+)% / (-?[\d.]+|-inf) dB,   front-right: \d+ / +(\d+)% / (-?[\d.]+|-inf) dB").unwrap();

                    let caps = re.captures(&line).unwrap();
                    volume = caps[1].parse().unwrap();
                } else if line.starts_with("application.name = ") {
                    name = line[20..].to_string().replace('"', "");
                }
            }

            if let Some(source) = self.0.iter_mut()
                .find(|source| source.name == name) {
                source.id = id;
                source.volume = volume;
                source.mute = mute;
            } else {
                self.0.push(Source {
                    id,
                    name,
                    volume,
                    mute,
                    locked: false,
                });
            }
        }
    }
}
