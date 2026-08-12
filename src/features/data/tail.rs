use enum_ordinalize::Ordinalize;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TailData {
    pub mode: TailMode,
    pub segments: u8,
    pub bends: [f32; 4],
    pub animate: bool,
    pub swap_jacket_back: bool,
}

impl Default for TailData {
    fn default() -> Self {
        Self {
            mode: TailMode::default(),
            segments: 0,
            bends: [0.0; 4],
            animate: true,
            swap_jacket_back: false,
        }
    }
}

#[derive(Ordinalize, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum TailMode {
    None,
    #[default]
    Down,
    Back,
    Up,
    Vertical,
    Cross,
    CrossOverlap,
    Star,
    StarOverlap,
}
