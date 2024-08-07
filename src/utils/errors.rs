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
    #[error("IO error ({0}): {1}")]
    IoError(std::io::Error, &'static str),
    #[error("Cannot fit {0} into a long.")]
    NotEnoughSpaceInLongForBitsError(u8),
    #[error("Cannot fit {0} into an int.")]
    NotEnoughSpaceInIntForBitsError(u8),
    #[error("Invalid Alfalfa version: {0}")]
    InvalidAlfalfaVersion(u8),
    #[error("Cannot write an entry with name {0} - it must start with an ASCII character with value 64 (@) or greater")]
    InvalidAlfalfaEntryName(String),
    #[error("Cannot write an entry with name {0} - it must only contain ASCII characters")]
    InvalidAlfalfaEntryNameAscii(String),
    /* Cannot write more than 1428 bytes of data (got "+bys.length+" bytes) */
    #[error("Cannot write more than 1428 bytes of data (got {0} bytes)")]
    AlfalfaDataTooLarge(usize),
    #[error("Unable to convert big uint to u32")]
    UnableToConvertBigUintToU32,
}

impl From<(std::io::Error, &'static str)> for EarsError {
    fn from((err, msg): (std::io::Error, &'static str)) -> Self {
        Self::IoError(err, msg)
    }
}

pub(crate) type Result<T> = core::result::Result<T, EarsError>;
