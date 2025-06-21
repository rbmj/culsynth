//! Contains structures to support patch memory/serialization

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum PatchType {
    Bass,
    Lead,
    Pad,
    Percussion,
    Brass,
    Strings,
}

pub struct Patch {
    name_arr: [u8; 32],
    params: crate::voice::VoiceParams<i16>,
}
