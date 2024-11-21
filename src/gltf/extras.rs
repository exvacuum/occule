use std::borrow::Cow;

use base64::Engine;
use gltf::Gltf;
use serde_json::{json, value::to_raw_value, Value};

use crate::{Codec, Error};

/// Codec for embedding data in a GLTF file "extras" entry. It uses the extras entry in the first
/// scene in the file and stores the data as base64.
#[derive(Clone, Debug, Default)]
pub struct ExtrasEntryCodec;

impl Codec for ExtrasEntryCodec {
    fn encode(&self, carrier: &[u8], payload: &[u8]) -> Result<Vec<u8>, crate::Error> {
        let gltf = match Gltf::from_slice(carrier) {
            Ok(gltf) => gltf,
            Err(e) => return Err(Error::DependencyError(e.to_string())),
        };

        let mut json = gltf.document.into_json();
        let mut scene = json.scenes.remove(0);
        let mut extras = match serde_json::from_str::<Value>(scene.extras.clone().unwrap_or(to_raw_value(&json!({})).unwrap()).get()) {
            Ok(extras) => extras,
            Err(e) => return Err(Error::DependencyError(e.to_string())),
        };
        match &mut extras {
            Value::Object(object) => {
                let base64_payload = base64::engine::general_purpose::STANDARD.encode(payload);
                object.insert("occule".into(), Value::String(base64_payload));
            },
            _ => return Err(Error::DataInvalid("Carrier has extras in non-object format, not gonna mess with that.".into()))
        }
        let extras = match to_raw_value(&extras) {
            Ok(raw) => Some(raw),
            Err(e) => return Err(Error::DependencyError(e.to_string())),
        };
        scene.extras = extras;
        json.scenes.insert(0, scene);
        let json_string = match gltf_json::serialize::to_string(&json) {
            Ok(json_string) => json_string,
            Err(e) => return Err(Error::DependencyError(e.to_string())),
        };

        let mut glb = match gltf::binary::Glb::from_slice(carrier) {
            Ok(glb) => glb,
            Err(e) => return Err(Error::DependencyError(e.to_string())),
        };
        glb.header.length = (glb.header.length as usize - glb.json.len() + align_to_multiple_of_four(json_string.len())) as u32;
        glb.json = Cow::Owned(json_string.into_bytes());        

        Ok(match glb.to_vec() {
            Ok(vec) => vec,
            Err(e) => return Err(Error::DependencyError(e.to_string()))
        })
    }

    fn decode(&self, encoded: &[u8]) -> Result<(Vec<u8>, Vec<u8>), crate::Error> {
        let gltf = match Gltf::from_slice(encoded) {
            Ok(gltf) => gltf,
            Err(e) => return Err(Error::DependencyError(e.to_string())),
        };
        let mut json = gltf.document.into_json();
        let mut extras = match &json.scenes[0].extras {
            Some(extras) => match serde_json::from_str::<Value>(extras.get()) {
                Ok(Value::Object(value)) => value,
                _ => return Err(Error::DataNotEncoded),
            },
            None => return Err(Error::DataNotEncoded),
        };
        let payload = match extras.remove("occule".into()) {
            Some(Value::String(payload)) => match base64::engine::general_purpose::STANDARD.decode(payload) {
                Ok(payload) => payload,
                Err(e) => return Err(Error::DependencyError(e.to_string())),
            },
            _ => return Err(Error::DataNotEncoded),
        };

        json.scenes[0].extras = match to_raw_value(&Value::Object(extras)) {
            Ok(extras) => Some(extras),
            Err(e) => return Err(Error::DependencyError(e.to_string()))
        };

        // TODO: remove payload from carrier
        Ok((encoded.to_vec(), payload))      
    }
}

fn align_to_multiple_of_four(n: usize) -> usize {
    (n + 3) & !3
}
