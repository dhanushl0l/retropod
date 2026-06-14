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
    let _bins_clone = Arc::clone(&bins);

    let mut display = match setup_display() {
        Ok(v) => v,
        Err(e) => {
            panic!("Init display failed with: {}", e)
        }
    };

    thread::spawn(move || {
        audio_thread(audio_rx, bins);
    });
    thread::spawn(move || {
        update_display(&mut display, button_rx, audio_tx);
    });
    handle_button(&button_tx);
}

// use std::sync::atomic::AtomicU8;
// use std::sync::{Arc, Mutex};
// use std::{error, thread};

// use retropod::audio::decode::PlaybackSession;

// fn main() -> Result<(), Box<dyn error::Error>> {
//     let path = String::from("/home/dhanu/Downloads/ALONES - Aqua Timez.flac");

//     // Shared state: spectrum analyzer bins and per-band EQ gains.
//     // Adjust the bin/band count to whatever your FFT/EQ implementation expects.
//     let bins = Arc::new(Mutex::new(vec![0.0f32; 32]));
//     let eq_gains = Arc::new(Mutex::new(vec![1.0f32; 10])); // unity gain on all bands

//     let session = PlaybackSession::start(&path, bins.clone(), eq_gains.clone())?;

//     println!("Playing '{path}'. Press Enter to stop early...");

//     // Spawn a thread to wait for the user to press Enter so we can stop on demand.
//     let (tx, rx) = std::sync::mpsc::channel();
//     thread::spawn(move || {
//         loop {
//             let mut line = String::new();
//             let _ = std::io::stdin().read_line(&mut line);
//             let _ = tx.send(());
//         }
//     });

//     // Poll: exit either when the user presses Enter, or after a max duration
//     // as a fallback (since we don't have an explicit "finished" signal here).
//     let max_runtime = std::time::Duration::from_secs(600);
//     let start = std::time::Instant::now();
//     let mut paused = false;
//     use std::io::Write;

//     loop {
//         if rx.try_recv().is_ok() {
//             if paused {
//                 session.resume();
//                 println!("\nresume");
//             } else {
//                 session.pause();
//                 println!("\npause");
//             }

//             paused = !paused;
//         }

//         let pct = session.progress() as usize;

//         let bar_width = 30;

//         let bar = if pct >= 100 {
//             "=".repeat(bar_width)
//         } else {
//             let filled = (pct * (bar_width - 1)) / 100;

//             "=".repeat(filled) + ">" + &" ".repeat(bar_width - filled - 1)
//         };

//         print!("\r[{bar}] {:3}%", pct);
//         std::io::stdout().flush().unwrap();

//         if pct >= 100 {
//             println!("\nPlayback finished");
//             break;
//         }

//         if start.elapsed() > max_runtime {
//             println!("\nMax runtime reached, stopping...");
//             break;
//         }

//         std::thread::sleep(std::time::Duration::from_millis(100));
//     }

//     Ok(())
// }
