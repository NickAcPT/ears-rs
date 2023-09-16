use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::ops::{BitOr, Shl};

use byteorder::{BigEndian, ReadBytesExt};
use ibig::{ubig, UBig};
use image::RgbaImage;

use crate::parser::utils::to_argb_hex;
use crate::utils::errors::{EarsError, Result};
use crate::utils::model::{AlfalfaData, Rectangle};

const ENCODE_REGIONS: [Rectangle; 10] = [
    Rectangle {
        x1: 8,
        y1: 0,
        x2: 24,
        y2: 8,
    },
    Rectangle {
        x1: 0,
        y1: 8,
        x2: 8,
        y2: 16,
    },
    Rectangle {
        x1: 16,
        y1: 8,
        x2: 32,
        y2: 16,
    },
    Rectangle {
        x1: 4,
        y1: 16,
        x2: 12,
        y2: 20,
    },
    Rectangle {
        x1: 20,
        y1: 16,
        x2: 36,
        y2: 20,
    },
    Rectangle {
        x1: 44,
        y1: 16,
        x2: 52,
        y2: 20,
    },
    Rectangle {
        x1: 0,
        y1: 20,
        x2: 56,
        y2: 32,
    },
    Rectangle {
        x1: 20,
        y1: 48,
        x2: 28,
        y2: 52,
    },
    Rectangle {
        x1: 36,
        y1: 48,
        x2: 44,
        y2: 52,
    },
    Rectangle {
        x1: 16,
        y1: 52,
        x2: 48,
        y2: 64,
    },
];

const MAGIC: u32 = 0xEA1FA1FA; // EALFALFA
const PREDEF_KEYS: [&str; 4] = ["END", "wing", "erase", "cape"];

pub fn read_alfalfa(image: &RgbaImage) -> Result<Option<AlfalfaData>> {
    let data = decode_alfalfa(image)?;

    if data.is_none() {
        return Ok(None);
    }

    let mut data = Cursor::new(data.unwrap());

    let magic = data.read_u32::<BigEndian>().map_err(|e| (e, "Unable to read Magic data"))?;

    if magic != MAGIC {
        return Ok(None);
    }

    let version = data.read_u8().map_err(|e| (e, "Unable to read version"))?;

    if version != 1 {
        // Don't know how to read this version, ignoring
        return Ok(None);
    }

    let mut map = HashMap::with_capacity(PREDEF_KEYS.len());

    loop {
        let index = data.read_u8().map_err(|e| (e, "Unable to read alfalfa key index"))?;
        let key = if (index as usize) >= PREDEF_KEYS.len() {
            format!("!unk{}", index)
        } else {
            PREDEF_KEYS[index as usize].to_string()
        };

        if key == "END" {
            break;
        }

        let mut buf = Vec::with_capacity(256);

        loop {
            let len = data.read_u8().map_err(|e| (e, "Unable to read data length"))?;
            // Read len bytes into the end of the buffer
            let old_len = buf.len();
            let new_len = old_len + len as usize;
            buf.resize(new_len, 0);
            data.read_exact(&mut buf[old_len..new_len]).map_err(|e| (e, "Unable to read alfalfa data into buffer"))?;

            if len != 255 {
                break;
            }
        }

        map.insert(key, buf);
    }

    Ok(Some(AlfalfaData { version, data: map }))
}

fn decode_alfalfa(image: &RgbaImage) -> Result<Option<Vec<u8>>> {
    if image.width() != 64 || image.height() != 64 {
        return Ok(None);
    }

    let mut bi = ubig!(0);
    let mut read = 0u32;

    for rect in ENCODE_REGIONS {
        for x in rect.x1..rect.x2 {
            for y in rect.y1..rect.y2 {
                let pixel = image
                    .get_pixel_checked(x, y)
                    .ok_or(EarsError::InvalidAlfalfaPixelPosition(x, y))?;
                let a = to_argb_hex(pixel) >> 24 & 0xFF;
                if a == 0 {
                    continue;
                }

                let value = (0x7F - (a & 0x7F)) as u32;

                bi = bi.bitor(UBig::from(value).shl(read as usize * 7usize));
                read += 1;
            }
        }
    }

    let vec = bi.to_be_bytes();
    Ok(if vec.is_empty() { None } else { Some(vec) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alfalfa_read_works() -> Result<()> {
        let image = image::open("test_images/ears_v1_nickac_sample.png").unwrap();
        let image = image.to_rgba8();

        let map = read_alfalfa(&image)?;

        assert_eq!(
            map,
            Some(AlfalfaData {
                version: 1,
                data: HashMap::from([
                    (
                        "erase".to_string(),
                        [196, 131, 30, 2, 12, 122, 141, 24, 96, 152, 201].to_vec()
                    ),
                    (
                        "cape".to_string(),
                        [
                            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0,
                            20, 0, 0, 0, 16, 8, 6, 0, 0, 0, 22, 24, 95, 27, 0, 0, 2, 227, 73, 68,
                            65, 84, 120, 156, 141, 148, 91, 72, 84, 81, 24, 133, 215, 156, 57, 163,
                            51, 14, 131, 78, 83, 94, 38, 77, 71, 195, 176, 146, 66, 41, 181, 210,
                            210, 204, 202, 52, 145, 172, 36, 3, 49, 169, 4, 9, 11, 161, 52, 3, 43,
                            136, 160, 11, 61, 4, 249, 16, 37, 138, 189, 148, 21, 66, 55, 234, 165,
                            94, 194, 130, 160, 160, 32, 240, 33, 186, 80, 97, 151, 55, 159, 66, 86,
                            235, 204, 30, 26, 35, 168, 30, 190, 179, 255, 253, 239, 179, 214, 255,
                            159, 189, 247, 12, 38, 1, 78, 187, 193, 49, 141, 105, 34, 75, 164, 136,
                            27, 98, 196, 89, 179, 192, 207, 26, 243, 68, 241, 95, 40, 21, 171, 4,
                            214, 234, 113, 207, 17, 38, 9, 219, 22, 150, 10, 8, 75, 120, 193, 251,
                            90, 171, 18, 126, 145, 255, 31, 224, 226, 106, 61, 16, 39, 203, 47,
                            177, 13, 6, 236, 120, 110, 184, 74, 57, 143, 4, 201, 255, 32, 69, 239,
                            191, 235, 0, 235, 231, 131, 139, 66, 134, 12, 25, 214, 228, 128, 61,
                            203, 193, 83, 42, 86, 53, 15, 28, 88, 23, 51, 12, 198, 201, 14, 196,
                            153, 153, 199, 171, 93, 96, 97, 200, 136, 119, 20, 128, 117, 121, 224,
                            227, 22, 240, 102, 35, 120, 183, 9, 204, 85, 229, 103, 173, 127, 26,
                            166, 171, 112, 87, 145, 25, 127, 51, 156, 184, 3, 70, 194, 224, 143,
                            231, 46, 182, 54, 128, 71, 247, 130, 79, 134, 192, 241, 65, 240, 245,
                            85, 23, 195, 179, 193, 137, 81, 25, 250, 36, 80, 183, 249, 89, 134,
                            160, 58, 235, 104, 48, 227, 204, 60, 30, 141, 233, 48, 190, 197, 247,
                            171, 111, 63, 248, 252, 33, 232, 138, 205, 167, 191, 235, 96, 174, 201,
                            208, 233, 68, 221, 231, 231, 26, 130, 234, 188, 173, 217, 20, 202, 206,
                            140, 231, 209, 186, 93, 162, 169, 112, 84, 236, 211, 169, 78, 79, 122,
                            162, 88, 174, 152, 225, 84, 6, 175, 15, 198, 13, 29, 177, 131, 95, 183,
                            162, 101, 139, 33, 61, 117, 134, 225, 172, 78, 85, 235, 1, 83, 143,
                            218, 76, 222, 29, 239, 52, 212, 239, 224, 102, 64, 235, 161, 147, 234,
                            88, 157, 216, 18, 184, 37, 14, 56, 239, 233, 238, 250, 235, 13, 150,
                            186, 245, 104, 45, 193, 49, 76, 220, 172, 137, 174, 69, 210, 177, 152,
                            153, 29, 67, 177, 247, 136, 12, 22, 74, 212, 107, 114, 150, 58, 131,
                            140, 189, 58, 36, 232, 11, 18, 234, 13, 46, 25, 90, 234, 222, 142, 40,
                            239, 151, 161, 175, 78, 85, 15, 26, 19, 43, 205, 224, 196, 254, 3, 96,
                            98, 137, 126, 57, 125, 102, 30, 104, 51, 157, 249, 182, 25, 67, 111,
                            181, 226, 90, 211, 161, 99, 230, 201, 113, 12, 177, 138, 115, 208, 203,
                            124, 239, 179, 168, 40, 226, 121, 193, 37, 145, 55, 209, 56, 199, 247,
                            146, 33, 244, 51, 136, 125, 198, 16, 91, 77, 33, 212, 210, 135, 50,
                            122, 69, 18, 202, 117, 128, 126, 186, 17, 166, 7, 243, 156, 47, 89,
                            202, 160, 245, 148, 5, 89, 95, 149, 168, 149, 104, 152, 11, 194, 95,
                            20, 151, 50, 197, 61, 46, 113, 175, 196, 221, 102, 11, 208, 28, 29, 19,
                            80, 163, 245, 149, 49, 202, 53, 175, 165, 133, 116, 153, 230, 58, 91,
                            210, 204, 160, 107, 148, 57, 41, 159, 88, 144, 246, 81, 134, 135, 185,
                            56, 227, 61, 51, 189, 111, 89, 152, 249, 65, 243, 61, 162, 195, 220, 2,
                            52, 48, 17, 21, 50, 46, 215, 88, 246, 11, 31, 54, 202, 48, 153, 54, 34,
                            68, 37, 150, 113, 192, 58, 206, 17, 251, 28, 175, 88, 103, 217, 133,
                            157, 188, 101, 13, 112, 208, 62, 193, 7, 174, 203, 60, 102, 117, 178,
                            29, 141, 81, 195, 6, 84, 178, 2, 197, 49, 138, 126, 197, 27, 176, 66,
                            95, 226, 99, 38, 210, 136, 22, 212, 243, 188, 246, 105, 216, 58, 195,
                            219, 184, 196, 94, 117, 51, 138, 11, 28, 178, 78, 235, 47, 237, 34,
                            187, 209, 206, 67, 234, 210, 185, 232, 77, 88, 207, 74, 109, 69, 37,
                            74, 98, 152, 184, 90, 227, 38, 172, 225, 92, 164, 242, 39, 177, 98,
                            209, 227, 149, 197, 218, 189, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96,
                            130
                        ]
                        .to_vec()
                    )
                ]),
            })
        );

        Ok(())
    }
}
