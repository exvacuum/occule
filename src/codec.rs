pub trait Codec {
    type Carrier;
    type Payload;
    type Output;
    type Error;

    fn encode(
        &self,
        carrier: impl Into<Self::Carrier>,
        payload: impl Into<Self::Payload>,
    ) -> Result<Self::Output, Self::Error>;

    fn decode(&self, encoded: impl Into<Self::Output>) -> Result<(Self::Carrier, Self::Payload), Self::Error>;
}
