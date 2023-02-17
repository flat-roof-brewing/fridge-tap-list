#![no_std]
#![no_main]

// Manually copied from https://github.com/unneback/LCD/blob/10709f03c5c57bdf737b54c35fc8c82c67a4ae96/GreenPAK/LCDx4.hex
const GREENPAK_DATA: [u8; 256] = [
    0xD0, 0x0A, 0xA5, 0x58, 0x5D, 0xD3, 0x0A, 0x43, 0x37, 0x08, 0x3D, 0x49, 0x75, 0x5A, 0xD3, 0x69,
    0x4D, 0xA7, 0x35, 0x9D, 0x2C, 0x34, 0x20, 0x1C, 0x74, 0x0D, 0xA4, 0x54, 0x81, 0x02, 0xE7, 0xFA,
    0x18, 0x2A, 0xB0, 0xD6, 0xF4, 0x5A, 0xD3, 0x6B, 0x4D, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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

use core::fmt::Write;

use esp_println::println;
use fugit::RateExtU32;

use esp32_hal::{
    clock::ClockControl, i2c::I2C, pac::Peripherals, prelude::*, timer::TimerGroup, Delay, Rtc, IO,
};
use esp_backtrace as _;

use greenpak::GreenPAK;
use lm75::LM75;
use shared_bus::BusManagerSimple;
use st7032i::ST7032i;

macro_rules! log_error {
    ($value:expr, $message:expr) => {
        match $value {
            Ok(_) => {},
            Err(err) => {
                println!(concat!($message, ": {:?}"), err)
            }
        }
    };
    ($value:expr, $message:expr$(, $args:expr)*) => {
        match $value {
            Ok(_) => {},
            Err(err) => {
                println!(concat!($message, ": {:?}"), $($args),*, err)
            }
        }
    };
}

fn select_lcd<I: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::WriteRead> (greenpak: &mut GreenPAK<I>, lcd: u8) -> Result<(), <I as embedded_hal::blocking::i2c::Write>::Error> {
    assert!(lcd < 4);

    greenpak.write_byte(0x7A, 0b01000000 | (lcd << 4))
}

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

    let i2c = BusManagerSimple::new(i2c);

    let mut greenpak = GreenPAK::new(i2c.acquire_i2c());
    log_error!(greenpak.write_program(&GREENPAK_DATA), "Failed to write program to GreenPAK");

    let mut delay = Delay::new(&clocks);
    let mut sensor = LM75::new(i2c.acquire_i2c());
    let mut lcd = ST7032i::new(i2c.acquire_i2c(), Delay::new(&clocks), 2, 0x20);

    for i in 0..4 {
        log_error!(select_lcd(&mut greenpak, i), "Failed to select LCD {}", i);
        log_error!(lcd.init(), "Failed to initialize LCD {}", i);
    }

    log_error!(select_lcd(&mut greenpak, 0), "Failed to select LCD 0");
    log_error!(write!(lcd, "White House Ale"), "Failed to write to LCD 0");

    log_error!(select_lcd(&mut greenpak, 1), "Failed to select LCD 1");
    log_error!(write!(lcd, "Milky Way"), "Failed to write to LCD 1");

    log_error!(select_lcd(&mut greenpak, 2), "Failed to select LCD 2");
    log_error!(write!(lcd, "Jul 2022"), "Failed to write to LCD 2");

    log_error!(select_lcd(&mut greenpak, 3), "Failed to select LCD 3");
    log_error!(write!(lcd, "Reservoir Hops"), "Failed to write to LCD 3");

    loop {
        let temp = sensor.measure().unwrap();

        log_error!(select_lcd(&mut greenpak, 0), "Failed to select LCD 0");
        log_error!(lcd.move_cursor(1, 0), "Failed to move cursor on LCD 0");
        log_error!(write!(lcd, "Temp: {: >5.1}°C", temp), "Failed to write to LCD 0");

        delay.delay_ms(1000u32);
    }
}
