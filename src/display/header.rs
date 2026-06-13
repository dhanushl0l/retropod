use std::fmt::Debug;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{Baseline, Text},
};

pub fn render_header<D>(
    display: &mut D,
    title: &str,
    time: &str,
    style: embedded_graphics::mono_font::MonoTextStyle<'_, BinaryColor>,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
    D::Error: Debug,
{
    // Title on the left
    Text::with_baseline(title, Point::new(0, 0), style, Baseline::Top).draw(display)?;

    // Time on the right (128 - 6*time.len())
    let time_x = 128 - 6 * i32::try_from(time.len()).unwrap_or(0);
    Text::with_baseline(time, Point::new(time_x, 0), style, Baseline::Top).draw(display)?;

    // Underline below header
    Line::new(Point::new(0, 11), Point::new(127, 11))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(display)?;

    Ok(())
}
