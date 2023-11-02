use std::io::Write;

use crate::utils::errors::EarsError;
use crate::utils::errors::Result;

pub(crate) struct BitWriter<W: Write> {
    data: u8,
    index: i32,
    writer: W,
}

impl<W: Write> Drop for BitWriter<W> {
    fn drop(&mut self) {
        self.align().expect("Unable to align bit writer");
    }
}

#[allow(dead_code)]
impl<W: Write> BitWriter<W> {
    pub(crate) fn new(writer: W) -> BitWriter<W> {
        BitWriter {
            data: 0,
            index: 0,
            writer,
        }
    }

    pub(crate) fn write_bit(&mut self, bit: u8) -> Result<()> {
        self.data <<= 1;
        self.data |= bit & 0x01;
        self.index += 1;

        if self.index >= 8 {
            self.writer
                .write_all(&[self.data])
                .map_err(|e| (e, "Unable to write bit data"))?;
            self.index = 0;
            self.data = 0;
        }

        Ok(())
    }

    pub(crate) fn write_byte(&mut self, value: u8) -> Result<()> {
        if self.index == -1 {
            self.writer
                .write_all(&[value])
                .map_err(|e| (e, "Unable to write byte data"))?;
        } else {
            for i in (0..8).rev() {
                self.write_bit((value >> i) & 1)?;
            }
        }

        Ok(())
    }

    pub(crate) fn write(&mut self, value: u32) -> Result<()> {
        if self.index == -1 {
            self.writer
                .write_all(&[value as u8])
                .map_err(|e| (e, "Unable to write byte data"))?;
        } else {
            for i in (0..8).rev() {
                self.write_bit(((value >> i) & 1) as u8)?;
            }
        }

        Ok(())
    }

    pub(crate) fn write_bool(&mut self, value: bool) -> Result<()> {
        self.write_bit(value as u8)
    }

    pub(crate) fn write_long(&mut self, bits: u8, value: u64) -> Result<()> {
        if !(1..64).contains(&bits) {
            Err(EarsError::NotEnoughSpaceInLongForBitsError(bits))
        } else {
            let mut cur = u64::reverse_bits(value) >> (64 - bits);
            for _ in 0..bits {
                self.write_bit((cur & 1) as u8)?;
                cur >>= 1;
            }

            Ok(())
        }
    }

    pub(crate) fn write_sam_unit(&mut self, bits: u8, value: f32) -> Result<()> {
        self.write_bool(value < 0.0)?;
        let max = (1 << bits) - 1;
        self.write_long(bits, (value.abs() * max as f32) as u64)
    }

    pub(crate) fn write_unit(&mut self, bits: u8, value: f32) -> Result<()> {
        let max = (1 << bits) - 1;
        self.write_long(bits, (value * max as f32) as u64)
    }

    /// Aligns the write marker to the start of the next byte.
    /// If the marker is already at the beginning of a byte, this method does nothing.
    pub(crate) fn align(&mut self) -> Result<()> {
        while self.index != 0 {
            self.write_bit(0)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::utils::bit_reader::BitReader;

    use super::*;

    #[test]
    fn write_complex_works() -> Result<()> {
        let mut output = Vec::new();

        let cursor = Cursor::new(&mut output);

        {
            let mut writer = BitWriter::new(cursor);

            writer.write_bool(true)?;
            writer.write_bool(false)?;

            writer.write_long(6, 63)?;
            writer.write_long(6, 0)?;

            writer.write_byte(255)?;
            writer.write_long(2, 2)?;
        }

        let len = output.len();
        let mut reader = BitReader::new(Cursor::new(&mut output), len);

        assert!(reader.read_bool()?);
        assert!(!(reader.read_bool()?));

        assert_eq!(reader.read_long(6)?, 63);
        assert_eq!(reader.read_long(6)?, 0);

        assert_eq!(reader.read_byte()?, 255);

        assert_eq!(reader.read(2)?, 2);

        Ok(())
    }
}
