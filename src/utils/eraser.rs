use image::RgbaImage;

use crate::alfalfa::utils::erase_utils::EraseRegionsProvider;
use crate::utils::errors::Result;

fn process_erase_regions(image: &mut RgbaImage) -> Result<()> {
    let alfalfa = crate::alfalfa::io::read_alfalfa(image)?;

    if let Some(alfalfa) = alfalfa {
        let regions = alfalfa.get_erase_regions()?;
        if let Some(regions) = regions {
            for region in regions {
                for x in region.x..region.x + region.width {
                    for y in region.y..region.y + region.height {
                        if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                            *pixel = image::Rgba([0, 0, 0, 0]);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::errors::Result;

    #[test]
    fn eraser_works() -> Result<()> {
        macro_rules! eraser_works {
            ($original: literal, $expected: literal) => {
                let image = image::open($original).unwrap();
                let mut image = image.to_rgba8();

                process_erase_regions(&mut image)?;
                let expected_image = image::open($expected).unwrap().to_rgba8();

                assert_eq!(image, expected_image);
            };
        }

        eraser_works!(
            "test_images/ears_v1_nickac_sample.png",
            "test_images/ears_v1_nickac_sample_erased.png"
        );

        Ok(())
    }
}