use super::*;

/// An iterator builder for [RingModInput]
///
/// Use this to easily build iterators to [RingModInput] out of iterators to
/// its constituent parts.
pub struct RingModInputIter<T, A, B>
where
    T: DspFormatBase,
    A: Iterator<Item = T::Sample>,
    B: Iterator<Item = T::Sample>,
{
    signal_a: A,
    signal_b: B,
    phantom: core::marker::PhantomData<T>,
}

impl<T, A, B> RingModInputIter<T, A, B>
where
    T: DspFormatBase,
    A: Iterator<Item = T::Sample>,
    B: Iterator<Item = T::Sample>,
{
    /// Replace the current carrier source with the one provided
    pub fn with_signal_a<New: Iterator<Item = T::Sample>>(
        self,
        new: New,
    ) -> RingModInputIter<T, New, B> {
        RingModInputIter {
            signal_a: new,
            signal_b: self.signal_b,
            phantom: self.phantom,
        }
    }
    /// Replace the current modulator source with the
    /// one provided
    pub fn with_signal_b<New: Iterator<Item = T::Sample>>(
        self,
        new: New,
    ) -> RingModInputIter<T, A, New> {
        RingModInputIter {
            signal_a: self.signal_a,
            signal_b: new,
            phantom: self.phantom,
        }
    }
}

impl<T, A, B> Iterator for RingModInputIter<T, A, B>
where
    T: DspFormatBase,
    A: Iterator<Item = T::Sample>,
    B: Iterator<Item = T::Sample>,
{
    type Item = RingModInput<T>;
    fn next(&mut self) -> Option<RingModInput<T>> {
        Some(RingModInput {
            signal_a: self.signal_a.next()?,
            signal_b: self.signal_b.next()?,
        })
    }
}

/// Create a new [RingModInputIter], which initially creates instances of
/// with zero signals for both the carrier and modulator
/// until calling the `with_*()` methods.
pub fn new_ringmod_input_iter<T: DspFormatBase>(
) -> RingModInputIter<T, Repeat<T::Sample>, Repeat<T::Sample>> {
    RingModInputIter {
        signal_a: repeat(T::Sample::zero()),
        signal_b: repeat(T::Sample::zero()),
        phantom: Default::default(),
    }
}

/// An iterator builder for [RingModParams]
///
/// Use this to easily build iterators to [RingModParams] out of iterators
/// to its constituent parts.
pub struct RingModParamIter<T, A, B, C>
where
    T: DspFormatBase,
    A: Iterator<Item = T::Scalar>,
    B: Iterator<Item = T::Scalar>,
    C: Iterator<Item = T::Scalar>,
{
    mix_a: A,
    mix_b: B,
    mix_mod: C,
    phantom: core::marker::PhantomData<T>,
}

impl<T, A, B, C> RingModParamIter<T, A, B, C>
where
    T: DspFormatBase,
    A: Iterator<Item = T::Scalar>,
    B: Iterator<Item = T::Scalar>,
    C: Iterator<Item = T::Scalar>,
{
    /// Replace the current carrier gain source with the one provided
    pub fn with_mix_a<New: Iterator<Item = T::Scalar>>(
        self,
        new: New,
    ) -> RingModParamIter<T, New, B, C> {
        RingModParamIter {
            mix_a: new,
            mix_b: self.mix_b,
            mix_mod: self.mix_mod,
            phantom: self.phantom,
        }
    }
    /// Replace the current modulator gain source with the one provided
    pub fn with_mix_b<New: Iterator<Item = T::Scalar>>(
        self,
        new: New,
    ) -> RingModParamIter<T, A, New, C> {
        RingModParamIter {
            mix_a: self.mix_a,
            mix_b: new,
            mix_mod: self.mix_mod,
            phantom: self.phantom,
        }
    }
    /// Replace the current output gain source with the one provided
    pub fn with_mix_mod<New: Iterator<Item = T::Scalar>>(
        self,
        new: New,
    ) -> RingModParamIter<T, A, B, New> {
        RingModParamIter {
            mix_a: self.mix_a,
            mix_b: self.mix_b,
            mix_mod: new,
            phantom: self.phantom,
        }
    }
}

impl<T, A, B, C> Iterator for RingModParamIter<T, A, B, C>
where
    T: DspFormatBase,
    A: Iterator<Item = T::Scalar>,
    B: Iterator<Item = T::Scalar>,
    C: Iterator<Item = T::Scalar>,
{
    type Item = RingModParams<T>;
    fn next(&mut self) -> Option<RingModParams<T>> {
        Some(RingModParams {
            mix_a: self.mix_a.next()?,
            mix_b: self.mix_b.next()?,
            mix_mod: self.mix_mod.next()?,
        })
    }
}

/// Create a new [RingModParamIter], which initially creates instances of
/// [RingModParams] with the unity gain for the carrier signal and zero gain
/// for the modulator and output signals
pub fn new_ringmod_param_iter<T: DspFormatBase>(
) -> RingModParamIter<T, Repeat<T::Scalar>, Repeat<T::Scalar>, Repeat<T::Scalar>> {
    RingModParamIter {
        mix_a: repeat(T::Scalar::one()),
        mix_b: repeat(T::Scalar::zero()),
        mix_mod: repeat(T::Scalar::zero()),
        phantom: Default::default(),
    }
}
