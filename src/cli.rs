use std::process::exit;

use crate::source::*;
use sarge::*;

pub fn cli<R>(app: Option<&dyn Fn() -> Result<(), R>>) -> Result<(), R> {
    let mut sources = Sources::new();

    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'l', "list"));
    parser.add(arg!(int, both, 'i', "id"));
    parser.add(arg!(str, both, 'n', "name"));
    parser.add(arg!(int, both, 'v', "volume"));
    parser.add(arg!(flag, both, 'm', "mute"));
    parser.add(arg!(flag, both, 'u', "unmute"));

    parser.parse().unwrap();

    if get_flag!(parser, both, 'l', "list") {
        sources.iter().for_each(|source| {
            println!("{}", source.name());
            println!("      id: {}", source.id());
            println!("    mute: {}", source.mute);
            println!("  volume: {}%\n", source.volume);
        });

        if !get_flag!(parser, both, 'u', "unmute")
            && !get_flag!(parser, both, 'm', "mute")
            && get_val!(parser, both, 'v', "volume").is_none()
        {
            exit(0);
        }
    }

    let source;
    if let Some(id) = get_val!(parser, both, 'i', "id").map(|val| val.get_int()) {
        source = sources.by_id(id).unwrap_or_else(|| {
            eprintln!("error: no source matches {id}");
            exit(1);
        });
    } else if let Some(name) = get_val!(parser, both, 'n', "name").map(|val| val.get_str()) {
        source = sources.by_name(&name).unwrap_or_else(|| {
            eprintln!("error: no source matches '{name}'");
            exit(1);
        });
    } else if let Some(app) = app {
        return app();
    } else {
        eprintln!("error: must provide a name or id");
        exit(1);
    }

    if get_flag!(parser, both, 'u', "unmute") {
        source.mute = false;
    } else if get_flag!(parser, both, 'm', "mute") {
        source.mute = true;
    }

    if let Some(vol) = get_val!(parser, both, 'v', "volume").map(|val| val.get_int()) {
        source.volume = vol;
    }

    sources.flush().unwrap_or_else(|e| {
        eprintln!("error (when setting sources):\n{e}");
        exit(1);
    });

    if let Some(app) = app {
        app()
    } else {
        Ok(())
    }
}
