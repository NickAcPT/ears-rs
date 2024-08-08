use image::{Rgb, Rgba, RgbaImage};

use crate::{parser::EarsParser, utils::errors::Result};

pub struct EarsEmissivePalette(pub Vec<Rgb<u8>>);

pub fn extract_emissive_palette(skin: &RgbaImage) -> Result<Option<EarsEmissivePalette>> {
    if EarsParser::parse(&skin)?.filter(|f| f.emissive).is_none() {
        return Ok(None);
    }

    let mut emissive_palette = vec![];

    for x in 52..56 {
        for y in 32..36 {
            let color = skin.get_pixel(x, y);
            if color.0[3] /* alpha */ > 0 {
                emissive_palette.push(Rgb([color.0[0], color.0[1], color.0[2]]));
            }
        }
    }

    if emissive_palette.is_empty() {
        return Ok(None);
    }

    Ok(Some(EarsEmissivePalette(emissive_palette)))
}

pub fn apply_emissive_palette(
    texture: &mut RgbaImage,
    emissive_palette: &EarsEmissivePalette,
) -> Result<RgbaImage> {
    let mut emissive_texture = RgbaImage::new(texture.width(), texture.height());

    for (x, y, pixel) in texture.enumerate_pixels_mut() {
        let [r, g, b, a] = pixel.0;

        if a > 0 && emissive_palette.0.contains(&Rgb([r, g, b])) {
            emissive_texture.put_pixel(x, y, Rgba([r, g, b, 255]));
            pixel.0 = [0, 0, 0, 0];
        }
    }

    Ok(emissive_texture)
}

#[cfg(test)]
mod tests {
    use crate::utils::{apply_emissive_palette, errors::Result, extract_emissive_palette};

    #[test]
    fn emissive_texture_works() -> Result<()> {
        fn emissive_works(original: &str, expected_remainder: &str, expected: &str) -> Result<()> {
            let image = image::open(original).unwrap();
            let mut image = image.to_rgba8();

            let palette = extract_emissive_palette(&image)?.unwrap();

            let remainder_emissive = apply_emissive_palette(&mut image, &palette)?;
            let expected_image = image::open(expected).unwrap().to_rgba8();
            let expected_remainder_image = image::open(expected_remainder).unwrap().to_rgba8();

            image
                .enumerate_pixels()
                .zip(expected_image.enumerate_pixels())
                .for_each(|((x, y, pixel), (_, _, expected_pixel))| {
                    assert_eq!(pixel, expected_pixel, "Pixel at ({}, {})", x, y);
                });

            remainder_emissive
                .enumerate_pixels()
                .zip(expected_remainder_image.enumerate_pixels())
                .for_each(|((x, y, pixel), (_, _, expected_pixel))| {
                    assert_eq!(pixel, expected_pixel, "Pixel at ({}, {})", x, y);
                });

            Ok(())
        }

        emissive_works(
            "test_images/emissive-before.png",
            "test_images/emissive-remainder.png",
            "test_images/emissive-after.png",
        )?;

        Ok(())
    }
}
