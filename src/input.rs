use std::sync::mpsc::Sender;
use std::time::Duration;

use rppal::gpio::{Gpio, InputPin, Trigger};

pub enum ButtonEvent {
    Up,
    Down,
    Left,
    Right,
    Enter,
}

/// this function handles the user input and never exit if does should kill the process
/// # Panics
pub fn handle_button(send: &Sender<ButtonEvent>) -> () {
    let gpio = Gpio::new().unwrap();

    let mut down = gpio.get(17).unwrap().into_input_pullup();
    let mut ender = gpio.get(27).unwrap().into_input_pullup();
    let mut left = gpio.get(22).unwrap().into_input_pullup();
    let mut up = gpio.get(23).unwrap().into_input_pullup();
    let mut right = gpio.get(24).unwrap().into_input_pullup();

    let debounce = Some(Duration::from_millis(50));

    down.set_interrupt(Trigger::FallingEdge, debounce).unwrap();
    ender.set_interrupt(Trigger::FallingEdge, debounce).unwrap();
    left.set_interrupt(Trigger::FallingEdge, debounce).unwrap();
    up.set_interrupt(Trigger::FallingEdge, debounce).unwrap();
    right.set_interrupt(Trigger::FallingEdge, debounce).unwrap();
    let pins: [&InputPin; 5] = [&down, &ender, &left, &up, &right];

    loop {
        match gpio.poll_interrupts(&pins, true, None).unwrap() {
            Some((pin, _event)) => match pin {
                p if p == down => send.send(ButtonEvent::Down).unwrap(),
                p if p == ender => send.send(ButtonEvent::Enter).unwrap(),
                p if p == left => send.send(ButtonEvent::Left).unwrap(),
                p if p == up => send.send(ButtonEvent::Up).unwrap(),
                p if p == right => send.send(ButtonEvent::Right).unwrap(),
                _ => {
                    eprintln!("wtf");
                }
            },
            None => {
                eprint!("Error detcting user input");
            }
        }
    }
}
