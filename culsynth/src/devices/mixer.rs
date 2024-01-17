use super::*;
use crate::{DspFormat, DspType};
use core::iter::zip;

#[derive(Clone, Default)]
pub struct Mixer<T: DspFormat, const N: usize> {
    phantom: core::marker::PhantomData<T>,
}

impl<T: DspFormat, const N: usize> Device<T> for Mixer<T, N> {
    type Input = [T::Sample; N];
    type Params = [T::Scalar; N];
    type Output = T::Sample;
    fn next(
        &mut self,
        context: &T::Context,
        input: Self::Input,
        params: Self::Params,
    ) -> T::Sample {
        T::narrow_sample(
            zip(input.iter(), params.iter())
                .fold(T::WideSample::default(), |acc, (signal, scale_factor)| {
                    acc + T::widen_sample(signal.scale(*scale_factor))
                }),
        )
    }
}
