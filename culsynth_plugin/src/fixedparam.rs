use std::sync::Arc;

use culsynth::{EnvParamFxP, Fixed16, LfoFreqFxP, NoteFxP, ScalarFxP};
use fixed::traits::Fixed;
use lazy_static::lazy_static;
use nih_plug::prelude::*;
use regex::Regex;

lazy_static! {
    static ref FREQ_REGEX: Regex =
        Regex::new(r"^([0-9]+\.?[0-9]*) ?(?:()|([kK]?[hH][zZ]))$").unwrap();
    static ref ENV_REGEX: Regex = Regex::new(r"^([0-9]+\.?[0-9]*) ?(?:()|([mM]?[sS]))$").unwrap();
}

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
    use fixed::types::I16F16;
    let percent = I16F16::from_bits(x) * 100;
    percent.round().to_num::<i32>().to_string() + "%"
}

fn fixed_s2v_percent(s: &str) -> Option<i32> {
    s.trim_end_matches(&[' ', '%'])
        .parse::<f32>()
        .map(|x| ScalarFxP::saturating_from_num(x / 100.0).to_bits() as i32)
        .ok()
}

fn fixed_v2s_freq(x: i32) -> String {
    let mut freq = culsynth::midi_note_to_frequency(NoteFxP::from_bits(x as u16));
    let khz = freq > 1000;
    if khz {
        freq /= 1000;
    }
    let mut s = if freq > 100 {
        freq.to_num::<i32>().to_string()
    } else if freq > 10 {
        format!("{:.1}", freq)
    } else {
        format!("{:.2}", freq)
    };
    s += if khz { " kHz" } else { " Hz" };
    s
}

fn fixed_s2v_freq(s: &str) -> Option<i32> {
    let groups = FREQ_REGEX.captures(s)?;
    let khz = groups.get(2)?.len() == 3; //khz is 3 chars
    let mut freq = groups.get(1)?.as_str().parse::<f32>().ok()?;
    if khz {
        freq *= 1000f32;
    }
    Some(NoteFxP::saturating_from_num(((freq / 440f32).log2() * 12f32) + 69f32).to_bits() as i32)
}

fn fixed_v2s_time<T: Fixed16>(x: i32) -> String
where
    u16: Into<T::Bits>,
{
    let x = x as u16;
    let mut value = T::from_bits(x.into()).to_num::<f32>();
    let ms = value < 1.;
    if ms {
        value *= 1000.;
    }
    let mut s = if value > 100. {
        (value as i32).to_string()
    } else if value > 10. {
        format!("{:.1}", value)
    } else {
        format!("{:.2}", value)
    };
    s += if ms { " ms" } else { " s" };
    s
}

fn fixed_s2v_time<T: Fixed16>(s: &str) -> Option<i32>
where
    T::Bits: Into<i32>,
{
    let groups = ENV_REGEX.captures(s)?;
    let ms = groups.get(2)?.len() == 2; //ms is 2 chars vs 1 for s
    let mut time = groups.get(1)?.as_str().parse::<f32>().ok()?;
    if ms {
        time /= 1000f32;
    }
    Some(T::saturating_from_num(time).to_bits().into())
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

/// Helper function to create a new `nih_plug::IntParam` for a [`EnvParamFxP`]
/// as a percentage (maps the fixed point number to 0% to 100%)
pub fn new_fixed_param_env(name: impl Into<String>, default: EnvParamFxP) -> IntParam {
    IntParam::new(
        name,
        default.to_bits().into(),
        IntRange::Linear {
            min: EnvParamFxP::MIN.to_bits().into(),
            max: EnvParamFxP::MAX.to_bits().into(),
        },
    )
    .with_smoother(SmoothingStyle::Linear(50.0))
    .with_value_to_string(Arc::new(fixed_v2s_time::<EnvParamFxP>))
    .with_string_to_value(Arc::new(fixed_s2v_time::<EnvParamFxP>))
}

/// Helper function to create a new `nih_plug::IntParam` for a [`LfoFreqFxP`]
/// as a percentage (maps the fixed point number to 0% to 100%)
pub fn new_fixed_param_lfo(name: impl Into<String>, default: LfoFreqFxP) -> IntParam {
    IntParam::new(
        name,
        default.to_bits().into(),
        IntRange::Linear {
            min: LfoFreqFxP::MIN.to_bits().into(),
            max: LfoFreqFxP::MAX.to_bits().into(),
        },
    )
    .with_smoother(SmoothingStyle::Linear(50.0))
    .with_value_to_string(Arc::new(fixed_v2s_time::<LfoFreqFxP>))
    .with_string_to_value(Arc::new(fixed_s2v_time::<LfoFreqFxP>))
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
}
