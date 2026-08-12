use std::io::{Cursor, Write};

use crate::{
    features::{
        data::{
            ear::EarMode,
            tail::TailMode,
            wing::{WingAnimationMode, WingMode},
        },
        EarsFeatures,
    },
    parser::{utils::from_argb_hex, v1::parser::EarsParserV1, EarsFeaturesWriter},
    utils::{bit_writer::BitWriter, errors::Result},
};
use enum_ordinalize::Ordinalize;

pub struct EarsWriterV1;

impl EarsWriterV1 {
    fn write_features<W: Write>(feat: &EarsFeatures, writer: &mut BitWriter<W>) -> Result<()> {
        let version = EarsParserV1::get_required_version_for_features(feat);
        writer.write_long(8, version.into())?;

        let ears = if feat.ear_mode == EarMode::None {
            0u64
        } else {
            let ears = ((feat.ear_mode.ordinal() - 1) * 3) as u64;
            let anchor = (feat.ear_anchor.ordinal()) as u64;
            ears + anchor + 1
        };

        writer.write_long(6, ears)?;

        writer.write_long(2, feat.protrusions.ordinal() as u64 & 0b11)?;

        let tail_mode = feat.tail.map(|tail| tail.mode).unwrap_or(TailMode::None);
        writer.write_long(
            3,
            if tail_mode.ordinal() > 6 {
                7
            } else {
                tail_mode.ordinal() as u64
            },
        )?;

        if let Some(tail) = feat.tail {
            if tail.mode != TailMode::None {
                writer.write_long(2, (tail.segments - 1) as u64)?;
                let [bend_0, bend_1, bend_2, bend_3] = tail.bends;

                writer.write_sam_unit(6, bend_0 / 90.0f32)?;
                if tail.segments > 1 {
                    writer.write_sam_unit(6, bend_1 / 90.0f32)?;
                }

                if tail.segments > 2 {
                    writer.write_sam_unit(6, bend_2 / 90.0f32)?;
                }

                if tail.segments > 3 {
                    writer.write_sam_unit(6, bend_3 / 90.0f32)?;
                }
            }
        }

        if let Some(snout) = feat.snout {
            // ears-rs: Our parser returns Some if the following conditions are met:
            // snoutWidth > 0 && snoutHeight > 0 && snoutDepth > 0
            // We can assume that the snout is enabled if we have a snout, so we don't need to check for that
            writer.write_long(3, (snout.width) as u64)?;
            writer.write_long(2, (snout.height - 1) as u64)?;
            writer.write_long(3, (snout.depth - 1) as u64)?;
            writer.write_long(3, (snout.offset) as u64)?;
        } else {
            writer.write_long(3, 0)?;
        }

        writer.write_unit(5, feat.chest_size)?;

        let wing_mode = feat.wing.map(|wing| wing.mode).unwrap_or(WingMode::None);
        writer.write_long(3, wing_mode.ordinal() as u64)?;

        if wing_mode != WingMode::None {
            writer.write_bool(
                feat.wing
                    .is_some_and(|wing| wing.animation_mode == WingAnimationMode::Normal),
            )?;
        }

        writer.write_bool(feat.cape_enabled)?;
        writer.write_bool(feat.emissive)?;

        if version >= 1 && tail_mode.ordinal() > 6 {
            writer.write_long(3, (tail_mode.ordinal() - 7) as u64)?;
        }

        if version >= 2 {
            writer.write_long(3, feat.leg_mode.ordinal() as u64)?;
            if wing_mode != WingMode::None {
                writer.write_long(
                    3,
                    feat.wing
                        .map(|wing| wing.animation_mode.ordinal() as u64)
                        .unwrap_or(WingAnimationMode::Normal.ordinal() as u64),
                )?;
            }
            writer.write_bool(feat.tail.is_none_or(|tail| tail.animate))?;
            writer.write_bool(feat.tail.is_some_and(|tail| tail.swap_jacket_back))?;
        }

        if version >= 3 {
            writer.write_long(2, feat.protrusions.ordinal() as u64 >> 2)?;
        }

        writer.align()?;

        Ok(())
    }
}

impl EarsFeaturesWriter for EarsWriterV1 {
    fn write(image: &mut image::RgbaImage, features: &crate::features::EarsFeatures) -> Result<()> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        {
            let mut writer = BitWriter::new(&mut cursor);

            Self::write_features(features, &mut writer)?;
        };
        let mut data = cursor.into_inner().into_iter();

        for y in 0..4 {
            for x in 0..4 {
                let c = if x == 0 && y == 0 {
                    0xFFEA2501
                } else {
                    let mut c: u32 = 0xFF000000;
                    c |= (((data.next()).as_deref().unwrap_or(&0) & 0xFF) as u32) << 16;
                    c |= (((data.next()).as_deref().unwrap_or(&0) & 0xFF) as u32) << 8;
                    c |= ((data.next()).as_deref().unwrap_or(&0) & 0xFF) as u32;

                    c
                };

                let pixel = image
                    .get_pixel_mut_checked(x, 32 + y)
                    .ok_or_else(|| crate::utils::errors::EarsError::InvalidPixelLocation(x, y))?;
                *pixel = from_argb_hex(c);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use image::RgbaImage;

    use super::*;
    use crate::{
        features::{
            data::{
                ear::EarAnchor, leg::LegMode, protrusions::Protrusions, snout::SnoutData,
                tail::TailData, wing::WingData,
            },
            DataVersion,
        },
        parser::{v1::parser::EarsParserV1, EarsFeaturesParser},
    };

    #[test]
    fn v1_roundtrip_write_works() -> Result<()> {
        let features = EarsFeatures {
            ear_mode: EarMode::Around,
            ear_anchor: EarAnchor::Center,
            tail: Some(TailData {
                mode: TailMode::Down,
                segments: 2,
                bends: [-10.0, -14.285715, 0.0, 0.0],
                animate: true,
                swap_jacket_back: false,
            }),
            snout: Some(SnoutData {
                offset: 1,
                width: 4,
                height: 2,
                depth: 2,
            }),
            wing: None,
            protrusions: Protrusions::Claws,
            leg_mode: LegMode::Plantigrade,
            chest_size: 0.0,
            cape_enabled: true,
            emissive: false,
            data_version: DataVersion::V1(0),
        };

        let mut image = RgbaImage::new(64, 64);

        EarsWriterV1::write(&mut image, &features)?;
        let result = EarsParserV1::parse(&image)?;

        assert_eq!(result, Some(features));

        Ok(())
    }

    #[test]
    fn v1_version_zero_roundtrip_preserves_image() -> Result<()> {
        let features = EarsFeatures {
            ear_mode: EarMode::Around,
            ear_anchor: EarAnchor::Center,
            tail: Some(TailData {
                mode: TailMode::Down,
                segments: 2,
                bends: [-10.0, -14.285715, 0.0, 0.0],
                ..Default::default()
            }),
            snout: Some(SnoutData {
                offset: 1,
                width: 4,
                height: 2,
                depth: 2,
            }),
            cape_enabled: true,
            ..Default::default()
        };
        let mut source = RgbaImage::new(64, 64);
        EarsWriterV1::write(&mut source, &features)?;

        let parsed = EarsParserV1::parse(&source)?.expect("v1 data should parse");
        assert_eq!(parsed.data_version, DataVersion::V1(0));

        let mut roundtripped = source.clone();
        EarsWriterV1::write(&mut roundtripped, &parsed)?;
        assert_eq!(roundtripped, source);

        Ok(())
    }

    #[test]
    fn v1_roundtrip_write_works_with_new_data() -> Result<()> {
        let features = EarsFeatures {
            ear_mode: EarMode::Around,
            ear_anchor: EarAnchor::Center,
            tail: Some(TailData {
                mode: TailMode::StarOverlap,
                segments: 2,
                bends: [-10.0, -14.285715, 0.0, 0.0],
                animate: true,
                swap_jacket_back: false,
            }),
            snout: Some(SnoutData {
                offset: 1,
                width: 4,
                height: 2,
                depth: 2,
            }),
            wing: Some(WingData {
                mode: WingMode::Flat,
                ..Default::default()
            }),
            protrusions: Protrusions::Claws,
            leg_mode: LegMode::Plantigrade,
            chest_size: 0.0,
            cape_enabled: true,
            emissive: false,
            data_version: DataVersion::V1(1),
        };

        let mut image = RgbaImage::new(64, 64);

        EarsWriterV1::write(&mut image, &features)?;
        let result = EarsParserV1::parse(&image)?;

        assert_eq!(result, Some(features));

        Ok(())
    }

    #[test]
    fn v1_roundtrip_preserves_extended_feature_data() -> Result<()> {
        let features = EarsFeatures {
            tail: Some(TailData {
                mode: TailMode::StarOverlap,
                segments: 1,
                bends: [0.0; 4],
                animate: false,
                swap_jacket_back: true,
            }),
            wing: Some(WingData {
                mode: WingMode::Flat,
                animation_mode: WingAnimationMode::NoFlight,
            }),
            protrusions: Protrusions::ClawsAndDoubleHalo,
            leg_mode: LegMode::DigitigradeFull,
            data_version: DataVersion::V1(3),
            ..Default::default()
        };

        let mut image = RgbaImage::new(64, 64);
        EarsWriterV1::write(&mut image, &features)?;

        assert_eq!(EarsParserV1::parse(&image)?, Some(features));

        Ok(())
    }
}
