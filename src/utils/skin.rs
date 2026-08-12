use image::{Rgba, RgbaImage};

use crate::{
    alfalfa::{AlfalfaData, read_alfalfa},
    features::{EarsFeatures, data::leg::LegMode},
    parser::EarsParser,
    utils::{
        alpha::{FORCED_OPAQUE_REGIONS, LEG_BOTTOM_HALF_REGIONS, LEG_REGIONS, Rectangle},
        eraser::apply_erase_regions,
        errors::Result,
    },
};

/// The data retained while preprocessing a skin for rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct SkinPreprocessResult {
    /// Alfalfa data decoded before its backing pixels are made opaque or erased.
    pub alfalfa: Option<AlfalfaData>,
    /// Features parsed before digitigrade processing clears texture pixels.
    pub features: Option<EarsFeatures>,
    /// Leg texture pixels removed from the skin for digitigrade rendering.
    pub displaced_skin: Option<RgbaImage>,
}

pub fn preprocess_skin(image: &mut RgbaImage) -> Result<SkinPreprocessResult> {
    let alfalfa = read_alfalfa(image)?;
    if let Some(alfalfa) = alfalfa.as_ref() {
        if !alfalfa.is_empty() {
            force_opaque_regions(image, FORCED_OPAQUE_REGIONS);
        }
        apply_erase_regions(image, alfalfa)?;
    }

    let features = EarsParser::parse(image)?;
    let displaced_skin = features.as_ref().and_then(|features| {
        let regions = match features.leg_mode {
            LegMode::Plantigrade => return None,
            LegMode::DigitigradePartial => LEG_BOTTOM_HALF_REGIONS,
            LegMode::DigitigradeFull => LEG_REGIONS,
        };
        Some(displace_regions(image, regions))
    });

    if features
        .as_ref()
        .and_then(|features| features.tail.as_ref())
        .is_some_and(|tail| tail.swap_jacket_back)
    {
        swap_jacket_back_and_tail(image);
    }

    Ok(SkinPreprocessResult {
        alfalfa,
        features,
        displaced_skin,
    })
}

fn force_opaque_regions(image: &mut RgbaImage, regions: &[Rectangle]) {
    for region in regions {
        for y in region.y1..region.y2 {
            for x in region.x1..region.x2 {
                if let Some(pixel) = image.get_pixel_mut_checked(x, y) {
                    pixel.0[3] = u8::MAX;
                }
            }
        }
    }
}

fn displace_regions(image: &mut RgbaImage, regions: &[Rectangle]) -> RgbaImage {
    let mut displaced = RgbaImage::new(64, 64);
    for region in regions {
        for y in region.y1..region.y2 {
            for x in region.x1..region.x2 {
                let Some(source) = image.get_pixel_checked(x, y).copied() else {
                    continue;
                };
                let mut copied = source;
                if region.force_opaque {
                    copied.0[3] = u8::MAX;
                }
                displaced.put_pixel(x, y, copied);
                image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
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
        parser::{EarsFeaturesWriter, v0::writer::EarsWriterV0},
        utils::{apply_emissive_palette, extract_emissive_palette, strip_alpha_for_features},
    };

    #[test]
    fn preprocess_displaces_partial_digitigrade_regions() {
        let mut image = digitigrade_skin(LegMode::DigitigradePartial);
        image.put_pixel(24, 48, Rgba([1, 2, 3, 4]));
        image.put_pixel(8, 32, Rgba([5, 6, 7, 8]));

        let result = preprocess_skin(&mut image).unwrap();
        let displaced = result.displaced_skin.unwrap();

        assert_eq!(
            result.features.unwrap().leg_mode,
            LegMode::DigitigradePartial
        );
        assert_eq!(*image.get_pixel(24, 48), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(8, 32), Rgba([0, 0, 0, 0]));
        assert_eq!(*displaced.get_pixel(24, 48), Rgba([1, 2, 3, 255]));
        assert_eq!(*displaced.get_pixel(8, 32), Rgba([5, 6, 7, 8]));
    }

    #[test]
    fn preprocess_displaces_full_digitigrade_regions() {
        let mut image = digitigrade_skin(LegMode::DigitigradeFull);
        image.put_pixel(20, 48, Rgba([1, 2, 3, 4]));
        image.put_pixel(4, 32, Rgba([5, 6, 7, 8]));

        let result = preprocess_skin(&mut image).unwrap();
        let displaced = result.displaced_skin.unwrap();

        assert_eq!(result.features.unwrap().leg_mode, LegMode::DigitigradeFull);
        assert_eq!(*image.get_pixel(20, 48), Rgba([0, 0, 0, 0]));
        assert_eq!(*displaced.get_pixel(20, 48), Rgba([1, 2, 3, 255]));
        assert_eq!(*displaced.get_pixel(4, 32), Rgba([5, 6, 7, 8]));
    }

    #[test]
    fn preprocessing_matches_ears_tail_swap_fixtures() {
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

            let result = preprocess_skin(&mut actual).unwrap();
            assert!(
                result
                    .features
                    .as_ref()
                    .and_then(|features| features.tail.as_ref())
                    .is_some_and(|tail| tail.swap_jacket_back),
                "{original} did not parse its jacket swap flag"
            );

            assert_eq!(*expected.get_pixel(56, 16), Rgba([0, 0, 255, 255]));
            assert_eq!(*expected.get_pixel(63, 16), Rgba([255, 0, 255, 255]));
            assert_eq!(*expected.get_pixel(56, 27), Rgba([0, 255, 255, 255]));
            assert_eq!(*expected.get_pixel(32, 36), Rgba([0, 255, 0, 255]));
            assert_eq!(*expected.get_pixel(39, 47), Rgba([255, 255, 0, 255]));
            for ((x, y, actual_pixel), (_, _, expected_pixel)) in
                actual.enumerate_pixels().zip(expected.enumerate_pixels())
            {
                assert_eq!(
                    actual_pixel, expected_pixel,
                    "processed {original} did not match {swapped} at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn preprocessing_matches_ears_digitigrade_displacement_fixtures() {
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
            assert_ears_displacement(original, displaced, leg_mode);
        }
    }

    #[test]
    fn preprocessing_matches_ears_emissive_digitigrade_displacement_fixtures() {
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
            let mut original_image = image::open(original).unwrap().to_rgba8();
            let palette = extract_emissive_palette(&original_image).unwrap().unwrap();
            let result = preprocess_skin(&mut original_image).unwrap();
            let mut displaced_image = result.displaced_skin.unwrap();

            assert_eq!(result.features.unwrap().leg_mode, leg_mode);
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

    fn assert_ears_displacement(original: &str, displaced: &str, leg_mode: LegMode) {
        let before = image::open(original).unwrap().to_rgba8();
        let mut actual = before.clone();
        let result = preprocess_skin(&mut actual).unwrap();
        let expected = image::open(displaced).unwrap().to_rgba8();

        assert_eq!(result.features.unwrap().leg_mode, leg_mode);
        assert_image_eq(
            result.displaced_skin.as_ref().unwrap(),
            &expected,
            displaced,
        );
        for ((x, y, before_pixel), (_, _, actual_pixel)) in
            before.enumerate_pixels().zip(actual.enumerate_pixels())
        {
            if expected.get_pixel(x, y).0[3] > 0 {
                assert_eq!(
                    *actual_pixel,
                    Rgba([0, 0, 0, 0]),
                    "{original} retained displaced pixel at ({x}, {y})"
                );
            } else {
                assert_eq!(
                    *actual_pixel, *before_pixel,
                    "{original} changed non-displaced pixel at ({x}, {y})"
                );
            }
        }
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
