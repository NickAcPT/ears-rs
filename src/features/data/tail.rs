use enum_ordinalize::Ordinalize;

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct TailData {
    pub mode: TailMode,
    pub segments: u8,
    pub bends: [f32; 4],
}

#[derive(Ordinalize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum TailMode {
    None,
    Down,
    Back,
    Up,
    Vertical,
}

impl Default for TailMode {
    fn default() -> Self {
        TailMode::Down
    }
}
