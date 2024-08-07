use enum_ordinalize::Ordinalize;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct WingData {
    pub mode: WingMode,
    pub animated: bool,
}

#[derive(Ordinalize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[derive(Default)]
pub enum WingMode {
    None,
    #[default]
    SymmetricDual,
    SymmetricSingle,
    AsymmetricL,
    AsymmetricR,
}


