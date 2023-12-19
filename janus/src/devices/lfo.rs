use super::osc::PhaseFxP;
use super::*;
use crate::fixedmath::apply_scalar_i;
use core::mem::transmute;
use core::option::Option;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

/// Default random seed to use if not provided a seed
const RANDOM_SEED: u64 = 0xce607a9d25ec3d88u64; //random 64 bit integer

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
pub struct LfoParams<'a, Smp: Float> {
    /// The frequency of the LFO, in Hz
    pub freq: &'a [Smp],
    /// The depth of oscillation, between 0 and 1
    pub depth: &'a [Smp],
    /// The options, including waveform and retriggering (see [LfoOptions])
    pub opts: &'a [LfoOptions],
}

impl<'a, Smp: Float> LfoParams<'a, Smp> {
    /// The length of this parameter pack is defined as the length of the
    /// smallest subslice
    pub fn len(&self) -> usize {
        min_size(&[self.freq.len(), self.depth.len(), self.opts.len()])
    }
    /// True if any slice is zero length
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A mutable LFO parameter pack (see [LfoParams])
pub struct MutLfoParams<'a, Smp: Float> {
    /// The frequency of the LFO, in Hz
    pub freq: &'a mut [Smp],
    /// The depth of oscillation, between 0 and 1
    pub depth: &'a mut [Smp],
    /// The options, including waveform and retriggering (see [LfoOptions])
    pub opts: &'a mut [LfoOptions],
}

impl<'a, Smp: Float> MutLfoParams<'a, Smp> {
    /// The length of this parameter pack is defined as the length of the
    /// smallest subslice
    pub fn len(&self) -> usize {
        min_size(&[self.freq.len(), self.depth.len(), self.opts.len()])
    }
    /// True if any slice is zero length
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, Smp: Float> From<MutLfoParams<'a, Smp>> for LfoParams<'a, Smp> {
    fn from(value: MutLfoParams<'a, Smp>) -> Self {
        Self {
            freq: value.freq,
            depth: value.depth,
            opts: value.opts,
        }
    }
}

/// A floating-point LFO
#[derive(Clone)]
pub struct Lfo<Smp> {
    outbuf: BufferT<Smp>,
    rng: SmallRng,
    phase: Smp,
    rand_smps: [Smp; 2],
    last_gate: bool,
}

impl<Smp: Float> Lfo<Smp> {
    /// Constructor
    pub fn new(seed: u64) -> Self {
        let mut retval = Self {
            outbuf: [Smp::ZERO; STATIC_BUFFER_SIZE],
            rng: SmallRng::seed_from_u64(seed),
            phase: Smp::ZERO,
            rand_smps: [Smp::ZERO; 2],
            last_gate: false,
        };
        retval.update_rands();
        retval.update_rands();
        retval
    }
    fn update_rands(&mut self) {
        self.rand_smps[1] = self.rand_smps[0];
        let rand_num = self.rng.next_u32() & (u16::MAX as u32);
        self.rand_smps[0] = Smp::from_u16(rand_num as u16) / Smp::from_u16(u16::MAX);
    }
    /// Generate the LFO signal
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(&mut self, ctx: &Context<Smp>, gate: &[Smp], params: LfoParams<Smp>) -> &[Smp] {
        let freq = params.freq;
        let depth = params.depth;
        let opts = params.opts;
        let numsamples = min_size(&[
            freq.len(),
            gate.len(),
            opts.len(),
            depth.len(),
            STATIC_BUFFER_SIZE,
        ]);
        for i in 0..numsamples {
            let this_gate = gate[i] > Smp::ONE_HALF;
            if opts[i].retrigger() && this_gate && !self.last_gate {
                self.phase = Smp::ZERO;
            }
            self.last_gate = this_gate;
            //generate waveforms (piecewise defined)
            let frac_2phase_pi = (self.phase + self.phase) / Smp::PI();
            let mut value = match opts[i].wave().unwrap_or_default() {
                LfoWave::Saw => frac_2phase_pi / Smp::TWO,
                LfoWave::Square => {
                    if self.phase < Smp::ZERO {
                        Smp::ONE.neg()
                    } else {
                        Smp::ONE
                    }
                }
                LfoWave::Triangle => {
                    if self.phase < Smp::FRAC_PI_2().neg() {
                        frac_2phase_pi.neg() - Smp::TWO
                    } else if self.phase > Smp::FRAC_PI_2() {
                        Smp::TWO - frac_2phase_pi
                    } else {
                        frac_2phase_pi
                    }
                }
                LfoWave::Sine => {
                    if self.phase < Smp::FRAC_PI_2().neg() {
                        // phase in [-pi, pi/2)
                        // Use the identity sin(x) = -cos(x+pi/2) since our taylor series
                        // approximations are centered about zero and this will be more accurate
                        Smp::fcos(self.phase + Smp::FRAC_PI_2()).neg()
                    } else if self.phase < Smp::FRAC_PI_2() {
                        // phase in [pi/2, pi)
                        // sin(x) = cos(x-pi/2)
                        Smp::fcos(self.phase - Smp::FRAC_PI_2())
                    } else {
                        Smp::fsin(self.phase)
                    }
                }
                LfoWave::SampleHold => self.rand_smps[0],
                LfoWave::SampleGlide => {
                    self.rand_smps[0] + (frac_2phase_pi * (self.rand_smps[1] - self.rand_smps[0]))
                }
            };
            if !opts[i].bipolar() {
                value = (value + Smp::ONE) / Smp::TWO;
            }
            self.outbuf[i] = value * depth[i];
            let phase_per_sample = (freq[i] * Smp::TAU()) / ctx.sample_rate;
            self.phase = self.phase + phase_per_sample;
            // Check if we've crossed from positive phase back to negative:
            if self.phase >= Smp::PI() {
                self.phase = self.phase - Smp::TAU();
                self.update_rands();
            }
        }
        &self.outbuf[0..numsamples]
    }
}

impl<Smp: Float> Default for Lfo<Smp> {
    fn default() -> Self {
        Self::new(RANDOM_SEED)
    }
}

/// A fixed-point LFO parameter pack
pub struct LfoParamsFxP<'a> {
    /// The frequency of the LFO, in Hz
    pub freq: &'a [LfoFreqFxP],
    /// The depth of oscillation, as a number between 0 and 1
    pub depth: &'a [ScalarFxP],
    /// The options for this LFO (see [LfoOptions])
    pub opts: &'a [LfoOptions],
}

impl<'a> LfoParamsFxP<'a> {
    /// The length of this parameter pack is defined as the length of the
    /// smallest subslice
    pub fn len(&self) -> usize {
        min_size(&[self.freq.len(), self.depth.len(), self.opts.len()])
    }
    /// Returns true if any subslice is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A mutable fixed-point LFO parameter pack (see [LfoParamsFxP])
pub struct MutLfoParamsFxP<'a> {
    /// The frequency of the LFO, in Hz
    pub freq: &'a mut [LfoFreqFxP],
    /// The depth of oscillation, as a number between 0 and 1
    pub depth: &'a mut [ScalarFxP],
    /// The options for this LFO (see [LfoOptions])
    pub opts: &'a mut [LfoOptions],
}

impl<'a> MutLfoParamsFxP<'a> {
    /// The length of this parameter pack is defined as the length of the
    /// smallest subslice
    pub fn len(&self) -> usize {
        min_size(&[self.freq.len(), self.depth.len(), self.opts.len()])
    }
    /// Returns true if any subslice is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> From<MutLfoParamsFxP<'a>> for LfoParamsFxP<'a> {
    fn from(value: MutLfoParamsFxP<'a>) -> Self {
        Self {
            freq: value.freq,
            depth: value.depth,
            opts: value.opts,
        }
    }
}

/// A fixed-point LFO:
#[derive(Clone)]
pub struct LfoFxP {
    outbuf: BufferT<SampleFxP>,
    rng: SmallRng,
    rand_smps: [SampleFxP; 2],
    phase: PhaseFxP,
    last_gate: bool,
}

impl LfoFxP {
    /// Constructor
    pub fn new(seed: u64) -> LfoFxP {
        let mut retval = LfoFxP {
            outbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            rng: SmallRng::seed_from_u64(seed),
            rand_smps: [SampleFxP::ZERO; 2],
            phase: PhaseFxP::ZERO,
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
        self.rand_smps[0] = SampleFxP::from_num(rand_scalar);
    }
    /// Generate the LFO signal
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(
        &mut self,
        ctx: &ContextFxP,
        gate: &[SampleFxP],
        params: LfoParamsFxP,
    ) -> &[SampleFxP] {
        let freq = params.freq;
        let depth = params.depth;
        let opts = params.opts;
        let numsamples = min_size(&[
            freq.len(),
            gate.len(),
            opts.len(),
            depth.len(),
            STATIC_BUFFER_SIZE,
        ]);
        const FRAC_2_PI: ScalarFxP = ScalarFxP::lit("0x0.a2fa");
        for i in 0..numsamples {
            let this_gate = gate[i] > SampleFxP::lit("0.5");
            if opts[i].retrigger() && this_gate && !self.last_gate {
                self.phase = PhaseFxP::ZERO;
            }
            self.last_gate = this_gate;
            //generate waveforms (piecewise defined)
            let frac_2phase_pi = apply_scalar_i(SampleFxP::from_num(self.phase), FRAC_2_PI);
            let mut value = match opts[i].wave().unwrap_or_default() {
                LfoWave::Saw => frac_2phase_pi.unwrapped_shr(1),
                LfoWave::Square => {
                    if self.phase < 0 {
                        SampleFxP::NEG_ONE
                    } else {
                        SampleFxP::ONE
                    }
                }
                LfoWave::Triangle => {
                    if self.phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {
                        frac_2phase_pi.unwrapped_neg() - SampleFxP::lit("2")
                    } else if self.phase > PhaseFxP::FRAC_PI_2 {
                        SampleFxP::lit("2") - frac_2phase_pi
                    } else {
                        frac_2phase_pi
                    }
                }
                LfoWave::Sine => {
                    if self.phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {
                        // phase in [-pi, pi/2)
                        // Use the identity sin(x) = -cos(x+pi/2) since our taylor series
                        // approximations are centered about zero and this will be more accurate
                        fixedmath::cos_fixed(SampleFxP::from_num(self.phase + PhaseFxP::FRAC_PI_2))
                            .unwrapped_neg()
                    } else if self.phase < PhaseFxP::FRAC_PI_2 {
                        // phase in [pi/2, pi)
                        // sin(x) = cos(x-pi/2)
                        fixedmath::cos_fixed(SampleFxP::from_num(self.phase - PhaseFxP::FRAC_PI_2))
                    } else {
                        fixedmath::sin_fixed(SampleFxP::from_num(self.phase))
                    }
                }
                LfoWave::SampleHold => self.rand_smps[0],
                LfoWave::SampleGlide => {
                    self.rand_smps[0]
                        + SampleFxP::from_num(
                            SampleFxP::from_num(frac_2phase_pi)
                                .wide_mul(self.rand_smps[1] - self.rand_smps[0]),
                        )
                }
            };
            if !opts[i].bipolar() {
                value = (value + SampleFxP::ONE).unwrapped_shr(1);
            }
            self.outbuf[i] = SampleFxP::from_num(value.wide_mul_unsigned(depth[i]));
            let phase_per_sample = PhaseFxP::from_num(
                freq[i]
                    .wide_mul(ctx.sample_rate.frac_2pi4096_sr())
                    .unwrapped_shr(12),
            );
            self.phase += phase_per_sample;
            // Check if we've crossed from positive phase back to negative:
            if self.phase >= PhaseFxP::PI {
                self.phase -= PhaseFxP::TAU;
                self.update_rands();
            }
        }
        &self.outbuf[0..numsamples]
    }
}

impl Default for LfoFxP {
    fn default() -> Self {
        Self::new(RANDOM_SEED)
    }
}
