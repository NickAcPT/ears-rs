use crate::utils::errors::{EarsError, Result};
use std::io::Read;

pub(crate) struct BitReader<R: Read> {
    data: u8,
    index: i32,
    length: usize,
    current_index: usize,
    reader: R,
}

#[allow(dead_code)]
impl<R: Read> BitReader<R> {
    pub(crate) fn new(reader: R, len: usize) -> BitReader<R> {
        BitReader {
            data: 0,
            index: -1,
            length: len,
            current_index: 0,
            reader,
        }
    }

    pub(crate) fn available(&self) -> usize {
        self.length - self.current_index
    }

    pub(crate) fn read_bit(&mut self) -> Result<u8> {
        Ok(if self.index < 0 {
            let mut buf = [0u8; 1];
            self.reader
                .read_exact(&mut buf)
                .map_err(|e| (e, "Unable to read bit data"))?;
            self.current_index += 1;

            self.data = buf[0];
            self.index = 6;

            (self.data >> 7) & 0x01
        } else {
            self.index -= 1;
            (self.data >> (self.index + 1)) & 0x01
        })
    }

    pub(crate) fn read_bool(&mut self) -> Result<bool> {
        self.read_bit().map(|b| b == 1)
    }

    pub(crate) fn read_byte(&mut self) -> Result<u8> {
        let mut result = 0u8;
        for _ in 0..8 {
            let cur = self.read_bit()?;
            result = (result << 1) | cur;
        }

        Ok(result)
    }

    pub(crate) fn read_long(&mut self, bits: u8) -> Result<u64> {
        if bits == 0 {
            Ok(0u64)
        } else if bits > 64 {
            Err(EarsError::NotEnoughSpaceInLongForBitsError(bits))
        } else {
            let mut result = 0u64;
            for _ in 0..bits {
                let cur = self.read_bit()?;
                result = (result << 1) | (cur as u64);
            }

            Ok(result)
        }
    }

    pub(crate) fn read(&mut self, bits: u8) -> Result<u32> {
        if bits > 32 {
            Err(EarsError::NotEnoughSpaceInIntForBitsError(bits))
        } else {
            Ok(self.read_long(bits)? as u32)
        }
    }

    /// Reads a sign-and-magnitude signed number of the given length (excluding sign bit), then
    /// divides it by the max value, giving a unit value from -1 to 1.
    pub(crate) fn read_sam_unit(&mut self, bits: u8) -> Result<f32> {
        let sign = self.read_bool()?;
        let value = self.read(bits)?;

        let max = (1 << bits) - 1;
        let float = value as f32 / max as f32;

        if sign {
            Ok(-float)
        } else {
            Ok(float)
        }
    }

    pub(crate) fn read_unit(&mut self, bits: u8) -> Result<f32> {
        let value = self.read(bits)? as f32;
        Ok(value / (((1 << bits) - 1) as f32))
    }

    /// Aligns the read marker to the start of the next byte.
    /// If the marker is already at the beginning of a byte, this method does nothing.
    pub(crate) fn align(&mut self) -> Result<()> {
        while self.index > 0 {
            self.read_bit()?;
        }

        Ok(())
    }
}
