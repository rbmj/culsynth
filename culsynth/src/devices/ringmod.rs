use super::*;
use crate::{DspFormatBase, DspType, DspFloat};

#[derive(Clone, Default)]
pub struct RingModInput<T: DspFormatBase> {
    pub signal_a: T::Sample,
    pub signal_b: T::Sample,
}

#[derive(Clone, Default)]
pub struct RingModParams<T: DspFormatBase> {
    pub mix_a: T::Scalar,
    pub mix_b: T::Scalar,
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
