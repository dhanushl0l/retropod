pub enum AudioControlEvent {
    Play(Option<String>),
    Pause,
    Stop,
}
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use super::player::PlaybackSession;

pub fn audio_thread(audio_rx: Receiver<AudioControlEvent>, bins: Arc<Mutex<Vec<f32>>>) {
    let mut is_paused = false;
    let mut player = None;
    loop {
        while let Ok(event) = audio_rx.try_recv() {
            match event {
                AudioControlEvent::Stop => return,
                AudioControlEvent::Play(path) => {
                    if let Some(path) = path {
                        player = match PlaybackSession::start(&path, bins.clone()) {
                            Ok(player) => {
                                is_paused = false;
                                Some(player)
                            }
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            }
                        };
                    } else {
                        if let Some(player) = &player {
                            player.resume();
                            is_paused = false;
                        } else {
                            eprintln!("Player not initalized")
                        }
                    }
                }
                AudioControlEvent::Pause => {
                    if let Some(player) = &player {
                        player.pause();
                        is_paused = true;
                    }
                }
            }
            if is_paused {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
        }
    }
}
