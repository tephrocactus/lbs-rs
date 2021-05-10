pub use lbs_derive::*;
pub use read::LBSRead;
pub use write::LBSWrite;

pub mod read;
pub mod write;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "smallvec")]
mod smallvec;

pub fn err_invalid_data(text: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, text.to_owned())
}
