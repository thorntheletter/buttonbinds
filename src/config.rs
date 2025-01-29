use clap::Parser;
use enigo::Key;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

/// a program to quickly bind gamepad controls for offline multiplayer of games with no / bad
/// controller support
#[derive(Parser, Debug, Serialize)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Output all events
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    #[arg(short, long, default_value = "bindings.json")]
    pub file: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Controls {
    pub directions: HashMap<String, Key>,
    pub actions: Vec<(String, Key)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub controls: Vec<Controls>,
}

pub fn load_config(filename: String) -> Config {
    let config: Config = match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
            //
        }
        Err(_) => {
            println!("Configuration file not loaded, defaulting to Wonderful World.");
            let p1 = Controls {
                // likely a cleaner way but i do not feel like dealing with str lifetimes rn
                directions: HashMap::from([
                    (String::from("Up"), Key::Unicode('t')),
                    (String::from("Down"), Key::Unicode('b')),
                    (String::from("Left"), Key::Unicode('f')),
                    (String::from("Right"), Key::Unicode('h')),
                ]),
                actions: vec![
                    (String::from("Punch"), Key::Unicode('a')),
                    (String::from("Kick"), Key::Unicode('s')),
                    (String::from("Slash"), Key::Unicode('d')),
                    (String::from("Heavy Slash"), Key::Unicode('q')),
                    (String::from("Original Action"), Key::Unicode('w')),
                    (String::from("Special Action"), Key::Unicode('e')),
                    (String::from("Pause"), Key::Escape),
                ],
            };
            let p2 = Controls {
                directions: HashMap::from([
                    (String::from("Up"), Key::Numpad8),
                    (String::from("Down"), Key::Numpad2),
                    (String::from("Left"), Key::Numpad4),
                    (String::from("Right"), Key::Numpad6),
                ]),
                actions: vec![
                    (String::from("Punch"), Key::Unicode('j')),
                    (String::from("Kick"), Key::Unicode('k')),
                    (String::from("Slash"), Key::Unicode('l')),
                    (String::from("Heavy Slash"), Key::Unicode('i')),
                    (String::from("Original Action"), Key::Unicode('o')),
                    (String::from("Special Action"), Key::Unicode('p')),
                    (String::from("Pause"), Key::Escape),
                ],
            };
            Config {
                name: String::from("Wonderful World"),
                controls: vec![p1, p2],
            }
        }
    };
    println!("Loaded configuration for {}.", config.name);
    return config;
}
