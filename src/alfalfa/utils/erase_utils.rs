use std::io::{Cursor, Read};

use crate::utils::bit_reader::BitReader;
use crate::utils::errors::Result;
use crate::utils::model::AlfalfaData;

#[derive(Debug, Clone, Eq, PartialEq)]
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
}

pub trait EraseRegionsProvider {
    fn get_erase_regions(&self) -> Result<Option<Vec<EraseRegion>>>;
}

const ERASE_KEY: &str = "erase";

impl EraseRegionsProvider for AlfalfaData {
    fn get_erase_regions(&self) -> Result<Option<Vec<EraseRegion>>> {
        if self.data.contains_key(ERASE_KEY) {
            let data = &self.data[ERASE_KEY];
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
}
