macro_rules! read_magic_pixel {
    ($image: expr, $idx: literal) => {
        {
           use crate::parser::utils::to_argb_hex;
           use crate::utils::errors::EarsError;

           $image.get_pixel_checked($idx % 4, 32 + ($idx / 4)).ok_or_else(|| EarsError::InvalidMagicPixelLocation($idx)).map(|p| to_argb_hex(&p))
        }
    };

    ($image: expr, $idx: literal, $default: expr, $($magic_pixel:pat => $result: expr),+) => {
        read_magic_pixel!($image, $idx, $default, true, $($magic_pixel => $result),+)?.ok_or_else(|| EarsError::InvalidMagicPixelLocation($idx))
    };

    ($image: expr, $idx: literal, $default: expr, $relevant: expr, $($magic_pixel:pat => $result: expr),+) => {
        {
            use crate::parser::utils::to_argb_hex;
            use crate::utils::errors::EarsError;

            let pixel = to_argb_hex($image.get_pixel_checked($idx % 4, 32 + ($idx / 4)).ok_or_else(|| EarsError::InvalidMagicPixelLocation($idx))?);
            let magic_pixel = MagicPixelsV0::get_by_argb_hex(pixel);

            Result::Ok(if $relevant {
                Some(match magic_pixel {
                    $($magic_pixel => $result,)+
                    _ => $default
                })
            } else {
                None
            })
        }
    };
}

use std::{collections::HashMap, hash::Hash};

use image::RgbaImage;
pub(crate) use read_magic_pixel;

use crate::{
    parser::{utils::from_argb_hex, v0::magic_pixels::MagicPixelsV0},
    utils::errors::{EarsError, Result},
};

pub(crate) fn write_magic_pixel<K: Eq + Hash>(
    image: &mut RgbaImage,
    idx: u32,
    value: K,
    value_map: impl Into<HashMap<K, MagicPixelsV0>>,
) -> Result<()> {
    let map: HashMap<K, MagicPixelsV0> = value_map.into();

    let magic_pixel = *map.get(&value).unwrap_or(&MagicPixelsV0::Unknown);

    write_raw_magic_pixel(image, idx, magic_pixel.get_hex())
}

pub(crate) fn write_raw_magic_pixel(image: &mut RgbaImage, idx: u32, value: u32) -> Result<()> {
    let magic_pixel = from_argb_hex(value);

    let dst_pixel = image
        .get_pixel_mut_checked(idx % 4, 32 + (idx / 4))
        .ok_or_else(|| EarsError::InvalidMagicPixelLocation(idx))?;

    dst_pixel.0 = magic_pixel.0;

    Ok(())
}
