use std::sync::Arc;

use culsynth::{NoteFxP, ScalarFxP};
use fixed::traits::Fixed;
use nih_plug::prelude::*;
fn fixed_v2s<F: Fixed>(x: i32) -> String
where
    i32: TryFrom<F::Bits>,
{
    F::from_bits(F::Bits::try_from(x).unwrap_or_default()).to_string()
}

fn fixed_s2v<F: Fixed>(s: &str) -> Option<i32>
where
    F::Bits: Into<i32>,
{
    F::from_str(s).map(|x| x.to_bits().into()).ok()
}

fn fixed_v2s_percent(x: i32) -> String {
    let percent = ScalarFxP::from_bits(x as u16).to_num::<f32>() * 100f32;
    format!("{}", percent)
}

fn fixed_s2v_percent(s: &str) -> Option<i32> {
    s.trim_end_matches(&[' ', '%'])
        .parse::<f32>()
        .map(|x| ScalarFxP::saturating_from_num(x / 100.0).to_bits() as i32)
        .ok()
}

fn fixed_v2s_freq(x: i32) -> String {
    culsynth::midi_note_to_frequency(NoteFxP::from_bits(x as u16)).to_string()
}

fn fixed_s2v_freq(s: &str) -> Option<i32> {
    s.trim_end_matches(&[' ', 'H', 'h', 'Z', 'z'])
        .parse::<f32>()
        .map(|x| {
            NoteFxP::saturating_from_num(((x / 440f32).log2() * 12f32) + 69f32).to_bits() as i32
        })
        .ok()
}

/// Helper function to create a new `nih_plug::IntParam` for a fixed point number
///
/// The integer value will be the raw fixed point representation (i.e. 0 -> 2^16 - 1),
/// but the string representation in the DAW/GUI will be the logical value.
pub fn new_fixed_param<F: Fixed>(name: impl Into<String>, default: F) -> IntParam
where
    F::Bits: Into<i32>,
{
    IntParam::new(
        name,
        default.to_bits().into(),
        IntRange::Linear {
            min: F::MIN.to_bits().into(),
            max: F::MAX.to_bits().into(),
        },
    )
    .with_smoother(SmoothingStyle::Linear(50.0))
    .with_value_to_string(Arc::new(fixed_v2s::<F>))
    .with_string_to_value(Arc::new(fixed_s2v::<F>))
}

/// Helper function to create a new `nih_plug::IntParam` for a [`ScalarFxP`]
/// as a percentage (maps the fixed point number to 0% to 100%)
pub fn new_fixed_param_percent(name: impl Into<String>, default: ScalarFxP) -> IntParam {
    IntParam::new(
        name,
        default.to_bits().into(),
        IntRange::Linear {
            min: ScalarFxP::MIN.to_bits().into(),
            max: ScalarFxP::MAX.to_bits().into(),
        },
    )
    .with_smoother(SmoothingStyle::Linear(50.0))
    .with_value_to_string(Arc::new(fixed_v2s_percent))
    .with_string_to_value(Arc::new(fixed_s2v_percent))
    .with_unit(" %")
}

/// Helper function to create a new `nih_plug::IntParam` for a [`NoteFxP`]
/// as a frequency (maps the fixed point number to the MIDI tuning range in Hz)
pub fn new_fixed_param_freq(name: impl Into<String>, default: NoteFxP) -> IntParam {
    IntParam::new(
        name,
        default.to_bits().into(),
        IntRange::Linear {
            min: NoteFxP::MIN.to_bits().into(),
            max: NoteFxP::MAX.to_bits().into(),
        },
    )
    .with_smoother(SmoothingStyle::Linear(50.0))
    .with_value_to_string(Arc::new(fixed_v2s_freq))
    .with_string_to_value(Arc::new(fixed_s2v_freq))
    .with_unit(" Hz")
}
