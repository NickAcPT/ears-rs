use image::Rgba;

pub(crate) fn to_argb_hex(value: &Rgba<u8>) -> u32 {
    (value[3] as u32) << 24 | (value[0] as u32) << 16 | (value[1] as u32) << 8 | value[2] as u32
}
