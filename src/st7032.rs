use embedded_hal::blocking::i2c::Write;
use hex_literal::hex;

const ST7032_I2CADDR: u8 = 0x3E;

const CLEAR: [u8; 2] = hex!("0001");
const HOME: [u8; 2] = hex!("0002");
const INIT: [u8; 10] = hex!("00383914785e6d0c0106");
const SEND: [u8; 1] = hex!("40");

fn format_send<'a>(buffer: &'a mut [u8], text: &str, pad_with_spaces: bool) -> &'a [u8] {
    assert!(text.len() <= 16, "Text too long");
    assert!(text.is_ascii(), "Text must be ASCII");

    buffer[0] = SEND[0];

    for (idx, byte) in text.as_bytes().iter().enumerate() {
        buffer[idx + 1] = *byte;
    }

    if pad_with_spaces {
        for byte in buffer.iter_mut().skip(text.len() + 1) {
            *byte = b' ';
        }

        &buffer[0..]
    } else {
        &buffer[..(text.len() + 1)]
    }
}

pub struct ST7032<I> {
    device: I,
}

impl<I: Write> ST7032<I> {
    /// Create device driver instance.
    pub fn new(i2c: I) -> Self {
        ST7032 { device: i2c }
    }

    pub fn init(&mut self) -> Result<(), I::Error> {
        self.device.write(ST7032_I2CADDR, &INIT)
    }

    pub fn clear(&mut self) -> Result<(), I::Error> {
        self.device.write(ST7032_I2CADDR, &CLEAR)
    }

    pub fn home(&mut self) -> Result<(), I::Error> {
        self.device.write(ST7032_I2CADDR, &HOME)
    }

    /// # Panics
    ///
    /// Panics if `x` is not in range `0..=15`.
    /// Panics if `y` is not in range `0..=3`.
    pub fn set_cursor(&mut self, x: u8, y: u8) -> Result<(), I::Error> {
        assert!(x <= 15, "Invalid column: {}", x);

        let row = match y {
            0 => 0x00,
            1 => 0x40,
            2 => 0x14,
            3 => 0x54,
            _ => panic!("Invalid row: {}", y),
        };

        let cmd = [0x00, 0x80 | (row + x)];

        self.device.write(ST7032_I2CADDR, &cmd)
    }

    /// # Panics
    ///
    /// Panics if `text` is longer than 16 characters.
    /// Panics if `text` contains non-ASCII characters.
    pub fn put_text(&mut self, text: &str) -> Result<(), I::Error> {
        let mut buffer = [0u8; 17];
        let cmd = format_send(&mut buffer, text, false);

        self.device.write(ST7032_I2CADDR, &cmd)
    }

    /// # Panics
    ///
    /// Panics if `y` is not in range `0..=3`.
    /// Panics if `text` is longer than 16 characters.
    /// Panics if `text` contains non-ASCII characters.
    pub fn set_line(&mut self, y: u8, text: &str) -> Result<(), I::Error> {
        self.set_cursor(0, y)?;

        let mut buffer = [0u8; 17];
        let cmd = format_send(&mut buffer, text, true);

        self.device.write(ST7032_I2CADDR, &cmd)
    }
}

impl<I, E> core::fmt::Write for ST7032<I>
where
    I: Write<Error = E>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        let mut command = [0x40, 0, 0, 0];
        c.encode_utf8(&mut command[1..]);
        self.device.write(ST7032_I2CADDR, &command[..2]).ok();
        Ok(())
    }
}
