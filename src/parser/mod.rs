use image::RgbaImage;

use crate::features::EarsFeatures;
use crate::parser::utils::to_argb_hex;
use crate::utils::errors::Result;

pub(crate) mod utils;
mod v0;
mod v1;

pub(crate) trait EarsFeaturesParser {
    fn get_data_version() -> u8;

    fn detect_magic_pixel() -> u32;

    fn detect(image: &RgbaImage) -> bool {
        let magic_pixel = image.get_pixel_checked(0, 32);
        if let Some(magic_pixel) = magic_pixel {
            let magic_pixel = to_argb_hex(magic_pixel);

            magic_pixel == Self::detect_magic_pixel()
        } else {
            false
        }
    }

    fn parse(image: &RgbaImage) -> Result<Option<EarsFeatures>>;
}

pub struct EarsParser;

impl EarsParser {
    pub fn parse(image: &RgbaImage) -> Result<Option<EarsFeatures>> {
        Ok(if let Some(features) = v0::parser::EarsParserV0::parse(image)? {
            Some(features)
        } else {
            v1::parser::EarsParserV1::parse(image)?
        })
    }
}
