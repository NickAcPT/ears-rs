use std::io::{Cursor, Write};

use crate::{
    features::{
        data::{ear::EarMode, tail::TailMode, wing::WingMode},
        EarsFeatures,
    },
    parser::{utils::from_argb_hex, EarsFeaturesWriter},
    utils::{bit_writer::BitWriter, errors::Result},
};
use enum_ordinalize::Ordinalize;

pub struct EarsWriterV1;

impl EarsWriterV1 {
    fn write_features<W: Write>(feat: &EarsFeatures, writer: &mut BitWriter<W>) -> Result<()> {
        writer.write_long(8, 0)?; // version

        let ears = if feat.ear_mode == EarMode::None {
            0u64
        } else {
            let ears = ((feat.ear_mode.ordinal() - 1) * 3) as u64;
            let anchor = (feat.ear_anchor.ordinal()) as u64;
            ears + anchor + 1
        };

        writer.write_long(6, ears)?;

        writer.write_bool(feat.claws)?;
        writer.write_bool(feat.horn)?;

        writer.write_long(
            3,
            feat.tail
                .map(|t| t.mode)
                .unwrap_or(TailMode::None)
                .ordinal() as u64,
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

        writer.write_long(
            3,
            feat.wing.map(|w| w.mode).unwrap_or(WingMode::None) as u64,
        )?;

        if let Some(animated) = feat
            .wing
            .map(|w| w.animated)
            .filter(|_| feat.wing.is_some_and(|w| w.mode != WingMode::None))
        {
            writer.write_bool(animated)?;
        }

        writer.write_bool(feat.cape_enabled)?;
        writer.write_bool(feat.emissive)?;

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
        features::data::{ear::EarAnchor, snout::SnoutData, tail::TailData},
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
            }),
            snout: Some(SnoutData {
                offset: 1,
                width: 4,
                height: 2,
                depth: 2,
            }),
            wing: None,
            claws: true,
            horn: false,
            chest_size: 0.0,
            cape_enabled: true,
            emissive: false,
            data_version: 1,
        };

        let mut image = RgbaImage::new(64, 64);

        EarsWriterV1::write(&mut image, &features)?;
        let result = EarsParserV1::parse(&image)?;

        assert_eq!(result, Some(features));

        Ok(())
    }
}
