use crate::{Codec, Error};

/// Reverses payload binary data and writes it ass-first past the end of the original data. A
/// length marker is also prepended to the payload *before reversing* so the decoder knows how long
/// the payload is.
#[derive(Clone, Debug)]
pub struct BinaryReverseAppendixCodec;

impl Codec for BinaryReverseAppendixCodec {
    fn encode(&self, carrier: &[u8], payload: &[u8]) -> Result<Vec<u8>, crate::Error> {
        let mut encoded = Vec::<u8>::new();
        encoded.extend(carrier.iter());
        let payload_len = (payload.len() as u64 + 8).to_le_bytes();
        encoded.extend(payload_len.iter().chain(payload.iter()).rev());
        Ok(encoded)
    }

    fn decode(&self, encoded: &[u8]) -> Result<(Vec<u8>, Vec<u8>), crate::Error> {
        if encoded.len() < 8 {
            return Err(Error::DataNotEncoded);
        }

        let encoded_len = encoded.len();
        let payload_len = u64::from_le_bytes(encoded.iter().rev().take(8).cloned().collect::<Vec<_>>().try_into().unwrap()) as usize;
        if encoded_len < payload_len + 8 || payload_len < 8 {
            return Err(Error::DataNotEncoded);
        }

        let carrier_len = encoded_len - payload_len;

        let carrier = encoded[..carrier_len].to_vec();
        let payload = encoded.iter().rev().skip(8).take(payload_len - 8).cloned().collect::<Vec<_>>();

        Ok((carrier, payload))
    }
}
