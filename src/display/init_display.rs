use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use rppal::i2c::I2c;
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::BufferedGraphicsMode, prelude::*};

pub fn setup_display()
-> Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>> {
    let i2c = I2c::new().unwrap();
    let interface = I2CDisplayInterface::new(i2c);
    let display_size = DisplaySize128x64; // ← swap this out

    let mut display = Ssd1306::new(interface, display_size, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display
}
