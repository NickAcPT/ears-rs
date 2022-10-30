#[derive(Default, Debug, PartialEq)]
pub struct TailData {
    pub mode: TailMode,
    pub segments: u8,
    pub bends: [f32; 4],
}

#[derive(Debug, PartialEq, Eq)]
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
