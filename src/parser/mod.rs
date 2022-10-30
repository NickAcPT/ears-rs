pub(crate) mod utils;
mod v0;
mod v1;

use crate::features::EarsFeatures;
use crate::parser::utils::to_argb_hex;
use image::RgbaImage;

pub(crate) trait EarsFeaturesParser {
    fn get_data_version() -> u8;

    fn detect_magic_pixel() -> i32;

    fn detect(image: &RgbaImage) -> bool {
        let magic_pixel = image.get_pixel(0, 32);
        let magic_pixel = to_argb_hex(magic_pixel);

        magic_pixel == Self::detect_magic_pixel()
    }

    fn parse(image: &RgbaImage) -> Option<EarsFeatures>;
}
