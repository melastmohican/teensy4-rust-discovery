#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;

use bsp::board;
use bsp::hal::timer::Blocking;

#[bsp::rt::entry]
fn main() -> ! {
   
    let board::Resources {
        pit,
        pins, mut gpio2, ..
    } = board::t40(board::instances());
    let led = board::led(&mut gpio2, pins.p13);

    let mut delay = Blocking::<_, { board::PERCLK_FREQUENCY }>::from_pit(pit.0);
    loop {
        led.toggle();
        delay.block_ms(1000);
    }
}