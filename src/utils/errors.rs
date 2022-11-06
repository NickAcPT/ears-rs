use image::ImageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EarsError {
    #[error("Image error: {0}")]
    ImageError(#[from] ImageError),
    #[error("Invalid pixel location: (idx: {0})")]
    InvalidMagicPixelLocation(u32)
}

pub(crate) type Result<T> = core::result::Result<T, EarsError>;