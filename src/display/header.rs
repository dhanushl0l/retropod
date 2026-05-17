use std::fmt::Debug;

use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{Baseline, Text},
};

pub fn render_header<D>(display: &mut D, title: &str, time: &str)
where
    D: DrawTarget<Color = BinaryColor>,
    D::Error: Debug,
{
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // Title on the left
    Text::with_baseline(title, Point::new(0, 0), style, Baseline::Top)
        .draw(display)
        .unwrap();

    // Time on the right (128 - 6*time.len())
    let time_x = 128 - (6 * time.len()) as i32;
    Text::with_baseline(time, Point::new(time_x, 0), style, Baseline::Top)
        .draw(display)
        .unwrap();

    // Underline below header
    Line::new(Point::new(0, 11), Point::new(127, 11))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(display)
        .unwrap();
}
