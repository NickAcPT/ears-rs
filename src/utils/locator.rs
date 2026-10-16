use image::{RgbaImage, imageops::crop_imm};

pub const LOCATOR_ATLAS_WIDTH: u32 = 16;
pub const LOCATOR_ATLAS_HEIGHT: u32 = 12;

const LOCATOR_SPRITE_REGIONS: [(u32, u32, u32); 4] = [(0, 0, 9), (9, 0, 7), (9, 7, 5), (6, 9, 3)];

pub fn extract_locator_sprites(atlas: &RgbaImage) -> Option<[RgbaImage; 4]> {
    if atlas.width() != LOCATOR_ATLAS_WIDTH || atlas.height() != LOCATOR_ATLAS_HEIGHT {
        return None;
    }

    Some(LOCATOR_SPRITE_REGIONS.map(|(x, y, size)| crop_imm(atlas, x, y, size, size).to_image()))
}

#[cfg(test)]
mod tests {
    use image::Rgba;

    use super::*;

    #[test]
    fn locator_sprites_are_cropped_from_atlas_regions() {
        let atlas = RgbaImage::from_fn(LOCATOR_ATLAS_WIDTH, LOCATOR_ATLAS_HEIGHT, |x, y| {
            Rgba([x as u8, y as u8, 0, 255])
        });

        let sprites = extract_locator_sprites(&atlas).unwrap();

        for (sprite, (ox, oy, size)) in sprites.iter().zip(LOCATOR_SPRITE_REGIONS) {
            assert_eq!(sprite.dimensions(), (size, size));
            for (x, y, pixel) in sprite.enumerate_pixels() {
                assert_eq!(pixel, atlas.get_pixel(ox + x, oy + y));
            }
        }
    }

    #[test]
    fn locator_sprites_require_exact_atlas_size() {
        assert!(extract_locator_sprites(&RgbaImage::new(16, 16)).is_none());
        assert!(extract_locator_sprites(&RgbaImage::new(32, 24)).is_none());
    }
}
