use regex::Regex;
use std::process::{exit, Command, Stdio};
use sarge::*;

#[cfg(feature = "gui")]
use eframe::egui;


#[derive(Debug, Clone, PartialEq, Eq)]
struct Sink {
    id: i32,
    name: String,

    volume: i32,
    mute: bool,
}

fn get_sinks() -> Vec<Sink> {
    let mut sinks = Vec::with_capacity(6);

    let raw = String::from_utf8(
        Command::new("pactl")
            .args(["list", "sink-inputs"])
            .stdout(Stdio::piped())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let raw_sinks: Vec<Vec<String>> = raw
        .replace('\t', "")
        .split("\n\n")
        .map(|line| line.split('\n').map(String::from).collect::<Vec<String>>())
        .collect();

    for raw_sink in raw_sinks {
        let id: i32 = raw_sink[0][12..].parse().unwrap();
        let mut volume: i32 = 0;
        let mut mute = false;
        let mut name = String::new();

        for line in raw_sink {
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

        sinks.push(Sink {
            id,
            name,
            volume,
            mute,
        });
    }

    sinks
}

fn set_mute(id: i32, mute: bool) -> Result<std::process::ExitStatus, std::io::Error> {
    let mute = if mute { "1" } else { "0" };

    Command::new("pactl")
        .arg("set-sink-input-mute")
        .arg(id.to_string())
        .arg(mute)
        .output()
        .map(|out| out.status)
}

fn set_volume(id: i32, volume: i32) -> Result<std::process::ExitStatus, std::io::Error> {
    let volume = format!("{volume}%");

    Command::new("pactl")
        .arg("set-sink-input-volume")
        .arg(id.to_string())
        .arg(volume)
        .output()
        .map(|out| out.status)
}

fn set_sink(sink: &Sink) -> Result<(), std::io::Error> {
    let status = set_volume(sink.id, sink.volume)?.code().unwrap_or(0);
    if status != 0 {
        return Err(std::io::Error::from_raw_os_error(status));
    }

    let status = set_mute(sink.id, sink.mute)?.code().unwrap_or(0);
    if status != 0 {
        return Err(std::io::Error::from_raw_os_error(status));
    }

    Ok(())
}

fn set_sinks(sinks: &Vec<Sink>) -> Result<(), std::io::Error> {
    for sink in sinks {
        set_sink(sink)?;
    }

    Ok(())
}

fn by_id(sinks: &mut [Sink], id: i32) -> Option<&mut Sink> {
    sinks.iter_mut().find(|sink| sink.id == id)
}

fn by_name<'a>(sinks: &'a mut [Sink], name: &str) -> Option<&'a mut Sink> {
    sinks.iter_mut().find(|sink| sink.name.contains(name))
}

#[cfg(not(feature = "gui"))]
fn main() {
    let mut sinks = get_sinks();

    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'l', "list"));
    parser.add(arg!(int, both, 'i', "id"));
    parser.add(arg!(str, both, 'n', "name"));
    parser.add(arg!(int, both, 'v', "volume"));
    parser.add(arg!(flag, both, 'm', "mute"));
    parser.add(arg!(flag, both, 'u', "unmute"));

    parser.parse().unwrap();

    if get_flag!(parser, both, 'l', "list") {
        sinks.iter().for_each(|sink| {
            println!("{}", sink.name);
            println!("      id: {}", sink.id);
            println!("    mute: {}", sink.mute);
            println!("  volume: {}%\n", sink.volume);
        });

        if !get_flag!(parser, both, 'u', "unmute")
            && !get_flag!(parser, both, 'm', "mute")
            && get_val!(parser, both, 'v', "volume").is_none()
        {
            return;
        }
    }

    let sink;
    if let Some(id) = get_val!(parser, both, 'i', "id").map(|val| val.get_int()) {
        sink = by_id(&mut sinks, id).unwrap_or_else(|| {
            eprintln!("error: no sink matches {id}");
            exit(1);
        });
    } else if let Some(name) = get_val!(parser, both, 'n', "name").map(|val| val.get_str()) {
        sink = by_name(&mut sinks, &name).unwrap_or_else(|| {
            eprintln!("error: no sink matches '{name}'");
            exit(1);
        });
    } else {
        eprintln!("error: must provide either an id or a name");
        exit(1);
    }

    if get_flag!(parser, both, 'u', "unmute") {
        sink.mute = false;
    } else if get_flag!(parser, both, 'm', "mute") {
        sink.mute = true;
    }

    if let Some(vol) = get_val!(parser, both, 'v', "volume").map(|val| val.get_int()) {
        sink.volume = vol;
    }

    set_sinks(&sinks).unwrap_or_else(|e| {
        eprintln!("error (when setting sinks):\n{e}");
        exit(1);
    });
}

#[cfg(feature = "gui")]
fn main() -> Result<(), eframe::Error> {
    let mut sinks = get_sinks();

    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'l', "list"));
    parser.add(arg!(int, both, 'i', "id"));
    parser.add(arg!(str, both, 'n', "name"));
    parser.add(arg!(int, both, 'v', "volume"));
    parser.add(arg!(flag, both, 'm', "mute"));
    parser.add(arg!(flag, both, 'u', "unmute"));

    parser.parse().unwrap();

    if get_flag!(parser, both, 'l', "list") {
        sinks.iter().for_each(|sink| {
            println!("{}", sink.name);
            println!("      id: {}", sink.id);
            println!("    mute: {}", sink.mute);
            println!("  volume: {}%\n", sink.volume);
        });

        if !get_flag!(parser, both, 'u', "unmute")
            && !get_flag!(parser, both, 'm', "mute")
            && get_val!(parser, both, 'v', "volume").is_none()
        {
            exit(0);
        }
    }

    let sink;
    if let Some(id) = get_val!(parser, both, 'i', "id").map(|val| val.get_int()) {
        sink = by_id(&mut sinks, id).unwrap_or_else(|| {
            eprintln!("error: no sink matches {id}");
            exit(1);
        });
    } else if let Some(name) = get_val!(parser, both, 'n', "name").map(|val| val.get_str()) {
        sink = by_name(&mut sinks, &name).unwrap_or_else(|| {
            eprintln!("error: no sink matches '{name}'");
            exit(1);
        });
    } else {
        eprintln!("error: must provide either an id or a name");
        exit(1);
    }

    if get_flag!(parser, both, 'u', "unmute") {
        sink.mute = false;
    } else if get_flag!(parser, both, 'm', "mute") {
        sink.mute = true;
    }

    if let Some(vol) = get_val!(parser, both, 'v', "volume").map(|val| val.get_int()) {
        sink.volume = vol;
    }

    set_sinks(&sinks).unwrap_or_else(|e| {
        eprintln!("error (when setting sinks):\n{e}");
        exit(1);
    });

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native(
        "PACTL Mixer",
        options,
        Box::new(|_cc| Box::<Mixer>::default()),
    )
}

#[cfg(feature = "gui")]
struct Mixer {
    sinks: Vec<Sink>,
}

#[cfg(feature = "gui")]
impl Default for Mixer {
    fn default() -> Self {
        Self { sinks: get_sinks() }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for Mixer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mixer");

            let old_sinks = self.sinks.clone();
            ui.vertical(|ui| {
                for sink in self.sinks.iter_mut() {
                    ui.label(format!("{} (id: {})", sink.name, sink.id));
                    ui.horizontal(|ui| {
                        ui.label("Volume");
                        ui.add(egui::Slider::new(&mut sink.volume, 0..=200));
                        if sink.mute && ui.button("Unmute").clicked() {
                            sink.mute = false;
                        } else if ui.button("Mute").clicked() {
                            sink.mute = true;
                        }
                    });
                }
            });

            if old_sinks != self.sinks {
                _ = set_sinks(&self.sinks);
            }

            if ui.button("Update").clicked() {
                self.sinks = get_sinks();
            }

            if ui.button("Exit").clicked() {
                exit(0);
            }
        });
    }
}
