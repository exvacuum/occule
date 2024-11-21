use std::{cmp::Ordering, io::{BufWriter, Cursor}};

use image::{DynamicImage, GenericImageView, Pixel};

use crate::{codec::Codec, Error};

/// Least-significant bit (LSB) steganography encodes data in the least-significant bits of colors
/// in an image. This implementation reduces the colors in the carrier (irreversibly) in order to
/// allow a byte of data to fit in each pixel of the image. 3 bits of data are encoded per pixel,
/// and the 9th bit is used to signal the end of data.
#[derive(Clone, Debug, Default)]
pub struct LsbCodec;

impl Codec for LsbCodec {
    fn encode(&self, carrier: &[u8], payload: &[u8]) -> Result<Vec<u8>, Error>
    {
        let image_format = image::guess_format(carrier).unwrap();
        let mut image: DynamicImage = image::load_from_memory(carrier).unwrap();
        let payload: &[u8] = payload;

        if image.pixels().count() < payload.len() {
            return Err(Error::DataInvalid("Payload Too Big for Carrier".into()));
        }

        let mut payload_iter = payload.iter();

        match image {
            DynamicImage::ImageRgba8(ref mut image) => {
                for pixel in image.pixels_mut() {
                    if let Some(payload_byte) = payload_iter.next() {
                        encode_pixel(pixel, *payload_byte, false);
                    } else {
                        encode_pixel(pixel, 0, true);
                    }
                }
            },
            DynamicImage::ImageRgb8(ref mut image) => {
                for pixel in image.pixels_mut() {
                    if let Some(payload_byte) = payload_iter.next() {
                        encode_pixel(pixel, *payload_byte, false);
                    } else {
                        encode_pixel(pixel, 0, true);
                    }
                }
            },
            _ => return Err(Error::DataInvalid("Unsupported Image Color Format".into()))
        }

        let mut buf = BufWriter::new(Cursor::new(Vec::<u8>::new()));
        if let Err(e) = image.write_to(&mut buf, image_format) {
            return Err(Error::DependencyError(e.to_string()))
        }
        Ok(buf.into_inner().unwrap().into_inner())
    }

    fn decode(&self, carrier: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Error>
    {
        let image_format = image::guess_format(carrier).unwrap();
        let mut image: DynamicImage = image::load_from_memory(carrier).unwrap();
        let mut payload: Vec<u8> = Vec::new();

        match image {
            DynamicImage::ImageRgba8(ref mut image) => {
                for pixel in image.pixels_mut() {
                    if let Some(payload_byte) = decode_pixel(pixel) {
                        payload.push(payload_byte);
                    } else {
                        break;
                    }
                }
            },
            DynamicImage::ImageRgb8(ref mut image) => {
                for pixel in image.pixels_mut() {
                    if let Some(payload_byte) = decode_pixel(pixel) {
                        payload.push(payload_byte);
                    } else {
                        break;
                    }
                }
            },
            _ => return Err(Error::DataInvalid("Unsupported Image Color Format".into()))
        }
        
        let mut buf = BufWriter::new(Cursor::new(Vec::<u8>::new()));
        if let Err(e) = image.write_to(&mut buf, image_format) {
            return Err(Error::DependencyError(e.to_string()))
        }
        Ok((buf.into_inner().unwrap().into_inner(), payload))
    }
}

fn encode_pixel<P: Pixel<Subpixel = u8>>(pixel: &mut P, payload_byte: u8, end_of_data: bool) {
    let mut bits_remaining: i32 = 8;
    for channel in pixel.channels_mut() {
        *channel &= 0b11111000;
        bits_remaining -= 3;
        if bits_remaining <= -3 {
            break;
        }

        let mask = match bits_remaining.cmp(&0) {
            Ordering::Less => payload_byte << -bits_remaining,
            _ => payload_byte >> bits_remaining,
        } & 0b00000111;

        *channel |= mask;
    }

    // Add end-of-data marker to final bit if necessary
    if end_of_data {
        *pixel.channels_mut().last_mut().unwrap() |= 1;
    }
}

fn decode_pixel<P: Pixel<Subpixel = u8>>(pixel: &mut P) -> Option<u8> {
    
    // Final bit as end-of-data marker
    if pixel.channels().last().unwrap() & 1 == 1 {
        return None;
    }

    let mut bits_remaining: i32 = 8;
    let mut payload_byte: u8 = 0;
    for channel in pixel.channels_mut() {
        bits_remaining -= 3;
        if bits_remaining <= -3 {
            break;
        }

        let channel_bits = *channel & 0b00000111;
        *channel &= 0b11111000;
        let mask = match bits_remaining.cmp(&0) {
            Ordering::Less => channel_bits >> -bits_remaining,
            _ => channel_bits << bits_remaining,
        };
        payload_byte |= mask;
    }
    Some(payload_byte)
}
