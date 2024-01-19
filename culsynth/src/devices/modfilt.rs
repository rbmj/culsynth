use super::*;

/// Input for a [ModFilt]
pub struct ModFiltInput<T: DspFormatBase> {
    /// The signal being filtered
    pub signal: T::Sample,
    /// The envelope signal
    pub env: T::Scalar,
    /// The velocity signal
    pub vel: T::Scalar,
    /// The keyboard signal
    pub kbd: T::Note,
}

/// A parameter pack for a [ModFiltFxP]
#[derive(Clone, Default)]
pub struct ModFiltParams<T: DspFormatBase> {
    /// The amount of envelope modulation, from 0 (none) to 1 (the envelope
    /// will, at peak, fully open the filter)
    pub env_mod: T::Scalar,
    /// The amount of velocity modulation, from 0 (none) to 1 (max velocity
    /// will fully open the filter)
    pub vel_mod: T::Scalar,
    /// The amount of keyboard tracking, from 0 (none) to 1 (1:1)
    pub kbd_tracking: T::Scalar,
    /// The cutoff frequency of the filter, as a MIDI note number
    pub cutoff: T::Note,
    /// The resonance of the filter, from 0 (none) to 1 (nearly self-resonant)
    pub resonance: T::Scalar,
    /// The mix of the low-pass output of the filter
    pub low_mix: T::Scalar,
    /// The mix of the band-pass output of the filter
    pub band_mix: T::Scalar,
    /// The mix of the high-pass output of the filter
    pub high_mix: T::Scalar,
}

impl<T: DspFloat> From<&ModFiltParams<i16>> for ModFiltParams<T> {
    fn from(value: &ModFiltParams<i16>) -> Self {
        Self {
            env_mod: value.env_mod.to_num(),
            vel_mod: value.vel_mod.to_num(),
            kbd_tracking: value.kbd_tracking.to_num(),
            cutoff: value.cutoff.to_num(),
            resonance: value.resonance.to_num(),
            low_mix: value.low_mix.to_num(),
            band_mix: value.band_mix.to_num(),
            high_mix: value.high_mix.to_num(),
        }
    }
}

impl<T: DspFormatBase> ModFiltParams<T> {
    /// Extract the [FiltParams] from this parameter pack, taking into account
    /// any modulation from the [ModFiltInput].
    pub fn to_filt_params(&self, input: &ModFiltInput<T>) -> FiltParams<T> {
        let mut cutoff = self.cutoff;
        let kbd = input.kbd.scale(self.kbd_tracking);
        let vel = T::note_from_scalar(input.vel.scale(self.vel_mod));
        let env = T::note_from_scalar(input.env.scale(self.env_mod));
        cutoff = cutoff.dsp_saturating_add(kbd).dsp_saturating_add(vel).dsp_saturating_add(env);
        FiltParams {
            cutoff,
            resonance: self.resonance,
        }
    }
}

/// Wraps a [Filt], with a series of additional parameters to make it easier
/// to use and with many more modulation options.  It takes parameters for
/// low, band, and high-pass gain, and mixes the outputs together, and adds
/// parameters to modulate the cutoff frequency with keyboard tracking,
/// envelope, and velocity modulation.
#[derive(Clone, Default)]
pub struct ModFilt<T: DspFormat> {
    filter: Filt<T>,
    mixer: Mixer<T, 3>,
}

impl<T: DspFormat> Device<T> for ModFilt<T> {
    type Input = ModFiltInput<T>;
    type Params = ModFiltParams<T>;
    type Output = T::Sample;
    fn next(
        &mut self,
        context: &T::Context,
        input: ModFiltInput<T>,
        params: ModFiltParams<T>,
    ) -> T::Sample {
        let filt_out = self.filter.next(context, input.signal, params.to_filt_params(&input));
        self.mixer.next(
            context,
            [filt_out.low, filt_out.band, filt_out.high],
            [params.low_mix, params.band_mix, params.high_mix],
        )
    }
}
