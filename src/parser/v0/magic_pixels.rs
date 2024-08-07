macro_rules! define_v0_magic_pixels {
    ($($name: ident: $hex:expr),+) => {
        #[allow(dead_code, overflowing_literals)]
        #[derive(Copy, Clone, Hash, PartialEq, Eq)]
        pub(crate) enum MagicPixelsV0 {
            $(
                $name,
            )*
        }

        impl MagicPixelsV0 {
            #[allow(dead_code, overflowing_literals)]
            pub(crate) fn get_hex(&self) -> u32 {
                match &self {
                    $(
                        MagicPixelsV0::$name => $hex,
                    )*
                }
            }

            #[allow(dead_code, overflowing_literals)]
            pub(crate) fn get_by_argb_hex(hex: u32) -> MagicPixelsV0 {
                match hex {
                    $(
                        $hex => MagicPixelsV0::$name,
                    )*
                    _ => MagicPixelsV0::Unknown
                }
            }
        }
    };
}

define_v0_magic_pixels!(
    Unknown: 0xFF000000,
    Blue: 0xFF3F23D8,
    Green: 0xFF23D848,
    Red: 0xFFD82350,
    Purple: 0xFFB923D8,
    Cyan: 0xFF23D8C6,
    Orange: 0xFFD87823,
    Pink: 0xFFD823B7,
    Purple2: 0xFFD823FF,
    White: 0xFFFEFDF2,
    Gray: 0xFF5E605A
);
