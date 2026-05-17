use std::fmt::Debug;

use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

// Your icon as raw 1-bit bytes (1 byte = 8 pixels wide)
// Generate with: https://javl.github.io/image2cpp/
// Settings: 1-bit, vertical, MSB first
const ICON_WIDTH: u32 = 24;
const ICON_HEIGHT: u32 = 24;
const ICON_DATA: &[u8] = &[
    0x00, 0x00, 0x00, // row 0
    0x00, 0x00, 0x00, // row 1
    0x00, 0x00, 0x00, // row 2
    0x00, 0x00, 0x00, // row 3
    0x00, 0x00, 0x03, // row 4
    0x00, 0x00, 0xFF, // row 5
    0x00, 0x3F, 0x83, // row 6
    0x00, 0x3C, 0x03, // row 7
    0x00, 0x37, 0x03, // row 8
    0x00, 0x31, 0xFF, // row 9
    0x00, 0x3F, 0xA3, // row 10
    0x00, 0x3C, 0x03, // row 11
    0x00, 0x37, 0x03, // row 12
    0x00, 0x31, 0xE3, // row 13
    0x00, 0x30, 0xFB, // row 14
    0x00, 0x31, 0xFF, // row 15
    0x0F, 0xB3, 0xFF, // row 16
    0x1F, 0xF3, 0xFF, // row 17
    0x3F, 0xF3, 0xFE, // row 18
    0x3F, 0xF1, 0xFC, // row 19
    0x3F, 0xE0, 0xF8, // row 20
    0x1F, 0xC0, 0x00, // row 21
    0x0F, 0x80, 0x00, // row 22
    0x00, 0x00, 0x00, // row 23
];

pub fn render_body<D>(display: &mut D, label: &str)
where
    D: DrawTarget<Color = BinaryColor>,
    D::Error: Debug,
{
    let box_x = 4;
    let box_y = 14;
    let padding = 3;
    let box_w = ICON_WIDTH + (padding * 2);
    let box_h = ICON_HEIGHT + (padding * 2);

    // Draw the border box
    Rectangle::new(Point::new(box_x, box_y), Size::new(box_w, box_h))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(display)
        .unwrap();

    // Draw the icon centered inside the box
    let icon_raw: ImageRaw<BinaryColor> = ImageRaw::new(ICON_DATA, ICON_WIDTH);

    Image::new(
        &icon_raw,
        Point::new(box_x + padding as i32, box_y + padding as i32),
    )
    .draw(display)
    .unwrap();

    // Draw label text below the box
    let text_y = box_y + box_h as i32 + 8;
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new(label, Point::new(box_x, text_y), style)
        .draw(display)
        .unwrap();
}
