macro_rules! read_magic_pixel {
     ($image: expr, $idx: literal) => {
         {
            use crate::parser::utils::to_argb_hex;

            to_argb_hex($image.get_pixel($idx % 4, 32 + ($idx / 4)))
         }
     };

     ($image: expr, $idx: literal, $default: expr, $($magic_pixel:pat => $result: expr),+) => {
         read_magic_pixel!($image, $idx, $default, true, $($magic_pixel => $result),+).unwrap()
     };

     ($image: expr, $idx: literal, $default: expr, $relevant: expr, $($magic_pixel:pat => $result: expr),+) => {
        {
            use crate::parser::utils::to_argb_hex;

            let pixel = to_argb_hex($image.get_pixel($idx % 4, 32 + ($idx / 4)));
            let magic_pixel = MagicPixelsV0::get_by_argb_hex(pixel);

            if $relevant {
                Some(match magic_pixel {
                    $($magic_pixel => $result,)+
                    _ => $default
                })
            } else {
                None
            }
        }
     };
 }

pub(crate) use read_magic_pixel;
