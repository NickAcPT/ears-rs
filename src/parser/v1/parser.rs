use crate::features::EarsFeatures;
use crate::parser::utils::to_argb_hex;
use crate::parser::EarsFeaturesParser;
use crate::utils::errors::{EarsError, Result};
use image::RgbaImage;
use std::io::{BufWriter, Write};

const V1_PARSER_MAGIC: u32 = 0xFFEA2501;

struct EarsParserV1;

impl EarsFeaturesParser for EarsParserV1 {
    fn get_data_version() -> u8 {
        1
    }

    fn detect_magic_pixel() -> u32 {
        V1_PARSER_MAGIC
    }

    fn parse(image: &RgbaImage) -> Result<Option<EarsFeatures>> {
        let data = Vec::new();
        let mut data_writer = BufWriter::new(data);

        for y in 0..4 {
            for x in 0..4 {
                if x == 0 && y == 0 {
                    continue;
                }
                let c = to_argb_hex(
                    image
                        .get_pixel_checked(x, 32 + y)
                        .ok_or(EarsError::InvalidPixelLocation(x, y))?,
                );

                data_writer.write_all(((c >> 16) & 0xFF).to_be_bytes().as_slice())?;
                data_writer.write_all(((c >> 8) & 0xFF).to_be_bytes().as_slice())?;
                data_writer.write_all((c & 0xFF).to_be_bytes().as_slice())?;
            }
        }

        Ok(None)
    }
}
