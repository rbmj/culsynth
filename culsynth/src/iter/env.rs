use super::*;
use detail::{EnvMode, EnvOps, EnvType};

struct EnvState<T: DspFormat> {
    setpoint: T::EnvSignal,
    signal: T::EnvSignal,
    context: T::Context,
    mode: EnvMode,
}

impl<T: DspFormat> EnvState<T> {
    fn new(context: T::Context) -> Self {
        Self {
            setpoint: T::EnvSignal::default(),
            signal: T::EnvSignal::default(),
            context,
            mode: EnvMode::Release,
        }
    }
}

pub struct Env<
    T: DspFormat,
    Gate: Source<T::Sample>,
    Attack: Source<T::EnvParam>,
    Decay: Source<T::EnvParam>,
    Sustain: Source<T::Scalar>,
    Release: Source<T::EnvParam>,
> {
    gate: Gate,
    attack: Attack,
    decay: Decay,
    sustain: Sustain,
    release: Release,
    state: EnvState<T>,
}

impl<
        T: DspFormat,
        Gate: Source<T::Sample>,
        Attack: Source<T::EnvParam>,
        Decay: Source<T::EnvParam>,
        Sustain: Source<T::Scalar>,
        Release: Source<T::EnvParam>,
    > Env<T, Gate, Attack, Decay, Sustain, Release>
{
    pub fn with_gate<NewGate: Source<T::Sample>>(
        self,
        new_gate: NewGate,
    ) -> Env<T, NewGate, Attack, Decay, Sustain, Release> {
        Env {
            gate: new_gate,
            attack: self.attack,
            decay: self.decay,
            sustain: self.sustain,
            release: self.release,
            state: self.state,
        }
    }
    pub fn with_attack<NewAttack: Source<T::EnvParam>>(
        self,
        new_attack: NewAttack,
    ) -> Env<T, Gate, NewAttack, Decay, Sustain, Release> {
        Env {
            gate: self.gate,
            attack: new_attack,
            decay: self.decay,
            sustain: self.sustain,
            release: self.release,
            state: self.state,
        }
    }
    pub fn with_decay<NewDecay: Source<T::EnvParam>>(
        self,
        new_decay: NewDecay,
    ) -> Env<T, Gate, Attack, NewDecay, Sustain, Release> {
        Env {
            gate: self.gate,
            attack: self.attack,
            decay: new_decay,
            sustain: self.sustain,
            release: self.release,
            state: self.state,
        }
    }
    pub fn with_sustain<NewSustain: Source<T::Scalar>>(
        self,
        new_sustain: NewSustain,
    ) -> Env<T, Gate, Attack, Decay, NewSustain, Release> {
        Env {
            gate: self.gate,
            attack: self.attack,
            decay: self.decay,
            sustain: new_sustain,
            release: self.release,
            state: self.state,
        }
    }
    pub fn with_release<NewRelease: Source<T::EnvParam>>(
        self,
        new_release: NewRelease,
    ) -> Env<T, Gate, Attack, Decay, Sustain, NewRelease> {
        Env {
            gate: self.gate,
            attack: self.attack,
            decay: self.decay,
            sustain: self.sustain,
            release: new_release,
            state: self.state,
        }
    }
    pub fn iter<'a>(&'a mut self) -> EnvIter<'a, T, Gate, Attack, Decay, Sustain, Release> {
        EnvIter {
            env_state: &mut self.state,
            gate: self.gate.get(),
            attack: self.attack.get(),
            decay: self.decay.get(),
            sustain: self.sustain.get(),
            release: self.release.get(),
        }
    }
}

pub fn new<T: DspFormat>(
    context: T::Context,
) -> Env<
    T,
    IteratorSource<Repeat<T::Sample>>,
    IteratorSource<Repeat<T::EnvParam>>,
    IteratorSource<Repeat<T::EnvParam>>,
    IteratorSource<Repeat<T::Scalar>>,
    IteratorSource<Repeat<T::EnvParam>>,
> {
    Env {
        gate: repeat(T::Sample::zero()).into(),
        attack: repeat(T::ADR_DEFAULT).into(),
        decay: repeat(T::ADR_DEFAULT).into(),
        sustain: repeat(T::Scalar::one()).into(),
        release: repeat(T::ADR_DEFAULT).into(),
        state: EnvState::new(context),
    }
}

struct EnvIter<
    'a,
    T: DspFormat,
    Gate: Source<T::Sample> + 'a,
    Attack: Source<T::EnvParam> + 'a,
    Decay: Source<T::EnvParam> + 'a,
    Sustain: Source<T::Scalar> + 'a,
    Release: Source<T::EnvParam> + 'a,
> {
    env_state: &'a mut EnvState<T>,
    gate: Gate::It<'a>,
    attack: Attack::It<'a>,
    decay: Decay::It<'a>,
    sustain: Sustain::It<'a>,
    release: Release::It<'a>,
}

impl<
        'a,
        T: DspFormat,
        Gate: Source<T::Sample> + 'a,
        Attack: Source<T::EnvParam> + 'a,
        Decay: Source<T::EnvParam> + 'a,
        Sustain: Source<T::Scalar> + 'a,
        Release: Source<T::EnvParam> + 'a,
    > Iterator for EnvIter<'a, T, Gate, Attack, Decay, Sustain, Release>
{
    type Item = T::Scalar;
    fn next(&mut self) -> Option<Self::Item> {
        let gate = self.gate.next()?;
        let attack = self.attack.next()?;
        let decay = self.decay.next()?;
        let sustain = self.sustain.next()?;
        let release = self.release.next()?;
        let setpoint_old = self.env_state.setpoint;
        let last = self.env_state.signal;
        if gate <= T::GATE_THRESHOLD {
            self.env_state.mode = EnvMode::Release;
            self.env_state.setpoint = T::SIGNAL_MIN;
        } else if self.env_state.mode == EnvMode::Release {
            self.env_state.mode = EnvMode::Attack;
            self.env_state.setpoint = T::SIGNAL_MAX;
        } else if self.env_state.mode == EnvMode::Attack && last > T::ATTACK_THRESHOLD {
            self.env_state.mode = EnvMode::Decay;
        }
        let rise = if self.env_state.mode == EnvMode::Attack {
            attack
        } else if self.env_state.mode == EnvMode::Decay {
            // Need setpoint control here since the state transition will only
            // fire once, and we might be modulated
            self.env_state.setpoint = sustain.into();
            decay
        } else {
            release
        };
        self.env_state.signal = T::calc_env(
            &self.env_state.context,
            self.env_state.setpoint,
            setpoint_old,
            last,
            rise,
        );
        Some(self.env_state.signal.to_scalar())
    }
}

impl<T: DspFloat> EnvOps for T {
    const SIGNAL_MIN: T = T::ZERO;
    const SIGNAL_MAX: T = T::ONE;
    const ATTACK_THRESHOLD: T = T::POINT_NINE_EIGHT;
    const GATE_THRESHOLD: T = T::ONE_HALF;
    const ADR_DEFAULT: T = T::POINT_ONE;
    fn calc_env(context: &Context<T>, setpoint: T, setpoint_old: T, last: T, rise_time: T) -> T {
        // This is equivalen to saying rise time = 4 time constants...
        let k = rise_time * (context.sample_rate / T::TWO) + T::ONE;
        let pro = setpoint_old + setpoint - last - last;
        let delta = pro / k;
        last + delta
    }
}

impl EnvOps for i16 {
    const GATE_THRESHOLD: Sample = Sample::lit("0.5");
    const ATTACK_THRESHOLD: EnvSignalFxP = EnvSignalFxP::lit("0.98");
    const SIGNAL_MAX: EnvSignalFxP = EnvSignalFxP::lit("0x0.FFFC");
    const SIGNAL_MIN: EnvSignalFxP = EnvSignalFxP::lit("0x0.0004");
    const ADR_DEFAULT: EnvParamFxP = EnvParamFxP::lit("0.1");
    fn calc_env(
        context: &ContextFxP,
        setpoint: EnvSignalFxP,
        setpoint_old: EnvSignalFxP,
        last: EnvSignalFxP,
        rise_time: EnvParamFxP,
    ) -> EnvSignalFxP {
        use crate::fixedmath::{one_over_one_plus, I2F14, U16F0};
        // This is equivalent to saying rise time = 4 time constants...
        let sr = U16F0::from_bits(context.sample_rate.value() >> 1);
        let k = rise_time.wide_mul(sr);
        let (gain, shift) = one_over_one_plus(k);
        // Need saturating here to avoid panic if A == 0 && S == 0:
        let pro = I2F14::saturating_from_num(setpoint_old + setpoint - last.unwrapped_shl(1));
        let delta = pro.wide_mul_unsigned(gain).unwrapped_shr(shift);
        last + delta
    }
}
