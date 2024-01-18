use super::*;

/// An iterator builder for [LfoParams]
///
/// Use this to easily build iterators to [LfoParams] out of iterators to
/// its constituent parts.
pub struct LfoParamIter<T: DspFormatBase, F, D, O>
where
    F: Iterator<Item = T::LfoFreq>,
    D: Iterator<Item = T::Scalar>,
    O: Iterator<Item = LfoOptions>,
{
    f: F,
    d: D,
    o: O,
    phantom: core::marker::PhantomData<T>,
}

impl<T: DspFormatBase, F, D, O> LfoParamIter<T, F, D, O>
where
    F: Iterator<Item = T::LfoFreq>,
    D: Iterator<Item = T::Scalar>,
    O: Iterator<Item = LfoOptions>,
{
    /// Replace the current frequenchy source with the one provided
    pub fn with_freq<New: Iterator<Item = T::LfoFreq>>(
        self,
        new: New,
    ) -> LfoParamIter<T, New, D, O> {
        LfoParamIter {
            f: new,
            d: self.d,
            o: self.o,
            phantom: self.phantom,
        }
    }
    /// Replace the current depth source with the one provided
    pub fn with_depth<New: Iterator<Item = T::Scalar>>(
        self,
        new: New,
    ) -> LfoParamIter<T, F, New, O> {
        LfoParamIter {
            f: self.f,
            d: new,
            o: self.o,
            phantom: self.phantom,
        }
    }
    /// Replace the current depth source with the one provided
    pub fn with_options<New: Iterator<Item = LfoOptions>>(
        self,
        new: New,
    ) -> LfoParamIter<T, F, D, New> {
        LfoParamIter {
            f: self.f,
            d: self.d,
            o: new,
            phantom: self.phantom,
        }
    }
}

impl<T: DspFormatBase, F, D, O> Iterator for LfoParamIter<T, F, D, O>
where
    F: Iterator<Item = T::LfoFreq>,
    D: Iterator<Item = T::Scalar>,
    O: Iterator<Item = LfoOptions>,
{
    type Item = LfoParams<T>;
    fn next(&mut self) -> Option<LfoParams<T>> {
        Some(LfoParams {
            freq: self.f.next()?,
            depth: self.d.next()?,
            opts: self.o.next()?,
        })
    }
}

/// Create a new [LfoParamIter], which initially creates instances of
/// [LfoParams] with frequency 1Hz, depth 1, and default [LfoOptions] until
/// calling the `with_*()` methods.
#[allow(clippy::type_complexity)]
pub fn new_lfo_param_iter<T: DspFormatBase>(
) -> LfoParamIter<T, Repeat<T::LfoFreq>, Repeat<T::Scalar>, Repeat<LfoOptions>> {
    LfoParamIter {
        f: repeat(T::LfoFreq::one()),
        d: repeat(T::Scalar::one()),
        o: repeat(LfoOptions::default()),
        phantom: Default::default(),
    }
}
