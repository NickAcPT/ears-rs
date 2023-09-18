pub mod data;
use crate::features::data::ear::{EarAnchor, EarMode};
use data::{snout::SnoutData, tail::TailData, wing::WingData};

#[derive(Default, Debug, Clone, Copy)]
pub struct EarsFeatures {
    pub ear_mode: EarMode,
    pub ear_anchor: Option<EarAnchor>,
    pub tail: Option<TailData>,
    pub snout: Option<SnoutData>,
    pub wing: Option<WingData>,

    pub claws: bool,
    pub horn: bool,
    pub chest_size: f32,

    pub cape_enabled: bool,
    pub emissive: bool,
}
