use thiserror::Error;

/// Codecs enable the concealment of payload data inside the data of a carrier.
pub trait Codec {
    /// Embeds payload data inside carrier, returning the result.
    fn encode(&self, carrier: &[u8], payload: &[u8]) -> Result<Vec<u8>, CodecError>;

    /// Extracts payload data from an encoded carrier, returning the carrier with data removed and the
    /// payload data.
    fn decode(&self, encoded: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CodecError>;
}

/// Errors produced by a codec
#[derive(Debug, Error)]
pub enum CodecError {
    /// Variant used when data is determined not to be encoded. Note that a codec may have no way
    /// of knowing this, so this may not be returned even if the data was not encoded
    #[error("Data was not encoded with this codec")]
    DataNotEncoded,

    /// Variant used when data is invalid in some way. Allows a message string for further context
    #[error("Provided data invalid: {0}")]
    DataInvalid(String),

    /// Variant used when some dependency, such as a file load, fails
    #[error("Error occured in dependency: {0}")]
    DependencyError(String),
}
