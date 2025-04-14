#![no_std]
#![no_main]

use teensy4_bsp as bsp;

use defmt::*;
use defmt_rtt as _;
use display_interface_spi::SPIInterface;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::Image;
use embedded_graphics::mono_font::ascii::FONT_9X18;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb666, RgbColor};
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_hal_compat::ForwardCompat;
use mipidsi::models::ILI9341Rgb666;
use mipidsi::Builder;
use panic_probe as _;
use teensy4_bsp::board;
use teensy4_bsp::hal::timer::Blocking;
use tinybmp::Bmp;

#[bsp::rt::entry]
fn main() -> ! {
    info!("Program start");
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
    let cs = gpio2.output(pins.p7).forward();
    let dc = gpio2.output(pins.p9).forward(); // Data/Command on pin 9
    let rst = gpio2.output(pins.p8).forward(); // Reset on pin 8
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

    let spi_device = ExclusiveDevice::new_no_delay(spi.forward(), cs).unwrap();
    let di = SPIInterface::new(spi_device, dc);

    let mut display = Builder::new(ILI9341Rgb666, di)
        .reset_pin(rst)
        //.invert_colors(ColorInversion::Inverted)
        //.color_order(ColorOrder::Bgr)
        //.orientation(Orientation::new().rotate(Rotation::Deg180).flip_vertical())
        //.display_size(320, 240)
        .init(&mut delay)
        .unwrap();

    display.clear(Rgb666::BLACK).unwrap();

    // Create text style
    let mut style = MonoTextStyle::new(&FONT_9X18, Rgb666::WHITE);

    // Position x:5, y: 10
    Text::new("Hello", Point::new(5, 10), style)
        .draw(&mut display)
        .unwrap();

    // Turn text to blue
    style.set_text_color(Some(Rgb666::BLUE));
    Text::new("World", Point::new(160, 26), style)
        .draw(&mut display)
        .unwrap();

    let raw_image: Bmp<Rgb666> = Bmp::from_slice(include_bytes!("rust.bmp")).unwrap();
    let image = Image::new(&raw_image, Point::new(100, 40));
    image.draw(&mut display).unwrap();

    loop {}
}
