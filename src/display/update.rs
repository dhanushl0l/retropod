use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use rppal::i2c::I2c;
use ssd1306::Ssd1306;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::I2CInterface;
use ssd1306::size::DisplaySize128x64;

use crate::audio::init::AudioControlEvent;
use crate::input::ButtonEvent;
use crate::{APP_TITLE, STYLE};

use super::header::render_header;

pub fn update_display(
    display: &mut Ssd1306<
        I2CInterface<I2c>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
    button_rx: mpsc::Receiver<ButtonEvent>,
    _audio_tx: mpsc::Sender<AudioControlEvent>,
) {
    let mut last_second = 0u64;
    loop {
        let event = button_rx.recv_timeout(Duration::from_secs(1));
        match event {
            Ok(ButtonEvent::Enter) => {
                println!("enter")
            }
            Ok(_) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(e) => {
                eprint!("This nver supposed to happen: {}", e)
            }
        }
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if secs != last_second {
            last_second = secs;
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
            display.flush().unwrap();
        }
    }
}
