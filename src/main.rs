use enigo::{
    Direction,
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::collections::HashMap;
use std::io;
use std::thread;
extern crate sdl2;
use sdl2::controller::{Axis, Button, GameController};
use sdl2::event::Event;
use std::os::raw::c_void;
use std::time;

#[derive(Eq, Hash, PartialEq)]
enum ControllerInput {
    Analog(Axis),
    Digital(Button),
}

fn bind(
    bindings: &mut [HashMap<u32, HashMap<ControllerInput, Key>>; 2],
    p_idx: usize,
    c_idx: u32,
    input: ControllerInput,
    k: Key,
) -> bool {
    // could probably do this easier with try_insert if it ever gets added
    match bindings[p_idx].get_mut(&c_idx) {
        Some(controller_bindings) => {
			match controller_bindings.get(&input) {
				Some(_) => return false,
				None => {
					controller_bindings.insert(input, k);
					return true;
				}
			}
        }
        None => {
            bindings[p_idx].insert(c_idx, HashMap::new());
            let controller_bindings = bindings[p_idx].get_mut(&c_idx).unwrap();
            controller_bindings.insert(input, k);
			return true;
        }
    }
}

fn press(
    bindings: &[HashMap<u32, HashMap<ControllerInput, Key>>; 2],
    enigo: &mut Enigo,
    c_idx: u32,
    input: ControllerInput,
    action: Direction,
) {
    for binding in bindings {
        match binding.get(&c_idx) {
            Some(controller_bindings) => match controller_bindings.get(&input) {
                Some(key) => {
                    let _ = enigo.key(*key, action);
                }
                _ => (),
            },
            _ => (),
        }
    }
}

fn main() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");
    sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1"); // might not be necessary

    let sdl_context = sdl2::init().unwrap();
    let game_controller_subsystem = sdl_context.game_controller().unwrap();
    let event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let bind_mode_event_type_id = unsafe { event_subsystem.register_event().unwrap() };
    let event_sender = event_subsystem.event_sender();

    let mut controllers: HashMap<u32, GameController> = HashMap::new();
    let mut controller_analog_states: HashMap<u32, [bool; 4]> = HashMap::new();
    let mut bindings: [HashMap<u32, HashMap<ControllerInput, Key>>; 2] =
        [HashMap::new(), HashMap::new()];

    #[cfg(target_os = "linux")]
    let directions: [HashMap<&str, Key>; 2] = [
        HashMap::from([
            ("Up", Key::Unicode('t')),
            ("Down", Key::Unicode('b')),
            ("Left", Key::Unicode('f')),
            ("Right", Key::Unicode('h')),
        ]),
        HashMap::from([
            // should probably combine these into one type but quick and hacky is the way for right now
            ("Up", Key::Other(0xffb8)),
            ("Down", Key::Other(0xffb2)),
            ("Left", Key::Other(0xffb4)),
            ("Right", Key::Other(0xffb6)),
        ]),
    ];

    #[cfg(target_os = "windows")]
    let directions: [HashMap<&str, Key>; 2] = [
        HashMap::from([
            ("Up", Key::Unicode('t')),
            ("Down", Key::Unicode('b')),
            ("Left", Key::Unicode('f')),
            ("Right", Key::Unicode('h')),
        ]),
        HashMap::from([
            ("Up", Key::Other(0x68)),
            ("Down", Key::Other(0x62)),
            ("Left", Key::Other(0x64)),
            ("Right", Key::Other(0x66)),
        ]),
    ];

    let actions: [[(&str, Key); 7]; 2] = [
        [
            ("Punch", Key::Unicode('a')),
            ("Kick", Key::Unicode('s')),
            ("Slash", Key::Unicode('d')),
            ("Heavy Slash", Key::Unicode('q')),
            ("Original Action", Key::Unicode('w')),
            ("Special Action", Key::Unicode('e')),
            ("Pause", Key::Escape),
        ],
        [
            ("Punch", Key::Unicode('j')),
            ("Kick", Key::Unicode('k')),
            ("Slash", Key::Unicode('l')),
            ("Heavy Slash", Key::Unicode('i')),
            ("Original Action", Key::Unicode('o')),
            ("Special Action", Key::Unicode('p')),
            ("Pause", Key::Escape),
        ],
    ];

    // let (tx, rx) = mpsc::channel();
    println!("Please enter either 1 or 2.");
    thread::spawn(move || loop {
        // event_sender.push_custom_event::<u32>(3);
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<i32>() {
            Ok(p_num) => {
                let _ = event_sender.push_event(Event::User {
                    timestamp: 0,
                    window_id: 0,
                    type_: bind_mode_event_type_id,
                    code: p_num - 1,
                    data1: 0 as *mut c_void,
                    data2: 0 as *mut c_void,
                });
            }
            Err(_) => println!("Please enter either 1 or 2."),
        }
    });

    loop {
        let event = event_pump.wait_event();
        // println!("{event:?}");
        match event {
            Event::ControllerButtonDown {
                timestamp: _,
                which,
                button,
            } => press(
                &bindings,
                &mut enigo,
                which,
                ControllerInput::Digital(button),
                Press,
            ),

            Event::ControllerButtonUp {
                timestamp: _,
                which,
                button,
            } => press(
                &bindings,
                &mut enigo,
                which,
                ControllerInput::Digital(button),
                Release,
            ),

            Event::ControllerAxisMotion {
                timestamp: _,
                which,
                axis,
                value,
            } => match axis {
                Axis::TriggerRight | Axis::TriggerLeft => {
                    let state_idx = match axis {
                        Axis::TriggerRight => 0,
                        Axis::TriggerLeft => 1,
                        _ => panic!(),
                    };
                    let old_state = controller_analog_states.get(&which).unwrap()[state_idx];
                    let new_state = value.unsigned_abs() > i16::MAX as u16 / 2;
                    if old_state != new_state {
                        controller_analog_states.get_mut(&which).unwrap()[state_idx] = new_state;
                        if new_state {
                            press(
                                &bindings,
                                &mut enigo,
                                which,
                                ControllerInput::Analog(axis),
                                Press,
                            );
                        } else {
                            press(
                                &bindings,
                                &mut enigo,
                                which,
                                ControllerInput::Analog(axis),
                                Release,
                            );
                        }
                    }
                }
                Axis::LeftX | Axis::LeftY => {
                    let (state_idx, dpad) = match (axis, value > 0) {
                        (Axis::LeftX, true) => (2, Button::DPadRight),
                        (Axis::LeftX, false) => (2, Button::DPadLeft),
                        (Axis::LeftY, true) => (3, Button::DPadDown),
                        (Axis::LeftY, false) => (3, Button::DPadUp),
                        _ => panic!(),
                    };
                    let old_state = controller_analog_states.get(&which).unwrap()[state_idx];
                    let new_state = value.unsigned_abs() > i16::MAX as u16 / 2;

                    if old_state != new_state {
                        controller_analog_states.get_mut(&which).unwrap()[state_idx] = new_state;
                        if new_state {
                            press(
                                &bindings,
                                &mut enigo,
                                which,
                                ControllerInput::Digital(dpad),
                                Press,
                            );
                        } else {
                            // should be able to simplify with changing dpad var above, is fine for now
                            if state_idx == 2 {
                                press(
                                    &bindings,
                                    &mut enigo,
                                    which,
                                    ControllerInput::Digital(Button::DPadRight),
                                    Release,
                                );
                                press(
                                    &bindings,
                                    &mut enigo,
                                    which,
                                    ControllerInput::Digital(Button::DPadLeft),
                                    Release,
                                );
                            } else {
                                press(
                                    &bindings,
                                    &mut enigo,
                                    which,
                                    ControllerInput::Digital(Button::DPadDown),
                                    Release,
                                );
                                press(
                                    &bindings,
                                    &mut enigo,
                                    which,
                                    ControllerInput::Digital(Button::DPadUp),
                                    Release,
                                );
                            }
                        }
                    }
                }
                _ => (),
            },
            Event::User {
                timestamp: _,
                window_id: _,
                type_: _,
                code,
                data1: _,
                data2: _,
            } => {
                if code > 1 {
                    // maybe should have kept this out of the main event loop, just broken with
                    println!("Please enter either 1 or 2.")
                } else {
                    println!("Binding buttons for Player {}. Press the buttons you would like for the corresponding actions:", code + 1);
                    let p_idx = code as usize;
                    bindings[p_idx].drain();
                    let mut controller_id: u32 = 0; // should probably be option again, too lazy to change it back
                    for (action, key) in actions[p_idx] {
                        println!("{}:", action);
                        'waiting_input: loop {
                            // swap this to next_event_blocking w/ filters later if possible
                            let event = event_pump.wait_event();
							println!("{event:?}");
                            match event {
                                Event::ControllerButtonDown {
                                    timestamp: _,
                                    which,
                                    button,
                                } => {
                                    controller_id = which;
                                    match bind(
                                        &mut bindings,
                                        p_idx,
                                        which,
                                        ControllerInput::Digital(button),
                                        key,
                                    ) {
										true => break 'waiting_input,
										false => ()
									}
                                }
                                Event::ControllerDeviceAdded {
                                    timestamp: _,
                                    which,
                                } => match game_controller_subsystem.open(which) {
                                    Ok(c) => {
                                        controller_analog_states
                                            .insert(c.instance_id(), [false; 4]);
                                        controllers.insert(c.instance_id(), c);
                                    }
                                    Err(_) => (),
                                },
                                Event::ControllerDeviceRemoved {
                                    timestamp: _,
                                    which,
                                } => {
                                    controllers.remove(&which);
                                    controller_analog_states.remove(&which);
                                }
                                Event::ControllerAxisMotion {
                                    timestamp: _,
                                    which,
                                    axis,
                                    value,
                                } => match axis {
                                    Axis::TriggerRight | Axis::TriggerLeft => {
                                        let state_idx = match axis {
                                            Axis::TriggerRight => 0,
                                            Axis::TriggerLeft => 1,
                                            _ => panic!(),
                                        };
                                        let old_state = controller_analog_states
                                            .get(&which)
                                            .unwrap()[state_idx];
                                        let new_state = value.unsigned_abs() > i16::MAX as u16 / 2;
                                        if old_state != new_state {
                                            controller_analog_states.get_mut(&which).unwrap()
                                                [state_idx] = new_state;
                                            if new_state {
                                                match bind(
                                                    &mut bindings,
                                                    p_idx,
                                                    which,
                                                    ControllerInput::Analog(axis),
                                                    key,
                                                ) {
													true => break 'waiting_input,
													false => ()
												}
                                            }
                                        }
                                    }
                                    _ => (),
                                },
                                Event::Quit { .. } => return,
                                _ => (),
                            }
                        }
                    }

                    // should probably just initialize bindings map with these already in it once they have controller select
                    // let controller_bindings = bindings[p_idx].get_mut(&controller_id).unwrap();
                    bind(
                        &mut bindings,
                        p_idx,
                        controller_id,
                        ControllerInput::Digital(Button::DPadUp),
                        *directions[p_idx].get("Up").unwrap(),
                    );
                    bind(
                        &mut bindings,
                        p_idx,
                        controller_id,
                        ControllerInput::Digital(Button::DPadDown),
                        *directions[p_idx].get("Down").unwrap(),
                    );
                    bind(
                        &mut bindings,
                        p_idx,
                        controller_id,
                        ControllerInput::Digital(Button::DPadLeft),
                        *directions[p_idx].get("Left").unwrap(),
                    );
                    bind(
                        &mut bindings,
                        p_idx,
                        controller_id,
                        ControllerInput::Digital(Button::DPadRight),
                        *directions[p_idx].get("Right").unwrap(),
                    );
                    println!("Finished with binding, please double check bindings in game.");
                    println!("Please enter either 1 or 2.");
                }
            }
            Event::ControllerDeviceAdded {
                timestamp: _,
                which,
            } => match game_controller_subsystem.open(which) {
                Ok(c) => {
                    controller_analog_states.insert(c.instance_id(), [false; 4]);
                    controllers.insert(c.instance_id(), c);
                }
                Err(_) => (),
            },
            Event::ControllerDeviceRemoved {
                timestamp: _,
                which,
            } => {
                controllers.remove(&which);
                controller_analog_states.remove(&which);
            }
            Event::Quit { .. } => return,
            _ => (),
        }
    }
}
