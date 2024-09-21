use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::io;
// use std::string;
use gilrs::{Gilrs, Event, EventType, Button, GamepadId};
use enigo::{Enigo, Settings, Key, Keyboard, Direction::{Press, Release}};


fn main() {
    let mut gilrs = Gilrs::new().unwrap(); // if running into issues with shoulder sensitivity, use gilrsbuilder and set_axis_to_btn
    let mut enigo = Enigo::new (&Settings::default()).unwrap();

    // let mut bindings: HashMap<GamepadId, HashMap<Button, Key>> = HashMap::new(); // can mess around with types later
    let player_1_directions = HashMap::from([ // should probably combine these into one type but quick and hacky is the way for right now
        ("Up", Key::Unicode('t')),
        ("Down", Key::Unicode('b')),
        ("Left", Key::Unicode('f')),
        ("Right", Key::Unicode('h')),
    ]);
    let player_1_actions = [
        ("Punch", Key::Unicode('a')),
        ("Kick", Key::Unicode('s')),
        ("Slash", Key::Unicode('d')),
        ("Heavy Slash", Key::Unicode('q')),
        ("Original Action", Key::Unicode('w')),
        ("Special Action", Key::Unicode('e')),
		("Pause", Key::Escape),
    ];
    #[cfg(target_os = "linux")]
    let player_2_directions = HashMap::from([ // should probably combine these into one type but quick and hacky is the way for right now
        ("Up", Key::Other(0xffb8)),
        ("Down", Key::Other(0xffb2)),
        ("Left", Key::Other(0xffb4)),
        ("Right", Key::Other(0xffb6)),
    ]);
    #[cfg(target_os = "windows")]
    let player_2_directions = HashMap::from([
        ("Up", Key::Other(0x68)),
        ("Down", Key::Other(0x62)),
        ("Left", Key::Other(0x64)),
        ("Right", Key::Other(0x66)),
    ]);
    let player_2_actions = [
        ("Punch", Key::Unicode('j')),
        ("Kick", Key::Unicode('k')),
        ("Slash", Key::Unicode('l')),
        ("Heavy Slash", Key::Unicode('i')),
        ("Original Action", Key::Unicode('o')),
        ("Special Action", Key::Unicode('p')),
		("Pause", Key::Escape),
    ];
    let mut player_1_bindings: HashMap<GamepadId, HashMap<Button, Key>> = HashMap::new(); // can mess around with types later
    let mut player_2_bindings: HashMap<GamepadId, HashMap<Button, Key>> = HashMap::new(); // again should maybe be together but we can figure that out later

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop{
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            match input.trim() {
                "1" => tx.send("1").unwrap(),
                "2" => tx.send("2").unwrap(),
                _ => println!("Please enter either 1 or 2.")
            }
        }
    });


    loop{
        match rx.try_recv() {
            Ok(received) => { // can probably move binding to its own function and have parameters for player and such
                match received {
                    "1" => {
                        // bind p1 buttons
                        println!("Binding buttons for Player 1. Press the buttons you would like for the corresponding actions:");
                        player_1_bindings.drain();
                        let mut controller_id: Option<GamepadId> = None;
                        for (action, key) in player_1_actions{
                            println!("{}:", action);
                            'waiting_input: loop { // swap this to next_event_blocking w/ filters later if possible
                                while let Some(Event {id, event, ..}) = gilrs.next_event() {
                                    controller_id = Some(id);
                                    match event {
                                        EventType::ButtonPressed(button, _) => {
                                            match player_1_bindings.get_mut(&id) { // can probably do this easier with try_insert but not too fond of thinking rn // nvm it has still not been fully added for the past 3 years
                                                Some(controller_bindings) => {
                                                    controller_bindings.insert(button, key);
                                                },
                                                None => {
                                                    player_1_bindings.insert(id, HashMap::new());
                                                    let controller_bindings = player_1_bindings.get_mut(&id).unwrap();
                                                    controller_bindings.insert(button, key);
                                                }
                                            }
                                            break 'waiting_input;
                                        }
                                        _ => ()
                                    }
                                }
                            }
                        }

                        // should probably just initialize bindings map with these already in it
                        let controller_bindings = player_1_bindings.get_mut(&controller_id.unwrap()).unwrap();
                        controller_bindings.insert(Button::DPadUp, *player_1_directions.get("Up").unwrap());
                        controller_bindings.insert(Button::DPadDown, *player_1_directions.get("Down").unwrap());
                        controller_bindings.insert(Button::DPadLeft, *player_1_directions.get("Left").unwrap());
                        controller_bindings.insert(Button::DPadRight, *player_1_directions.get("Right").unwrap());

                        // println!("{:?}", player_1_bindings);
                    },
                    "2" => {
                        // bind p2 buttons
                        println!("Binding buttons for Player 2. Press the buttons you would like for the corresponding actions:");
                        player_2_bindings.drain();
                        let mut controller_id: Option<GamepadId> = None;
                        for (action, key) in player_2_actions{
                            println!("{}:", action);
                            'waiting_input: loop { // swap this to next_event_blocking w/ filters later if possible
                                while let Some(Event {id, event, ..}) = gilrs.next_event() {
                                    controller_id = Some(id);
                                    match event {
                                        EventType::ButtonPressed(button, _) => {
                                            match player_2_bindings.get_mut(&id) { // can probably do this easier with try_insert but not too fond of thinking rn // nvm it has still not been fully added for the past 3 years
                                                Some(controller_bindings) => {
                                                    controller_bindings.insert(button, key);
                                                },
                                                None => {
                                                    player_2_bindings.insert(id, HashMap::new());
                                                    let controller_bindings = player_2_bindings.get_mut(&id).unwrap();
                                                    controller_bindings.insert(button, key);
                                                }
                                            }
                                            break 'waiting_input;
                                        }
                                        _ => ()
                                    }
                                }
                            }
                        }

                        // should probably just initialize bindings map with these already in it
                        let controller_bindings = player_2_bindings.get_mut(&controller_id.unwrap()).unwrap();
                        controller_bindings.insert(Button::DPadUp, *player_2_directions.get("Up").unwrap());
                        controller_bindings.insert(Button::DPadDown, *player_2_directions.get("Down").unwrap());
                        controller_bindings.insert(Button::DPadLeft, *player_2_directions.get("Left").unwrap());
                        controller_bindings.insert(Button::DPadRight, *player_2_directions.get("Right").unwrap());

                        // println!("{:?}", player_2_bindings);
                    },
                    _ => println!("received something else somehow, tell this to thorn"),
                }}
            Err(_) => (),
        }
        while let Some(Event {id, event, ..}) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(b, _c) => {
                    match player_1_bindings.get(&id) {
                        Some(controller_bindings) => {
							println!("{event:?}");
                            match controller_bindings.get(&b) {
                                Some(key) => {
                                    let res = enigo.key(*key, Press);
									match res {
										Err(e) => println!("{e:?}"),
										_ => (),
									}
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                    match player_2_bindings.get(&id) {
                        Some(controller_bindings) => {
                            match controller_bindings.get(&b) {
                                Some(key) => {
                                    let res = enigo.key(*key, Press);
									match res {
										Err(e) => println!("{e:?}"),
										_ => (),
									}
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                },
                EventType::ButtonReleased(b, _c) => {
                    match player_1_bindings.get(&id) {
                        Some(controller_bindings) => {
                            match controller_bindings.get(&b) {
                                Some(key) => {
                                    let _ = enigo.key(*key, Release);
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                    match player_2_bindings.get(&id) {
                        Some(controller_bindings) => {
                            match controller_bindings.get(&b) {
                                Some(key) => {
                                    let _ = enigo.key(*key, Release);
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }
    }
}
