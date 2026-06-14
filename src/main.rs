use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use retropod::audio::init::audio_thread;
use retropod::display::init::setup_display;
use retropod::display::update::update_display;
use retropod::input::handle_button;

fn main() {
    let (button_tx, button_rx) = mpsc::channel();
    let (audio_tx, audio_rx) = mpsc::channel();

    let bins = Arc::new(Mutex::new(vec![0.0f32; 128]));
    let bins_clone = Arc::clone(&bins);

    let mut display = match setup_display() {
        Ok(v) => v,
        Err(e) => {
            panic!("Init display failed with: {}", e)
        }
    };

    thread::spawn(move || {
        audio_thread(audio_rx, bins);
        panic!("Unexpected behaviour audio thread crashed")
    });
    thread::spawn(move || {
        update_display(&mut display, button_rx, audio_tx, bins_clone);
        panic!("Unexpected behaviour display thread crashed")
    });
    handle_button(&button_tx);
    panic!("Unexpected behaviour button thread crashed")
}
