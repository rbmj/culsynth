use pipewire as pw;
use pipewire::spa::param::audio::AudioFormat;
pub struct PwSupportedFormats {}

impl PwSupportedFormats {
    pub const REQUESTED_BUFSZ: usize = 1024;
    pub const REQUESTED_SAMPLE_RATE: usize = 48000;
    pub const LATENCY_STR: &'static str = "1024/48000";
    pub const FMT_F32: AudioFormat = if cfg!(target_endian = "little") {
        AudioFormat::F32LE
    } else {
        AudioFormat::F32BE
    };
    pub const FMT_I16: AudioFormat = if cfg!(target_endian = "little") {
        AudioFormat::S16LE
    } else {
        AudioFormat::S16BE
    };
}

impl pw::spa::pod::serialize::PodSerialize for PwSupportedFormats {
    fn serialize<O: std::io::Write + std::io::Seek>(
        &self,
        serializer: pw::spa::pod::serialize::PodSerializer<O>,
    ) -> Result<pw::spa::pod::serialize::SerializeSuccess<O>, pw::spa::pod::serialize::GenError>
    {
        use pw::spa::param::{format, ParamType};
        use pw::spa::pod::PropertyFlags;
        use pw::spa::utils::{Choice, ChoiceEnum, ChoiceFlags, Id, SpaTypes};
        let mut s = serializer.serialize_object(
            SpaTypes::ObjectParamFormat.as_raw(),
            ParamType::EnumFormat.as_raw(),
        )?;
        s.serialize_property(
            format::FormatProperties::MediaType.as_raw(),
            &Id(format::MediaType::Audio.0),
            PropertyFlags::empty(),
        )?;
        s.serialize_property(
            format::FormatProperties::MediaSubtype.as_raw(),
            &Id(format::MediaSubtype::Raw.0),
            PropertyFlags::empty(),
        )?;
        s.serialize_property(
            format::FormatProperties::AudioRate.as_raw(),
            &Choice::<i32>(
                ChoiceFlags::empty(),
                ChoiceEnum::Enum {
                    default: Self::REQUESTED_SAMPLE_RATE as i32,
                    alternatives: vec![44100, 48000],
                },
            ),
            PropertyFlags::empty(),
        )?;
        s.serialize_property(
            format::FormatProperties::AudioChannels.as_raw(),
            &Choice::<i32>(
                ChoiceFlags::empty(),
                ChoiceEnum::Enum {
                    default: 2,
                    alternatives: vec![1, 2],
                },
            ),
            PropertyFlags::empty(),
        )?;
        s.serialize_property(
            format::FormatProperties::AudioFormat.as_raw(),
            &Choice::<Id>(
                ChoiceFlags::empty(),
                ChoiceEnum::Enum {
                    default: Id(Self::FMT_F32.as_raw()),
                    alternatives: vec![Id(Self::FMT_I16.as_raw()), Id(Self::FMT_F32.as_raw())],
                },
            ),
            PropertyFlags::empty(),
        )?;
        s.end()
    }
}
