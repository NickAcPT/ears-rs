use image::RgbaImage;

use crate::features::{EarsFeatures, data::leg::LegMode};

#[derive(Clone, Copy)]
pub(crate) struct Rectangle {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
    pub force_opaque: bool,
}

const fn rectangle(x: u32, y: u32, width: u32, height: u32, force_opaque: bool) -> Rectangle {
    Rectangle {
        x1: x,
        y1: y,
        x2: x + width,
        y2: y + height,
        force_opaque,
    }
}

pub(crate) const FORCED_OPAQUE_REGIONS: &[Rectangle] = &[
    rectangle(8, 0, 16, 8, false),
    rectangle(0, 8, 32, 8, false),
    rectangle(4, 16, 8, 4, false),
    rectangle(20, 16, 16, 4, false),
    rectangle(44, 16, 8, 4, false),
    rectangle(0, 20, 56, 12, false),
    rectangle(20, 48, 8, 4, false),
    rectangle(36, 48, 8, 4, false),
    rectangle(16, 52, 32, 12, false),
];

pub(crate) const LEG_BOTTOM_HALF_REGIONS: &[Rectangle] = &[
    rectangle(24, 48, 4, 4, true),  // left leg bottom
    rectangle(16, 58, 16, 6, true), // left leg
    rectangle(8, 16, 4, 4, true),   // right leg bottom
    rectangle(0, 26, 16, 6, true),  // right leg
    rectangle(8, 32, 4, 4, false),  // right leg pant bottom
    rectangle(0, 42, 16, 6, false), // right leg pant
    rectangle(8, 48, 4, 4, false),  // left leg pant bottom
    rectangle(0, 58, 16, 6, false), // left leg pant
];

/// A list of the rectangles that are displaced if full digitigrade legs are enabled.
pub(crate) const LEG_REGIONS: &[Rectangle] = &[
    rectangle(20, 48, 8, 4, true),   // left leg pole
    rectangle(16, 52, 16, 12, true), // left leg
    rectangle(4, 16, 8, 4, true),    // right leg pole
    rectangle(0, 20, 16, 12, true),  // right leg
    rectangle(4, 32, 8, 4, false),   // right leg pant pole
    rectangle(0, 36, 16, 12, false), // right leg pant
    rectangle(4, 48, 8, 4, false),   // left leg pant pole
    rectangle(0, 52, 16, 12, false), // left leg pant
];

const FORCED_OPAQUE_REGIONS_WITHOUT_LEG_BOTTOM_REGIONS: &[Rectangle] = &[
    rectangle(8, 0, 16, 8, false),
    rectangle(0, 8, 32, 8, false),
    rectangle(4, 16, 4, 4, false),
    rectangle(20, 16, 16, 4, false),
    rectangle(44, 16, 8, 4, false),
    rectangle(0, 20, 16, 18, false),
    rectangle(16, 20, 40, 12, false),
    rectangle(20, 48, 4, 4, false),
    rectangle(36, 48, 8, 4, false),
    rectangle(16, 52, 16, 6, false),
    rectangle(32, 52, 16, 12, false),
];

const FORCED_OPAQUE_REGIONS_WITHOUT_LEG_REGIONS: &[Rectangle] = &[
    rectangle(8, 0, 16, 8, false),
    rectangle(0, 8, 32, 8, false),
    rectangle(4, 16, 4, 4, false),
    rectangle(20, 16, 16, 4, false),
    rectangle(44, 16, 8, 4, false),
    rectangle(16, 20, 40, 12, false),
    rectangle(20, 48, 4, 4, false),
    rectangle(36, 48, 8, 4, false),
    rectangle(32, 52, 16, 12, false),
];

pub fn strip_alpha(image: &mut RgbaImage) {
    strip_alpha_for_features(image, None);
}

pub fn strip_alpha_for_features(image: &mut RgbaImage, features: Option<&EarsFeatures>) {
    let leg_mode = features.map(|features| features.leg_mode);
    let regions = match leg_mode {
        Some(LegMode::DigitigradePartial) => FORCED_OPAQUE_REGIONS_WITHOUT_LEG_BOTTOM_REGIONS,
        Some(LegMode::DigitigradeFull) => FORCED_OPAQUE_REGIONS_WITHOUT_LEG_REGIONS,
        _ => FORCED_OPAQUE_REGIONS,
    };
    strip_alpha_regions(image, regions);
}

fn strip_alpha_regions(image: &mut RgbaImage, regions: &[Rectangle]) {
    let x_scale = image.width() as f32 / 64.0;
    let y_scale = image.height() as f32 / 64.0;
    for region in regions {
        let x1 = (region.x1 as f32 * x_scale) as u32;
        let y1 = (region.y1 as f32 * y_scale) as u32;
        let x2 = (region.x2 as f32 * x_scale) as u32;
        let y2 = (region.y2 as f32 * y_scale) as u32;
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
    use crate::utils::alpha::{strip_alpha};

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
}
