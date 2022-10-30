pub(crate) fn to_argb_hex(value: [u8; 4]) -> i32 {
    (value[3] as i32) << 24 | (value[0] as i32) << 16 | (value[1] as i32) << 8 | value[2] as i32
}
