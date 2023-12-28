#![no_std]
#![no_main]

mod st7032;

use esp_backtrace as _;
use esp_println::println;
use hal::{clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*, Delay, IO};

use crate::st7032::ST7032;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut reset_pin = io.pins.gpio1.into_open_drain_output();

    reset_pin.set_low().unwrap();
    delay.delay_ms(250u32);
    reset_pin.set_high().unwrap();
    delay.delay_ms(250u32);

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio6.into_open_drain_output(),
        io.pins.gpio7.into_open_drain_output(),
        400u32.kHz(),
        &clocks,
    );

    println!("Hello world!");

    let mut lcd = ST7032::new(i2c);

    lcd.init().unwrap();
    delay.delay_ms(500u32);

    lcd.set_line(0, "Jul 2023").unwrap();

    loop {
        println!("Loop...");
        delay.delay_ms(500u32);
    }
}
