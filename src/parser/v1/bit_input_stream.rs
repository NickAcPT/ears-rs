use crate::utils::errors::Result;
use std::io::{Cursor, Read};

struct BitReader<R: Read> {
    data: u8,
    index: i32,
    reader: R,
}

impl<R: Read> BitReader<R> {
    fn new(reader: R) -> BitReader<R> {
        BitReader {
            data: 0,
            index: -1,
            reader,
        }
    }

    fn read_bit(&mut self) -> Result<u8> {
        Ok(if self.index < 0 {
            let mut buf = [0u8; 1];
            self.reader.read_exact(&mut buf)?;

            self.data = buf[0];
            self.index = 6;

            (self.data >> 7) & 0x01
        } else {
            self.index -= 1;
            (self.data >> (self.index + 1)) & 0x01
        })
    }

    fn read_bool(&mut self) -> Result<bool> {
        self.read_bit().map(|b| b == 1)
    }
}
