use image::{Rgb, Rgba, RgbaImage};

use crate::{parser::EarsParser, utils::errors::Result};

#[derive(Debug, Clone, PartialEq)]
pub struct EarsEmissivePalette(pub Vec<Rgb<u8>>);

pub const MAX_EMISSIVE_COLORS: usize = 16;

impl From<Vec<Rgb<u8>>> for EarsEmissivePalette {
    fn from(palette: Vec<Rgb<u8>>) -> Self {
        Self(palette)
    }
}

impl From<EarsEmissivePalette> for Vec<Rgb<u8>> {
    fn from(palette: EarsEmissivePalette) -> Self {
        palette.0
    }
}

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

pub fn write_emissive_palette(
    skin: &mut RgbaImage,
    emissive_palette: &EarsEmissivePalette,
) -> Result<()> {
    let pixels: &[Rgb<u8>] = &emissive_palette.0;

    let mut idx = 0;
    for x in 52..56 {
        for y in 32..36 {
            if let Some(px) = skin.get_pixel_mut_checked(x, y) {
                let pixel = pixels
                    .get(idx)
                    .map_or(Rgba([0, 0, 0, 0]), |&Rgb([r, g, b])| Rgba([r, g, b, 255]));

                *px = pixel;
            }

            idx += 1;
        }
    }

    Ok(())
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
    use image::RgbaImage;

    use crate::{
        features::EarsFeatures,
        parser::{v0::writer::EarsWriterV0, EarsFeaturesWriter},
        utils::{
            apply_emissive_palette, errors::Result, extract_emissive_palette,
            write_emissive_palette,
        },
    };

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

    #[test]
    fn emissive_writing_works() -> Result<()> {
        let original = image::open("test_images/emissive-before.png")
            .unwrap()
            .to_rgba8();
        let palette = extract_emissive_palette(&original)?.unwrap();

        let mut image = RgbaImage::new(64, 64);

        EarsWriterV0::write(
            &mut image,
            &EarsFeatures {
                emissive: true,
                ..Default::default()
            },
        )?;

        write_emissive_palette(&mut image, &palette)?;

        let written_palette = extract_emissive_palette(&image)?;

        assert_eq!(Some(palette), written_palette);

        Ok(())
    }
}
