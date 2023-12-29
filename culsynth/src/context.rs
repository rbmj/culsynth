//! This module provides objects to reason about the processing context.
//! Currently, the only information wrapped is the current audio sample rate.

use crate::devices::Float;
use crate::ScalarFxP;

/// A trait to provide a generic interface to the various types of
/// fixed/floating point contexts in this module.
pub trait GenericContext {
    /// Returns the sample rate, in Hz.
    fn sample_rate(&self) -> u32;
    /// Returns true if processing using fixed-point logic.
    fn is_fixed_point(&self) -> bool;
}

#[derive(Clone, Copy)]
/// A floating point (using the type `Smp`) processing context
pub struct Context<Smp: Float> {
    /// The sample rate, in Hz, with the same type as a processing type
    pub sample_rate: Smp,
}

impl<Smp: Float> Context<Smp> {
    /// Create a new `Context`
    pub fn new(sample_rate: Smp) -> Self {
        Self { sample_rate }
    }
}

impl<Smp: Float> Default for Context<Smp> {
    fn default() -> Self {
        Self::new(<Smp as From<u16>>::from(44100u16))
    }
}

impl<Smp: Float> GenericContext for Context<Smp> {
    fn sample_rate(&self) -> u32 {
        self.sample_rate.to_u32().unwrap_or_default()
    }
    fn is_fixed_point(&self) -> bool {
        false
    }
}

#[derive(Default, Clone, Copy)]
/// A fixed-point processing context.  Currently this is only supported for a
/// handful of different sample rates, as properly implementing the fixed-point
/// arithmetic requires some assumptions about the ranges of parameters, and
/// this library uses a few hard-coded constants to optimize fixed-point
/// division.
pub struct ContextFxP {
    /// The sample rate, as one of the supported FixedSampleRates:
    pub sample_rate: FixedSampleRate,
}

impl ContextFxP {
    /// Create a new fixed-point context with a sample rate of 44.1kHz
    pub const fn new_441() -> Self {
        Self {
            sample_rate: FixedSampleRate::Khz44_1,
        }
    }
    /// Create a new fixed-point context with a sample rate of 48kHz
    pub const fn new_480() -> Self {
        Self {
            sample_rate: FixedSampleRate::Khz48_0,
        }
    }
    /// Create a fixed-point processing context if the sample rate provided is
    /// a supported sample rate, or return `None` otherwise.
    pub fn maybe_create(value: u32) -> Option<Self> {
        if let Ok(val) = FixedSampleRate::try_from(value) {
            Some(Self { sample_rate: val })
        } else {
            None
        }
    }
}

impl GenericContext for ContextFxP {
    /// The sample rate of this fixed-point context
    fn sample_rate(&self) -> u32 {
        self.sample_rate.value() as u32
    }
    /// Always returns true
    fn is_fixed_point(&self) -> bool {
        true
    }
}

#[derive(Default, Clone, Copy)]
/// An enum representing all of the supported sample rates for fixed-point logic
pub enum FixedSampleRate {
    /// 44.1kHz sample rate
    #[default]
    Khz44_1,
    /// 48kHz sample rate
    Khz48_0,
}

impl FixedSampleRate {
    /// Converts this sample rate to a u16
    pub const fn value(&self) -> u16 {
        match self {
            Self::Khz44_1 => 44100u16,
            Self::Khz48_0 => 48000u16,
        }
    }
    /// An unsigned, 16 bit fixed-point number representing the quantity
    /// (2 * pi * 4096) / (sample_rate)
    pub const fn frac_2pi4096_sr(&self) -> ScalarFxP {
        const RET441: ScalarFxP = ScalarFxP::lit("0x0.9566");
        const RET480: ScalarFxP = ScalarFxP::lit("0x0.8942");
        match self {
            Self::Khz44_1 => RET441,
            Self::Khz48_0 => RET480,
        }
    }
}

impl TryFrom<u32> for FixedSampleRate {
    type Error = &'static str;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            44100 => Ok(Self::Khz44_1),
            48000 => Ok(Self::Khz48_0),
            _ => Err("Unsupported Sample Rate"),
        }
    }
}
