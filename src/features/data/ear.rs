use enum_ordinalize::Ordinalize;

#[derive(Ordinalize, Debug, PartialEq, Eq, Clone, Copy)]
#[derive(Default)]
pub enum EarMode {
    #[default]
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
#[derive(Default)]
pub enum EarAnchor {
    #[default]
    Center,
    Front,
    Back,
}




