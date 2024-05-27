/// Codecs enable the concealment of payload data inside the data of a carrier.
pub trait Codec {
    /// Data type representing the carrier.
    type Carrier;

    /// Data type representing the payload.
    type Payload;

    /// Data type representing encoder output/decoder input (usually the same as the carrier).
    type Output;

    /// Type of errors produced by this codec.
    type Error;

    /// Embeds payload data inside carrier, returning the result.
    fn encode<C, P>(&self, carrier: C, payload: P) -> Result<Self::Output, Self::Error>
    where
        C: Into<Self::Carrier>,
        P: Into<Self::Payload>;

    /// Extracts payload data from an encoded carrier, returning the carrier with data removed and the
    /// payload data.
    fn decode<E>(&self, encoded: E) -> Result<(Self::Carrier, Self::Payload), Self::Error>
    where
        E: Into<Self::Output>;
}
