use std::cmp::Ordering;

use image::{ColorType, DynamicImage, GenericImageView, Pixel};
use thiserror::Error;

use crate::codec::Codec;

#[derive(Debug)]
pub struct ChampleveCodec;

impl Codec for ChampleveCodec {
    type Carrier = DynamicImage;
    type Payload = Vec<u8>;
    type Output = Self::Carrier;
    type Error = ChampleveError;

    fn encode(&self, carrier: impl Into<Self::Carrier>, payload: impl Into<Self::Payload>) -> Result<Self::Output, Self::Error> {
        let mut image: DynamicImage = carrier.into();
        let payload: Vec<u8> = payload.into();

        if image.pixels().count() < payload.len() {
            return Err(ChampleveError::PayloadTooBig);
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
            _ => return Err(ChampleveError::UnsupportedFormat { format: image.color() })
        }

        Ok(image)
    }

    fn decode(&self, carrier: impl Into<Self::Output>) -> Result<(Self::Carrier, Self::Payload), ChampleveError> {
        let mut image: DynamicImage = carrier.into();
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
            _ => return Err(ChampleveError::UnsupportedFormat { format: image.color() })
        }
        
        Ok((image, payload))
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

#[derive(Error, Debug)]
pub enum ChampleveError {
    #[error("Payload is too big for the carrier. Choose a smaller payload or an image with greater pixel dimensions.")]
    PayloadTooBig,
    #[error("Specified image format ({format:?}) is unsupported.")]
    UnsupportedFormat {
        format: ColorType
    },
}
