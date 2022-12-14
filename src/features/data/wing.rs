use enum_ordinalize::Ordinalize;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct WingData {
    pub mode: WingMode,
    pub animated: bool,
}

#[derive(Ordinalize, Debug, PartialEq, Eq)]
pub enum WingMode {
    None,
    SymmetricDual,
    SymmetricSingle,
    AsymmetricL,
    AsymmetricR,
}

impl Default for WingMode {
    fn default() -> Self {
        WingMode::SymmetricDual
    }
}
