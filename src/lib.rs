#![warn(missing_docs)]

//! Library providing steganography codecs for various carrier and payload types, designed to be
//! extensible.

mod codec;
pub use codec::*;

/// Codecs for carriers in JPEG format.
#[cfg(feature = "jpeg")]
pub mod jpeg;

/// Codecs for carriers in lossless image formats (PNG, WebP, etc.).
#[cfg(feature = "lossless")]
pub mod lossless;
