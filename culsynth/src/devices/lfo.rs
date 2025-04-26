use super::*;
use crate::{IScalarFxP, LfoFreqFxP, PhaseFxP};
use core::mem::transmute;
use core::option::Option;
use oorandom::Rand32;

/// Default random seed to use if not provided a seed
const RANDOM_SEED: u64 = 0xce607a9d25ec3d88u64; //random 64 bit integer

pub(crate) mod detail {
    use super::*;

    pub trait LfoOps: crate::DspFormatBase {
        fn next_phase(
            phase: Self::Phase,
            context: &Self::Context,
            frequency: Self::LfoFreq,
        ) -> Self::Phase;
        fn calc_lfo(
            phase: Self::Phase,
            wave: lfo::LfoWave,
            rands: &[Self::Sample; 2],
        ) -> Self::Sample;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(from = "LfoOptionsSerde")]
#[serde(into = "LfoOptionsSerde")]
/// A struct to package together the various LFO configuration options in one
/// convenient struct that fits in 16 bits.  We could get away with packing
/// it in 8 bits, but we'll use 16 to allow for future expansion
pub struct LfoOptions {
    bits: u16,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct LfoOptionsSerde {
    wave: LfoWave,
    bipolar: bool,
    retrigger: bool,
}

impl From<LfoOptionsSerde> for LfoOptions {
    fn from(value: LfoOptionsSerde) -> Self {
        LfoOptions::new(value.wave, value.bipolar, value.retrigger)
    }
}
impl From<LfoOptions> for LfoOptionsSerde {
    fn from(value: LfoOptions) -> Self {
        LfoOptionsSerde {
            wave: value.wave().unwrap_or_default(),
            bipolar: value.bipolar(),
            retrigger: value.retrigger(),
        }
    }
}

impl LfoOptions {
    const BIPOLAR: u16 = 1 << 8;
    const RETRIGGER: u16 = 1 << 9;
    /// The LFO Waveform (Sine, Square, Sample+Hold, etc.)
    pub const fn wave(&self) -> Option<LfoWave> {
        let value = (self.bits & 0xFF) as u8;
        LfoWave::new_from_u8(value)
    }
    /// Set the LFO Waveform
    pub const fn set_wave(&mut self, wave: LfoWave) {
        self.bits &= 0xFF00;
        self.bits |= wave as u16;
    }
    /// Is this LFO unipolar (0:1) or bipolar (-1:1)?
    pub const fn bipolar(&self) -> bool {
        self.bits & Self::BIPOLAR != 0
    }
    /// Set bipolar (true) or unipolar (false)
    pub const fn set_bipolar(&mut self, bipolar: bool) {
        self.bits &= !Self::BIPOLAR;
        self.bits |= if bipolar { Self::BIPOLAR } else { 0 };
    }
    /// Does this LFO retrigger/reset on each gate?
    pub const fn retrigger(&self) -> bool {
        self.bits & Self::RETRIGGER != 0
    }
    /// Turn on (true)/off (false) retriggering
    pub const fn set_retrigger(&mut self, retrigger: bool) {
        self.bits &= !Self::RETRIGGER;
        self.bits |= if retrigger { Self::RETRIGGER } else { 0 };
    }
    /// Pack the LFO parameters into a `LfoOptions` value
    pub const fn new(wave: LfoWave, bipolar: bool, retrigger: bool) -> Self {
        LfoOptions {
            bits: (wave as u16)
                | if bipolar { Self::BIPOLAR } else { 0 }
                | if retrigger { Self::RETRIGGER } else { 0 },
        }
    }
}

impl Default for LfoOptions {
    /// The default value is a bipolar, retriggering sine wave
    fn default() -> Self {
        Self::new(LfoWave::Sine, true, true)
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
#[repr(u8)]
/// The LFO waveform in use
pub enum LfoWave {
    /// Sine wave is default
    #[default]
    Sine,
    /// Square wave
    Square,
    /// Triangle wave
    Triangle,
    /// Sawtooth wave
    Saw,
    /// Sample and Hold
    SampleHold,
    /// Sample and Glide
    SampleGlide,
}

impl Serialize for LfoWave {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

struct LfoWaveVisitor;
impl<'de> serde::de::Visitor<'de> for LfoWaveVisitor {
    type Value = u8;
    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value)
    }
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("An integer corresponding to a valid LfoWave")
    }
}

impl<'de> Deserialize<'de> for LfoWave {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let as_int = deserializer.deserialize_u8(LfoWaveVisitor {})?;
        Ok(LfoWave::new_from_u8(as_int).unwrap_or_default())
    }
}

impl LfoWave {
    const ELEM: [LfoWave; 6] = [
        Self::Sine,
        Self::Square,
        Self::Triangle,
        Self::Saw,
        Self::SampleHold,
        Self::SampleGlide,
    ];
    /// Returns a slice to all of the possible LfoWaves
    pub const fn waves() -> &'static [LfoWave] {
        &Self::ELEM
    }
    /// Provides the name of the waveform (long-format)
    pub const fn to_str(&self) -> &'static str {
        [
            "Sine",
            "Square",
            "Triangle",
            "Saw",
            "Sample & Hold",
            "Sample & Glide",
        ][*self as usize]
    }
    /// Provides the name of the waveform (long-format)
    ///
    /// This is a single character (for waveforms with unicode representations)
    /// or up to three character abbreviation (e.g. "S+H")
    pub const fn to_str_short(&self) -> &'static str {
        [
            crate::util::SIN_CHARSTR,
            crate::util::SQ_CHARSTR,
            crate::util::TRI_CHARSTR,
            crate::util::SAW_CHARSTR,
            "S+H",
            "S+G",
        ][*self as usize]
    }
    /// Try to create a LfoWave from a u8
    pub const fn new_from_u8(value: u8) -> Option<Self> {
        if value >= LfoWave::Sine as u8 && value <= LfoWave::SampleGlide as u8 {
            unsafe { Some(transmute::<u8, LfoWave>(value)) }
        } else {
            None
        }
    }
}

impl From<LfoWave> for &'static str {
    fn from(value: LfoWave) -> Self {
        value.to_str()
    }
}

impl TryFrom<u8> for LfoWave {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, &'static str> {
        Self::new_from_u8(value).ok_or("Conversion of u8 to LfoWave Overflowed")
    }
}

/// A struct packaging together several slices to act as parameters for an LFO
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LfoParams<T: DspFormatBase> {
    /// The frequency of the LFO, in Hz
    pub freq: T::LfoFreq,
    /// The depth of oscillation, between 0 and 1
    pub depth: T::Scalar,
    /// The options, including waveform and retriggering (see [LfoOptions])
    pub opts: LfoOptions,
}

impl<T: DspFloat> From<&LfoParams<i16>> for LfoParams<T> {
    fn from(value: &LfoParams<i16>) -> Self {
        LfoParams::<T> {
            freq: value.freq.to_num(),
            depth: value.depth.to_num(),
            opts: value.opts,
        }
    }
}

impl<T: DspFormatBase> LfoParams<T> {
    /// Default constructor
    pub const fn new() -> Self {
        Self {
            freq: T::LfoFreq::ZERO,
            depth: T::Scalar::ZERO,
            opts: LfoOptions::new(LfoWave::Sine, true, true),
        }
    }
}

/// An LFO
#[derive(Clone)]
pub struct Lfo<T: DspFormatBase + detail::LfoOps> {
    rng: Rand32,
    phase: T::Phase,
    rand_smps: [T::Sample; 2],
    last_gate: bool,
}

impl<T: DspFormatBase + detail::LfoOps> Lfo<T> {
    /// Constructor
    pub fn new(seed: u64) -> Self {
        let mut retval = Self {
            rng: Rand32::new(seed),
            phase: T::Phase::zero(),
            rand_smps: [T::Sample::zero(); 2],
            last_gate: false,
        };
        retval.update_rands();
        retval.update_rands();
        retval
    }
    fn update_rands(&mut self) {
        self.rand_smps[1] = self.rand_smps[0];
        let rand_num = self.rng.rand_u32() & (u16::MAX as u32);
        let rand_scalar = ScalarFxP::from_bits(rand_num as u16);
        self.rand_smps[0] = T::sample_from_fixed(IScalarFxP::from_num(rand_scalar));
    }
}

impl<T: DspFormat> Device<T> for Lfo<T> {
    type Input = bool;
    type Params = LfoParams<T>;
    type Output = T::Sample;
    /// Generate the LFO signal
    fn next(&mut self, context: &T::Context, gate: bool, params: LfoParams<T>) -> T::Sample {
        if params.opts.retrigger() && gate && !self.last_gate {
            self.phase = T::Phase::zero();
        }
        self.last_gate = gate;
        let mut value = T::calc_lfo(
            self.phase,
            params.opts.wave().unwrap_or_default(),
            &self.rand_smps,
        );
        if !params.opts.bipolar() {
            value = (value + T::Sample::one()).divide_by_two();
        }
        value = value.scale(params.depth);
        let old_phase = self.phase;
        self.phase = T::next_phase(old_phase, context, params.freq);
        // Check if we've crossed from positive phase back to negative:
        if old_phase >= self.phase {
            self.update_rands();
        }
        value
    }
}

impl<T: DspFormatBase + detail::LfoOps> Default for Lfo<T> {
    fn default() -> Self {
        Self::new(RANDOM_SEED)
    }
}

impl detail::LfoOps for i16 {
    fn calc_lfo(phase: PhaseFxP, wave: lfo::LfoWave, rands: &[SampleFxP; 2]) -> SampleFxP {
        match wave {
            LfoWave::Saw => SampleFxP::from_num(phase),
            LfoWave::Square => {
                if phase < PhaseFxP::ZERO {
                    SampleFxP::NEG_ONE
                } else {
                    SampleFxP::ONE
                }
            }
            LfoWave::Triangle => {
                SampleFxP::ONE - (SampleFxP::from_num(phase).abs().unwrapped_shl(1))
            }
            LfoWave::Sine => SampleFxP::from_num(fixedmath::sin_pi(IScalarFxP::from_num(phase))),
            LfoWave::SampleHold => rands[0],
            LfoWave::SampleGlide => {
                let phase_plus_one = SampleFxP::from_num(phase) + SampleFxP::ONE;
                rands[0] + SampleFxP::multiply(phase_plus_one.unwrapped_shr(1), rands[1] - rands[0])
            }
        }
    }
    fn next_phase(phase: PhaseFxP, context: &ContextFxP, frequency: LfoFreqFxP) -> Self::Phase {
        let phase_per_smp = PhaseFxP::from_num(
            frequency.wide_mul(context.sample_rate.frac_32768_sr()).unwrapped_shr(15),
        );
        phase.wrapping_add(phase_per_smp)
    }
}

impl<T: DspFloat> detail::LfoOps for T {
    fn calc_lfo(phase: T, wave: lfo::LfoWave, rands: &[T; 2]) -> T {
        match wave {
            LfoWave::Saw => phase,
            LfoWave::Square => {
                if phase < T::ZERO {
                    T::ONE.neg()
                } else {
                    T::ONE
                }
            }
            LfoWave::Triangle => T::ONE - (phase.abs() * T::TWO),
            LfoWave::Sine => (phase * T::PI).fsin(),
            LfoWave::SampleHold => rands[0],
            LfoWave::SampleGlide => {
                rands[0] + (((phase + T::ONE) / T::TWO) * (rands[1] - rands[0]))
            }
        }
    }
    fn next_phase(mut phase: T, context: &Context<T>, frequency: T) -> T {
        phase = phase + (frequency / context.sample_rate);
        if phase >= T::ONE {
            phase = phase - T::TWO;
        }
        phase
    }
}
