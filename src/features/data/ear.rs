use enum_ordinalize::Ordinalize;

#[derive(Ordinalize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum EarMode {
    None,
    Above,
    Sides,
    Behind,
    Around,
    Floppy,
    Cross,
    Out,
    Tall,
    TallCross,
}

#[derive(Ordinalize, Debug, PartialEq, Eq, Clone, Copy)]
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
