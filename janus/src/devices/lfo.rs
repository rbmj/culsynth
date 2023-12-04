use super::osc::PhaseFxP;
use super::*;
use crate::fixedmath::apply_scalar_i;
use core::mem::transmute;
use core::option::Option;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

/// Default random seed to use if not provided a seed
const RANDOM_SEED: u64 = 0xce607a9d25ec3d88u64; //random 64 bit integer

#[repr(transparent)]
pub struct LfoParam {
    bits: u16,
}

impl LfoParam {
    const BIPOLAR: u16 = 1 << 8;
    const RETRIGGER: u16 = 1 << 9;
    pub fn wave(&self) -> Option<LfoWave> {
        let value = (self.bits & 0xFF) as u8;
        LfoWave::try_from(value).ok()
    }
    pub fn bipolar(&self) -> bool {
        self.bits & Self::BIPOLAR != 0
    }
    pub fn retrigger(&self) -> bool {
        self.bits & Self::RETRIGGER != 0
    }
    pub fn new(wave: LfoWave, bipolar: bool, retrigger: bool) -> Self {
        LfoParam {
            bits: (wave as u16)
                | if bipolar { Self::BIPOLAR } else { 0 }
                | if retrigger { Self::RETRIGGER } else { 0 },
        }
    }
}

#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum LfoWave {
    #[default]
    Sine,
    Square,
    Triangle,
    Saw,
    SampleHold,
    SampleGlide,
}

impl From<LfoWave> for &'static str {
    fn from(value: LfoWave) -> Self {
        [
            "Sine",
            "Square",
            "Triangle",
            "Saw",
            "Sample & Hold",
            "Sample & Glide",
        ][value as usize]
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
    pub fn process(
        &mut self,
        ctx: &Context<Smp>,
        freq: &[Smp],
        gate: &[Smp],
        params: &[LfoParam],
    ) -> &[Smp] {
        let numsamples = min_size(&[freq.len(), gate.len(), params.len(), STATIC_BUFFER_SIZE]);
        for i in 0..numsamples {
            let this_gate = gate[i] > Smp::ONE_HALF;
            if params[i].retrigger() && this_gate && !self.last_gate {
                self.phase = Smp::ZERO;
            }
            self.last_gate = this_gate;
            //generate waveforms (piecewise defined)
            let frac_2phase_pi = (self.phase + self.phase) / Smp::PI();
            let mut value = match params[i].wave().unwrap_or_default() {
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
                        Smp::cos(self.phase + Smp::FRAC_PI_2()).neg()
                    } else if self.phase < Smp::FRAC_PI_2() {
                        // phase in [pi/2, pi)
                        // sin(x) = cos(x-pi/2)
                        Smp::cos(self.phase - Smp::FRAC_PI_2())
                    } else {
                        Smp::sin(self.phase)
                    }
                }
                LfoWave::SampleHold => self.rand_smps[0],
                LfoWave::SampleGlide => {
                    self.rand_smps[0] + (frac_2phase_pi * (self.rand_smps[1] - self.rand_smps[0]))
                }
            };
            if !params[i].bipolar() {
                value = (value + Smp::ONE) / Smp::TWO;
            }
            self.outbuf[i] = value;
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

/// A fixed-point LFO:
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
        freq: &[LfoFreqFxP],
        gate: &[SampleFxP],
        params: &[LfoParam],
    ) -> &[SampleFxP] {
        let numsamples = min_size(&[freq.len(), gate.len(), params.len(), STATIC_BUFFER_SIZE]);
        const FRAC_2_PI: ScalarFxP = ScalarFxP::lit("0x0.a2fa");
        for i in 0..numsamples {
            let this_gate = gate[i] > SampleFxP::lit("0.5");
            if params[i].retrigger() && this_gate && !self.last_gate {
                self.phase = PhaseFxP::ZERO;
            }
            self.last_gate = this_gate;
            //generate waveforms (piecewise defined)
            let frac_2phase_pi = apply_scalar_i(SampleFxP::from_num(self.phase), FRAC_2_PI);
            let mut value = match params[i].wave().unwrap_or_default() {
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
            if !params[i].bipolar() {
                value = (value + SampleFxP::ONE).unwrapped_shr(1);
            }
            self.outbuf[i] = value;
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
