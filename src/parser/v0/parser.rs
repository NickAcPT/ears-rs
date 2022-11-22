use crate::{
    features::data::ear::{EarAnchor, EarMode},
    features::data::snout::SnoutData,
    features::data::tail::{TailData, TailMode},
    features::data::wing::{WingData, WingMode},
    features::EarsFeatures,
    parser::v0::macros::read_magic_pixel,
    parser::v0::magic_pixels::MagicPixelsV0,
    parser::EarsFeaturesParser,
    utils::errors::{EarsError, Result},
};
use image::RgbaImage;

pub(crate) struct EarsParserV0;

impl EarsFeaturesParser for EarsParserV0 {
    fn get_data_version() -> u8 {
        0
    }

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
        )?;

        let (claws, horn) = read_magic_pixel!(
            image, 3, (false, false),
            MagicPixelsV0::Green => (true, false),
            MagicPixelsV0::Purple => (false, true),
            MagicPixelsV0::Cyan => (true, true)
        )?;

        features.claws = claws;
        features.horn = horn;

        features.tail = read_tail_data(image)?;
        features.snout = read_snout_data(image)?;

        let etc = read_magic_pixel!(image, 7)?;

        features.chest_size = (((etc & 0x00FF0000) >> 16) as f32 / 128f32).clamp(0.0, 1.0);
        features.cape_enabled = (etc & 16) != 0;

        features.wing = read_wing_data(image)?;

        features.emissive = read_magic_pixel!(image, 10)? == MagicPixelsV0::Orange.get_hex();

        Ok(Some(features))
    }
}

fn read_wing_data(image: &RgbaImage) -> Result<Option<WingData>> {
    let mode = read_magic_pixel!(
        image, 8, WingMode::None,
        MagicPixelsV0::Pink => WingMode::SymmetricDual,
        MagicPixelsV0::Green => WingMode::SymmetricSingle,
        MagicPixelsV0::Cyan => WingMode::AsymmetricL,
        MagicPixelsV0::Orange => WingMode::AsymmetricR
    )?;

    if mode == WingMode::None {
        return Ok(None);
    }

    let animated = read_magic_pixel!(image, 9)? != MagicPixelsV0::Red.get_hex();

    Ok(Some(WingData { mode, animated }))
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
        MagicPixelsV0::Orange => TailMode::Vertical
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
    fn v0_works() {
        let image = image::open("test_images/ears_v0_sample_ear_out_front_claws_horn_tail_back_3_snout_4x3x4-0,2_wings_symmetric_dual_normal.png").unwrap();
        let image = image.to_rgba8();
        let features = EarsParserV0::parse(&image).unwrap().unwrap();

        assert_eq!(features.ear_mode, EarMode::Out);
        assert_eq!(features.ear_anchor, Some(EarAnchor::Front));
        assert!(features.claws);
        assert!(features.horn);

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
                animated: true,
            })
        )
    }
}
