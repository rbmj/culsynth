use super::*;

/// An iterater builder for [ModFiltParams]
///
/// By default (see [new_mod_filt_param_iter]), this constructs instances of
/// [ModFiltParams] where the filter is fully open (cutoff is max) and zero
/// resonance.  All modulation sources are set to zero, and the low pass gain
/// is set to one while band and high pass are set to zero.  This can then be
/// modified by calling the `with_*` methods.
pub struct ModFiltParamIter<T, A, B, C, D, E, F, G, H>
where
    T: DspFormat,
    A: Iterator<Item = T::Scalar>,
    B: Iterator<Item = T::Scalar>,
    C: Iterator<Item = T::Scalar>,
    D: Iterator<Item = T::Note>,
    E: Iterator<Item = T::Scalar>,
    F: Iterator<Item = T::Scalar>,
    G: Iterator<Item = T::Scalar>,
    H: Iterator<Item = T::Scalar>,
{
    env_mod: A,
    vel_mod: B,
    kbd_tracking: C,
    cutoff: D,
    resonance: E,
    low_mix: F,
    band_mix: G,
    high_mix: H,
    phantom: core::marker::PhantomData<T>,
}

impl<T, A, B, C, D, E, F, G, H> ModFiltParamIter<T, A, B, C, D, E, F, G, H>
where
    T: DspFormat,
    A: Iterator<Item = T::Scalar>,
    B: Iterator<Item = T::Scalar>,
    C: Iterator<Item = T::Scalar>,
    D: Iterator<Item = T::Note>,
    E: Iterator<Item = T::Scalar>,
    F: Iterator<Item = T::Scalar>,
    G: Iterator<Item = T::Scalar>,
    H: Iterator<Item = T::Scalar>,
{
    /// Replace the envelope modulation amount with the provided iterator
    pub fn with_env_mod<New>(self, new: New) -> ModFiltParamIter<T, New, B, C, D, E, F, G, H>
    where
        New: Iterator<Item = T::Scalar>,
    {
        ModFiltParamIter {
            phantom: self.phantom,
            env_mod: new,
            vel_mod: self.vel_mod,
            kbd_tracking: self.kbd_tracking,
            cutoff: self.cutoff,
            resonance: self.resonance,
            low_mix: self.low_mix,
            band_mix: self.band_mix,
            high_mix: self.high_mix,
        }
    }
    /// Replace the velocity modulation amount with the provided iterator
    pub fn with_vel_mod<New>(self, new: New) -> ModFiltParamIter<T, A, New, C, D, E, F, G, H>
    where
        New: Iterator<Item = T::Scalar>,
    {
        ModFiltParamIter {
            phantom: self.phantom,
            env_mod: self.env_mod,
            vel_mod: new,
            kbd_tracking: self.kbd_tracking,
            cutoff: self.cutoff,
            resonance: self.resonance,
            low_mix: self.low_mix,
            band_mix: self.band_mix,
            high_mix: self.high_mix,
        }
    }
    /// Replace the keyboard tracking amount with the provided iterator
    pub fn with_kbd_tracking<New>(self, new: New) -> ModFiltParamIter<T, A, B, New, D, E, F, G, H>
    where
        New: Iterator<Item = T::Scalar>,
    {
        ModFiltParamIter {
            phantom: self.phantom,
            env_mod: self.env_mod,
            vel_mod: self.vel_mod,
            kbd_tracking: new,
            cutoff: self.cutoff,
            resonance: self.resonance,
            low_mix: self.low_mix,
            band_mix: self.band_mix,
            high_mix: self.high_mix,
        }
    }
    /// Replace the filter cutoff with the provided iterator
    pub fn with_cutoff<New>(self, new: New) -> ModFiltParamIter<T, A, B, C, New, E, F, G, H>
    where
        New: Iterator<Item = T::Note>,
    {
        ModFiltParamIter {
            phantom: self.phantom,
            env_mod: self.env_mod,
            vel_mod: self.vel_mod,
            kbd_tracking: self.kbd_tracking,
            cutoff: new,
            resonance: self.resonance,
            low_mix: self.low_mix,
            band_mix: self.band_mix,
            high_mix: self.high_mix,
        }
    }
    /// Replace the filter resonance with the provided resonance
    pub fn with_resonance<New>(self, new: New) -> ModFiltParamIter<T, A, B, C, D, New, F, G, H>
    where
        New: Iterator<Item = T::Scalar>,
    {
        ModFiltParamIter {
            phantom: self.phantom,
            env_mod: self.env_mod,
            vel_mod: self.vel_mod,
            kbd_tracking: self.kbd_tracking,
            cutoff: self.cutoff,
            resonance: new,
            low_mix: self.low_mix,
            band_mix: self.band_mix,
            high_mix: self.high_mix,
        }
    }
    /// Replace the low-pass signal gain (mix) with the provided value
    pub fn with_low_mix<New>(self, new: New) -> ModFiltParamIter<T, A, B, C, D, E, New, G, H>
    where
        New: Iterator<Item = T::Scalar>,
    {
        ModFiltParamIter {
            phantom: self.phantom,
            env_mod: self.env_mod,
            vel_mod: self.vel_mod,
            kbd_tracking: self.kbd_tracking,
            cutoff: self.cutoff,
            resonance: self.resonance,
            low_mix: new,
            band_mix: self.band_mix,
            high_mix: self.high_mix,
        }
    }
    /// Replace the band-pass signal gain (mix) with the provided value
    pub fn with_band_mix<New>(self, new: New) -> ModFiltParamIter<T, A, B, C, D, E, F, New, H>
    where
        New: Iterator<Item = T::Scalar>,
    {
        ModFiltParamIter {
            phantom: self.phantom,
            env_mod: self.env_mod,
            vel_mod: self.vel_mod,
            kbd_tracking: self.kbd_tracking,
            cutoff: self.cutoff,
            resonance: self.resonance,
            low_mix: self.low_mix,
            band_mix: new,
            high_mix: self.high_mix,
        }
    }
    /// Replace the high-pass signal gain (mix) with the provided value
    pub fn with_high_mix<New>(self, new: New) -> ModFiltParamIter<T, A, B, C, D, E, F, G, New>
    where
        New: Iterator<Item = T::Scalar>,
    {
        ModFiltParamIter {
            phantom: self.phantom,
            env_mod: self.env_mod,
            vel_mod: self.vel_mod,
            kbd_tracking: self.kbd_tracking,
            cutoff: self.cutoff,
            resonance: self.resonance,
            low_mix: self.low_mix,
            band_mix: self.band_mix,
            high_mix: new,
        }
    }
}

impl<T, A, B, C, D, E, F, G, H> Iterator for ModFiltParamIter<T, A, B, C, D, E, F, G, H>
where
    T: DspFormat,
    A: Iterator<Item = T::Scalar>,
    B: Iterator<Item = T::Scalar>,
    C: Iterator<Item = T::Scalar>,
    D: Iterator<Item = T::Note>,
    E: Iterator<Item = T::Scalar>,
    F: Iterator<Item = T::Scalar>,
    G: Iterator<Item = T::Scalar>,
    H: Iterator<Item = T::Scalar>,
{
    type Item = ModFiltParams<T>;
    fn next(&mut self) -> Option<ModFiltParams<T>> {
        Some(ModFiltParams {
            env_mod: self.env_mod.next()?,
            vel_mod: self.vel_mod.next()?,
            kbd_tracking: self.kbd_tracking.next()?,
            cutoff: self.cutoff.next()?,
            resonance: self.resonance.next()?,
            low_mix: self.low_mix.next()?,
            band_mix: self.band_mix.next()?,
            high_mix: self.high_mix.next()?,
        })
    }
}

/// Construct a [ModFiltParamIter]
pub fn new_modfilt_param_iter<T: DspFormat>() -> ModFiltParamIter<
    T,
    Repeat<T::Scalar>,
    Repeat<T::Scalar>,
    Repeat<T::Scalar>,
    Repeat<T::Note>,
    Repeat<T::Scalar>,
    Repeat<T::Scalar>,
    Repeat<T::Scalar>,
    Repeat<T::Scalar>,
> {
    ModFiltParamIter {
        phantom: Default::default(),
        env_mod: repeat(T::Scalar::zero()),
        vel_mod: repeat(T::Scalar::zero()),
        kbd_tracking: repeat(T::Scalar::zero()),
        cutoff: repeat(T::note_from_scalar(T::Scalar::one())),
        resonance: repeat(T::Scalar::zero()),
        low_mix: repeat(T::Scalar::one()),
        band_mix: repeat(T::Scalar::zero()),
        high_mix: repeat(T::Scalar::zero()),
    }
}

/// An iterator builder for [ModFiltInput]
///
/// Use this to easily build iterators to [ModFiltInput] out of iterators to
/// its constituent parts.
pub struct ModFiltInputIter<T: DspFormatBase, S, E, V, K>
where
    S: Iterator<Item = T::Sample>,
    E: Iterator<Item = T::Scalar>,
    V: Iterator<Item = T::Scalar>,
    K: Iterator<Item = T::Note>,
{
    s: S,
    e: E,
    v: V,
    k: K,
    phantom: core::marker::PhantomData<T>,
}

impl<T: DspFormatBase, S, E, V, K> ModFiltInputIter<T, S, E, V, K>
where
    S: Iterator<Item = T::Sample>,
    E: Iterator<Item = T::Scalar>,
    V: Iterator<Item = T::Scalar>,
    K: Iterator<Item = T::Note>,
{
    /// Replace the current signal source with the one provided
    pub fn with_signal<NewS: Iterator<Item = T::Sample>>(
        self,
        news: NewS,
    ) -> ModFiltInputIter<T, NewS, E, V, K> {
        ModFiltInputIter {
            s: news,
            e: self.e,
            v: self.v,
            k: self.k,
            phantom: self.phantom,
        }
    }
    /// Replace the current envelope source with the one provided
    pub fn with_env<NewE: Iterator<Item = T::Scalar>>(
        self,
        newe: NewE,
    ) -> ModFiltInputIter<T, S, NewE, V, K> {
        ModFiltInputIter {
            s: self.s,
            e: newe,
            v: self.v,
            k: self.k,
            phantom: self.phantom,
        }
    }
    /// Replace the current velocity source with the one provided
    pub fn with_vel<NewV: Iterator<Item = T::Scalar>>(
        self,
        newv: NewV,
    ) -> ModFiltInputIter<T, S, E, NewV, K> {
        ModFiltInputIter {
            s: self.s,
            e: self.e,
            v: newv,
            k: self.k,
            phantom: self.phantom,
        }
    }
    /// Replace the current keyboard (note) source with the one provided
    pub fn with_kbd<NewK: Iterator<Item = T::Note>>(
        self,
        newk: NewK,
    ) -> ModFiltInputIter<T, S, E, V, NewK> {
        ModFiltInputIter {
            s: self.s,
            e: self.e,
            v: self.v,
            k: newk,
            phantom: self.phantom,
        }
    }
}

impl<T, S, E, V, K> Iterator for ModFiltInputIter<T, S, E, V, K>
where
    T: DspFormatBase,
    S: Iterator<Item = T::Sample>,
    E: Iterator<Item = T::Scalar>,
    V: Iterator<Item = T::Scalar>,
    K: Iterator<Item = T::Note>,
{
    type Item = ModFiltInput<T>;
    fn next(&mut self) -> Option<ModFiltInput<T>> {
        Some(ModFiltInput {
            signal: self.s.next()?,
            env: self.e.next()?,
            vel: self.v.next()?,
            kbd: self.k.next()?,
        })
    }
}

/// Create a new [ModFiltInputIter], which initially creates instances of
/// [ModFiltInput::default] until calling the `with_*()` methods.
pub fn new_modfilt_input_iter<T: DspFormatBase>(
) -> ModFiltInputIter<T, Repeat<T::Sample>, Repeat<T::Scalar>, Repeat<T::Scalar>, Repeat<T::Note>> {
    ModFiltInputIter {
        s: repeat(T::Sample::zero()),
        e: repeat(T::Scalar::zero()),
        v: repeat(T::Scalar::zero()),
        k: repeat(T::Note::zero()),
        phantom: Default::default(),
    }
}
