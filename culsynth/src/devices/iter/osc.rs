use super::*;

/// An iterator builder for [OscParams]
///
/// Use this to easily build iterators to [OscParams] out of iterators to
/// its constituent parts.
pub struct OscParamIter<T, A, B>
where
    T: DspFormatBase,
    A: Iterator<Item = T::NoteOffset>,
    B: Iterator<Item = T::Scalar>,
{
    tune: A,
    shape: B,
    phantom: core::marker::PhantomData<T>,
}

impl<T, A, B> OscParamIter<T, A, B>
where
    T: DspFormatBase,
    A: Iterator<Item = T::NoteOffset>,
    B: Iterator<Item = T::Scalar>,
{
    /// Replace the current tuning source with the one provided
    pub fn with_tune<New: Iterator<Item = T::NoteOffset>>(
        self,
        new: New,
    ) -> OscParamIter<T, New, B> {
        OscParamIter {
            tune: new,
            shape: self.shape,
            phantom: self.phantom,
        }
    }
    /// Replace the current wave shape (phase distortion) source with the
    /// one provided
    pub fn with_shape<New: Iterator<Item = T::Scalar>>(self, new: New) -> OscParamIter<T, A, New> {
        OscParamIter {
            tune: self.tune,
            shape: new,
            phantom: self.phantom,
        }
    }
}

impl<T, A, B> Iterator for OscParamIter<T, A, B>
where
    T: DspFormatBase,
    A: Iterator<Item = T::NoteOffset>,
    B: Iterator<Item = T::Scalar>,
{
    type Item = OscParams<T>;
    fn next(&mut self) -> Option<OscParams<T>> {
        Some(OscParams {
            tune: self.tune.next()?,
            shape: self.shape.next()?,
        })
    }
}

/// Create a new [OscParamIter], which initially creates instances of
/// [OscParams] zero tuning offset and zero wave shaping (phase distortion)
/// until calling the `with_*()` methods.
pub fn new_osc_param_iter<T: DspFormatBase>(
) -> OscParamIter<T, Repeat<T::NoteOffset>, Repeat<T::Scalar>> {
    OscParamIter {
        tune: repeat(T::NoteOffset::zero()),
        shape: repeat(T::Scalar::zero()),
        phantom: Default::default(),
    }
}

/// An iterator builder for [SyncedOscsParams]
///
/// Use this to easily build iterators to [SyncedOscsParams] out of iterators
/// to its constituent parts.
pub struct SyncedOscsParamIter<T, A, B, C>
where
    T: DspFormatBase,
    A: Iterator<Item = OscParams<T>>,
    B: Iterator<Item = OscParams<T>>,
    C: Iterator<Item = bool>,
{
    primary: A,
    secondary: B,
    sync: C,
    phantom: core::marker::PhantomData<T>,
}

impl<T, A, B, C> SyncedOscsParamIter<T, A, B, C>
where
    T: DspFormatBase,
    A: Iterator<Item = OscParams<T>>,
    B: Iterator<Item = OscParams<T>>,
    C: Iterator<Item = bool>,
{
    /// Replace the current primary OscParams source with the one provided
    pub fn with_primary<New: Iterator<Item = OscParams<T>>>(
        self,
        new: New,
    ) -> SyncedOscsParamIter<T, New, B, C> {
        SyncedOscsParamIter {
            primary: new,
            secondary: self.secondary,
            sync: self.sync,
            phantom: self.phantom,
        }
    }
    /// Replace the current secondary OscParams source with the one provided
    pub fn with_secondary<New: Iterator<Item = OscParams<T>>>(
        self,
        new: New,
    ) -> SyncedOscsParamIter<T, A, New, C> {
        SyncedOscsParamIter {
            primary: self.primary,
            secondary: new,
            sync: self.sync,
            phantom: self.phantom,
        }
    }
    /// Replace the current oscillator sync source with the one provided
    pub fn with_sync<New: Iterator<Item = bool>>(
        self,
        new: New,
    ) -> SyncedOscsParamIter<T, A, B, New> {
        SyncedOscsParamIter {
            primary: self.primary,
            secondary: self.secondary,
            sync: new,
            phantom: self.phantom,
        }
    }
}

impl<T, A, B, C> Iterator for SyncedOscsParamIter<T, A, B, C>
where
    T: DspFormatBase,
    A: Iterator<Item = OscParams<T>>,
    B: Iterator<Item = OscParams<T>>,
    C: Iterator<Item = bool>,
{
    type Item = SyncedOscsParams<T>;
    fn next(&mut self) -> Option<SyncedOscsParams<T>> {
        Some(SyncedOscsParams {
            primary: self.primary.next()?,
            secondary: self.secondary.next()?,
            sync: self.sync.next()?,
        })
    }
}

/// Create a new [SyncedOscsParamIter], which initially creates instances of
/// [SyncedOscsParams] with the defaults for each
pub fn new_synced_oscs_param_iter<T: DspFormatBase>(
) -> SyncedOscsParamIter<T, Repeat<OscParams<T>>, Repeat<OscParams<T>>, Repeat<bool>> {
    SyncedOscsParamIter {
        primary: repeat(OscParams {
            tune: T::NoteOffset::zero(),
            shape: T::Scalar::zero(),
        }),
        secondary: repeat(OscParams {
            tune: T::NoteOffset::zero(),
            shape: T::Scalar::zero(),
        }),
        sync: repeat(false),
        phantom: Default::default(),
    }
}
