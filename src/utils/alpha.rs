use image::RgbaImage;

use crate::features::{EarsFeatures, data::leg::LegMode};

#[derive(Clone, Copy)]
pub(crate) struct SkinRegion {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    kind: SkinRegionKind,
}

impl SkinRegion {
    const fn new(x: u32, y: u32, width: u32, height: u32, kind: SkinRegionKind) -> Self {
        Self {
            x,
            y,
            width,
            height,
            kind,
        }
    }

    const fn displaced_by(self, leg_mode: LegMode) -> bool {
        match self.kind {
            SkinRegionKind::LegSideTopHalf
            | SkinRegionKind::LegTop
            | SkinRegionKind::PantSideTopHalf
            | SkinRegionKind::PantTop => matches!(leg_mode, LegMode::DigitigradeFull),
            SkinRegionKind::LegSideBottomHalf
            | SkinRegionKind::LegBottom
            | SkinRegionKind::PantSideBottomHalf
            | SkinRegionKind::PantBottom => {
                !matches!(leg_mode, LegMode::Plantigrade)
            }
            _ => false,
        }
    }

    const fn force_opaque(self) -> bool {
        matches!(
            self.kind,
            SkinRegionKind::Head
                | SkinRegionKind::Face
                | SkinRegionKind::Arm
                | SkinRegionKind::Torso
                | SkinRegionKind::LegSideTopHalf
                | SkinRegionKind::LegSideBottomHalf
                | SkinRegionKind::LegTop
                | SkinRegionKind::LegBottom
        )
    }
}

#[derive(Clone, Copy)]
enum SkinRegionKind {
    Head,
    Face,
    Hat,
    Arm,
    Sleeve,
    Torso,
    Jacket,
    LegSideTopHalf,
    LegSideBottomHalf,
    LegTop,
    LegBottom,
    PantSideTopHalf,
    PantSideBottomHalf,
    PantTop,
    PantBottom,
}

use SkinRegionKind::*;

const SKIN_REGIONS: &[SkinRegion] = &[
    SkinRegion::new(0, 8, 8, 8, Head),
    SkinRegion::new(8, 8, 8, 8, Face),
    SkinRegion::new(16, 8, 16, 8, Head),
    SkinRegion::new(8, 0, 16, 8, Head),
    SkinRegion::new(32, 8, 32, 8, Hat),
    SkinRegion::new(40, 0, 16, 8, Hat),
    SkinRegion::new(0, 20, 16, 6, LegSideTopHalf),
    SkinRegion::new(0, 26, 16, 6, LegSideBottomHalf),
    SkinRegion::new(4, 16, 4, 4, LegTop),
    SkinRegion::new(8, 16, 4, 4, LegBottom),
    SkinRegion::new(16, 20, 24, 12, Torso),
    SkinRegion::new(20, 16, 16, 4, Torso),
    SkinRegion::new(40, 20, 16, 12, Arm),
    SkinRegion::new(44, 16, 8, 4, Arm),
    SkinRegion::new(0, 36, 16, 6, PantSideTopHalf),
    SkinRegion::new(0, 42, 16, 6, PantSideBottomHalf),
    SkinRegion::new(4, 32, 4, 4, PantTop),
    SkinRegion::new(8, 32, 4, 4, PantBottom),
    SkinRegion::new(16, 36, 24, 12, Jacket),
    SkinRegion::new(20, 32, 16, 4, Jacket),
    SkinRegion::new(40, 36, 16, 12, Sleeve),
    SkinRegion::new(44, 32, 8, 4, Sleeve),
    SkinRegion::new(0, 52, 16, 6, PantSideTopHalf),
    SkinRegion::new(0, 58, 16, 6, PantSideBottomHalf),
    SkinRegion::new(4, 48, 4, 4, PantTop),
    SkinRegion::new(8, 48, 4, 4, PantBottom),
    SkinRegion::new(16, 52, 16, 6, LegSideTopHalf),
    SkinRegion::new(16, 58, 16, 6, LegSideBottomHalf),
    SkinRegion::new(20, 48, 4, 4, LegTop),
    SkinRegion::new(24, 48, 4, 4, LegBottom),
    SkinRegion::new(32, 52, 16, 12, Arm),
    SkinRegion::new(36, 48, 8, 4, Arm),
    SkinRegion::new(48, 52, 16, 12, Sleeve),
    SkinRegion::new(52, 48, 8, 4, Sleeve),
];

pub(crate) fn displaced_regions(leg_mode: LegMode) -> impl Iterator<Item = SkinRegion> {
    SKIN_REGIONS
        .iter()
        .copied()
        .filter(move |region| region.displaced_by(leg_mode))
}

pub fn strip_alpha(image: &mut RgbaImage) {
    strip_alpha_for_features(image, None);
}

pub fn strip_alpha_for_features(image: &mut RgbaImage, features: Option<&EarsFeatures>) {
    let leg_mode = features.map_or(LegMode::Plantigrade, |features| features.leg_mode);
    let x_scale = image.width() as f32 / 64.0;
    let y_scale = image.height() as f32 / 64.0;

    for region in SKIN_REGIONS {
        if !region.force_opaque() || region.displaced_by(leg_mode) {
            continue;
        }
        let x1 = (region.x as f32 * x_scale) as u32;
        let y1 = (region.y as f32 * y_scale) as u32;
        let x2 = ((region.x + region.width) as f32 * x_scale) as u32;
        let y2 = ((region.y + region.height) as f32 * y_scale) as u32;
        for y in y1..y2 {
            for x in x1..x2 {
                if let Some(pixel) = image.get_pixel_mut_checked(x, y) {
                    pixel.0[3] = u8::MAX;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use image::RgbaImage;

    use super::strip_alpha_for_features;
    use crate::features::{EarsFeatures, data::leg::LegMode};
    use crate::utils::alpha::strip_alpha;

    #[test]
    fn alpha_stripper_works() {
        fn alpha_strip_works(original: &str, expected: &str) {
            let mut image = image::open(original).unwrap().to_rgba8();
            strip_alpha(&mut image);
            let expected_image = image::open(expected).unwrap().to_rgba8();

            for (x, y, pixel) in expected_image.enumerate_pixels() {
                let real_pixel = image.get_pixel(x, y).0;
                assert_eq!(pixel.0, real_pixel, "Pixel at ({}, {}) is different", x, y);
            }
        }

        alpha_strip_works(
            "test_images/ears_v0_sample1.png",
            "test_images/ears_v0_sample1.png",
        );
        alpha_strip_works(
            "test_images/ears_v1_nickac_sample.png",
            "test_images/ears_v1_nickac_alpha_stripped.png",
        );
        alpha_strip_works(
            "test_images/notch_upgraded.png",
            "test_images/notch_upgraded_alpha_stripped.png",
        );
        alpha_strip_works(
            "test_images/notch_upgraded_hd.png",
            "test_images/notch_upgraded_alpha_stripped_hd.png",
        );
    }

    #[test]
    fn alpha_stripping_keeps_only_digitigrade_leg_regions_transparent() {
        let mut partial = RgbaImage::new(64, 64);
        strip_alpha_for_features(
            &mut partial,
            Some(&EarsFeatures {
                leg_mode: LegMode::DigitigradePartial,
                ..EarsFeatures::default()
            }),
        );
        assert_eq!(partial.get_pixel(4, 16).0[3], 255);
        assert_eq!(partial.get_pixel(8, 16).0[3], 0);
        assert_eq!(partial.get_pixel(0, 20).0[3], 255);
        assert_eq!(partial.get_pixel(0, 26).0[3], 0);

        let mut full = RgbaImage::new(64, 64);
        strip_alpha_for_features(
            &mut full,
            Some(&EarsFeatures {
                leg_mode: LegMode::DigitigradeFull,
                ..EarsFeatures::default()
            }),
        );
        assert_eq!(full.get_pixel(4, 16).0[3], 0);
        assert_eq!(full.get_pixel(8, 16).0[3], 0);
        assert_eq!(full.get_pixel(0, 20).0[3], 0);
        assert_eq!(full.get_pixel(16, 20).0[3], 255);
    }
}
