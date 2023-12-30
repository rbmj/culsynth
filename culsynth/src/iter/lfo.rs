use super::*;
use detail::{LfoOps, OscOps};
use rand::{rngs::SmallRng, RngCore, SeedableRng};

pub use crate::devices::{LfoOptions, LfoWave};

struct LfoState<T: DspFormat> {
    rng: SmallRng,
    context: T::Context,
    phase: T::Phase,
    rand_smps: [T::Sample; 2],
    last_gate: bool,
}

impl<T: DspFormat> LfoState<T> {
    fn new(context: T::Context, seed: u64) -> Self {
        let mut ret = LfoState {
            rng: SmallRng::seed_from_u64(seed),
            context,
            phase: T::Phase::zero(),
            rand_smps: [T::Sample::zero(); 2],
            last_gate: false,
        };
        ret.update_rands();
        ret.update_rands();
        ret
    }
    fn update_rands(&mut self) {
        let mut bytes = [0u8; 2];
        self.rng.fill_bytes(&mut bytes);
        let rand_iscalar = IScalarFxP::from_ne_bytes(bytes);
        self.rand_smps[0] = self.rand_smps[1];
        self.rand_smps[1] = T::sample_from_fixed(rand_iscalar);
    }
}

pub struct Lfo<
    T: DspFormat,
    Gate: Source<T::Sample>,
    Freq: Source<T::LfoFreq>,
    Depth: Source<T::Scalar>,
    Opts: Source<LfoOptions>,
> {
    state: LfoState<T>,
    gate: Gate,
    freq: Freq,
    depth: Depth,
    opts: Opts,
}

impl<
        T: DspFormat,
        Gate: Source<T::Sample>,
        Freq: Source<T::LfoFreq>,
        Depth: Source<T::Scalar>,
        Opts: Source<LfoOptions>,
    > Lfo<T, Gate, Freq, Depth, Opts>
{
    pub fn with_gate<NewGate: Source<T::Sample>>(
        self,
        new_gate: NewGate,
    ) -> Lfo<T, NewGate, Freq, Depth, Opts> {
        Lfo {
            state: self.state,
            gate: new_gate,
            freq: self.freq,
            depth: self.depth,
            opts: self.opts,
        }
    }
    pub fn with_freq<NewFreq: Source<T::LfoFreq>>(
        self,
        new_freq: NewFreq,
    ) -> Lfo<T, Gate, NewFreq, Depth, Opts> {
        Lfo {
            state: self.state,
            gate: self.gate,
            freq: new_freq,
            depth: self.depth,
            opts: self.opts,
        }
    }
    pub fn with_depth<NewDepth: Source<T::Scalar>>(
        self,
        new_depth: NewDepth,
    ) -> Lfo<T, Gate, Freq, NewDepth, Opts> {
        Lfo {
            state: self.state,
            gate: self.gate,
            freq: self.freq,
            depth: new_depth,
            opts: self.opts,
        }
    }
    pub fn with_options<NewOpts: Source<LfoOptions>>(
        self,
        new_opts: NewOpts,
    ) -> Lfo<T, Gate, Freq, Depth, NewOpts> {
        Lfo {
            state: self.state,
            gate: self.gate,
            freq: self.freq,
            depth: self.depth,
            opts: new_opts,
        }
    }
}

pub fn new<T: DspFormat>(
    context: T::Context,
    seed: u64,
) -> Lfo<
    T,
    IteratorSource<Repeat<T::Sample>>,
    IteratorSource<Repeat<T::LfoFreq>>,
    IteratorSource<Repeat<T::Scalar>>,
    IteratorSource<Repeat<LfoOptions>>,
> {
    Lfo {
        state: LfoState::new(context, seed),
        gate: repeat(T::Sample::zero()).into(),
        freq: repeat(T::LfoFreq::one()).into(),
        depth: repeat(T::Scalar::one()).into(),
        opts: repeat(LfoOptions::default()).into(),
    }
}

pub struct LfoIter<
    'a,
    T: DspFormat,
    Gate: Source<T::Sample> + 'a,
    Freq: Source<T::LfoFreq> + 'a,
    Depth: Source<T::Scalar> + 'a,
    Opts: Source<LfoOptions> + 'a,
> {
    state: &'a mut LfoState<T>,
    gate: Gate::It<'a>,
    freq: Freq::It<'a>,
    depth: Depth::It<'a>,
    opts: Opts::It<'a>,
}

impl<
        'a,
        T: DspFormat,
        Gate: Source<T::Sample> + 'a,
        Freq: Source<T::LfoFreq> + 'a,
        Depth: Source<T::Scalar> + 'a,
        Opts: Source<LfoOptions> + 'a,
    > Iterator for LfoIter<'a, T, Gate, Freq, Depth, Opts>
{
    type Item = T::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        let gate = self.gate.next()?;
        let freq = self.freq.next()?;
        let depth = self.depth.next()?;
        let opts = self.opts.next()?;
        let this_gate = gate > T::GATE_THRESHOLD;
        if opts.retrigger() && this_gate && !self.state.last_gate {
            self.state.phase = T::Phase::zero();
        }
        self.state.last_gate = this_gate;
        let mut value = T::calc_lfo(
            self.state.phase,
            opts.wave().unwrap_or_default(),
            &self.state.rand_smps,
        );

        if !opts.bipolar() {
            value = (value + T::Sample::one()).divide_by_two();
        }
        value = T::scale_sample(value, depth);
        self.state.phase = self.state.phase + T::phase_per_smp(&self.state.context, freq);
        // Check if we've crossed from positive phase back to negative:
        if self.state.phase >= T::Phase::PI {
            self.state.phase = self.state.phase - T::Phase::TAU;
            self.state.update_rands();
        }

        Some(value)
    }
}

impl LfoOps for i16 {
    fn calc_lfo(phase: Self::Phase, wave: lfo::LfoWave, rands: &[Self::Sample; 2]) -> Self::Sample {
        use crate::fixedmath::{apply_scalar_i, cos_fixed, sin_fixed};
        const TWO: Sample = Sample::lit("2");
        let frac_2phase_pi = apply_scalar_i(Sample::from_num(phase), Self::FRAC_2_PI);
        match wave {
            LfoWave::Saw => frac_2phase_pi.unwrapped_shr(1),
            LfoWave::Square => {
                if phase < 0 {
                    Sample::NEG_ONE
                } else {
                    Sample::ONE
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
                    cos_fixed(Sample::from_num(phase + PhaseFxP::FRAC_PI_2)).unwrapped_neg()
                } else if phase < PhaseFxP::FRAC_PI_2 {
                    // phase in [pi/2, pi)
                    // sin(x) = cos(x-pi/2)
                    cos_fixed(Sample::from_num(phase - PhaseFxP::FRAC_PI_2))
                } else {
                    sin_fixed(Sample::from_num(phase))
                }
            }
            LfoWave::SampleHold => rands[0],
            LfoWave::SampleGlide => {
                rands[0] + Sample::multiply(frac_2phase_pi, rands[1] - rands[0])
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

impl<T: DspFloat> LfoOps for T {
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
