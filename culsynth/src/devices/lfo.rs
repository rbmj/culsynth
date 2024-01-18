use super::*;
use crate::{IScalarFxP, PhaseFxP};
use core::mem::transmute;
use core::option::Option;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

/// Default random seed to use if not provided a seed
const RANDOM_SEED: u64 = 0xce607a9d25ec3d88u64; //random 64 bit integer

pub(crate) mod detail {
    use super::*;

    pub trait LfoOps: crate::DspFormatBase + crate::devices::osc::detail::OscOps {
        fn phase_per_smp(context: &Self::Context, frequency: Self::LfoFreq) -> Self::Phase;
        fn calc_lfo(
            phase: Self::Phase,
            wave: lfo::LfoWave,
            rands: &[Self::Sample; 2],
        ) -> Self::Sample;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
/// A struct to package together the various LFO configuration options in one
/// convenient struct that fits in 16 bits.  We could get away with packing
/// it in 8 bits, but we'll use 16 to allow for future expansion
pub struct LfoOptions {
    bits: u16,
}

impl LfoOptions {
    const BIPOLAR: u16 = 1 << 8;
    const RETRIGGER: u16 = 1 << 9;
    /// The LFO Waveform (Sine, Square, Sample+Hold, etc.)
    pub fn wave(&self) -> Option<LfoWave> {
        let value = (self.bits & 0xFF) as u8;
        LfoWave::try_from(value).ok()
    }
    /// Is this LFO unipolar (0:1) or bipolar (-1:1)?
    pub fn bipolar(&self) -> bool {
        self.bits & Self::BIPOLAR != 0
    }
    /// Does this LFO retrigger/reset on each gate?
    pub fn retrigger(&self) -> bool {
        self.bits & Self::RETRIGGER != 0
    }
    /// Pack the LFO parameters into a `LfoOptions` value
    pub fn new(wave: LfoWave, bipolar: bool, retrigger: bool) -> Self {
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

#[derive(Default, Clone, Copy)]
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
}

impl From<LfoWave> for &'static str {
    fn from(value: LfoWave) -> Self {
        value.to_str()
    }
}

impl TryFrom<u8> for LfoWave {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, &'static str> {
        if value >= LfoWave::Sine as u8 && value <= LfoWave::SampleGlide as u8 {
            unsafe { Ok(transmute::<u8, LfoWave>(value)) }
        } else {
            Err("Conversion of u8 to LfoWave Overflowed")
        }
    }
}

/// A struct packaging together several slices to act as parameters for an LFO
#[derive(Default, Clone)]
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

/// An LFO
#[derive(Clone)]
pub struct Lfo<T: DspFormatBase + detail::LfoOps> {
    rng: SmallRng,
    phase: T::Phase,
    rand_smps: [T::Sample; 2],
    last_gate: bool,
}

impl<T: DspFormatBase + detail::LfoOps> Lfo<T> {
    /// Constructor
    pub fn new(seed: u64) -> Self {
        let mut retval = Self {
            rng: SmallRng::seed_from_u64(seed),
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
        let rand_num = self.rng.next_u32() & (u16::MAX as u32);
        let rand_scalar = ScalarFxP::from_bits(rand_num as u16);
        self.rand_smps[0] = T::sample_from_fixed(IScalarFxP::from_num(rand_scalar));
    }
}

impl<T: DspFormat> Device<T> for Lfo<T> {
    type Input = T::Sample;
    type Params = LfoParams<T>;
    type Output = T::Sample;
    /// Generate the LFO signal
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    fn next(&mut self, context: &T::Context, gate: T::Sample, params: LfoParams<T>) -> T::Sample {
        let this_gate = gate > T::GATE_THRESHOLD;
        if params.opts.retrigger() && this_gate && !self.last_gate {
            self.phase = T::Phase::zero();
        }
        self.last_gate = this_gate;
        let mut value = T::calc_lfo(
            self.phase,
            params.opts.wave().unwrap_or_default(),
            &self.rand_smps,
        );
        if !params.opts.bipolar() {
            value = (value + T::Sample::one()).divide_by_two();
        }
        value = value.scale(params.depth);
        self.phase = self.phase + T::phase_per_smp(context, params.freq);
        // Check if we've crossed from positive phase back to negative:
        if self.phase >= T::Phase::PI {
            self.phase = self.phase - T::Phase::TAU;
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
        use crate::fixed_traits::Fixed16;
        use crate::fixedmath::{cos_fixed, sin_fixed};
        const TWO: SampleFxP = SampleFxP::lit("2");
        let frac_2phase_pi = SampleFxP::from_num(phase).scale_fixed(ScalarFxP::FRAC_2_PI);
        match wave {
            LfoWave::Saw => frac_2phase_pi.unwrapped_shr(1),
            LfoWave::Square => {
                if phase < 0 {
                    SampleFxP::NEG_ONE
                } else {
                    SampleFxP::ONE
                }
            }
            LfoWave::Triangle => {
                if phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {
                    frac_2phase_pi.unwrapped_neg() - TWO
                } else if phase > PhaseFxP::FRAC_PI_2 {
                    TWO - frac_2phase_pi
                } else {
                    frac_2phase_pi
                }
            }
            LfoWave::Sine => {
                if phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {
                    // phase in [-pi, pi/2)
                    // Use the identity sin(x) = -cos(x+pi/2) since our taylor series
                    // approximations are centered about zero and this will be more accurate
                    cos_fixed(SampleFxP::from_num(phase + PhaseFxP::FRAC_PI_2)).unwrapped_neg()
                } else if phase < PhaseFxP::FRAC_PI_2 {
                    // phase in [pi/2, pi)
                    // sin(x) = cos(x-pi/2)
                    cos_fixed(SampleFxP::from_num(phase - PhaseFxP::FRAC_PI_2))
                } else {
                    sin_fixed(SampleFxP::from_num(phase))
                }
            }
            LfoWave::SampleHold => rands[0],
            LfoWave::SampleGlide => {
                rands[0] + SampleFxP::multiply(frac_2phase_pi, rands[1] - rands[0])
            }
        }
    }
    fn phase_per_smp(context: &ContextFxP, frequency: Self::LfoFreq) -> Self::Phase {
        PhaseFxP::from_num(
            frequency
                .wide_mul(context.sample_rate.frac_2pi4096_sr())
                .unwrapped_shr(12),
        )
    }
}

impl<T: DspFloat> detail::LfoOps for T {
    fn calc_lfo(phase: T, wave: lfo::LfoWave, rands: &[T; 2]) -> T {
        let frac_2phase_pi = (phase + phase) / T::PI;
        let pi_2 = T::FRAC_PI_2;
        match wave {
            LfoWave::Saw => frac_2phase_pi / T::TWO,
            LfoWave::Square => {
                if phase < T::ZERO {
                    T::ONE.neg()
                } else {
                    T::ONE
                }
            }
            LfoWave::Triangle => {
                if phase < pi_2.neg() {
                    frac_2phase_pi.neg() - T::TWO
                } else if phase > pi_2 {
                    T::TWO - frac_2phase_pi
                } else {
                    frac_2phase_pi
                }
            }
            LfoWave::Sine => {
                if phase < pi_2.neg() {
                    // phase in [-pi, pi/2)
                    // Use the identity sin(x) = -cos(x+pi/2) since our taylor series
                    // approximations are centered about zero and this will be more accurate
                    T::fcos(phase + pi_2).neg()
                } else if phase < pi_2 {
                    // phase in [pi/2, pi)
                    // sin(x) = cos(x-pi/2)
                    T::fcos(phase - pi_2)
                } else {
                    T::fsin(phase)
                }
            }
            LfoWave::SampleHold => rands[0],
            LfoWave::SampleGlide => rands[0] + (frac_2phase_pi * (rands[1] - rands[0])),
        }
    }
    fn phase_per_smp(context: &Context<T>, frequency: T) -> T {
        (frequency * T::TAU) / context.sample_rate
    }
}
