use crate::{
    features::{
        DataVersion, EarsFeatures,
        data::{
            ear::{EarAnchor, EarMode},
            leg::LegMode,
            protrusions::Protrusions,
            snout::SnoutData,
            tail::{TailData, TailMode},
            wing::{WingAnimationMode, WingData, WingMode},
        },
    },
    parser::EarsFeaturesParser,
    parser::v0::macros::read_magic_pixel,
    parser::v0::magic_pixels::MagicPixelsV0,
    utils::errors::{EarsError, Result},
};
use image::RgbaImage;

pub(crate) struct EarsParserV0;

impl EarsFeaturesParser for EarsParserV0 {
    fn detect_magic_pixel() -> u32 {
        MagicPixelsV0::Blue.get_hex()
    }

    fn parse(image: &RgbaImage) -> Result<Option<EarsFeatures>> {
        if !Self::detect(image) {
            return Ok(None);
        }
        let mut features = EarsFeatures::default();

        features.ear_mode = read_magic_pixel!(
            image, 1, EarMode::None,
            MagicPixelsV0::Blue => EarMode::Above,
            MagicPixelsV0::Green => EarMode::Sides,
            MagicPixelsV0::Purple => EarMode::Behind,
            MagicPixelsV0::Cyan => EarMode::Around,
            MagicPixelsV0::Orange => EarMode::Floppy,
            MagicPixelsV0::Pink => EarMode::Cross,
            MagicPixelsV0::Purple2 => EarMode::Out,
            MagicPixelsV0::White => EarMode::Tall,
            MagicPixelsV0::Gray => EarMode::TallCross
        )?;

        features.ear_anchor = read_magic_pixel!(
            image, 2, EarAnchor::Center, features.ear_mode != EarMode::None && features.ear_mode != EarMode::Behind,
            MagicPixelsV0::Blue => EarAnchor::Center,
            MagicPixelsV0::Green => EarAnchor::Front,
            MagicPixelsV0::Red => EarAnchor::Back
        )?.unwrap_or_default();

        features.protrusions = read_magic_pixel!(
            image, 3, Protrusions::None,
            MagicPixelsV0::Green => Protrusions::Claws,
            MagicPixelsV0::Purple => Protrusions::Horn,
            MagicPixelsV0::Cyan => Protrusions::ClawsAndHorn,
            MagicPixelsV0::White => Protrusions::Halo,
            MagicPixelsV0::Gray => Protrusions::DoubleHalo,
            MagicPixelsV0::Purple2 => Protrusions::ClawsAndHalo,
            MagicPixelsV0::Pink => Protrusions::ClawsAndDoubleHalo
        )?;

        features.tail = read_tail_data(image)?;
        features.snout = read_snout_data(image)?;

        let etc = read_magic_pixel!(image, 7)?;

        features.chest_size = (((etc & 0x00FF0000) >> 16) as f32 / 128f32).clamp(0.0, 1.0);
        features.cape_enabled = (etc & 16) != 0;

        features.wing = read_wing_data(image)?;

        features.leg_mode = read_magic_pixel!(
            image, 11, LegMode::Plantigrade,
            MagicPixelsV0::Green => LegMode::DigitigradePartial,
            MagicPixelsV0::Pink => LegMode::DigitigradeFull
        )?;

        let bitflags = read_magic_pixel!(image, 12)? & 0x00FF_FFFF;
        if bitflags != MagicPixelsV0::Blue.get_hex() & 0x00FF_FFFF {
            if let Some(tail) = features.tail.as_mut() {
                tail.animate = bitflags & 1 != 0;
                tail.swap_jacket_back = bitflags & 2 != 0;
            }
        }

        features.emissive = read_magic_pixel!(image, 10)? == MagicPixelsV0::Orange.get_hex();

        features.data_version = DataVersion::V0;

        Ok(Some(features))
    }

    fn detect(image: &RgbaImage) -> bool {
        return read_magic_pixel!(image, 0)
            .is_ok_and(|p| MagicPixelsV0::get_by_argb_hex(p) == MagicPixelsV0::Blue);
    }
}

fn read_wing_data(image: &RgbaImage) -> Result<Option<WingData>> {
    let mode = read_magic_pixel!(
        image, 8, WingMode::None,
        MagicPixelsV0::Pink => WingMode::SymmetricDual,
        MagicPixelsV0::Green => WingMode::SymmetricSingle,
        MagicPixelsV0::Cyan => WingMode::AsymmetricL,
        MagicPixelsV0::Orange => WingMode::AsymmetricR,
        MagicPixelsV0::Purple => WingMode::AsymmetricDual,
        MagicPixelsV0::Purple2 => WingMode::Flat
    )?;

    if mode == WingMode::None {
        return Ok(None);
    }

    let animation_mode = read_magic_pixel!(
        image, 9, WingAnimationMode::Normal,
        MagicPixelsV0::Red => WingAnimationMode::None,
        MagicPixelsV0::Green => WingAnimationMode::NoFlight
    )?;

    Ok(Some(WingData {
        mode,
        animation_mode,
    }))
}

fn read_snout_data(image: &RgbaImage) -> Result<Option<SnoutData>> {
    let snout = read_magic_pixel!(image, 6)?;
    let etc = read_magic_pixel!(image, 7)?;

    let mut snout_offset = ((etc & 0x0000FF00) >> 8) as u8;
    let mut snout_width = ((snout & 0x00FF0000) >> 16) as u8;
    let mut snout_height = ((snout & 0x0000FF00) >> 8) as u8;
    let mut snout_depth = (snout & 0x000000FF) as u8;

    if snout_offset > 8 - snout_height {
        snout_offset = 8 - snout_height;
    }
    if snout_width > 7 {
        snout_width = 7;
    }
    if snout_height > 4 {
        snout_height = 4;
    }
    if snout_depth > 8 {
        snout_depth = 8;
    }

    if snout_width == 0 && snout_height == 0 && snout_depth == 0 {
        return Ok(None);
    }

    Ok(Some(SnoutData {
        width: snout_width,
        height: snout_height,
        depth: snout_depth,
        offset: snout_offset,
    }))
}

fn read_tail_data(image: &RgbaImage) -> Result<Option<TailData>> {
    /// Convert a pixel value to a float from -1 to 1, using an encoding that puts 0 at pixel value
    /// 0, thereby shifting all other possible values forward by one.
    ///
    /// This allows a black pixel to mean 0 for all of its values.
    ///
    /// Taken from https://github.com/unascribed/Ears/blob/7bf6c80a7d14ab8425926551fb1f46aba390b720/common/src/main/java/com/unascribed/ears/common/EarsFeaturesParserV0.java#L226
    fn px_val_to_unit(i: i32) -> f32 {
        if i == 0 {
            return 0.0;
        }
        let mut j = i - 128;
        if j < 0 {
            j -= 1;
        }
        if j >= 0 {
            j += 1;
        }
        j as f32 / 128.0
    }

    let mode = read_magic_pixel!(
        image, 4, TailMode::None,
        MagicPixelsV0::Blue => TailMode::Down,
        MagicPixelsV0::Green => TailMode::Back,
        MagicPixelsV0::Purple => TailMode::Up,
        MagicPixelsV0::Orange => TailMode::Vertical,
        MagicPixelsV0::Pink => TailMode::Cross,
        MagicPixelsV0::Purple2 => TailMode::CrossOverlap,
        MagicPixelsV0::White => TailMode::Star,
        MagicPixelsV0::Gray => TailMode::StarOverlap
    )?;

    if mode == TailMode::None {
        return Ok(None);
    }

    let tail_bend = read_magic_pixel!(image, 5)?;

    let tail_bend0 =
        px_val_to_unit((255 - ((tail_bend as i64 & 0xFF000000_i64) >> 24)) as i32) * 90.0;
    let tail_bend1 = px_val_to_unit(((tail_bend & 0x00FF0000) >> 16) as i32) * 90.0;
    let tail_bend2 = px_val_to_unit(((tail_bend & 0x0000FF00) >> 8) as i32) * 90.0;
    let tail_bend3 = px_val_to_unit((tail_bend & 0x000000FF) as i32) * 90.0;
    let mut data = TailData::default();

    data.mode = mode;
    data.bends = [tail_bend0, tail_bend1, tail_bend2, tail_bend3];
    data.segments = 1 + data.bends.iter().skip(1).filter(|&&x| x != 0.0).count() as u8;

    Ok(Some(data))
}

#[cfg(test)]
mod tests {
    use crate::parser::EarsParser;

    use super::*;

    #[test]
    fn v0_detection_works() {
        let image = image::open("test_images/ears_v0_sample1.png").unwrap();
        let image = image.to_rgba8();
        assert!(EarsParserV0::detect(&image));
    }

    #[test]
    fn v0_ear_mode_none() {
        let image = image::open("test_images/ears_v0_sample1.png").unwrap();
        let image = image.to_rgba8();
        let features = EarsParserV0::parse(&image).unwrap().unwrap();
        assert_eq!(features.ear_mode, EarMode::None);
    }

    #[test]
    fn v0_ear_mode_above() {
        let image = image::open("test_images/ears_v0_sample_earmode_above.png").unwrap();
        let image = image.to_rgba8();
        let features = EarsParserV0::parse(&image).unwrap().unwrap();
        assert_eq!(features.ear_mode, EarMode::Above);
    }

    #[test]
    fn v0_tail_works() {
        let image = image::open("test_images/ears_v0_sample_tail_3_down_30_-30_60.png").unwrap();
        let image = image.to_rgba8();
        let features = EarsParserV0::parse(&image).unwrap().unwrap();
        let tail = features.tail.unwrap();

        assert_eq!(tail.mode, TailMode::Down);
        assert_eq!(tail.segments, 3);
        assert_eq!(tail.bends, [30.234375, -29.53125, 60.46875, 0.0]); // Rounding go BRRRRRR
    }

    #[test]
    fn v0_works_with_alpha_in_ears_data() {
        let image = image::open("test_images/aa7e0904a404417b944d909b994f3abb.png").unwrap();
        let image = image.to_rgba8();
        let features = EarsParserV0::parse(&image).unwrap().unwrap();

        assert_eq!(
            EarsParser::parse(&image).unwrap().unwrap(),
            features,
            "Rip don't match"
        );

        assert_eq!(
            features,
            EarsFeatures {
                ear_mode: EarMode::Around,
                ear_anchor: EarAnchor::Center,
                tail: Some(TailData {
                    mode: TailMode::Vertical,
                    segments: 1,
                    bends: [14.765625, 0.0, 0.0, 0.0],
                    animate: true,
                    swap_jacket_back: false,
                }),
                snout: Some(SnoutData {
                    offset: 0,
                    width: 4,
                    height: 2,
                    depth: 2,
                }),
                wing: None,
                protrusions: Protrusions::None,
                leg_mode: LegMode::Plantigrade,
                chest_size: 0.40625,
                cape_enabled: false,
                emissive: false,
                data_version: DataVersion::V0,
            }
        );
    }

    #[test]
    fn v0_new_tail_modes_work() {
        #[rustfmt::skip]
        let modes = [
            ("test_images/ears-cross-overlapping.png",TailMode::CrossOverlap),
            ("test_images/ears-cross-tail.png", TailMode::Cross),
            ("test_images/ears-star-45.png", TailMode::Star),
            ("test_images/ears-overlapstar-45.png", TailMode::StarOverlap),
        ];

        for (file, mode) in modes {
            let image = image::open(file).unwrap();
            let image = image.to_rgba8();
            let features = EarsParserV0::parse(&image).unwrap().unwrap();
            let tail = features.tail.unwrap();

            assert_eq!(tail.mode, mode);
        }
    }

    #[test]
    fn v0_new_wing_modes_work() {
        #[rustfmt::skip]
        let modes = [
            ("test_images/ears-wing-flat.png", WingMode::Flat),
            ("test_images/ears-wing-asymmetricdual.png", WingMode::AsymmetricDual),
        ];

        for (file, mode) in modes {
            let image = image::open(file).unwrap();
            let image = image.to_rgba8();
            let features = EarsParserV0::parse(&image).unwrap().unwrap();
            let wing = features.wing.unwrap();

            assert_eq!(wing.mode, mode);
        }
    }

    #[test]
    fn v0_works() {
        let image = image::open("test_images/ears_v0_sample_ear_out_front_claws_horn_tail_back_3_snout_4x3x4-0,2_wings_symmetric_dual_normal.png").unwrap();
        let image = image.to_rgba8();
        let features = EarsParserV0::parse(&image).unwrap().unwrap();

        assert_eq!(features.ear_mode, EarMode::Out);
        assert_eq!(features.ear_anchor, EarAnchor::Front);
        assert_eq!(features.protrusions, Protrusions::ClawsAndHorn);

        let tail = features.tail.unwrap();
        assert_eq!(tail.segments, 3);
        assert_eq!(tail.mode, TailMode::Back);

        let snout = features.snout.unwrap();
        assert_eq!(snout.width, 4);
        assert_eq!(snout.height, 3);
        assert_eq!(snout.depth, 4);
        assert_eq!(snout.offset, 2);

        assert_eq!(
            features.wing,
            Some(WingData {
                mode: WingMode::SymmetricDual,
                animation_mode: WingAnimationMode::Normal,
            })
        )
    }
}
