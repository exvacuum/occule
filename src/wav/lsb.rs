use std::io::Cursor;

use crate::{Codec, Error};

use hound::{self, Sample, SampleFormat, WavReader, WavSamples, WavWriter};
use itertools::{Chunk, Itertools};
use num_traits::{FromBytes, ToBytes};

/// A Least-Significant Bit (LSB) Codec for WAV files. Stores 1 bit of payload data in each sample
/// of a WAV file. Supported sample formats are 8, 16, and 32-bit PCM, and 32-bit float.
#[derive(Clone, Debug)]
pub struct LsbCodec;

impl Codec for LsbCodec {
    fn encode(&self, carrier: &[u8], payload: &[u8]) -> Result<Vec<u8>, crate::Error> {
        if let Ok(mut reader) = hound::WavReader::new(Cursor::new(carrier)) {
            let mut encoded = vec![];
            {
                let mut writer = WavWriter::new(Cursor::new(&mut encoded), reader.spec()).unwrap();
                match reader.spec().sample_format {
                    hound::SampleFormat::Float => {
                        encode::<f32, 4>(payload, &mut reader, &mut writer);
                    }
                    hound::SampleFormat::Int => {
                        match reader.spec().bits_per_sample {
                            8 => {
                                encode::<i8, 1>(payload, &mut reader, &mut writer);
                            }
                            16 => {
                                encode::<i16, 2>(payload, &mut reader, &mut writer);
                            }
                            32 => {
                                encode::<i32, 4>(payload, &mut reader, &mut writer);
                            }
                            _ => return Err(Error::DataInvalid(
                                "Provided WAV data has an unsupported number of bits per sample."
                                    .into(),
                            )),
                        }
                    }
                }
                writer.flush().unwrap();
            }
            Ok(encoded)
        } else {
            Err(Error::DataInvalid(
                "Could not create WAV reader from provided data".into(),
            ))
        }
    }

    fn decode(&self, encoded: &[u8]) -> Result<(Vec<u8>, Vec<u8>), crate::Error> {
        if let Ok(mut reader) = hound::WavReader::new(Cursor::new(encoded)) {
            let decoded = match reader.spec().sample_format {
                SampleFormat::Float => decode::<f32, 4>(&mut reader)?,
                SampleFormat::Int => match reader.spec().bits_per_sample {
                    8 => decode::<i8, 1>(&mut reader)?,
                    16 => decode::<i16, 2>(&mut reader)?,
                    32 => decode::<i32, 4>(&mut reader)?,
                    _ => return Err(Error::DataNotEncoded),
                },
            };
            Ok((encoded.to_vec(), decoded))
        } else {
            Err(Error::DataInvalid(
                "Could not create WAV reader from provided data".into(),
            ))
        }
    }
}

fn encode<T, const N: usize>(
    payload: &[u8],
    reader: &mut WavReader<Cursor<&[u8]>>,
    writer: &mut WavWriter<Cursor<&mut Vec<u8>>>,
) where
    T: Sample + ToBytes<Bytes = [u8; N]> + FromBytes<Bytes = [u8; N]>,
{
    let payload_len = ((payload.len() + size_of::<u32>()) as u32).to_le_bytes();
    let mut payload_iter = payload_len.iter().chain(payload.iter());
    for sample_chunk in &reader.samples::<T>().chunks(8) {
        match payload_iter.next() {
            Some(payload_byte) => {
                encode_byte(writer, *payload_byte, sample_chunk);
            }
            None => {
                for sample in sample_chunk {
                    writer.write_sample(sample.unwrap()).unwrap();
                }
            }
        }
    }
}

fn encode_byte<T, const N: usize>(
    writer: &mut WavWriter<Cursor<&mut Vec<u8>>>,
    payload_byte: u8,
    sample_chunk: Chunk<WavSamples<Cursor<&[u8]>, T>>,
) where
    T: Sample + ToBytes<Bytes = [u8; N]> + FromBytes<Bytes = [u8; N]>,
{
    for (i, sample) in sample_chunk.enumerate() {
        let sample = sample.unwrap();
        let mut sample_bytes = sample.to_le_bytes();
        let payload_bit = (payload_byte >> (7 - i)) & 0b0000_0001;
        sample_bytes[1] &= 0b1111_1110;
        sample_bytes[1] |= payload_bit;
        writer
            .write_sample(T::from_le_bytes(&sample_bytes))
            .unwrap();
    }
}

fn decode<T, const N: usize>(reader: &mut WavReader<Cursor<&[u8]>>) -> Result<Vec<u8>, Error>
where
    T: Sample + ToBytes<Bytes = [u8; N]> + FromBytes<Bytes = [u8; N]>,
{
    let mut decoded = vec![];
    let mut length_bytes = [0_u8; 4];
    for (i, sample_chunk) in reader
        .samples::<T>()
        .take(8 * 4)
        .chunks(8)
        .into_iter()
        .enumerate()
    {
        for (j, sample) in sample_chunk.enumerate() {
            let sample = sample.unwrap();
            let sample_bytes = sample.to_le_bytes();
            let payload_bit = (sample_bytes[1] & 0b0000_0001) << (7 - j);
            length_bytes[i] |= payload_bit;
        }
    }

    let payload_length = u32::from_le_bytes(length_bytes) as usize - size_of::<u32>();
    if payload_length > reader.samples::<T>().len() {
        return Err(Error::DataNotEncoded);
    }

    for sample_chunk in &reader.samples::<T>().chunks(8) {
        let mut byte = 0_u8;
        for (i, sample) in sample_chunk.enumerate() {
            let sample = sample.unwrap();
            let sample_bytes = sample.to_le_bytes();
            let payload_bit = (sample_bytes[1] & 0b0000_0001) << (7 - i);
            byte |= payload_bit;
        }
        decoded.push(byte);
        if decoded.len() >= payload_length {
            break;
        }
    }
    Ok(decoded)
}
