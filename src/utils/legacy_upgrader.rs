use image::{imageops, RgbaImage};
use itertools::Either;

fn check_has_transparency(image: &RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32) -> bool {
    let min_dy = y1.min(y2);
    let max_dy = y1.max(y2);
    let min_dx = x1.min(x2);
    let max_dx = x1.max(x2);

    for y in min_dy..max_dy {
        for x in min_dx..max_dx {
            if let Some(pixel) = image.get_pixel_checked(x, y) {
                if pixel.0[3] < 128 {
                    return true;
                }
            }
        }
    }

    false
}

fn set_area_transparent(image: &mut RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32) {
    let has_transparency = check_has_transparency(image, x1, y1, x2, y2);
    if has_transparency {
        return;
    }

    let min_dy = y1.min(y2);
    let max_dy = y1.max(y2);
    let min_dx = x1.min(x2);
    let max_dx = x1.max(x2);

    for y in min_dy..max_dy {
        for x in min_dx..max_dx {
            if let Some(pixel) = image.get_pixel_mut_checked(x, y) {
                pixel.0[3] = 0;
            }
        }
    }
}

fn copy_rect(
    image: &mut RgbaImage,
    d1: (u32, u32),
    d2: (u32, u32),
    s1: (u32, u32),
    s2: (u32, u32),
) -> Option<()> {
    let (dx1, dy1) = d1;
    let (dx2, dy2) = d2;
    let (sx1, sy1) = s1;
    let (sx2, sy2) = s2;

    let dy_range = if dy1 < dy2 {
        Either::Left(dy1..dy2)
    } else {
        Either::Right((dy2..dy1).rev())
    };

    let dx_range = if dx1 < dx2 {
        Either::Left(dx1..dx2)
    } else {
        Either::Right((dx2..dx1).rev())
    };

    let sy_range = if sy1 < sy2 {
        Either::Left(sy1..sy2)
    } else {
        Either::Right((sy2..sy1).rev())
    };

    let sx_range = if sx1 < sx2 {
        Either::Left(sx1..sx2)
    } else {
        Either::Right((sx2..sx1).rev())
    };

    for (dy, sy) in dy_range.zip(sy_range) {
        for (dx, sx) in dx_range.clone().zip(sx_range.clone()) {
            let pixel = image.get_pixel_checked(sx, sy)?;
            image.put_pixel(dx, dy, *pixel);
        }
    }

    Some(())
}

pub fn upgrade_skin_if_needed(image: RgbaImage) -> Option<RgbaImage> {
    if image.height() == 64 {
        Some(image)
    } else {
        let mut new_image = RgbaImage::new(64, 64);
        imageops::replace(&mut new_image, &image, 0, 0);

        copy_rect(&mut new_image, (24, 48), (20, 52), (4, 16), (8, 20))?;
        copy_rect(&mut new_image, (28, 48), (24, 52), (8, 16), (12, 20))?;
        copy_rect(&mut new_image, (20, 52), (16, 64), (8, 20), (12, 32))?;
        copy_rect(&mut new_image, (24, 52), (20, 64), (4, 20), (8, 32))?;
        copy_rect(&mut new_image, (28, 52), (24, 64), (0, 20), (4, 32))?;
        copy_rect(&mut new_image, (32, 52), (28, 64), (12, 20), (16, 32))?;
        copy_rect(&mut new_image, (40, 48), (36, 52), (44, 16), (48, 20))?;
        copy_rect(&mut new_image, (44, 48), (40, 52), (48, 16), (52, 20))?;
        copy_rect(&mut new_image, (36, 52), (32, 64), (48, 20), (52, 32))?;
        copy_rect(&mut new_image, (40, 52), (36, 64), (44, 20), (48, 32))?;
        copy_rect(&mut new_image, (44, 52), (40, 64), (40, 20), (44, 32))?;
        copy_rect(&mut new_image, (48, 52), (44, 64), (52, 20), (56, 32))?;

        set_area_transparent(&mut new_image, 32, 0, 64, 32);
        set_area_transparent(&mut new_image, 0, 32, 16, 48);
        set_area_transparent(&mut new_image, 16, 32, 40, 48);
        set_area_transparent(&mut new_image, 40, 32, 56, 48);
        set_area_transparent(&mut new_image, 0, 48, 16, 64);
        set_area_transparent(&mut new_image, 48, 48, 64, 64);

        Some(new_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrader_works() {
        macro_rules! upgrader_works {
            ($original: literal, $expected: literal) => {
                let image = image::open($original).unwrap();
                let image = image.to_rgba8();

                let new_image = upgrade_skin_if_needed(image).unwrap();
                let expected_image = image::open($expected).unwrap().to_rgba8();

                assert_eq!(new_image, expected_image);
            };
        }

        upgrader_works!(
            "test_images/notch_original.png",
            "test_images/notch_upgraded.png"
        );
        upgrader_works!(
            "test_images/mister_fix_original.png",
            "test_images/mister_fix_upgraded.png"
        );
    }
}
