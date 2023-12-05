use image::Rgba;

pub(crate) fn to_argb_hex(value: &Rgba<u8>) -> u32 {
    (value[3] as u32) << 24 | (value[0] as u32) << 16 | (value[1] as u32) << 8 | value[2] as u32
}

pub(crate) fn from_argb_hex(value: u32) -> Rgba<u8> {
    let bytes = value.to_be_bytes();
    
    Rgba(
        [bytes[1], bytes[2], bytes[3], bytes[0]]
    )
}