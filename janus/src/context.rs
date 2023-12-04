use crate::devices::Float;
use crate::ScalarFxP;

pub trait GenericContext {
    fn sample_rate(&self) -> u32;
    fn is_fixed_point(&self) -> bool;
}

pub struct Context<Smp: Float> {
    pub sample_rate: Smp,
}

impl<Smp: Float> Context<Smp> {
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

#[derive(Default)]
pub struct ContextFxP {
    pub sample_rate: FixedSampleRate,
}

impl ContextFxP {
    pub fn new_441() -> Self {
        Self {
            sample_rate: FixedSampleRate::Khz44_1,
        }
    }
    pub fn new_480() -> Self {
        Self {
            sample_rate: FixedSampleRate::Khz48_0,
        }
    }
    pub fn maybe_create(value: u32) -> Option<Self> {
        if let Ok(val) = FixedSampleRate::try_from(value) {
            Some(Self { sample_rate: val })
        } else {
            None
        }
    }
}

impl GenericContext for ContextFxP {
    fn sample_rate(&self) -> u32 {
        self.sample_rate.value() as u32
    }
    fn is_fixed_point(&self) -> bool {
        true
    }
}

#[derive(Default)]
pub enum FixedSampleRate {
    #[default]
    Khz44_1,
    Khz48_0,
}

impl FixedSampleRate {
    pub const fn value(&self) -> u16 {
        match self {
            Self::Khz44_1 => 44100u16,
            Self::Khz48_0 => 48000u16,
        }
    }
    pub const fn frac_2pi4096_sr(&self) -> ScalarFxP {
        match self {
            Self::Khz44_1 => ScalarFxP::lit("0x0.9566"),
            Self::Khz48_0 => ScalarFxP::lit("0x0.8942"),
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
