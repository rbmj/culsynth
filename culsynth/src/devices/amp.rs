use super::*;
use crate::{DspFormat, DspType};

#[derive(Default, Clone)]
pub struct Amp<T: DspFormat> {
    phantom: core::marker::PhantomData<T>,
}

impl<T: DspFormat> Device<T> for Amp<T> {
    type Input = T::Sample;
    type Params = T::Scalar;
    type Output = T::Sample;
    fn next(&mut self, _: &T::Context, signal: T::Sample, gain: T::Scalar) -> T::Sample {
        signal.scale(gain)
    }
}
