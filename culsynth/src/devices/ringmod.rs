use super::*;
use crate::{DspFormatBase, DspType, DspFloat};

/// Input for a [RingMod].
#[derive(Clone, Default)]
pub struct RingModInput<T: DspFormatBase> {
    /// The first (carrier) input signal
    pub signal_a: T::Sample,
    /// The second (modulator) input signal
    pub signal_b: T::Sample,
}

/// Params for a [RingMod]
#[derive(Clone, Default)]
pub struct RingModParams<T: DspFormatBase> {
    /// Gain of the original first (carrier) signal, to be mixed
    /// back into the device's output.
    pub mix_a: T::Scalar,
    /// Gain of the original second (modulator) signal, to be mixed
    /// back into the device's output.
    pub mix_b: T::Scalar,
    /// Gain of the modulated result to be mixed into the device's output
    pub mix_mod: T::Scalar,
}

impl<T: DspFloat> From<&RingModParams<i16>> for RingModParams<T> {
    fn from(value: &RingModParams<i16>) -> Self {
        Self {
            mix_a: value.mix_a.to_num(),
            mix_b: value.mix_b.to_num(),
            mix_mod: value.mix_mod.to_num(),
        }
    }
}

/// A Ring Modulator and Mixer
/// 
/// This supports ring modulation of two signals and control over the mix of
/// both the input signals and the modulation signal in the final device output
/// 
/// This implements [Device], taking a [RingModInput] as input and
/// [RingModParams] as parameters and outputting a Sample.
#[derive(Clone, Default)]
pub struct RingMod<T: DspFormat> {
    mixer: Mixer<T, 3>,
}

impl<T: DspFormat> Device<T> for RingMod<T> {
    type Input = RingModInput<T>;
    type Params = RingModParams<T>;
    type Output = T::Sample;
    fn next(
        &mut self,
        context: &T::Context,
        input: RingModInput<T>,
        params: RingModParams<T>,
    ) -> T::Sample {
        let ring = input.signal_a.multiply(input.signal_b);
        self.mixer.next(
            context,
            [input.signal_a, input.signal_b, ring],
            [params.mix_a, params.mix_b, params.mix_mod],
        )
    }
}
