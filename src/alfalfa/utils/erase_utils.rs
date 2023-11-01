use std::io::{Cursor, Read, Write};

use crate::utils::bit_reader::{BitReader, BitWriter};
use crate::utils::errors::Result;
use crate::utils::model::{AlfalfaData, AlfalfaDataKey};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct EraseRegion {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
}

impl EraseRegion {
    fn decode<R: Read>(reader: &mut BitReader<R>) -> Result<EraseRegion> {
        let x = reader.read(6)? as u8;
        let y = reader.read(6)? as u8;

        let width = reader.read(5)? as u8 + 1;
        let height = reader.read(5)? as u8 + 1;

        Ok(EraseRegion {
            x,
            y,
            width,
            height,
        })
    }

    fn encode<W: Write>(&self, writer: &mut BitWriter<W>) -> Result<()> {
        writer.write_long(6, self.x as u64)?;
        writer.write_long(6, self.y as u64)?;

        writer.write_long(5, (self.width - 1) as u64)?;
        writer.write_long(5, (self.height - 1) as u64)?;

        Ok(())
    }

    fn encode_regions<W: Write>(regions: &[EraseRegion], writer: &mut BitWriter<W>) -> Result<()> {
        for region in regions {
            region.encode(writer)?;
        }

        Ok(())
    }
}

pub trait EraseRegionsProvider {
    fn get_erase_regions(&self) -> Result<Option<Vec<EraseRegion>>>;
    fn set_erase_regions(&mut self, regions: Vec<EraseRegion>) -> Result<()>;
}

impl EraseRegionsProvider for AlfalfaData {
    fn get_erase_regions(&self) -> Result<Option<Vec<EraseRegion>>> {
        if let Some(data) = self.get_data(AlfalfaDataKey::Erase) {
            let mut reader = BitReader::new(Cursor::new(data), data.len());

            let mut regions = Vec::new();
            loop {
                if reader.available() == 0 {
                    break;
                }
                regions.push(EraseRegion::decode(&mut reader)?);
            }

            Ok(Some(regions))
        } else {
            Ok(None)
        }
    }

    fn set_erase_regions(&mut self, regions: Vec<EraseRegion>) -> Result<()> {
        let mut data = Vec::new();
        {
            let mut writer = BitWriter::new(Cursor::new(&mut data));
            EraseRegion::encode_regions(&regions, &mut writer)?;
        }

        self.set_data(AlfalfaDataKey::Erase, data);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::alfalfa::io::read_alfalfa;

    use super::*;

    #[test]
    fn erase_region_reading_works() -> Result<()> {
        let image = image::open("test_images/ears_v1_nickac_sample.png").unwrap();
        let image = image.to_rgba8();

        let data = read_alfalfa(&image)?;
        assert!(data.is_some());

        let data = data.unwrap();
        let regions = data.get_erase_regions()?;

        assert_eq!(
            regions,
            Some(vec![
                EraseRegion {
                    x: 49,
                    y: 8,
                    width: 7,
                    height: 8,
                },
                EraseRegion {
                    x: 32,
                    y: 8,
                    width: 7,
                    height: 8,
                },
                EraseRegion {
                    x: 42,
                    y: 13,
                    width: 4,
                    height: 2,
                },
                EraseRegion {
                    x: 32,
                    y: 38,
                    width: 7,
                    height: 10,
                },
            ])
        );

        Ok(())
    }

    #[test]
    fn erase_region_write_works() -> Result<()> {
        let data: Vec<u8> = vec![
            0xc4, 0x83, 0x1e, 0x02, 0x0c, 0x7a, 0x8d, 0x18, 0x60, 0x98, 0xc9,
        ];

        let regions = vec![
            EraseRegion {
                x: 49,
                y: 8,
                width: 7,
                height: 8,
            },
            EraseRegion {
                x: 32,
                y: 8,
                width: 7,
                height: 8,
            },
            EraseRegion {
                x: 42,
                y: 13,
                width: 4,
                height: 2,
            },
            EraseRegion {
                x: 32,
                y: 38,
                width: 7,
                height: 10,
            },
        ];

        let mut out = Vec::new();
        {
            let mut writer = BitWriter::new(Cursor::new(&mut out));
            EraseRegion::encode_regions(&regions, &mut writer)?;
        }
        
        assert_eq!(out, data);

        Ok(())
    }
}
