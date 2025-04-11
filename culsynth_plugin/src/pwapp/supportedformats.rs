use pipewire as pw;
pub struct PwSupportedFormats {}

impl pw::spa::pod::serialize::PodSerialize for PwSupportedFormats {
    fn serialize<O: std::io::Write + std::io::Seek>(
        &self,
        serializer: pw::spa::pod::serialize::PodSerializer<O>,
    ) -> Result<pw::spa::pod::serialize::SerializeSuccess<O>, pw::spa::pod::serialize::GenError>
    {
        use pw::spa::param::{audio::AudioFormat, format, ParamType};
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
                    default: 48000,
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
                    default: Id(AudioFormat::F32LE.as_raw()),
                    alternatives: vec![
                        Id(AudioFormat::S16LE.as_raw()),
                        Id(AudioFormat::F32LE.as_raw()),
                    ],
                },
            ),
            PropertyFlags::empty(),
        )?;
        s.end()
    }
}
