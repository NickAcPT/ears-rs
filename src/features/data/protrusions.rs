use enum_ordinalize::Ordinalize;

#[derive(Ordinalize, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum Protrusions {
    #[default]
    None,
    Horn,
    Claws,
    ClawsAndHorn,
    Halo,
    DoubleHalo,
    ClawsAndHalo,
    ClawsAndDoubleHalo,
}
