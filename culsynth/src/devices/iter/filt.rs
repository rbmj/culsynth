use super::*;

/// An iterater builder for [FiltParams]
///
/// By default (see [new_filt_param_iter]), this constructs instances of
/// [FiltParams] where the filter is fully open (cutoff is max) and zero
/// resonance.  This can then be modified by calling the `with_*` methods.
pub struct FiltParamIter<T, A, B>
where
    T: DspFormat,
    A: Iterator<Item = T::Note>,
    B: Iterator<Item = T::Scalar>,
{
    cutoff: A,
    resonance: B,
    phantom: core::marker::PhantomData<T>,
}

impl<T, A, B> FiltParamIter<T, A, B>
where
    T: DspFormat,
    A: Iterator<Item = T::Note>,
    B: Iterator<Item = T::Scalar>,
{
    /// Replace the filter cutoff with the provided iterator
    pub fn with_cutoff<New: Iterator<Item = T::Note>>(self, new: New) -> FiltParamIter<T, New, B> {
        FiltParamIter {
            phantom: self.phantom,
            cutoff: new,
            resonance: self.resonance,
        }
    }
    /// Replace the filter resonance with the provided resonance
    pub fn with_resonance<New>(self, new: New) -> FiltParamIter<T, A, New>
    where
        New: Iterator<Item = T::Scalar>,
    {
        FiltParamIter {
            phantom: self.phantom,
            cutoff: self.cutoff,
            resonance: new,
        }
    }
}

impl<T: DspFormat, Cutoff: Iterator<Item = T::Note>, Resonance: Iterator<Item = T::Scalar>> Iterator
    for FiltParamIter<T, Cutoff, Resonance>
{
    type Item = FiltParams<T>;
    fn next(&mut self) -> Option<FiltParams<T>> {
        Some(FiltParams {
            cutoff: self.cutoff.next()?,
            resonance: self.resonance.next()?,
        })
    }
}

/// Construct a [FiltParamIter]
pub fn new_filt_param_iter<T: DspFormat>() -> FiltParamIter<T, Repeat<T::Note>, Repeat<T::Scalar>> {
    FiltParamIter {
        phantom: Default::default(),
        cutoff: repeat(T::note_from_scalar(T::Scalar::one())),
        resonance: repeat(T::Scalar::zero()),
    }
}
