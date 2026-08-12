pub(crate) mod bit_reader;
pub(crate) mod bit_writer;
pub(crate) mod model;

mod alpha;
mod cape;
mod emissive;
mod eraser;
pub mod errors;
mod legacy_upgrader;
mod skin;

pub use alpha::{strip_alpha, strip_alpha_for_features};
pub use cape::convert_ears_cape_to_mojang_cape;
pub use eraser::process_erase_regions;
pub use legacy_upgrader::upgrade_skin_if_needed;
pub use skin::{SkinPreprocessResult, preprocess_skin};

pub use emissive::*;
