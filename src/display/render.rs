use std::{
    thread,
    time::{Duration, Instant},
};

use chrono::Local;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};
use rppal::i2c::I2c;
use ssd1306::{
    Ssd1306, mode::BufferedGraphicsMode, prelude::I2CInterface, size::DisplaySize128x64,
};

use crate::{
    APP_TITLE,
    display::{header::render_header, render_menu::render_body},
};

pub fn render_display(
    mut display: Ssd1306<
        I2CInterface<I2c>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
) {
    loop {
        display.clear(BinaryColor::Off).unwrap();

        let now = Local::now();
        let time_str = now.format("%I:%M:%S %P").to_string();
        render_header(&mut display, APP_TITLE, &time_str);
        render_body(&mut display, "music");
        display.flush().unwrap();

        thread::sleep(Duration::from_secs(1));
    }
}
