use super::*;
pub(crate) mod detail {
    use super::*;

    pub use crate::fixedmath::I3F29 as EnvSignalFxP;

    #[derive(Eq, PartialEq, Clone, Copy, Default)]
    pub enum EnvMode {
        #[default]
        Release,
        Attack,
        Decay,
    }

    pub trait EnvType<T: DspFormatBase>: Copy + Default + From<T::Scalar> + PartialOrd {
        fn to_scalar(self) -> T::Scalar;
    }

    impl EnvType<i16> for EnvSignalFxP {
        fn to_scalar(self) -> ScalarFxP {
            ScalarFxP::saturating_from_num(self)
        }
    }

    impl<T: crate::Float + Send> EnvType<T> for T
    where
        T: From<crate::IScalarFxP> + From<crate::NoteFxP>,
    {
        fn to_scalar(self) -> Self {
            self
        }
    }

    pub trait EnvOps: crate::DspFormatBase {
        const SIGNAL_MIN: Self::EnvSignal;
        const SIGNAL_MAX: Self::EnvSignal;
        const ATTACK_THRESHOLD: Self::EnvSignal;
        const GATE_THRESHOLD: Self::Sample;
        const ADR_DEFAULT: Self::EnvParam;
        fn calc_env(
            context: &Self::Context,
            setpoint: Self::EnvSignal,
            setpoint_old: Self::EnvSignal,
            last: Self::EnvSignal,
            rise_time: Self::EnvParam,
        ) -> Self::EnvSignal;
    }
}

use detail::{EnvMode, EnvSignalFxP, EnvType};

/// Parameters for an [Env].  Note that the time parameters are not
/// strictly time-accurate - the goal here is to give more of a qualitative feel
/// for the range of the parameters than allow for precise timing.  If precise
/// timing is desired, the data displayed to the user can be refined on the UI
/// side.
///
/// TODO:  Determine formula for converting to a precise rise/fall time
#[derive(Clone)]
pub struct EnvParams<T: DspFormatBase> {
    /// Attack time, in seconds (approx)
    pub attack: T::EnvParam,
    /// Decay time, in seconds (approx)
    pub decay: T::EnvParam,
    /// Sustain level, between 0 and 1
    pub sustain: T::Scalar,
    /// Release time, in seconds (approx)
    pub release: T::EnvParam,
}

impl<T: DspFormatBase + detail::EnvOps> Default for EnvParams<T> {
    fn default() -> Self {
        Self {
            attack: T::ADR_DEFAULT,
            decay: T::ADR_DEFAULT,
            sustain: T::Scalar::one(),
            release: T::ADR_DEFAULT,
        }
    }
}

impl<T: DspFloat> From<&EnvParams<i16>> for EnvParams<T> {
    fn from(value: &EnvParams<i16>) -> Self {
        EnvParams::<T> {
            attack: value.attack.to_num(),
            decay: value.decay.to_num(),
            sustain: value.sustain.to_num(),
            release: value.release.to_num(),
        }
    }
}

/// An ADSR Envelope Generator
#[derive(Clone, Default)]
pub struct Env<T: DspFormatBase + detail::EnvOps> {
    setpoint: T::EnvSignal,
    signal: T::EnvSignal,
    mode: EnvMode,
}

impl<T: DspFormat> Device<T> for Env<T> {
    type Input = T::Sample;
    type Params = EnvParams<T>;
    type Output = T::Scalar;
    fn next(&mut self, context: &T::Context, gate: T::Sample, params: EnvParams<T>) -> T::Scalar {
        let setpoint_old = self.setpoint;
        if gate <= T::GATE_THRESHOLD {
            self.mode = EnvMode::Release;
            self.setpoint = T::SIGNAL_MIN;
        } else if self.mode == EnvMode::Release {
            self.mode = EnvMode::Attack;
            self.setpoint = T::SIGNAL_MAX;
        } else if self.mode == EnvMode::Attack && self.signal > T::ATTACK_THRESHOLD {
            self.mode = EnvMode::Decay;
        }
        let rise = match self.mode {
            EnvMode::Attack => params.attack,
            EnvMode::Decay => {
                // Need setpoint control here since the state transition will only
                // fire once, and we might be modulated
                self.setpoint = params.sustain.into();
                params.decay
            }
            EnvMode::Release => params.release,
        };
        self.signal = T::calc_env(context, self.setpoint, setpoint_old, self.signal, rise);
        self.signal.to_scalar()
    }
}

impl<T: DspFloat> detail::EnvOps for T {
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

impl detail::EnvOps for i16 {
    const GATE_THRESHOLD: SampleFxP = SampleFxP::lit("0.5");
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
