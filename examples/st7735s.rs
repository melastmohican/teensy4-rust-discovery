//! Draw a image. On 80x160 ST7735 display over SPI.
//!
//! This example is for the Teensy 4.0 board using SPI0.
//!
//! Wiring connections are as follows for  display:
//!
//! ```
//! ST7735S | -> | Teensy 4.0
//! ----|----|-----
//! VCC | -> | 3.3V
//! GND | -> | GND
//! SCK | -> | 13
//! MISO | -> | NC
//! MOSI | -> | 11
//! CS | -> | 10
//! DC | -> | 9
//! RST | -> | 8
//! BL | -> | 7
//! ```
//!
//! Run on a Teensy 4.0 with `cargo run --example st7735s`.

#![no_std]
#![no_main]

use teensy4_bsp as bsp;

use bsp::board;
use bsp::hal::timer::Blocking;
use bsp::rt::entry;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::Drawable;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_hal_compat::eh1_0::digital::OutputPin;
use embedded_hal_compat::ForwardCompat;
use st7735_lcd::Orientation;
use tinybmp::Bmp;

#[entry]
fn main() -> ! {
    // Get all hardware resources
    let board::Resources {
        mut gpio2,
        pit,
        pins,
        lpspi4,
        ..
    } = board::t40(board::instances());

    // Delay using PIT
    let mut delay = Blocking::<_, { board::PERCLK_FREQUENCY }>::from_pit(pit.0).forward();

    // Configure pads
    let sck = pins.p13; // SCK
    let sdi = pins.p12; // MISO
    let sdo = pins.p11; // MOSI

    let cs0 = pins.p10; // Chip Select on pin 10
    let cs = gpio2.output(pins.p6).forward();
    let dc = gpio2.output(pins.p9).forward(); // Data/Command on pin 9
    let rst = gpio2.output(pins.p8).forward(); // Reset on pin 8
    let mut bl = gpio2.output(pins.p7).forward();
    let spi: board::Lpspi4 = board::lpspi(
        lpspi4,
        board::LpspiPins {
            sdo,
            sdi,
            sck,
            pcs0: cs0,
        },
        1_000_000,
    );

    let spi_dev = ExclusiveDevice::new_no_delay(spi.forward(), cs).unwrap();

    let mut display = st7735_lcd::ST7735::new(spi_dev, dc, rst, false, true, 160, 80);

    display.init(&mut delay).unwrap();
    display.set_offset(1, 26);
    display
        .set_orientation(&Orientation::LandscapeSwapped)
        .unwrap();
    display.clear(Rgb565::BLACK).unwrap();
    display.set_offset(0, 25);

    // draw ferris
    let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("ferris.raw"), 86);
    let image: Image<_> = Image::new(&image_raw, Point::new(80, 8));
    image.draw(&mut display).unwrap();
    // draw rust logo
    let logo = Bmp::from_slice(include_bytes!("rust.bmp")).unwrap();
    let logo = Image::new(&logo, Point::new(0, 0));
    logo.draw(&mut display).unwrap();

    bl.set_high().ok();

    loop {}
}
