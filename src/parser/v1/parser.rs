use crate::features::data::ear::{EarAnchor, EarMode};
use crate::features::data::snout::SnoutData;
use crate::features::data::tail::{TailData, TailMode};
use crate::features::data::wing::{WingData, WingMode};
use crate::features::EarsFeatures;
use crate::parser::utils::to_argb_hex;
use crate::parser::EarsFeaturesParser;
use crate::utils::bit_reader::BitReader;
use crate::utils::errors::{EarsError, Result};
use image::RgbaImage;
use std::io::Cursor;

const V1_PARSER_MAGIC: u32 = 0xFFEA2501;

pub(crate) struct EarsParserV1;

impl EarsFeaturesParser for EarsParserV1 {
    fn get_data_version() -> u8 {
        1
    }

    fn detect_magic_pixel() -> u32 {
        V1_PARSER_MAGIC
    }

    fn parse(image: &RgbaImage) -> Result<Option<EarsFeatures>> {
        macro_rules! by_ordinal_or {
            ($en: ty, $ordinal: expr, $default: expr) => {{
                use enum_ordinalize::Ordinalize;

                <$en>::from_ordinal($ordinal as i8).unwrap_or($default)
            }};
        }

        let mut data = Vec::new();

        for y in 0..4 {
            for x in 0..4 {
                if x == 0 && y == 0 {
                    continue;
                }
                let c = to_argb_hex(
                    image
                        .get_pixel_checked(x, 32 + y)
                        .ok_or(EarsError::InvalidPixelLocation(x, y))?,
                );

                data.push(((c >> 16) & 0xFF) as u8);
                data.push(((c >> 8) & 0xFF) as u8);
                data.push((c & 0xFF) as u8);
            }
        }

        let data_len = data.len();
        let mut reader = BitReader::new(Cursor::new(data), data_len);

        // currently, version means nothing. in the future it will indicate additional
        // data that has been added to the end of the format (earlier data mustn't change
        // format!)

        // budget: ((4*4)-1)*3 bytes (360 bits)
        let _version = reader.read(8)?;

        let ears = reader.read(6)?;
        // 6 bits has a range of 0-63
        // this means we can have up to 20 ear modes, since we're using "base-3" encoding
        // we're stuck with 3 anchors forever, though

        let (ear_mode, ear_anchor) = if ears == 0 {
            (EarMode::default(), EarAnchor::default())
        } else {
            (
                by_ordinal_or!(EarMode, ((ears - 1) / 3) + 1, EarMode::None),
                by_ordinal_or!(EarAnchor, (ears - 1) % 3, EarAnchor::Center),
            )
        };

        let claws = reader.read_bool()?;
        let horn = reader.read_bool()?;

        let tail_i = reader.read(3)?;
        // 3 bits has a range of 0-7 - if we run out, a value of 7 can mean "read elsewhere"

        let tail_mode = by_ordinal_or!(TailMode, tail_i, TailMode::None);
        let mut tail_segments = 0;
        let mut tail_bend_0 = 0.0f32;
        let mut tail_bend_1 = 0.0f32;
        let mut tail_bend_2 = 0.0f32;
        let mut tail_bend_3 = 0.0f32;
        if tail_mode != TailMode::None {
            tail_segments = reader.read(2)? + 1;
            tail_bend_0 = reader.read_sam_unit(6)? * 90.0f32;
            tail_bend_1 = if tail_segments > 1 {
                reader.read_sam_unit(6)? * 90.0f32
            } else {
                0.0f32
            };
            tail_bend_2 = if tail_segments > 2 {
                reader.read_sam_unit(6)? * 90.0f32
            } else {
                0.0f32
            };
            tail_bend_3 = if tail_segments > 3 {
                reader.read_sam_unit(6)? * 90.0f32
            } else {
                0.0f32
            };
        }

        let mut snout_offset = 0u32;
        let snout_width = reader.read(3)?; // 0-7; valid snout widths are 1-7, so this is perfect as we can use 0 to mean "none"
        let mut snout_height = 0u32;
        let mut snout_depth = 0u32;

        if snout_width > 0 {
            snout_height = reader.read(2)? + 1; // 1-4; perfect
            snout_depth = reader.read(3)? + 1; // 1-8; perfect (the limit used to be 6, but why not 8)
            snout_offset = reader.read(3)?; // 0-7, but we have to cap it based on height
        }

        let chest_size = reader.read_unit(5)?;

        let wing_i = reader.read(3)?;
        // 3 bits has a range of 0-7
        let wing_mode = by_ordinal_or!(WingMode, wing_i, WingMode::None);
        let animate_wings = wing_mode != WingMode::None && reader.read_bool()?;

        let cape_enabled = reader.read_bool()?;

        let emissive = reader.read_bool()?;

        let features = EarsFeatures {
            ear_mode,
            ear_anchor,
            tail: if tail_segments != 0 {
                Some(TailData {
                    mode: tail_mode,
                    segments: tail_segments as u8,
                    bends: [tail_bend_0, tail_bend_1, tail_bend_2, tail_bend_3],
                })
            } else {
                None
            },
            snout: if snout_width > 0 {
                Some(SnoutData {
                    offset: snout_offset as u8,
                    width: snout_width as u8,
                    height: snout_height as u8,
                    depth: snout_depth as u8,
                })
            } else {
                None
            },
            wing: if wing_mode != WingMode::None {
                Some(WingData {
                    mode: wing_mode,
                    animated: animate_wings,
                })
            } else {
                None
            },
            claws,
            horn,
            chest_size,
            cape_enabled,
            emissive,
            data_version: Self::get_data_version(),
        };

        Ok(Some(features))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_detection_works() {
        assert!(EarsParserV1::detect(
            &image::open("test_images/ears_v1_nickac_sample.png")
                .unwrap()
                .to_rgba8()
        ));
        assert!(!EarsParserV1::detect(
            &image::open("test_images/ears_v0_sample1.png")
                .unwrap()
                .to_rgba8()
        ));
    }

    #[test]
    fn v1_parse_works() {
        let result = EarsParserV1::parse(
            &image::open("test_images/ears_v1_nickac_sample.png")
                .unwrap()
                .to_rgba8(),
        );
        assert!(result.is_ok());
        let features = result.unwrap();
        assert!(features.is_some());
        let features = features.unwrap();

        assert_eq!(features.ear_mode, EarMode::Around);
        assert_eq!(features.ear_anchor, EarAnchor::Center);

        assert!(features.claws);
        assert!(!features.horn);

        assert_eq!(
            features.tail,
            Some(TailData {
                mode: TailMode::Down,
                segments: 2,
                bends: [-10.0, -14.285715, 0.0, 0.0]
            })
        );

        assert_eq!(
            features.snout,
            Some(SnoutData {
                offset: 1,
                width: 4,
                height: 2,
                depth: 2,
            })
        );

        assert_eq!(features.chest_size, 0.0);
        assert_eq!(features.wing, None);

        assert!(features.cape_enabled);
        assert!(!features.emissive);
    }
}
