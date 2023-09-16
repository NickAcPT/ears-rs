pub(crate) mod bit_reader;
pub(crate) mod model;

mod alpha;
mod eraser;
pub mod errors;
mod legacy_upgrader;
pub mod cape;

pub use alpha::strip_alpha;
pub use eraser::process_erase_regions;
pub use legacy_upgrader::upgrade_skin_if_needed;