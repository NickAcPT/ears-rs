use image::RgbaImage;

use crate::{
    features::{
        data::{
            ear::{EarAnchor, EarMode},
            snout::SnoutData,
            tail::{TailData, TailMode},
            wing::{WingData, WingMode},
        },
        EarsFeatures,
    },
    parser::{
        v0::{
            macros::{read_magic_pixel, write_magic_pixel, write_raw_magic_pixel},
            magic_pixels::MagicPixelsV0,
        },
        EarsFeaturesWriter,
    },
    utils::errors::Result,
};

pub struct EarsWriterV0;

impl EarsFeaturesWriter for EarsWriterV0 {
    fn write(image: &mut RgbaImage, features: &EarsFeatures) -> Result<()> {
        // Ears V0 detection pixel
        write_raw_magic_pixel(image, 0, MagicPixelsV0::Blue.get_hex())?;

        write_magic_pixel(
            image,
            1,
            features.ear_mode,
            [
                (EarMode::Above, MagicPixelsV0::Blue),
                (EarMode::Sides, MagicPixelsV0::Green),
                (EarMode::Behind, MagicPixelsV0::Purple),
                (EarMode::Around, MagicPixelsV0::Cyan),
                (EarMode::Floppy, MagicPixelsV0::Orange),
                (EarMode::Cross, MagicPixelsV0::Pink),
                (EarMode::Out, MagicPixelsV0::Purple2),
                (EarMode::Tall, MagicPixelsV0::White),
                (EarMode::TallCross, MagicPixelsV0::Gray),
            ],
        )?;

        if features.ear_mode != EarMode::None && features.ear_mode != EarMode::Behind {
            write_magic_pixel(
                image,
                2,
                features.ear_anchor,
                [
                    (EarAnchor::Center, MagicPixelsV0::Blue),
                    (EarAnchor::Front, MagicPixelsV0::Green),
                    (EarAnchor::Back, MagicPixelsV0::Red),
                ],
            )?
        }

        write_magic_pixel(
            image,
            3,
            (features.claws, features.horn),
            [
                ((true, false), MagicPixelsV0::Green),
                ((false, true), MagicPixelsV0::Purple),
                ((true, true), MagicPixelsV0::Cyan),
            ],
        )?;

        if let Some(tail_data) = features.tail {
            write_tail_data(image, &tail_data)?;
        }

        let etc = if let Some(snout_data) = features.snout {
            write_snout_data(image, &snout_data)?
        } else {
            0
        };

        let chest_size = (features.chest_size * 128.0) as u32;

        let etc = etc | (chest_size << 16) | (features.cape_enabled as u32) << 4;

        write_raw_magic_pixel(image, 7, etc)?;

        if let Some(wing_data) = features.wing {
            write_wing_data(image, &wing_data)?;
        }

        write_raw_magic_pixel(
            image,
            10,
            if features.emissive {
                MagicPixelsV0::Orange
            } else {
                MagicPixelsV0::Blue
            }
            .get_hex(),
        )?;

        Ok(())
    }
}

fn write_tail_data(image: &mut RgbaImage, tail: &TailData) -> Result<()> {
    /// Convert a float from -1 to 1 to a pixel value, using an encoding that puts 0 at pixel value
    /// 0, thereby shifting all other possible values forward by one.
    ///
    /// This allows a black pixel to mean 0 for all of its values.
    ///
    /// Adapted from https://github.com/unascribed/Ears/blob/7bf6c80a7d14ab8425926551fb1f46aba390b720/common/src/main/java/com/unascribed/ears/common/EarsFeaturesParserV0.java#L226
    fn unit_to_px_val(j: f32) -> i32 {
        if j == 0.0 {
            return 0;
        }

        let j = j * 128.0;
        let mut j = j as i32;
        if j >= 0 {
            j -= 1;
        }
        if j < 0 {
            j += 1;
        }

        return j + 128;
    }
    write_magic_pixel(
        image,
        4,
        tail.mode,
        [
            (TailMode::Down, MagicPixelsV0::Blue),
            (TailMode::Back, MagicPixelsV0::Green),
            (TailMode::Up, MagicPixelsV0::Purple),
            (TailMode::Vertical, MagicPixelsV0::Orange),
        ],
    )?;

    if tail.mode == TailMode::None {
        return Ok(());
    }

    let [tail_bend0, tail_bend1, tail_bend2, tail_bend3] = tail.bends;

    let tail_bend0 = (255 - unit_to_px_val(tail_bend0 / 90.0) as i64) as i32;
    let tail_bend1 = unit_to_px_val(tail_bend1 / 90.0);
    let tail_bend2 = unit_to_px_val(tail_bend2 / 90.0);
    let tail_bend3 = unit_to_px_val(tail_bend3 / 90.0);

    let tail_bend = (((tail_bend0) as i64) << 24) as i32
        | (tail_bend1) << 16
        | (tail_bend2) << 8
        | (tail_bend3);

    write_raw_magic_pixel(image, 5, tail_bend as u32)?;

    Ok(())
}

fn write_snout_data(image: &mut RgbaImage, snout: &SnoutData) -> Result<u32> {
    let snout_depth = snout.depth.min(8);
    let mut snout_height = snout.height.min(4);
    let snout_width = snout.width.min(7);
    let snout_offset = snout.offset;

    if snout_offset > 8 - snout_height {
        snout_height = snout.offset + 8;
    }

    let snout = (snout_width as u32) << 16 | (snout_height as u32) << 8 | snout_depth as u32;
    let etc = (snout_offset as u32) << 8;

    write_raw_magic_pixel(image, 6, snout as u32)?;
    write_raw_magic_pixel(image, 7, etc as u32)?;

    Ok(etc as u32)
}

fn write_wing_data(image: &mut RgbaImage, wing: &WingData) -> Result<()> {
    write_magic_pixel(
        image,
        8,
        wing.mode,
        [
            (WingMode::SymmetricDual, MagicPixelsV0::Pink),
            (WingMode::SymmetricSingle, MagicPixelsV0::Green),
            (WingMode::AsymmetricL, MagicPixelsV0::Cyan),
            (WingMode::AsymmetricR, MagicPixelsV0::Orange),
        ],
    )?;

    if wing.mode == WingMode::None {
        return Ok(());
    }

    write_raw_magic_pixel(
        image,
        9,
        if wing.animated {
            MagicPixelsV0::Blue
        } else {
            MagicPixelsV0::Red
        }
        .get_hex(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self},
        path::PathBuf,
    };

    use image::RgbaImage;

    use crate::{
        features::{data::tail::TailMode, EarsFeatures},
        parser::{
            v0::{parser::EarsParserV0, writer::EarsWriterV0},
            EarsFeaturesParser, EarsFeaturesWriter,
        },
    };

    #[test]
    fn v0_tail_works() {
        let image = image::open("test_images/ears_v0_sample_tail_3_down_30_-30_60.png").unwrap();
        let image = image.to_rgba8();
        let features = EarsParserV0::parse(&image).unwrap().unwrap();
        let tail = features.tail.unwrap();

        assert_eq!(tail.mode, TailMode::Down);
        assert_eq!(tail.segments, 3);
        assert_eq!(tail.bends, [30.234375, -29.53125, 60.46875, 0.0]); // Rounding go BRRRRRR

        let mut new_img = RgbaImage::new(64, 64);

        EarsWriterV0::write(&mut new_img, &features).expect("Expected it to work!");

        let features2 = EarsParserV0::parse(&new_img).unwrap().unwrap();
        let tail2 = features2.tail.unwrap();

        assert_eq!(tail2.mode, tail.mode);
        assert_eq!(tail2.segments, tail.segments);
        assert_eq!(tail2.bends, tail.bends);
    }

    #[test]
    fn v0_works() {
        let image = image::open("test_images/ears_v0_sample_snout_5x2x6-0,6.png")
            .unwrap()
            .to_rgba8();

        let features = EarsParserV0::parse(&image).unwrap().unwrap();

        let mut image2 = RgbaImage::new(64, 64);
        EarsWriterV0::write(&mut image2, &features).expect("Expected it to work!");

        let features2 = EarsParserV0::parse(&image2).unwrap().unwrap();

        assert_eq!(
            features2, features,
            "Features are not equal: {:#?} != {:#?}",
            features, features2
        );
    }

    #[ignore]
    #[test]
    fn v0_massive_test_works() -> Result<(), Box<dyn std::error::Error>> {
        let _ = fs::create_dir("test_images/textures_ears");
        for entry in fs::read_dir("test_images/textures")? {
            if let Ok(entry) = entry {
                //eprintln!("Parsing image {:?}", entry.file_name());

                let image_bytes = fs::read(entry.path())?;

                if let Ok(image) =
                    image::load_from_memory_with_format(&image_bytes, image::ImageFormat::Png)
                {
                    let image = image.into_rgba8();

                    let features = EarsParserV0::parse(&image);

                    if let Ok(Some(features)) = features {
                        let mut path = PathBuf::new();

                        path.push("test_images/textures_ears");
                        path.push(entry.file_name().to_str().unwrap().to_owned() + ".png");

                        fs::copy(entry.path(), path)?;

                        let mut image2 = RgbaImage::new(64, 64);
                        EarsWriterV0::write(&mut image2, &features).expect("Expected it to work!");

                        let features2 = EarsParserV0::parse(&image2).unwrap().unwrap();

                        assert_eq!(
                            features2, features,
                            "Features are not equal: {:#?} != {:#?}",
                            features, features2
                        );

                        println!("Successfully parsed image {:?}", entry.file_name());
                    } else if let Err(e) = features {
                        eprintln!("Error parsing image: {:?}", e);
                    } else {
                        //eprintln!("No features found in image {:?}", entry.file_name());
                    }
                } else {
                    eprintln!("Could not load image {:?}", entry.file_name());
                }
            }
        }

        Ok(())
    }

    #[test]
    fn v0_roundtrip_works() {
        let image = image::open("test_images/ears_v0_sample_ear_out_front_claws_horn_tail_back_3_snout_4x3x4-0,2_wings_symmetric_dual_normal.png").unwrap();
        let image = image.to_rgba8();
        let features = EarsParserV0::parse(&image).unwrap().unwrap();

        let tail = features.tail.unwrap();
        let snout = features.snout.unwrap();
        let wing = features.wing.unwrap();

        let mut new_image = RgbaImage::new(64, 64);

        EarsWriterV0::write(&mut new_image, &features).expect("Expected it to work!");

        let features2 = EarsParserV0::parse(&new_image).unwrap().unwrap();
        let tail2 = features2.tail.unwrap();
        let snout2 = features2.snout.unwrap();
        let wing2 = features2.wing.unwrap();

        assert_eq!(features2.ear_mode, features.ear_mode);
        assert_eq!(features2.ear_anchor, features.ear_anchor);
        assert_eq!(features2.tail, features.tail);
        assert_eq!(features2.snout, features.snout);
        assert_eq!(features2.wing, features.wing);
        assert_eq!(features2.claws, features.claws);
        assert_eq!(features2.horn, features.horn);
        assert_eq!(features2.chest_size, features.chest_size);
        assert_eq!(features2.cape_enabled, features.cape_enabled);
        assert_eq!(features2.emissive, features.emissive);
        assert_eq!(features2.data_version, features.data_version);

        assert_eq!(tail2.mode, tail.mode);
        assert_eq!(tail2.segments, tail.segments);
        assert_eq!(tail2.bends, tail.bends);

        assert_eq!(snout2.offset, snout.offset);
        assert_eq!(snout2.width, snout.width);
        assert_eq!(snout2.height, snout.height);
        assert_eq!(snout2.depth, snout.depth);

        assert_eq!(wing2.mode, wing.mode);
        assert_eq!(wing2.animated, wing.animated);
    }

    #[test]
    fn v0_can_change_chest_properly() {
        let mut image = RgbaImage::new(64, 64);

        let features = EarsFeatures {
            chest_size: 0.45,
            ..Default::default()
        };

        // First, we write the features to the image - in this case, it's just the chest size
        EarsWriterV0::write(&mut image, &features).expect("Expected it to work!");

        // Now, we parse the image to get the features back
        let mut features2 = EarsParserV0::parse(&image).unwrap().unwrap();

        // Sanity check
        assert!(
            (features2.chest_size - 0.45).abs() < 0.01,
            "Chest size is not equal: {} != {}",
            features2.chest_size,
            0.45
        );

        // Ears Manipulator workflow: User wants to change the chest size to 0.0
        features2.chest_size = 0.0;

        // Now, we write the features back to the image
        EarsWriterV0::write(&mut image, &features2).expect("Expected it to work!");

        // Check that our changes were successful
        let features3 = EarsParserV0::parse(&image).unwrap().unwrap();

        assert!(
            (features3.chest_size - 0.0).abs() < 0.01,
            "Chest size is not equal: {} != {}",
            features3.chest_size,
            0.0
        );
    }
}
