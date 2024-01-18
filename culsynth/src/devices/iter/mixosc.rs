use super::*;

/// An iterator builder for [MixOscParams]
///
/// Use this to easily build iterators to [MixOscParams] out of iterators to
/// its constituent parts.
pub struct MixOscParamIter<T, A, B, C, D, E, F>
where
    T: DspFormatBase,
    A: Iterator<Item = T::NoteOffset>,
    B: Iterator<Item = T::Scalar>,
    C: Iterator<Item = T::Scalar>,
    D: Iterator<Item = T::Scalar>,
    E: Iterator<Item = T::Scalar>,
    F: Iterator<Item = T::Scalar>,
{
    tune: A,
    shape: B,
    sin: C,
    sq: D,
    tri: E,
    saw: F,
    phantom: core::marker::PhantomData<T>,
}

impl<T, A, B, C, D, E, F> MixOscParamIter<T, A, B, C, D, E, F>
where
    T: DspFormatBase,
    A: Iterator<Item = T::NoteOffset>,
    B: Iterator<Item = T::Scalar>,
    C: Iterator<Item = T::Scalar>,
    D: Iterator<Item = T::Scalar>,
    E: Iterator<Item = T::Scalar>,
    F: Iterator<Item = T::Scalar>,
{
    /// Replace the current tuning source with the one provided
    pub fn with_tune<New: Iterator<Item = T::NoteOffset>>(
        self,
        new: New,
    ) -> MixOscParamIter<T, New, B, C, D, E, F> {
        MixOscParamIter {
            tune: new,
            shape: self.shape,
            sin: self.sin,
            sq: self.sq,
            tri: self.tri,
            saw: self.saw,
            phantom: self.phantom,
        }
    }
    /// Replace the current wave shape (phase distortion) source with the
    /// one provided
    pub fn with_shape<New: Iterator<Item = T::Scalar>>(
        self,
        new: New,
    ) -> MixOscParamIter<T, A, New, C, D, E, F> {
        MixOscParamIter {
            tune: self.tune,
            shape: new,
            sin: self.sin,
            sq: self.sq,
            tri: self.tri,
            saw: self.saw,
            phantom: self.phantom,
        }
    }
    /// Replace the current sine wave gain source with the one provided
    pub fn with_sin<New: Iterator<Item = T::Scalar>>(
        self,
        new: New,
    ) -> MixOscParamIter<T, A, B, New, D, E, F> {
        MixOscParamIter {
            tune: self.tune,
            shape: self.shape,
            sin: new,
            sq: self.sq,
            tri: self.tri,
            saw: self.saw,
            phantom: self.phantom,
        }
    }
    /// Replace the current square wave gain source with the one provided
    pub fn with_sq<New: Iterator<Item = T::Scalar>>(
        self,
        new: New,
    ) -> MixOscParamIter<T, A, B, C, New, E, F> {
        MixOscParamIter {
            tune: self.tune,
            shape: self.shape,
            sin: self.sin,
            sq: new,
            tri: self.tri,
            saw: self.saw,
            phantom: self.phantom,
        }
    }
    /// Replace the current triangle wave gain source with the one provided
    pub fn with_tri<New: Iterator<Item = T::Scalar>>(
        self,
        new: New,
    ) -> MixOscParamIter<T, A, B, C, D, New, F> {
        MixOscParamIter {
            tune: self.tune,
            shape: self.shape,
            sin: self.sin,
            sq: self.sq,
            tri: new,
            saw: self.saw,
            phantom: self.phantom,
        }
    }
    /// Replace the current sawtooth wave gain source with the one provided
    pub fn with_saw<New: Iterator<Item = T::Scalar>>(
        self,
        new: New,
    ) -> MixOscParamIter<T, A, B, C, D, E, New> {
        MixOscParamIter {
            tune: self.tune,
            shape: self.shape,
            sin: self.sin,
            sq: self.sq,
            tri: self.tri,
            saw: new,
            phantom: self.phantom,
        }
    }
}

impl<T, A, B, C, D, E, F> Iterator for MixOscParamIter<T, A, B, C, D, E, F>
where
    T: DspFormatBase,
    A: Iterator<Item = T::NoteOffset>,
    B: Iterator<Item = T::Scalar>,
    C: Iterator<Item = T::Scalar>,
    D: Iterator<Item = T::Scalar>,
    E: Iterator<Item = T::Scalar>,
    F: Iterator<Item = T::Scalar>,
{
    type Item = MixOscParams<T>;
    fn next(&mut self) -> Option<MixOscParams<T>> {
        Some(MixOscParams {
            tune: self.tune.next()?,
            shape: self.shape.next()?,
            sin: self.sin.next()?,
            sq: self.sq.next()?,
            tri: self.tri.next()?,
            saw: self.saw.next()?,
        })
    }
}

/// Create a new [LfoParamIter], which initially creates instances of
/// [LfoParams] with frequency 1Hz, depth 1, and default [LfoOptions] until
/// calling the `with_*()` methods.
pub fn new_mixosc_param_iter<T: DspFormatBase>() -> MixOscParamIter<
    T,
    Repeat<T::NoteOffset>,
    Repeat<T::Scalar>,
    Repeat<T::Scalar>,
    Repeat<T::Scalar>,
    Repeat<T::Scalar>,
    Repeat<T::Scalar>,
> {
    MixOscParamIter {
        tune: repeat(T::NoteOffset::zero()),
        shape: repeat(T::Scalar::zero()),
        sin: repeat(T::Scalar::zero()),
        sq: repeat(T::Scalar::zero()),
        tri: repeat(T::Scalar::zero()),
        saw: repeat(T::Scalar::one()),
        phantom: Default::default(),
    }
}

/// An iterator builder for [SyncedMixOscsParams]
///
/// Use this to easily build iterators to [SyncedMixOscsParams] out of iterators
/// to its constituent parts.
pub struct SyncedMixOscsParamIter<T, A, B, C>
where
    T: DspFormatBase,
    A: Iterator<Item = MixOscParams<T>>,
    B: Iterator<Item = MixOscParams<T>>,
    C: Iterator<Item = bool>,
{
    primary: A,
    secondary: B,
    sync: C,
    phantom: core::marker::PhantomData<T>,
}

impl<T, A, B, C> SyncedMixOscsParamIter<T, A, B, C>
where
    T: DspFormatBase,
    A: Iterator<Item = MixOscParams<T>>,
    B: Iterator<Item = MixOscParams<T>>,
    C: Iterator<Item = bool>,
{
    /// Replace the current tuning source with the one provided
    pub fn with_primary<New: Iterator<Item = MixOscParams<T>>>(
        self,
        new: New,
    ) -> SyncedMixOscsParamIter<T, New, B, C> {
        SyncedMixOscsParamIter {
            primary: new,
            secondary: self.secondary,
            sync: self.sync,
            phantom: self.phantom,
        }
    }
    /// Replace the current tuning source with the one provided
    pub fn with_secondary<New: Iterator<Item = MixOscParams<T>>>(
        self,
        new: New,
    ) -> SyncedMixOscsParamIter<T, A, New, C> {
        SyncedMixOscsParamIter {
            primary: self.primary,
            secondary: new,
            sync: self.sync,
            phantom: self.phantom,
        }
    }
    /// Replace the current tuning source with the one provided
    pub fn with_sync<New: Iterator<Item = bool>>(
        self,
        new: New,
    ) -> SyncedMixOscsParamIter<T, A, B, New> {
        SyncedMixOscsParamIter {
            primary: self.primary,
            secondary: self.secondary,
            sync: new,
            phantom: self.phantom,
        }
    }
}

impl<T, A, B, C> Iterator for SyncedMixOscsParamIter<T, A, B, C>
where
    T: DspFormatBase,
    A: Iterator<Item = MixOscParams<T>>,
    B: Iterator<Item = MixOscParams<T>>,
    C: Iterator<Item = bool>,
{
    type Item = SyncedMixOscsParams<T>;
    fn next(&mut self) -> Option<SyncedMixOscsParams<T>> {
        Some(SyncedMixOscsParams {
            primary: self.primary.next()?,
            secondary: self.secondary.next()?,
            sync: self.sync.next()?,
        })
    }
}

/// Create a new [SyncedMixOscsParamIter], which initially creates instances of
/// [SyncedMixOscsParams] with the defaults for each
pub fn new_synced_mixoscs_param_iter<T: DspFormatBase>(
) -> SyncedMixOscsParamIter<T, Repeat<MixOscParams<T>>, Repeat<MixOscParams<T>>, Repeat<bool>> {
    SyncedMixOscsParamIter {
        primary: repeat(MixOscParams {
            tune: T::NoteOffset::zero(),
            shape: T::Scalar::zero(),
            sin: T::Scalar::zero(),
            sq: T::Scalar::zero(),
            tri: T::Scalar::zero(),
            saw: T::Scalar::one(),
        }),
        secondary: repeat(MixOscParams {
            tune: T::NoteOffset::zero(),
            shape: T::Scalar::zero(),
            sin: T::Scalar::zero(),
            sq: T::Scalar::zero(),
            tri: T::Scalar::zero(),
            saw: T::Scalar::one(),
        }),
        sync: repeat(false),
        phantom: Default::default(),
    }
}
