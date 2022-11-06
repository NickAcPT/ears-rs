use image::ImageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EarsError {
    #[error("Image error: {0}")]
    ImageError(#[from] ImageError),
    #[error("Invalid pixel location: (idx: {0})")]
    InvalidMagicPixelLocation(u32),
    #[error("Invalid Alfalfa pixel position: ({0}, {1})")]
    InvalidAlfalfaPixelPosition(u32, u32),
    #[error("Invalid Alfalfa pixel position: ({0}, {1})")]
    InvalidPixelLocation(u32, u32),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub(crate) type Result<T> = core::result::Result<T, EarsError>;
