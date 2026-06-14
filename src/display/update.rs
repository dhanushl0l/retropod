use std::process::Command;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::Text;
use rppal::i2c::I2c;
use ssd1306::Ssd1306;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::I2CInterface;
use ssd1306::size::DisplaySize128x64;

use crate::audio::init::AudioControlEvent;
use crate::input::ButtonEvent;
use crate::state::{AppState, MenuItem, MusicView, Screen, Theme, get_state};
use crate::{APP_TITLE, STYLE};

use super::header::render_header;

pub fn update_display(
    display: &mut Ssd1306<
        I2CInterface<I2c>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
    button_rx: mpsc::Receiver<ButtonEvent>,
    audio_tx: mpsc::Sender<AudioControlEvent>,
    _bind: Arc<Mutex<Vec<f32>>>,
) {
    loop {
        let event = button_rx.recv_timeout(Duration::from_secs(1));
        match event {
            Ok(val) => handle_button(val, display, &audio_tx),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(e) => {
                eprint!("This nver supposed to happen: {}", e)
            }
        }
    }
}

fn handle_button(
    event: ButtonEvent,
    display: &mut Ssd1306<
        I2CInterface<I2c>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
    audio_tx: &mpsc::Sender<AudioControlEvent>,
) {
    let mut app_state = get_state().lock().unwrap();
    let AppState {
        nav,
        music,
        bt,
        settings,
    } = &mut *app_state;

    match nav.last_mut().unwrap() {
        Screen::MainMenu { selected, items } => match event {
            ButtonEvent::Up => {
                *selected = if *selected == 0 {
                    items.len() - 1
                } else {
                    *selected - 1
                };
            }
            ButtonEvent::Down => {
                *selected = (*selected + 1) % items.len();
            }
            ButtonEvent::Enter => {
                let chosen = items[*selected].clone();
                let next = match chosen {
                    MenuItem::Music => Screen::Music {
                        view: MusicView::List { selected: 0 },
                    },
                    MenuItem::Bluetooth => Screen::Bluetooth,
                    MenuItem::Settings => Screen::Settings,
                };
                drop(items);
                app_state.nav.push(next);
            }
            ButtonEvent::Left | ButtonEvent::Right => {}
        },

        Screen::Music { view } => match view {
            MusicView::List { selected } => match event {
                ButtonEvent::Up => {
                    if !music.collection.is_empty() {
                        *selected = if *selected == 0 {
                            music.collection.len() - 1
                        } else {
                            *selected - 1
                        };
                    }
                }
                ButtonEvent::Down => {
                    if !music.collection.is_empty() {
                        *selected = (*selected + 1) % music.collection.len();
                    }
                }
                ButtonEvent::Enter => {
                    if !music.collection.is_empty() {
                        let idx = *selected;
                        app_state.music.song_position = idx;
                        app_state.music.title = Some(
                            app_state.music.collection[idx]
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default(),
                        );
                        app_state.music.is_playing = true;
                        let path = app_state.music.collection[idx]
                            .to_string_lossy()
                            .to_string();
                        if let Err(e) = audio_tx.send(AudioControlEvent::Play(Some(path))) {
                            eprintln!("Unable to play: {}", e)
                        };
                    }
                }
                ButtonEvent::Right => {
                    *view = MusicView::NowPlaying;
                }
                ButtonEvent::Left => {
                    if app_state.nav.len() > 1 {
                        app_state.nav.pop();
                    }
                }
            },
            MusicView::NowPlaying => match event {
                ButtonEvent::Left => {
                    if app_state.nav.len() > 1 {
                        app_state.nav.pop();
                    }
                }
                _ => {}
            },
        },
        Screen::Bluetooth => match event {
            ButtonEvent::Down => {
                if app_state.nav.len() > 1 {
                    app_state.nav.pop();
                }
            }
            ButtonEvent::Enter => {
                app_state.bt.scanning = !app_state.bt.scanning;
            }
            _ => {}
        },

        Screen::Settings => match event {
            ButtonEvent::Down => {
                if app_state.nav.len() > 1 {
                    app_state.nav.pop();
                }
            }
            ButtonEvent::Left => {
                settings.volume = settings.volume.saturating_sub(5);
                set_system_volume(settings.volume);
            }
            ButtonEvent::Right => {
                settings.volume = settings.volume.saturating_add(5).min(100);
                set_system_volume(settings.volume);
            }
            _ => {}
        },
    }

    // Render after mutation, while still holding the lock
    display.clear(BinaryColor::Off).unwrap();
    render_screen(display, &app_state);
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let hh = (secs % 86400) / 3600;
    let mm = (secs % 3600) / 60;
    let ss = secs % 60;

    display.clear(BinaryColor::Off).unwrap();

    if let Err(e) = render_header(
        display,
        APP_TITLE,
        &format!("{hh:02}:{mm:02}:{ss:02}"),
        *STYLE,
    ) {
        eprintln!("Render error: {e:?}");
    }

    render_screen(display, &app_state);

    display.flush().unwrap();
}

fn render_screen(
    display: &mut Ssd1306<
        I2CInterface<I2c>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
    state: &AppState,
) {
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    match state.nav.last().unwrap() {
        Screen::MainMenu { selected, items } => {
            for (i, item) in items.iter().enumerate() {
                let label = match item {
                    MenuItem::Music => "Music",
                    MenuItem::Bluetooth => "Bluetooth",
                    MenuItem::Settings => "Settings",
                };
                let line = if i == *selected {
                    format!("> {label}")
                } else {
                    format!("  {label}")
                };
                let y = 24 + (i as i32) * 12; // start below header, 12px row height
                Text::new(&line, Point::new(4, y), text_style)
                    .draw(display)
                    .ok();
            }
        }

        Screen::Music { view } => match view {
            MusicView::List { selected } => {
                if state.music.collection.is_empty() {
                    Text::new("No songs found", Point::new(4, 24), text_style)
                        .draw(display)
                        .ok();
                } else {
                    // Show up to 3 visible rows, scrolling so `selected` stays in view
                    let visible_rows = 3;
                    let total = state.music.collection.len();
                    let start = if *selected >= visible_rows {
                        selected - (visible_rows - 1)
                    } else {
                        0
                    };
                    let end = (start + visible_rows).min(total);

                    for (row, idx) in (start..end).enumerate() {
                        let name = state.music.collection[idx]
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        let line = if idx == *selected {
                            format!("> {name}")
                        } else {
                            format!("  {name}")
                        };

                        let y = 24 + (row as i32) * 12;
                        Text::new(&line, Point::new(4, y), text_style)
                            .draw(display)
                            .ok();
                    }
                }
            }

            MusicView::NowPlaying => {
                let title = state.music.title.as_deref().unwrap_or("No track");
                let status = if state.music.is_playing {
                    "Playing"
                } else {
                    "Paused"
                };
                let track_num = if state.music.collection.is_empty() {
                    "0/0".to_string()
                } else {
                    format!(
                        "{}/{}",
                        state.music.song_position + 1,
                        state.music.collection.len()
                    )
                };

                Text::new(title, Point::new(4, 24), text_style)
                    .draw(display)
                    .ok();
                Text::new(status, Point::new(4, 38), text_style)
                    .draw(display)
                    .ok();
                Text::new(&track_num, Point::new(4, 52), text_style)
                    .draw(display)
                    .ok();
            }
        },
        Screen::Bluetooth => {
            let status_line = if state.bt.scanning {
                "Scanning..."
            } else {
                "Idle"
            };
            Text::new(status_line, Point::new(4, 24), text_style)
                .draw(display)
                .ok();

            if let Some(dev) = &state.bt.connected {
                let line = format!("Connected: {}", dev.name);
                Text::new(&line, Point::new(4, 38), text_style)
                    .draw(display)
                    .ok();
            } else {
                Text::new("Not connected", Point::new(4, 38), text_style)
                    .draw(display)
                    .ok();
            }

            for (i, dev) in state.bt.discovered.iter().take(2).enumerate() {
                let y = 50 + (i as i32) * 12;
                Text::new(&dev.name, Point::new(4, y), text_style)
                    .draw(display)
                    .ok();
            }
        }

        Screen::Settings => {
            let vol_line = format!("Volume: {}", state.settings.volume);
            let bright_line = format!("Brightness: {}", state.settings.brightness);
            let theme_line = match state.settings.theme {
                Theme::Light => "Theme: Light",
                Theme::Dark => "Theme: Dark",
                Theme::System => "Theme: System",
            };

            Text::new(&vol_line, Point::new(4, 24), text_style)
                .draw(display)
                .ok();
            Text::new(&bright_line, Point::new(4, 38), text_style)
                .draw(display)
                .ok();
            Text::new(theme_line, Point::new(4, 52), text_style)
                .draw(display)
                .ok();
        }
    }
}

// it is a temp implementation :)
fn set_system_volume(percent: u8) {
    let percent = percent.min(100);
    std::thread::spawn(move || {
        let result = Command::new("wpctl")
            .args([
                "set-volume",
                "@DEFAULT_AUDIO_SINK@",
                &format!("{}%", percent),
            ])
            .status();

        if let Err(e) = result {
            eprintln!("Failed to set volume: {e}");
        }
    });
}
