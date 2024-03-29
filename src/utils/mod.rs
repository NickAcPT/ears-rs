pub(crate) mod bit_reader;
pub(crate) mod bit_writer;
pub(crate) mod model;

mod alpha;
mod eraser;
pub mod errors;
mod legacy_upgrader;
mod cape;
mod emissive;

pub use alpha::strip_alpha;
pub use eraser::process_erase_regions;
pub use legacy_upgrader::upgrade_skin_if_needed;
pub use cape::convert_ears_cape_to_mojang_cape;

pub use emissive::*;