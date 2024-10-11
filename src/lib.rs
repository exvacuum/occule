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

/// Codecs for carriers in gltf model format.
#[cfg(feature = "gltf")]
pub mod gltf;

/// Codecs for binary files.
#[cfg(feature = "bin")]
pub mod binary;

/// Codecs for wav files
#[cfg(feature = "wav")]
pub mod wav;
