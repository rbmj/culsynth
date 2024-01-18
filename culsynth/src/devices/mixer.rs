use super::*;
use core::iter::zip;

/// A signal mixer.  This is a convenience device to mix several signals
/// together easily.  The number of signals to mix together is passed in the
/// const-generic parameter `N`.
///
/// This implements [Device] taking an array of Samples as input, and an array
/// of Scalars as parameters.  Each Sample is multiplied by the corresponding
/// Scalar, and then the results are summed together in an output Sample.
///
/// In the fixed-point case, this device performs saturation checking rather
/// than wrapping or panicing on overflow.
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
        _context: &T::Context,
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
