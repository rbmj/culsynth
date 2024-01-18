use super::*;
use crate::devices::env::detail;

/// An iterator builder for [EnvParams]
///
/// Use this to easily build iterators to [EnvParams] out of iterators to
/// its constituent parts.
pub struct EnvParamIter<T: DspFormatBase + detail::EnvOps, A, D, S, R>
where
    A: Iterator<Item = T::EnvParam>,
    D: Iterator<Item = T::EnvParam>,
    S: Iterator<Item = T::Scalar>,
    R: Iterator<Item = T::EnvParam>,
{
    a: A,
    d: D,
    s: S,
    r: R,
    phantom: core::marker::PhantomData<T>,
}

impl<T: DspFormatBase + detail::EnvOps, A, D, S, R> EnvParamIter<T, A, D, S, R>
where
    A: Iterator<Item = T::EnvParam>,
    D: Iterator<Item = T::EnvParam>,
    S: Iterator<Item = T::Scalar>,
    R: Iterator<Item = T::EnvParam>,
{
    /// Replace the current attack source with the one provided
    pub fn with_attack<NewA: Iterator<Item = T::EnvParam>>(
        self,
        newa: NewA,
    ) -> EnvParamIter<T, NewA, D, S, R> {
        EnvParamIter {
            a: newa,
            d: self.d,
            s: self.s,
            r: self.r,
            phantom: self.phantom,
        }
    }
    /// Replace the current decay source with the one provided
    pub fn with_decay<NewD: Iterator<Item = T::EnvParam>>(
        self,
        newd: NewD,
    ) -> EnvParamIter<T, A, NewD, S, R> {
        EnvParamIter {
            a: self.a,
            d: newd,
            s: self.s,
            r: self.r,
            phantom: self.phantom,
        }
    }
    /// Replace the current sustain source with the one provided
    pub fn with_sustain<NewS: Iterator<Item = T::Scalar>>(
        self,
        news: NewS,
    ) -> EnvParamIter<T, A, D, NewS, R> {
        EnvParamIter {
            a: self.a,
            d: self.d,
            s: news,
            r: self.r,
            phantom: self.phantom,
        }
    }
    /// Replace the current release source with the one provided
    pub fn with_release<NewR: Iterator<Item = T::EnvParam>>(
        self,
        newr: NewR,
    ) -> EnvParamIter<T, A, D, S, NewR> {
        EnvParamIter {
            a: self.a,
            d: self.d,
            s: self.s,
            r: newr,
            phantom: self.phantom,
        }
    }
}

impl<T, A, D, S, R> Iterator for EnvParamIter<T, A, D, S, R>
where
    T: DspFormatBase + detail::EnvOps,
    A: Iterator<Item = T::EnvParam>,
    D: Iterator<Item = T::EnvParam>,
    S: Iterator<Item = T::Scalar>,
    R: Iterator<Item = T::EnvParam>,
{
    type Item = EnvParams<T>;
    fn next(&mut self) -> Option<EnvParams<T>> {
        Some(EnvParams {
            attack: self.a.next()?,
            decay: self.d.next()?,
            sustain: self.s.next()?,
            release: self.r.next()?,
        })
    }
}

/// Create a new [EnvParamIter], which initially creates instances of
/// [EnvParams::default] until calling the `with_*()` methods.
pub fn new_env_param_iter<T: DspFormatBase + detail::EnvOps>(
) -> EnvParamIter<T, Repeat<T::EnvParam>, Repeat<T::EnvParam>, Repeat<T::Scalar>, Repeat<T::EnvParam>>
{
    EnvParamIter {
        a: repeat(T::ADR_DEFAULT),
        d: repeat(T::ADR_DEFAULT),
        s: repeat(T::Scalar::one()),
        r: repeat(T::ADR_DEFAULT),
        phantom: Default::default(),
    }
}
