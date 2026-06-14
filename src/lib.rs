use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;

use std::sync::LazyLock;

pub mod audio;
pub mod display;
pub mod input;

pub const APP_TITLE: &str = "retropod";

pub static STYLE: LazyLock<MonoTextStyle<'static, BinaryColor>> = LazyLock::new(|| {
    MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build()
});
