//! Draw a 1 bit per pixel black and white image. On a 128x64 SSD1306 display over I2C.
//!
//! This example is for the Teensy 4.0 board using I2C0.
//!
//! Wiring connections are as follows for  display:
//!
//! ```
//!      Display -> Teensy 4.0
//! (black)  GND -> GND
//! (red)    +5V -> 3.3V
//! (green) SDA -> Pin 18
//! (blue)  SCL -> Pin 19
//! ```
//!
//! Run on a Teensy 4.0 with `cargo run --example ssd1306`.

#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use teensy4_bsp as bsp;
use bsp::rt::entry;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use ssd1306::mode::DisplayConfig;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use teensy4_bsp::board;


#[entry]
fn main() -> ! {
    // Grab preconfigured resources for Teensy 4.0
    let board::Resources {
        pins,
        lpi2c1,
        ..
    } = board::t40(board::instances());
    

    let i2c = board::lpi2c(
        lpi2c1,
        pins.p19,
        pins.p18,
        board::Lpi2cClockSpeed::KHz400,
    );
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);

    let im = Image::new(&raw, Point::new(32, 0));

    im.draw(&mut display).unwrap();

    display.flush().unwrap();

    loop {
    }
}
