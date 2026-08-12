pub mod data;
use crate::features::data::ear::{EarAnchor, EarMode};
use data::{
    leg::LegMode, protrusions::Protrusions, snout::SnoutData, tail::TailData, wing::WingData,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataVersion {
    #[default]
    V0,
    V1(u8),
    Custom(u32),
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct EarsFeatures {
    pub ear_mode: EarMode,
    pub ear_anchor: EarAnchor,
    pub tail: Option<TailData>,
    pub snout: Option<SnoutData>,
    pub wing: Option<WingData>,

    pub protrusions: Protrusions,
    pub leg_mode: LegMode,
    pub chest_size: f32,

    pub cape_enabled: bool,
    pub emissive: bool,

    pub data_version: DataVersion,
}
