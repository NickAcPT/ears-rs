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
    let mut tail = [Rgba([0, 0, 0, 0]); 8 * 12];
    for y in 0u32..12 {
        for x in 0u32..8 {
            let index = (y * 8 + x) as usize;
            tail[index] = *image.get_pixel(56 + x, 16 + y);
        }
    }
    for y in 0u32..12 {
        for x in 0u32..8 {
            let jacket = *image.get_pixel(32 + x, 36 + 11 - y);
            image.put_pixel(56 + x, 16 + y, jacket);
            image.put_pixel(32 + x, 36 + y, tail[(y * 8 + x) as usize]);
        }
    }
}

#[cfg(test)]
mod tests {
    use image::Rgba;

    use super::*;
    use crate::{
        features::{
            EarsFeatures,
            data::tail::{TailData, TailMode},
        },
        parser::{EarsFeaturesWriter, v0::writer::EarsWriterV0},
        utils::strip_alpha_for_features,
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
    fn preprocessing_swaps_tail_and_flipped_jacket_back() {
        let mut image = RgbaImage::new(64, 64);
        let features = EarsFeatures {
            tail: Some(TailData {
                mode: TailMode::Down,
                segments: 1,
                swap_jacket_back: true,
                ..TailData::default()
            }),
            ..EarsFeatures::default()
        };
        EarsWriterV0::write(&mut image, &features).unwrap();
        image.put_pixel(56, 16, Rgba([1, 2, 3, 4]));
        image.put_pixel(32, 47, Rgba([5, 6, 7, 8]));

        preprocess_skin(&mut image).unwrap();

        assert_eq!(*image.get_pixel(32, 36), Rgba([1, 2, 3, 4]));
        assert_eq!(*image.get_pixel(56, 16), Rgba([5, 6, 7, 8]));
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
