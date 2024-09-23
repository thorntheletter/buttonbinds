use std::collections::HashMap;
use std::io;
use std::sync::mpsc;
use std::thread;

use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
extern crate sdl2;
use sdl2::controller::{Axis, Button, GameController};
use sdl2::event::Event;

#[derive(Eq, Hash, PartialEq)] // could work for adding triggers into bindings but i need to sleep
enum ControllerInput {
    Analog(Axis),
    Digital(Button),
}

fn main() -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");
    sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1"); // might not be necessary

    let sdl_context = sdl2::init()?;
    let game_controller_subsystem = sdl_context.game_controller()?;
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut controllers: HashMap<u32, GameController> = HashMap::new();
    let mut controller_analog_states: HashMap<u32, [bool; 4]> = HashMap::new();

    let mut player_1_bindings: HashMap<u32, HashMap<ControllerInput, Key>> = HashMap::new(); // can mess around with types later
    let mut player_2_bindings: HashMap<u32, HashMap<ControllerInput, Key>> = HashMap::new(); // <(u32, Button), Key> could work
    let player_1_directions = HashMap::from([
        // should probably combine these into one type but quick and hacky is the way for right now
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
    let player_2_directions = HashMap::from([
        // should probably combine these into one type but quick and hacky is the way for right now
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

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "1" => tx.send("1").unwrap(),
            "2" => tx.send("2").unwrap(),
            _ => println!("Please enter either 1 or 2."),
        }
    });

    loop {
        match rx.try_recv() {
            Ok(received) => {
                // can probably move binding to its own function and have parameters for player and such
                match received {
                    "1" => {
                        // bind p1 buttons
                        println!("Binding buttons for Player 1. Press the buttons you would like for the corresponding actions:");
                        player_1_bindings.drain();
                        let mut controller_id: u32 = 0; // should probably be option again, too lazy to change it back
                        for (action, key) in player_1_actions {
                            println!("{}:", action);
                            'waiting_input: loop {
                                // swap this to next_event_blocking w/ filters later if possible
                                for event in event_pump.poll_iter() {
                                    match event {
                                        Event::ControllerButtonDown {
                                            timestamp: _,
                                            which,
                                            button,
                                        } => {
                                            controller_id = which;
                                            match player_1_bindings.get_mut(&which) {
                                                // can probably do this easier with try_insert but not too fond of thinking rn // nvm it has still not been fully added for the past 3 years
                                                Some(controller_bindings) => {
                                                    controller_bindings.insert(
                                                        ControllerInput::Digital(button),
                                                        key,
                                                    );
                                                }
                                                None => {
                                                    player_1_bindings.insert(which, HashMap::new());
                                                    let controller_bindings =
                                                        player_1_bindings.get_mut(&which).unwrap();
                                                    controller_bindings.insert(
                                                        ControllerInput::Digital(button),
                                                        key,
                                                    );
                                                }
                                            }
                                            break 'waiting_input;
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
                                        } => {
                                            if value.unsigned_abs() > i16::MAX as u16 / 2 {
                                                match axis {
                                                    Axis::TriggerRight => {
                                                        if !controller_analog_states
                                                            .get(&which)
                                                            .unwrap()[0]
                                                        {
                                                            println!("right trigger activate");
                                                            controller_id = which;
                                                            controller_analog_states
                                                                .get_mut(&which)
                                                                .unwrap()[0] = true;
                                                            match player_1_bindings.get_mut(&which)
                                                            {
                                                                Some(controller_bindings) => {
                                                                    controller_bindings.insert(
                                                                        ControllerInput::Analog(
                                                                            axis,
                                                                        ),
                                                                        key,
                                                                    );
                                                                }
                                                                None => {
                                                                    player_1_bindings.insert(
                                                                        which,
                                                                        HashMap::new(),
                                                                    );
                                                                    let controller_bindings =
                                                                        player_1_bindings
                                                                            .get_mut(&which)
                                                                            .unwrap();
                                                                    controller_bindings.insert(
                                                                        ControllerInput::Analog(
                                                                            axis,
                                                                        ),
                                                                        key,
                                                                    );
                                                                }
                                                            }
                                                            break 'waiting_input;
                                                        }
                                                    }
                                                    Axis::TriggerLeft => {
                                                        if !controller_analog_states
                                                            .get(&which)
                                                            .unwrap()[1]
                                                        {
                                                            println!("right trigger activate");
                                                            controller_id = which;
                                                            controller_analog_states
                                                                .get_mut(&which)
                                                                .unwrap()[1] = true;
                                                            match player_1_bindings.get_mut(&which)
                                                            {
                                                                Some(controller_bindings) => {
                                                                    controller_bindings.insert(
                                                                        ControllerInput::Analog(
                                                                            axis,
                                                                        ),
                                                                        key,
                                                                    );
                                                                }
                                                                None => {
                                                                    player_1_bindings.insert(
                                                                        which,
                                                                        HashMap::new(),
                                                                    );
                                                                    let controller_bindings =
                                                                        player_1_bindings
                                                                            .get_mut(&which)
                                                                            .unwrap();
                                                                    controller_bindings.insert(
                                                                        ControllerInput::Analog(
                                                                            axis,
                                                                        ),
                                                                        key,
                                                                    );
                                                                }
                                                            }
                                                            break 'waiting_input;
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            } else {
                                                // deactivate
                                                match axis {
                                                    Axis::TriggerRight => {
                                                        if controller_analog_states
                                                            .get(&which)
                                                            .unwrap()[0]
                                                        {
                                                            println!("deactivate right trigger");
                                                            controller_analog_states
                                                                .get_mut(&which)
                                                                .unwrap()[0] = false;
                                                        }
                                                    }
                                                    Axis::TriggerLeft => {
                                                        if controller_analog_states
                                                            .get(&which)
                                                            .unwrap()[1]
                                                        {
                                                            println!("deactivate right trigger");
                                                            controller_analog_states
                                                                .get_mut(&which)
                                                                .unwrap()[1] = false;
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        }
                                        Event::Quit { .. } => return Ok(()),
                                        _ => (),
                                    }
                                }
                            }
                        }

                        // should probably just initialize bindings map with these already in it
                        let controller_bindings =
                            player_1_bindings.get_mut(&controller_id).unwrap();
                        controller_bindings.insert(
                            ControllerInput::Digital(Button::DPadUp),
                            *player_1_directions.get("Up").unwrap(),
                        );
                        controller_bindings.insert(
                            ControllerInput::Digital(Button::DPadDown),
                            *player_1_directions.get("Down").unwrap(),
                        );
                        controller_bindings.insert(
                            ControllerInput::Digital(Button::DPadLeft),
                            *player_1_directions.get("Left").unwrap(),
                        );
                        controller_bindings.insert(
                            ControllerInput::Digital(Button::DPadRight),
                            *player_1_directions.get("Right").unwrap(),
                        );

                        // println!("{:?}", player_1_bindings);
                    }
                    "2" => {
                        // bind p2 buttons
                        println!("Binding buttons for Player 2. Press the buttons you would like for the corresponding actions:");
                        player_2_bindings.drain();
                        let mut controller_id: u32 = 0; // should probably be option again, will change back later
                        for (action, key) in player_2_actions {
                            println!("{}:", action);
                            'waiting_input: loop {
                                // swap this to next_event_blocking w/ filters later if possible
                                for event in event_pump.poll_iter() {
                                    match event {
                                        Event::ControllerButtonDown {
                                            timestamp: _,
                                            which,
                                            button,
                                        } => {
                                            controller_id = which;
                                            match player_2_bindings.get_mut(&which) {
                                                // can probably do this easier with try_insert but not too fond of thinking rn // nvm it has still not been fully added for the past 3 years
                                                Some(controller_bindings) => {
                                                    controller_bindings.insert(
                                                        ControllerInput::Digital(button),
                                                        key,
                                                    );
                                                }
                                                None => {
                                                    player_2_bindings.insert(which, HashMap::new());
                                                    let controller_bindings =
                                                        player_2_bindings.get_mut(&which).unwrap();
                                                    controller_bindings.insert(
                                                        ControllerInput::Digital(button),
                                                        key,
                                                    );
                                                }
                                            }
                                            break 'waiting_input;
                                        }
                                        Event::ControllerAxisMotion {
                                            timestamp: _,
                                            which,
                                            axis,
                                            value,
                                        } => {
                                            if value.unsigned_abs() > i16::MAX as u16 / 2 {
                                                match axis {
                                                    Axis::TriggerRight => {
                                                        if !controller_analog_states
                                                            .get(&which)
                                                            .unwrap()[0]
                                                        {
                                                            println!("right trigger activate");
                                                            controller_id = which;
                                                            controller_analog_states
                                                                .get_mut(&which)
                                                                .unwrap()[0] = true;
                                                            match player_2_bindings.get_mut(&which)
                                                            {
                                                                Some(controller_bindings) => {
                                                                    controller_bindings.insert(
                                                                        ControllerInput::Analog(
                                                                            axis,
                                                                        ),
                                                                        key,
                                                                    );
                                                                }
                                                                None => {
                                                                    player_2_bindings.insert(
                                                                        which,
                                                                        HashMap::new(),
                                                                    );
                                                                    let controller_bindings =
                                                                        player_2_bindings
                                                                            .get_mut(&which)
                                                                            .unwrap();
                                                                    controller_bindings.insert(
                                                                        ControllerInput::Analog(
                                                                            axis,
                                                                        ),
                                                                        key,
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Axis::TriggerLeft => {
                                                        if !controller_analog_states
                                                            .get(&which)
                                                            .unwrap()[1]
                                                        {
                                                            println!("right trigger activate");
                                                            controller_id = which;
                                                            controller_analog_states
                                                                .get_mut(&which)
                                                                .unwrap()[1] = true;
                                                            match player_2_bindings.get_mut(&which)
                                                            {
                                                                Some(controller_bindings) => {
                                                                    controller_bindings.insert(
                                                                        ControllerInput::Analog(
                                                                            axis,
                                                                        ),
                                                                        key,
                                                                    );
                                                                }
                                                                None => {
                                                                    player_2_bindings.insert(
                                                                        which,
                                                                        HashMap::new(),
                                                                    );
                                                                    let controller_bindings =
                                                                        player_2_bindings
                                                                            .get_mut(&which)
                                                                            .unwrap();
                                                                    controller_bindings.insert(
                                                                        ControllerInput::Analog(
                                                                            axis,
                                                                        ),
                                                                        key,
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            } else {
                                                // deactivate
                                                match axis {
                                                    Axis::TriggerRight => {
                                                        if controller_analog_states
                                                            .get(&which)
                                                            .unwrap()[0]
                                                        {
                                                            println!("deactivate right trigger");
                                                            controller_analog_states
                                                                .get_mut(&which)
                                                                .unwrap()[0] = false;
                                                        }
                                                    }
                                                    Axis::TriggerLeft => {
                                                        if controller_analog_states
                                                            .get(&which)
                                                            .unwrap()[1]
                                                        {
                                                            println!("deactivate right trigger");
                                                            controller_analog_states
                                                                .get_mut(&which)
                                                                .unwrap()[1] = false;
                                                        }
                                                    }
                                                    _ => (),
                                                }
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
                                        Event::Quit { .. } => return Ok(()),
                                        _ => (),
                                    }
                                }
                            }
                        }

                        // should probably just initialize bindings map with these already in it
                        let controller_bindings =
                            player_2_bindings.get_mut(&controller_id).unwrap();
                        controller_bindings.insert(
                            ControllerInput::Digital(Button::DPadUp),
                            *player_2_directions.get("Up").unwrap(),
                        );
                        controller_bindings.insert(
                            ControllerInput::Digital(Button::DPadDown),
                            *player_2_directions.get("Down").unwrap(),
                        );
                        controller_bindings.insert(
                            ControllerInput::Digital(Button::DPadLeft),
                            *player_2_directions.get("Left").unwrap(),
                        );
                        controller_bindings.insert(
                            ControllerInput::Digital(Button::DPadRight),
                            *player_2_directions.get("Right").unwrap(),
                        );

                        // println!("{:?}", player_2_bindings);
                    }
                    _ => println!("received something else somehow, tell this to thorn"),
                }
            }
            Err(_) => (),
        }

        for event in event_pump.poll_iter() {
            // TODO: add analog analog stick
            match event {
                Event::ControllerButtonDown {
                    timestamp: _,
                    which,
                    button,
                } => {
                    match player_1_bindings.get(&which) {
                        Some(controller_bindings) => {
                            println!("{event:?}");
                            match controller_bindings.get(&ControllerInput::Digital(button)) {
                                Some(key) => {
                                    let res = enigo.key(*key, Press);
                                    match res {
                                        Err(e) => println!("{e:?}"),
                                        _ => (),
                                    }
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                    match player_2_bindings.get(&which) {
                        Some(controller_bindings) => {
                            match controller_bindings.get(&ControllerInput::Digital(button)) {
                                Some(key) => {
                                    let res = enigo.key(*key, Press);
                                    match res {
                                        Err(e) => println!("{e:?}"),
                                        _ => (),
                                    }
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                }
                Event::ControllerButtonUp {
                    timestamp: _,
                    which,
                    button,
                } => {
                    match player_1_bindings.get(&which) {
                        Some(controller_bindings) => {
                            match controller_bindings.get(&ControllerInput::Digital(button)) {
                                Some(key) => {
                                    let _ = enigo.key(*key, Release);
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                    match player_2_bindings.get(&which) {
                        Some(controller_bindings) => {
                            match controller_bindings.get(&ControllerInput::Digital(button)) {
                                Some(key) => {
                                    let _ = enigo.key(*key, Release);
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                }
                Event::ControllerAxisMotion {
                    timestamp: _,
                    which,
                    axis,
                    value,
                } => {
                    if value.unsigned_abs() > i16::MAX as u16 / 2 {
                        match axis {
                            Axis::TriggerRight => {
                                if !controller_analog_states.get(&which).unwrap()[0] {
                                    println!("right trigger activate");
                                    controller_analog_states.get_mut(&which).unwrap()[0] = true;
                                    match player_1_bindings.get(&which) {
                                        Some(controller_bindings) => {
                                            match controller_bindings
                                                .get(&ControllerInput::Analog(axis))
                                            {
                                                Some(key) => {
                                                    let res = enigo.key(*key, Press);
                                                    match res {
                                                        Err(e) => println!("{e:?}"),
                                                        _ => (),
                                                    }
                                                }
                                                _ => (),
                                            }
                                        }
                                        _ => (),
                                    }
                                    match player_2_bindings.get(&which) {
                                        Some(controller_bindings) => {
                                            match controller_bindings
                                                .get(&ControllerInput::Analog(axis))
                                            {
                                                Some(key) => {
                                                    let res = enigo.key(*key, Press);
                                                    match res {
                                                        Err(e) => println!("{e:?}"),
                                                        _ => (),
                                                    }
                                                }
                                                _ => (),
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            Axis::TriggerLeft => {
                                if !controller_analog_states.get(&which).unwrap()[1] {
                                    println!("left trigger activate");
                                    controller_analog_states.get_mut(&which).unwrap()[1] = true;
                                    match player_1_bindings.get(&which) {
                                        Some(controller_bindings) => {
                                            match controller_bindings
                                                .get(&ControllerInput::Analog(axis))
                                            {
                                                Some(key) => {
                                                    let res = enigo.key(*key, Press);
                                                    match res {
                                                        Err(e) => println!("{e:?}"),
                                                        _ => (),
                                                    }
                                                }
                                                _ => (),
                                            }
                                        }
                                        _ => (),
                                    }
                                    match player_2_bindings.get(&which) {
                                        Some(controller_bindings) => {
                                            match controller_bindings
                                                .get(&ControllerInput::Analog(axis))
                                            {
                                                Some(key) => {
                                                    let res = enigo.key(*key, Press);
                                                    match res {
                                                        Err(e) => println!("{e:?}"),
                                                        _ => (),
                                                    }
                                                }
                                                _ => (),
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            Axis::LeftX => {
                                if !controller_analog_states.get(&which).unwrap()[2] {
                                    if value > 0 {
                                        // right
                                        println!("leftx activate (positive)");
                                    } else {
                                        // left
                                        println!("leftx activate (negative)");
                                    }
                                    controller_analog_states.get_mut(&which).unwrap()[2] = true;
                                }
                            }
                            Axis::LeftY => {
                                if !controller_analog_states.get(&which).unwrap()[3] {
                                    if value > 0 {
                                        // down
                                        println!("lefty activate (positive)");
                                    } else {
                                        // up
                                        println!("lefty activate (negative)");
                                    }
                                    controller_analog_states.get_mut(&which).unwrap()[3] = true;
                                }
                            }
                            _ => (),
                        }
                    } else {
                        // deactivate
                        match axis {
                            Axis::TriggerRight => {
                                if controller_analog_states.get(&which).unwrap()[0] {
                                    println!("deactivate right trigger");
                                    controller_analog_states.get_mut(&which).unwrap()[0] = false;
                                    match player_1_bindings.get(&which) {
                                        Some(controller_bindings) => {
                                            match controller_bindings
                                                .get(&ControllerInput::Analog(axis))
                                            {
                                                Some(key) => {
                                                    let _ = enigo.key(*key, Release);
                                                }
                                                _ => (),
                                            }
                                        }
                                        _ => (),
                                    }
                                    match player_2_bindings.get(&which) {
                                        Some(controller_bindings) => {
                                            match controller_bindings
                                                .get(&ControllerInput::Analog(axis))
                                            {
                                                Some(key) => {
                                                    let _ = enigo.key(*key, Release);
                                                }
                                                _ => (),
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            Axis::TriggerLeft => {
                                if controller_analog_states.get(&which).unwrap()[1] {
                                    println!("deactivate right trigger");
                                    controller_analog_states.get_mut(&which).unwrap()[1] = false;
                                    match player_1_bindings.get(&which) {
                                        Some(controller_bindings) => {
                                            match controller_bindings
                                                .get(&ControllerInput::Analog(axis))
                                            {
                                                Some(key) => {
                                                    let _ = enigo.key(*key, Release);
                                                }
                                                _ => (),
                                            }
                                        }
                                        _ => (),
                                    }
                                    match player_2_bindings.get(&which) {
                                        Some(controller_bindings) => {
                                            match controller_bindings
                                                .get(&ControllerInput::Analog(axis))
                                            {
                                                Some(key) => {
                                                    let _ = enigo.key(*key, Release);
                                                }
                                                _ => (),
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            Axis::LeftX => {
                                if controller_analog_states.get(&which).unwrap()[2] {
                                    println!("leftx deactivate");
                                    controller_analog_states.get_mut(&which).unwrap()[2] = false;
                                }
                            }
                            Axis::LeftY => {
                                if controller_analog_states.get(&which).unwrap()[3] {
                                    println!("lefty deactivate");
                                    controller_analog_states.get_mut(&which).unwrap()[3] = false;
                                }
                            }
                            _ => (),
                        }
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
                Event::Quit { .. } => return Ok(()),
                _ => (),
            }
        }
    }
}
