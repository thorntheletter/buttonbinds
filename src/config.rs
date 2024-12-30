use clap::Parser;
use enigo::Key;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::fs;
use std::io::{Read, BufReader};

// testotest
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
        },
        Err(error) => {
            println!("Configuration file not loaded, defaulting to Wonderful World.");
            let p1 = Controls {
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
                #[cfg(target_os = "windows")]
                directions: HashMap::from([
                    (String::from("Up"), Key::Other(0x68)),
                    (String::from("Down"), Key::Other(0x62)),
                    (String::from("Left"), Key::Other(0x64)),
                    (String::from("Right"), Key::Other(0x66)),
                ]),
                #[cfg(target_os = "linux")]
                directions: HashMap::from([
                    (String::from("Up"), Key::Other(0xffb8)),
                    (String::from("Down"), Key::Other(0xffb2)),
                    (String::from("Left"), Key::Other(0xffb4)),
                    (String::from("Right"), Key::Other(0xffb6)),
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
    // let reader = BufReader::new(file);
}
