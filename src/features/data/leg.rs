use enum_ordinalize::Ordinalize;

#[derive(Ordinalize, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum LegMode {
    #[default]
    Plantigrade,
    DigitigradePartial,
    DigitigradeFull,
}
