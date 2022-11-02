macro_rules! define_strip_alpha_func {
    ($([($x1: expr, $y1: expr), ($x2: expr, $y2: expr)]),+) => {
        use image::RgbaImage;

        #[allow(dead_code)]
        pub fn strip_alpha(image: &mut RgbaImage) {
            $(
            for y in $y1..$y2 {
                for x in $x1..$x2 {
                    if let Some(pixel) = image.get_pixel_mut_checked(x, y) {
                        pixel.0[3] = u8::MAX;
                    }
                }
            }
            )+
        }
    };
}

define_strip_alpha_func!(
    [(8, 0), (24, 8)],
    [(0, 8), (32, 16)],
    [(4, 16), (12, 20)],
    [(20, 16), (36, 20)],
    [(44, 16), (52, 20)],
    [(0, 20), (56, 32)],
    [(20, 48), (28, 52)],
    [(36, 48), (44, 52)],
    [(16, 52), (48, 64)]
);
