use image::RgbaImage;

use crate::{
    features::{EarsFeatures, data::leg::LegMode},
    utils::alpha::{LEG_BOTTOM_HALF_REGIONS, LEG_REGIONS, Rectangle},
};

pub fn apply_erase_displaced_regions(
    image: &mut RgbaImage,
    features: &EarsFeatures,
) -> crate::utils::errors::Result<()> {
    let regions = match features.leg_mode {
        LegMode::Plantigrade => return Ok(()),
        LegMode::DigitigradePartial => LEG_BOTTOM_HALF_REGIONS,
        LegMode::DigitigradeFull => LEG_REGIONS,
    };
    for region in regions {
        for y in region.y1..region.y2 {
            for x in region.x1..region.x2 {
                if let Some(pixel) = image.get_pixel_mut_checked(x, y) {
                    *pixel = image::Rgba([0, 0, 0, 0]);
                }
            }
        }
    }
    Ok(())
}

pub fn extract_displaced_skin(image: &RgbaImage, features: &EarsFeatures) -> Option<RgbaImage> {
    let regions = match features.leg_mode {
        LegMode::Plantigrade => return None,
        LegMode::DigitigradePartial => LEG_BOTTOM_HALF_REGIONS,
        LegMode::DigitigradeFull => LEG_REGIONS,
    };

    Some(copy_displaced_regions(image, regions))
}

fn copy_displaced_regions(image: &RgbaImage, regions: &[Rectangle]) -> RgbaImage {
    let mut displaced = RgbaImage::new(64, 64);
    for region in regions {
        for y in region.y1..region.y2 {
            for x in region.x1..region.x2 {
                let Some(mut pixel) = image.get_pixel_checked(x, y).copied() else {
                    continue;
                };
                if region.force_opaque {
                    pixel.0[3] = u8::MAX;
                }
                displaced.put_pixel(x, y, pixel);
            }
        }
    }
    displaced
}

fn swap_jacket_back_and_tail(image: &mut RgbaImage) {
    const TAIL_X: u32 = 56;
    const TAIL_Y: u32 = 16;
    const JACKET_X: u32 = 32;
    const JACKET_Y: u32 = 36;
    const WIDTH: u32 = 8;
    const HEIGHT: u32 = 12;

    let tail = image::imageops::crop_imm(&*image, TAIL_X, TAIL_Y, WIDTH, HEIGHT).to_image();
    let jacket = image::imageops::crop_imm(&*image, JACKET_X, JACKET_Y, WIDTH, HEIGHT).to_image();
    let flipped_jacket = image::imageops::flip_vertical(&jacket);

    image::imageops::replace(image, &flipped_jacket, i64::from(TAIL_X), i64::from(TAIL_Y));
    image::imageops::replace(image, &tail, i64::from(JACKET_X), i64::from(JACKET_Y));
}

#[cfg(test)]
mod tests {
    use image::Rgba;

    use super::*;
    use crate::{
        features::EarsFeatures,
        parser::{EarsFeaturesWriter, EarsParser, v0::writer::EarsWriterV0},
        utils::{apply_emissive_palette, extract_emissive_palette, strip_alpha_for_features},
    };

    #[test]
    fn swap_jacket_back_and_tail_exchanges_flipped_jacket_texture() {
        let mut image = RgbaImage::new(64, 64);
        image.put_pixel(56, 16, Rgba([1, 2, 3, 4]));
        image.put_pixel(32, 47, Rgba([5, 6, 7, 8]));

        swap_jacket_back_and_tail(&mut image);

        assert_eq!(*image.get_pixel(32, 36), Rgba([1, 2, 3, 4]));
        assert_eq!(*image.get_pixel(56, 16), Rgba([5, 6, 7, 8]));
    }

    #[test]
    fn swap_matches_ears_tail_swap_fixtures() {
        for (original, swapped) in [
            (
                "test_images/ears_v0_tail_swap_original.png",
                "test_images/ears_v0_tail_swap_swapped.png",
            ),
            (
                "test_images/ears_v1_tail_swap_original.png",
                "test_images/ears_v1_tail_swap_swapped.png",
            ),
        ] {
            let mut actual = image::open(original).unwrap().to_rgba8();
            let expected = image::open(swapped).unwrap().to_rgba8();

            swap_jacket_back_and_tail(&mut actual);
            assert_image_eq(&actual, &expected, swapped);
        }
    }

    #[test]
    fn extract_displaced_skin_preserves_partial_digitigrade_source_pixels() {
        let mut image = digitigrade_skin(LegMode::DigitigradePartial);
        image.put_pixel(24, 48, Rgba([1, 2, 3, 4]));
        image.put_pixel(8, 32, Rgba([5, 6, 7, 8]));
        let original = image.clone();
        let features = EarsParser::parse(&image).unwrap().unwrap();

        let displaced = extract_displaced_skin(&image, &features).unwrap();

        assert_eq!(image, original);
        assert_eq!(*displaced.get_pixel(24, 48), Rgba([1, 2, 3, 255]));
        assert_eq!(*displaced.get_pixel(8, 32), Rgba([5, 6, 7, 8]));
    }

    #[test]
    fn extract_displaced_skin_preserves_full_digitigrade_source_pixels() {
        let mut image = digitigrade_skin(LegMode::DigitigradeFull);
        image.put_pixel(20, 48, Rgba([1, 2, 3, 4]));
        image.put_pixel(4, 32, Rgba([5, 6, 7, 8]));
        let original = image.clone();
        let features = EarsParser::parse(&image).unwrap().unwrap();

        let displaced = extract_displaced_skin(&image, &features).unwrap();

        assert_eq!(image, original);
        assert_eq!(*displaced.get_pixel(20, 48), Rgba([1, 2, 3, 255]));
        assert_eq!(*displaced.get_pixel(4, 32), Rgba([5, 6, 7, 8]));
    }

    #[test]
    fn extraction_matches_ears_digitigrade_displacement_fixtures() {
        for (original, displaced, leg_mode) in [
            (
                "test_images/ears_v0_digitigrade_partial_original.png",
                "test_images/ears_v0_digitigrade_partial_displaced.png",
                LegMode::DigitigradePartial,
            ),
            (
                "test_images/ears_v0_digitigrade_full_original.png",
                "test_images/ears_v0_digitigrade_full_displaced.png",
                LegMode::DigitigradeFull,
            ),
            (
                "test_images/ears_v1_digitigrade_partial_original.png",
                "test_images/ears_v1_digitigrade_partial_displaced.png",
                LegMode::DigitigradePartial,
            ),
            (
                "test_images/ears_v1_digitigrade_full_original.png",
                "test_images/ears_v1_digitigrade_full_displaced.png",
                LegMode::DigitigradeFull,
            ),
        ] {
            let image = image::open(original).unwrap().to_rgba8();
            let features = EarsParser::parse(&image).unwrap().unwrap();
            let actual = extract_displaced_skin(&image, &features).unwrap();
            let expected = image::open(displaced).unwrap().to_rgba8();

            assert_eq!(features.leg_mode, leg_mode);
            assert_image_eq(&actual, &expected, displaced);
        }
    }

    #[test]
    fn extraction_matches_ears_emissive_digitigrade_displacement_fixtures() {
        for (original, displaced, emissive_displaced, leg_mode) in [
            (
                "test_images/ears_v0_digitigrade_partial_emissive_original.png",
                "test_images/ears_v0_digitigrade_partial_emissive_displaced.png",
                "test_images/ears_v0_digitigrade_partial_emissive_emissive_displaced.png",
                LegMode::DigitigradePartial,
            ),
            (
                "test_images/ears_v0_digitigrade_full_emissive_original.png",
                "test_images/ears_v0_digitigrade_full_emissive_displaced.png",
                "test_images/ears_v0_digitigrade_full_emissive_emissive_displaced.png",
                LegMode::DigitigradeFull,
            ),
            (
                "test_images/ears_v1_digitigrade_partial_emissive_original.png",
                "test_images/ears_v1_digitigrade_partial_emissive_displaced.png",
                "test_images/ears_v1_digitigrade_partial_emissive_emissive_displaced.png",
                LegMode::DigitigradePartial,
            ),
            (
                "test_images/ears_v1_digitigrade_full_emissive_original.png",
                "test_images/ears_v1_digitigrade_full_emissive_displaced.png",
                "test_images/ears_v1_digitigrade_full_emissive_emissive_displaced.png",
                LegMode::DigitigradeFull,
            ),
        ] {
            let image = image::open(original).unwrap().to_rgba8();
            let palette = extract_emissive_palette(&image).unwrap().unwrap();
            let features = EarsParser::parse(&image).unwrap().unwrap();
            let mut displaced_image = extract_displaced_skin(&image, &features).unwrap();

            assert_eq!(features.leg_mode, leg_mode);
            assert_image_eq(
                &displaced_image,
                &image::open(displaced).unwrap().to_rgba8(),
                displaced,
            );
            let actual_emissive = apply_emissive_palette(&mut displaced_image, &palette).unwrap();
            assert_image_eq(
                &actual_emissive,
                &image::open(emissive_displaced).unwrap().to_rgba8(),
                emissive_displaced,
            );
        }
    }

    #[test]
    fn contextual_alpha_stripping_preserves_displaced_leg_pixels() {
        let mut image = RgbaImage::new(64, 64);
        let features = EarsFeatures {
            leg_mode: LegMode::DigitigradeFull,
            ..EarsFeatures::default()
        };

        strip_alpha_for_features(&mut image, Some(&features));

        assert_eq!(image.get_pixel(4, 20).0[3], 0);
        assert_eq!(image.get_pixel(20, 16).0[3], 255);
    }

    fn assert_image_eq(actual: &RgbaImage, expected: &RgbaImage, expected_path: &str) {
        for ((x, y, actual_pixel), (_, _, expected_pixel)) in
            actual.enumerate_pixels().zip(expected.enumerate_pixels())
        {
            assert_eq!(
                actual_pixel, expected_pixel,
                "did not match {expected_path} at ({x}, {y})"
            );
        }
    }

    fn digitigrade_skin(leg_mode: LegMode) -> RgbaImage {
        let mut image = RgbaImage::new(64, 64);
        let features = EarsFeatures {
            leg_mode,
            ..EarsFeatures::default()
        };
        EarsWriterV0::write(&mut image, &features).unwrap();
        image
    }
}
