use super::*;
use crate::{DspFormat, DspType};

/// A Voltage-Controlled Amplifier (VCA)
/// 
/// This is a fairly simple implementation of a voltage-controlled amplifier.
/// For simplicity's sake in interacting with some of the other devices, it
/// only functions as a 2-quadrant VCA with a gain between 0 and 1.  A full
/// 4-quadrant VCA with less restrictions on gain values will be implemented
/// in the future.
/// 
/// It implements [Device] taking a Sample as input, a Scalar parameter (the
/// gain) and outputting a Sample (see [DspFormat] for more information).
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
