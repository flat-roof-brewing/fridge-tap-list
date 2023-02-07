use embedded_hal::blocking::i2c::{Write, WriteRead};

const ADDR: u8 = 0b0001000;

pub struct GreenPAK<I> {
    device: I,
}

impl<I: Write + WriteRead> GreenPAK<I> {
    /// Create device driver instance.
    pub fn new(i2c: I) -> Self {
        GreenPAK { device: i2c }
    }

    pub fn write_byte(&mut self, offset: u8, byte: u8) -> Result<(), <I as Write>::Error> {
        self.device.write(ADDR, &[offset, byte])
    }

    pub fn write_program(&mut self, data: &[u8; 256]) -> Result<(), <I as Write>::Error> {
        for (idx, chunk) in data.chunks(16).enumerate() {
            let mut cmd = [0u8; 17];

            cmd[0] = (idx * 16) as u8;
            cmd[1..].copy_from_slice(chunk);

            self.device.write(ADDR, &cmd)?;
        }

        Ok(())
    }

    pub fn read_byte(&mut self, offset: u8) -> Result<u8, <I as WriteRead>::Error> {
        let mut buf = [0u8; 1];
        self.device.write_read(ADDR, &[offset], &mut buf)?;
        Ok(buf[0])
    }

    pub fn read_program(&mut self) -> Result<[u8; 256], <I as WriteRead>::Error> {
        let mut data = [0u8; 256];
        let mut buf = [0u8; 1];

        for idx in 0..256usize {
            self.device.write_read(ADDR, &[idx as u8], &mut buf)?;
            data[idx] = buf[0];
        }

        Ok(data)
    }

    pub fn free(self) -> I {
        self.device
    }
}
