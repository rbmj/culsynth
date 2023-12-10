use super::*;

use super::EnvParamFxP;

/// Internal tag representing the current state of the envelope
#[derive(Eq, PartialEq)]
enum EnvState {
    Release,
    Attack,
    Decay,
}

/// A wrapper struct for passing parameters into the floating-point envelope
/// generator (see [Env]).  Note that the time parameters are not
/// strictly time-accurate - the goal here is to give more of a qualitative feel
/// for the range of the parameters than allow for precise timing.  If precise
/// timing is desired, the data displayed to the user can be refined on the UI
/// side.
///
/// TODO:  Determine formula for converting to a precise rise/fall time
pub struct EnvParams<'a, Smp> {
    /// The attack (rise to peak) time of the envelope, in seconds.
    pub attack: &'a [Smp],
    /// The decay (fall from peak to steady state) time of the envelope, in seconds.
    pub decay: &'a [Smp],
    /// The sustain level of the envelops, as a number between 0 and 1.
    pub sustain: &'a [Smp],
    /// The release time of the envelope, in seconds.
    pub release: &'a [Smp],
}

impl<'a, Smp> EnvParams<'a, Smp> {
    /// The length of this parameter pack, defined as the shortest length
    /// among all the constituent slices.
    pub fn len(&self) -> usize {
        min_size(&[
            self.attack.len(),
            self.decay.len(),
            self.sustain.len(),
            self.release.len(),
        ])
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct MutEnvParams<'a, Smp> {
    /// The attack (rise to peak) time of the envelope, in seconds.
    pub attack: &'a mut [Smp],
    /// The decay (fall from peak to steady state) time of the envelope, in seconds.
    pub decay: &'a mut [Smp],
    /// The sustain level of the envelops, as a number between 0 and 1.
    pub sustain: &'a mut [Smp],
    /// The release time of the envelope, in seconds.
    pub release: &'a mut [Smp],
}

impl<'a, Smp> MutEnvParams<'a, Smp> {
    /// The length of this parameter pack, defined as the shortest length
    /// among all the constituent slices.
    pub fn len(&self) -> usize {
        min_size(&[
            self.attack.len(),
            self.decay.len(),
            self.sustain.len(),
            self.release.len(),
        ])
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, Smp: Float> From<MutEnvParams<'a, Smp>> for EnvParams<'a, Smp> {
    fn from(value: MutEnvParams<'a, Smp>) -> Self {
        Self {
            attack: value.attack,
            decay: value.decay,
            sustain: value.sustain,
            release: value.release,
        }
    }
}

/// A floating-point ADSR envelope generator.  See [EnvParams] for the definitions
/// of the parameters
pub struct Env<Smp> {
    state: EnvState,
    outbuf: BufferT<Smp>,
    setpoint: Smp,
    last: Smp,
}

impl<Smp: Float> Env<Smp> {
    // much silliness because generics...
    const GATE_THRESHOLD: Smp = Smp::ONE_HALF;
    const ATTACK_THRESHOLD: Smp = Smp::POINT_NINE_EIGHT;
    const SIGNAL_MAX: Smp = Smp::ONE;
    const SIGNAL_MIN: Smp = Smp::ZERO;
    /// Constructor
    pub fn new() -> Self {
        Self {
            state: EnvState::Release,
            outbuf: [Smp::ZERO; STATIC_BUFFER_SIZE],
            setpoint: Self::SIGNAL_MIN,
            last: Self::SIGNAL_MIN,
        }
    }
    /// Process the gate input and return an envelope signal according to the ADSR
    /// parameters passed into the function.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(&mut self, ctx: &Context<Smp>, gate: &[Smp], params: EnvParams<Smp>) -> &[Smp] {
        let attack = params.attack;
        let decay = params.decay;
        let sustain = params.sustain;
        let release = params.release;
        let numsamples = min_size(&[
            attack.len(),
            decay.len(),
            sustain.len(),
            release.len(),
            gate.len(),
            STATIC_BUFFER_SIZE,
        ]);
        let setpoint_old = self.setpoint;
        for i in 0..numsamples {
            if gate[i] <= Self::GATE_THRESHOLD {
                self.state = EnvState::Release;
                self.setpoint = Self::SIGNAL_MIN;
            } else if self.state == EnvState::Release {
                self.state = EnvState::Attack;
                self.setpoint = Self::SIGNAL_MAX;
            } else if self.state == EnvState::Attack && self.last > Self::ATTACK_THRESHOLD {
                self.state = EnvState::Decay;
            }
            let rise = if self.state == EnvState::Attack {
                attack[i]
            } else if self.state == EnvState::Decay {
                self.setpoint = sustain[i];
                decay[i]
            } else {
                release[i]
            };
            // This is equivalen to saying rise time = 4 time constants...
            let k = rise * (ctx.sample_rate / Smp::TWO) + Smp::ONE;
            let pro = setpoint_old + self.setpoint - self.last - self.last;
            let delta = pro / k;
            self.last = self.last + delta;
            self.outbuf[i] = self.last;
        }
        &self.outbuf[0..numsamples]
    }
}

impl<Smp: Float> Default for Env<Smp> {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper struct for passing parameters into the fixed-point envelope
/// generator (see [EnvFxP]).  Note that the time parameters are not
/// strictly time-accurate - the goal here is to give more of a qualitative feel
/// for the range of the parameters than allow for precise timing.  If precise
/// timing is desired, the data displayed to the user can be refined on the UI
/// side.
///
/// TODO:  Determine formula for converting to a precise rise/fall time
pub struct EnvParamsFxP<'a> {
    /// The attack (rise) time of the envelope, in seconds, as a fixed point number
    /// (see [EnvParamFxP]).
    pub attack: &'a [EnvParamFxP],
    /// The decay (fall from peak to steady state) time of the envelope, in seconds,
    /// as a fixed point number (see [EnvParamFxP]).
    pub decay: &'a [EnvParamFxP],
    /// The steady state sustain level of the envelope, as a fixed point number
    /// between zero and one.  Note that while zero is included in this interval
    /// one is not - so there will technically be a very, very small loss in signal
    /// associated with the envelope (approximately 0.00013dB... I'm not worried)
    pub sustain: &'a [ScalarFxP],
    /// The release (fall from steady state to zero) time of the envelope, in seconds,
    /// as a fixed point number (see [EnvParamFxP]).
    pub release: &'a [EnvParamFxP],
}

impl<'a> EnvParamsFxP<'a> {
    /// The length of this parameter pack, defined as the shortest length
    /// among all the constituent slices.
    pub fn len(&self) -> usize {
        min_size(&[
            self.attack.len(),
            self.decay.len(),
            self.sustain.len(),
            self.release.len(),
        ])
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct MutEnvParamsFxP<'a> {
    /// The attack (rise) time of the envelope, in seconds, as a fixed point number
    /// (see [EnvParamFxP]).
    pub attack: &'a mut [EnvParamFxP],
    /// The decay (fall from peak to steady state) time of the envelope, in seconds,
    /// as a fixed point number (see [EnvParamFxP]).
    pub decay: &'a mut [EnvParamFxP],
    /// The steady state sustain level of the envelope, as a fixed point number
    /// between zero and one.  Note that while zero is included in this interval
    /// one is not - so there will technically be a very, very small loss in signal
    /// associated with the envelope (approximately 0.00013dB... I'm not worried)
    pub sustain: &'a mut [ScalarFxP],
    /// The release (fall from steady state to zero) time of the envelope, in seconds,
    /// as a fixed point number (see [EnvParamFxP]).
    pub release: &'a mut [EnvParamFxP],
}

impl<'a> MutEnvParamsFxP<'a> {
    /// The length of this parameter pack, defined as the shortest length
    /// among all the constituent slices.
    pub fn len(&self) -> usize {
        min_size(&[
            self.attack.len(),
            self.decay.len(),
            self.sustain.len(),
            self.release.len(),
        ])
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> From<MutEnvParamsFxP<'a>> for EnvParamsFxP<'a> {
    fn from(value: MutEnvParamsFxP<'a>) -> Self {
        Self {
            attack: value.attack,
            decay: value.decay,
            sustain: value.sustain,
            release: value.release,
        }
    }
}

/// A fixed point ADSR envelope generator.  See [EnvParamsFxP] for the fixed point
/// definitions of the parameters.
pub struct EnvFxP {
    state: EnvState,
    outbuf: BufferT<ScalarFxP>,
    setpoint: fixedmath::I3F29,
    last: fixedmath::I3F29,
}

impl EnvFxP {
    const GATE_THRESHOLD: SampleFxP = SampleFxP::lit("0.5");
    const ATTACK_THRESHOLD: ScalarFxP = ScalarFxP::lit("0.98");
    const SIGNAL_MAX: ScalarFxP = ScalarFxP::lit("0x0.FFFC");
    const SIGNAL_MIN: fixedmath::I3F29 = fixedmath::I3F29::lit("0x0.0004");
    /// Constructor
    pub fn new() -> Self {
        Self {
            state: EnvState::Release,
            outbuf: [ScalarFxP::ZERO; STATIC_BUFFER_SIZE],
            setpoint: Self::SIGNAL_MIN,
            last: Self::SIGNAL_MIN,
        }
    }
    /// Process the gate input and return an envelope signal according to the ADSR
    /// parameters passed into the function.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(
        &mut self,
        ctx: &ContextFxP,
        gate: &[SampleFxP],
        params: EnvParamsFxP,
    ) -> &[ScalarFxP] {
        let attack = params.attack;
        let decay = params.decay;
        let sustain = params.sustain;
        let release = params.release;
        let numsamples = min_size(&[
            attack.len(),
            decay.len(),
            sustain.len(),
            release.len(),
            gate.len(),
            STATIC_BUFFER_SIZE,
        ]);
        let setpoint_old = self.setpoint;
        for i in 0..numsamples {
            if gate[i] <= Self::GATE_THRESHOLD {
                self.state = EnvState::Release;
                self.setpoint = Self::SIGNAL_MIN;
            } else if self.state == EnvState::Release {
                self.state = EnvState::Attack;
                self.setpoint = fixedmath::I3F29::from_num(Self::SIGNAL_MAX);
            } else if self.state == EnvState::Attack && self.last > Self::ATTACK_THRESHOLD {
                self.state = EnvState::Decay;
            }
            let rise = if self.state == EnvState::Attack {
                attack[i]
            } else if self.state == EnvState::Decay {
                self.setpoint = core::cmp::max(
                    fixedmath::I3F29::from_num(core::cmp::min(sustain[i], Self::SIGNAL_MAX)),
                    Self::SIGNAL_MIN,
                );
                decay[i]
            } else {
                release[i]
            };
            // This is equivalen to saying rise time = 4 time constants...
            let sr = fixedmath::U16F0::from_bits(ctx.sample_rate.value() >> 1);
            let k = rise.wide_mul(sr);
            let (gain, shift) = fixedmath::one_over_one_plus(k);
            // Need saturating here to avoid panic if A == 0 && S == 0:
            let pro = fixedmath::I2F14::saturating_from_num(
                setpoint_old + self.setpoint - self.last.unwrapped_shl(1),
            );
            let delta = pro.wide_mul_unsigned(gain).unwrapped_shr(shift);
            self.last += delta;
            self.outbuf[i] = ScalarFxP::saturating_from_num(self.last);
        }
        &self.outbuf[0..numsamples]
    }
}

impl Default for EnvFxP {
    fn default() -> Self {
        Self::new()
    }
}
