use std::{mem::size_of, usize};

use img_parts::jpeg::{markers, Jpeg, JpegSegment};

use crate::{codec::Codec, Error};

/// Codec for storing payload data in JPEG comment (COM) segments. Can store an arbitrary amount of
/// data, as long as the number of comment segments does not exceed u64::MAX.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JpegSegmentCodec {
    /// Index of segment to insert comments at.
    pub start_index: usize,
}

impl Codec for JpegSegmentCodec {
    fn encode(&self, carrier: &[u8], payload: &[u8]) -> Result<Vec<u8>, Error>
    {
        let mut jpeg = match Jpeg::from_bytes(carrier.to_vec().into()) {
            Ok(v) => v,
            Err(e) => return Err(Error::DependencyError(e.to_string()))
        };
        let mut payload_bytes = payload.to_vec();
        let segment_count = ((payload_bytes.len() + size_of::<u64>()) as u64).div_ceil((u16::MAX as usize - size_of::<u16>()) as u64);
        payload_bytes.splice(0..0, segment_count.to_le_bytes());
        for (index, payload_chunk) in payload_bytes.chunks(u16::MAX as usize - size_of::<u16>()).enumerate() {
            let segment = JpegSegment::new_with_contents(markers::COM, payload_chunk.to_vec().into());
            jpeg.segments_mut().insert(self.start_index + index, segment);
        }
        Ok(jpeg.encoder().bytes().to_vec())
    }

    fn decode(&self, encoded: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Error>
    {
        let mut jpeg = match Jpeg::from_bytes(encoded.to_vec().into()) {
            Ok(v) => v,
            Err(e) => return Err(Error::DependencyError(e.to_string()))
        };
        let segment = jpeg.segments_mut().remove(self.start_index);
        let segment_bytes = segment.contents();
        let segment_count = u64::from_le_bytes(segment_bytes[0..size_of::<u64>()].try_into().unwrap()) as usize;
        let mut payload_vec: Vec<u8> = Vec::with_capacity((u16::MAX as usize - size_of::<u16>()) * segment_count);
        payload_vec.extend(segment_bytes[size_of::<u64>()..].to_vec());

        for _ in 0..segment_count-1 {
            let segment = jpeg.segments_mut().remove(self.start_index);
            payload_vec.extend(segment.contents());
        }

        Ok((jpeg.encoder().bytes().to_vec(), payload_vec))
    }
}

impl Default for JpegSegmentCodec {
    fn default() -> Self {
        Self {
            start_index: 3,
        }
    }
}
