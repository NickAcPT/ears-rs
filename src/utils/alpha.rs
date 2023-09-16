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

#[cfg(test)]
mod tests {
    use crate::utils::alpha::strip_alpha;


    #[test]
    fn alpha_stripper_works() {
        fn alpha_strip_works(original: &str, expected: &str) {
            let mut image = image::open(original).unwrap().to_rgba8();
            strip_alpha(&mut image);
            let expected_image = image::open(expected).unwrap().to_rgba8();

            for (x, y, pixel) in expected_image.enumerate_pixels() {
                let real_pixel = image.get_pixel(x, y).0;
                assert_eq!(pixel.0, real_pixel);
            }
        }

        alpha_strip_works(
            "test_images/ears_v0_sample1.png",
            "test_images/ears_v0_sample1.png"
        );
        alpha_strip_works(
            "test_images/ears_v1_nickac_sample.png",
            "test_images/ears_v1_nickac_alpha_stripped.png"
        );
        alpha_strip_works(
            "test_images/notch_upgraded.png",
            "test_images/notch_upgraded_alpha_stripped.png"
        );
    }
}
