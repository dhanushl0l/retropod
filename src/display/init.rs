use core::error;
use std::io;

use rppal::i2c::I2c;
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::BufferedGraphicsMode, prelude::*};

pub fn setup_display() -> Result<
    Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
    Box<dyn error::Error>,
> {
    let i2c = I2c::new()?;
    let interface = I2CDisplayInterface::new(i2c);
    let display_size = DisplaySize128x64;

    let mut display = Ssd1306::new(interface, display_size, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display
        .init()
        .map_err(|e| io::Error::other(format!("{:?}", e)))?;
    Ok(display)
}
