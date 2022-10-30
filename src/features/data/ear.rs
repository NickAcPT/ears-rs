#[derive(Debug, PartialEq, Eq)]
pub enum EarMode {
    None,
    Above,
    Sides,
    Out,
    Around,
    Floppy,
    Cross,
    Tall,
    TallCross,
    Behind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EarAnchor {
    Center,
    Front,
    Back,
}

impl Default for EarMode {
    fn default() -> Self {
        EarMode::None
    }
}

impl Default for EarAnchor {
    fn default() -> Self {
        EarAnchor::Center
    }
}
