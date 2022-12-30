#![no_std]
#![no_main]

// Manually copied from https://github.com/unneback/LCD/blob/0e5aa31ff7fefae214e4248fd488072b02e9c5ff/GreenPAK/LCDx4.hex
const GREENPAK_DATA: [u8; 256] = [
    0xD0, 0x0A, 0xA5, 0x18, 0x41, 0x0C, 0x0A, 0x33, 0x34, 0xC8, 0x30, 0x49, 0x44, 0x4A, 0x0C, 0x29,
    0x31, 0xA4, 0xC4, 0x90, 0x2C, 0x03, 0x20, 0x1C, 0x03, 0x01, 0xA4, 0x54, 0x81, 0x02, 0xE7, 0xFA,
    0x18, 0x2A, 0xB0, 0x12, 0xC3, 0x4A, 0x0C, 0x2B, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x80, 0x03, 0x00, 0x00, 0x0A, 0xA8, 0xB7, 0x0D, 0xB8, 0xC0, 0x05, 0xB4, 0x80,
    0x05, 0xB0, 0x40, 0x05, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x30, 0x58, 0x00, 0x20, 0x20, 0x58, 0x58, 0x00, 0x00, 0x80, 0x80, 0x58, 0x00, 0x58, 0x58,
    0x58, 0x58, 0x58, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x14, 0x22, 0x30, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x28, 0x88, 0x00, 0x00, 0xAC, 0xAC, 0xAC, 0x02, 0x20, 0x08, 0x00, 0x00, 0xAC, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xAC, 0x20, 0x00, 0x01, 0x00, 0x14, 0x01, 0x10, 0x08, 0x60, 0x01, 0x10, 0x00, 0x08,
    0x14, 0x01, 0x10, 0x08, 0x60, 0x01, 0x10, 0x00, 0x08, 0x00, 0x02, 0x02, 0x01, 0x00, 0x20, 0x02,
    0x00, 0x01, 0x00, 0x08, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA5,
];

mod greenpak;
mod lm75;

use esp_println::println;
use fugit::RateExtU32;

use esp32_hal::{
    clock::ClockControl, i2c::I2C, pac::Peripherals, prelude::*, timer::TimerGroup, Delay, Rtc, IO,
};
use esp_backtrace as _;

use greenpak::GreenPAK;
use lm75::LM75;

#[xtensa_lx_rt::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21.into_open_drain_output(),
        io.pins.gpio22.into_open_drain_output(),
        400u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let mut greenpak = GreenPAK::new(i2c);
    greenpak.write_program(&GREENPAK_DATA).unwrap();
    let i2c = greenpak.free();

    let mut delay = Delay::new(&clocks);
    let mut sensor = LM75::new(i2c);

    loop {
        let temp = sensor.measure().unwrap();

        println!("Temperature: {}°C", temp);

        delay.delay_ms(1000u32);
    }
}
