#![no_std]
#![no_main]

mod lm75;
mod st7032;

use core::fmt::Write;

use esp_backtrace as _;
use esp_println::println;
use hal::{clock::ClockControl, i2c::I2C, peripherals::Peripherals, prelude::*, Delay, IO};
use shared_bus::BusManagerSimple;

use crate::{st7032::ST7032, lm75::LM75};

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

    let i2c = BusManagerSimple::new(i2c);

    println!("Hello world!");

    let mut sensor = LM75::new(i2c.acquire_i2c());
    let mut lcd = ST7032::new(i2c.acquire_i2c());

    lcd.init().unwrap();
    delay.delay_ms(500u32);

    lcd.set_line(0, "Jul 2023").unwrap();

    loop {
        let temp = sensor.measure().unwrap();
        lcd.set_cursor(9, 1).unwrap();
        write!(lcd, "{: >5.1} C", temp).unwrap();
        delay.delay_ms(2500u32);
    }
}
