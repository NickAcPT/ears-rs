pub(crate) mod utils;
mod v0;
mod v1;

use crate::features::EarsFeatures;
use crate::parser::utils::to_argb_hex;
use crate::utils::errors::Result;
use image::RgbaImage;

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
