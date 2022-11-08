macro_rules! read_magic_pixel {
     ($image: expr, $idx: literal) => {
         {
            use crate::parser::utils::to_argb_hex;
            use crate::utils::errors::EarsError;

            $image.get_pixel_checked($idx % 4, 32 + ($idx / 4)).ok_or_else(|| EarsError::InvalidMagicPixelLocation($idx)).map(|p| to_argb_hex(p))
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

pub(crate) use read_magic_pixel;
