use std::ops::{BitOr, Shl};
use ibig::{UBig, ubig};
use image::RgbaImage;
use crate::parser::utils::to_argb_hex;
use crate::utils::model::{AlfafaData, Rectangle};

pub(crate) const ENCODE_REGIONS: [Rectangle; 10] = [
    Rectangle { x1: 8, y1: 0, x2: 24, y2: 8 },
    Rectangle { x1: 0, y1: 8, x2: 8, y2: 16 },
    Rectangle { x1: 16, y1: 8, x2: 32, y2: 16 },
    Rectangle { x1: 4, y1: 16, x2: 12, y2: 20 },
    Rectangle { x1: 20, y1: 16, x2: 36, y2: 20 },
    Rectangle { x1: 44, y1: 16, x2: 52, y2: 20 },
    Rectangle { x1: 0, y1: 20, x2: 56, y2: 32 },
    Rectangle { x1: 20, y1: 48, x2: 28, y2: 52 },
    Rectangle { x1: 36, y1: 48, x2: 44, y2: 52 },
    Rectangle { x1: 16, y1: 52, x2: 48, y2: 64 },
];

pub(crate) const PREDEF_KEYS: [&str; 4] = ["END", "wing", "erase", "cape"];

pub fn read_alfalfa(image: &RgbaImage) -> Option<AlfafaData> {
    if image.width() != 64 || image.height() != 64 {
        return None;
    }

    let mut bi = ubig!(0);
    let mut read = 0u32;

    for rect in ENCODE_REGIONS {
        for x in rect.x1..rect.x2 {
            for y in rect.y1..rect.y2 {
                let a = to_argb_hex(image.get_pixel(x, y)) >> 24 & 0xFF;
                if a == 0 {
                    continue;
                }

                let value = (0x7F - (a & 0x7F)) as u32;

                bi = bi.bitor(UBig::from(value).shl(read as usize * 7usize));
                read += 1;
            }
        }
    }

    //TODO: Properly read the data off of the bytes (bi.to_be_bytes())
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v0_detection_works() {
        let image = image::open("test_images/ears_v1_nickac_sample.png").unwrap();
        let image = image.to_rgba8();

        read_alfalfa(&image);
    }
}