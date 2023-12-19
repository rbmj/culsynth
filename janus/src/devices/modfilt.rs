use super::*;
use fixedmath::apply_scalar_u;

/// A parameter pack for a [ModFilt]
pub struct ModFiltParams<'a, Smp> {
    /// The amount of envelope modulation, from 0 (none) to 1 (the envelope
    /// will, at peak, fully open the filter)
    pub env_mod: &'a [Smp],
    /// The amount of velocity modulation, from 0 (none) to 1 (max velocity
    /// will fully open the filter)
    pub vel_mod: &'a [Smp],
    /// The amount of keyboard tracking, from 0 (none) to 1 (1:1)
    pub kbd: &'a [Smp],
    /// The cutoff frequency of the filter, as a MIDI note number
    pub cutoff: &'a [Smp],
    /// The resonance of the filter, from 0 (none) to 1 (nearly self-resonant)
    pub resonance: &'a [Smp],
    /// The mix of the low-pass output of the filter
    pub low_mix: &'a [Smp],
    /// The mix of the band-pass output of the filter
    pub band_mix: &'a [Smp],
    /// The mix of the high-pass output of the filter
    pub high_mix: &'a [Smp],
}

impl<'a, Smp> ModFiltParams<'a, Smp> {
    /// The length of this parameter pack, defined as the length of
    /// the shortest subslice
    pub fn len(&self) -> usize {
        min_size(&[
            self.env_mod.len(),
            self.vel_mod.len(),
            self.kbd.len(),
            self.cutoff.len(),
            self.resonance.len(),
            self.low_mix.len(),
            self.band_mix.len(),
            self.high_mix.len(),
        ])
    }
    /// True if any subslice is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A mutable parameter pack for a [ModFilt] - see also [ModFiltParams]
pub struct MutModFiltParams<'a, Smp> {
    /// The amount of envelope modulation, from 0 (none) to 1 (the envelope
    /// will, at peak, fully open the filter)
    pub env_mod: &'a mut [Smp],
    /// The amount of velocity modulation, from 0 (none) to 1 (max velocity
    /// will fully open the filter)
    pub vel_mod: &'a mut [Smp],
    /// The amount of keyboard tracking, from 0 (none) to 1 (1:1)
    pub kbd: &'a mut [Smp],
    /// The cutoff frequency of the filter, as a MIDI note number
    pub cutoff: &'a mut [Smp],
    /// The resonance of the filter, from 0 (none) to 1 (nearly self-resonant)
    pub resonance: &'a mut [Smp],
    /// The mix of the low-pass output of the filter
    pub low_mix: &'a mut [Smp],
    /// The mix of the band-pass output of the filter
    pub band_mix: &'a mut [Smp],
    /// The mix of the high-pass output of the filter
    pub high_mix: &'a mut [Smp],
}

impl<'a, Smp> MutModFiltParams<'a, Smp> {
    /// The length of this parameter pack, defined as the length of
    /// the shortest subslice
    pub fn len(&self) -> usize {
        min_size(&[
            self.env_mod.len(),
            self.vel_mod.len(),
            self.kbd.len(),
            self.cutoff.len(),
            self.resonance.len(),
            self.low_mix.len(),
            self.band_mix.len(),
            self.high_mix.len(),
        ])
    }
    /// True if any subslice is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, Smp> From<MutModFiltParams<'a, Smp>> for ModFiltParams<'a, Smp> {
    fn from(value: MutModFiltParams<'a, Smp>) -> Self {
        Self {
            env_mod: value.env_mod,
            vel_mod: value.vel_mod,
            kbd: value.kbd,
            cutoff: value.cutoff,
            resonance: value.resonance,
            low_mix: value.low_mix,
            band_mix: value.band_mix,
            high_mix: value.high_mix,
        }
    }
}

/// Wraps a [Filt], with a series of additional parameters to make it easier
/// to use and with many more modulation options.  It takes parameters for
/// low, band, and high-pass gain, and mixes the outputs together, and adds
/// parameters to modulate the cutoff frequency with keyboard tracking,
/// envelope, and velocity modulation.
#[derive(Clone)]
pub struct ModFilt<Smp> {
    filter: Filt<Smp>,
    outbuf: BufferT<Smp>,
}

impl<Smp: Float> ModFilt<Smp> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            filter: Default::default(),
            outbuf: [Smp::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    /// Run the filter on the given input signal, taking into account all
    /// parameters as well as the provided envelope signal, voice MIDI note
    /// number signal, and velocity signal.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(
        &mut self,
        ctx: &Context<Smp>,
        input: &[Smp],
        env: &[Smp],
        note: &[Smp],
        vel: &[Smp],
        params: ModFiltParams<Smp>,
    ) -> &[Smp] {
        let numsamples = min_size(&[input.len(), env.len(), note.len(), vel.len(), params.len()]);
        for i in 0..numsamples {
            // use outbuf to hold the modulated filter cutoff
            self.outbuf[i] = Smp::NOTE_MAX * params.env_mod[i] * env[i];
            self.outbuf[i] = self.outbuf[i] + (Smp::NOTE_MAX * params.vel_mod[i] * vel[i]);
            self.outbuf[i] = self.outbuf[i] + params.cutoff[i] + (params.kbd[i] * note[i]);
            // saturate the cutoff if it's higher than note_max
            if self.outbuf[i] > Smp::NOTE_MAX {
                self.outbuf[i] = Smp::NOTE_MAX;
            }
        }
        //calculate filter output
        let filt_out = self.filter.process(
            ctx,
            input,
            FiltParams {
                cutoff: &self.outbuf[0..numsamples],
                resonance: params.resonance,
            },
        );
        //now mix the outputs
        for i in 0..numsamples {
            self.outbuf[i] = (params.low_mix[i] * filt_out.low[i])
                + (params.band_mix[i] * filt_out.band[i])
                + (params.high_mix[i] * filt_out.high[i]);
        }
        &self.outbuf[0..numsamples]
    }
}

impl<Smp: Float> Default for ModFilt<Smp> {
    fn default() -> Self {
        Self::new()
    }
}

/// A parameter pack for a [ModFiltFxP]
pub struct ModFiltParamsFxP<'a> {
    /// The amount of envelope modulation, from 0 (none) to 1 (the envelope
    /// will, at peak, fully open the filter)
    pub env_mod: &'a [ScalarFxP],
    /// The amount of velocity modulation, from 0 (none) to 1 (max velocity
    /// will fully open the filter)
    pub vel_mod: &'a [ScalarFxP],
    /// The amount of keyboard tracking, from 0 (none) to 1 (1:1)
    pub kbd: &'a [ScalarFxP],
    /// The cutoff frequency of the filter, as a MIDI note number
    pub cutoff: &'a [NoteFxP],
    /// The resonance of the filter, from 0 (none) to 1 (nearly self-resonant)
    pub resonance: &'a [ScalarFxP],
    /// The mix of the low-pass output of the filter
    pub low_mix: &'a [ScalarFxP],
    /// The mix of the band-pass output of the filter
    pub band_mix: &'a [ScalarFxP],
    /// The mix of the high-pass output of the filter
    pub high_mix: &'a [ScalarFxP],
}

impl<'a> ModFiltParamsFxP<'a> {
    /// The length of this parameter pack, defined as the length of
    /// the shortest subslice
    pub fn len(&self) -> usize {
        min_size(&[
            self.env_mod.len(),
            self.kbd.len(),
            self.cutoff.len(),
            self.resonance.len(),
            self.low_mix.len(),
            self.band_mix.len(),
            self.high_mix.len(),
        ])
    }
    /// True if any subslice is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A mutable parameter pack for a [ModFiltFxP] - see also [ModFiltParamsFxP]
pub struct MutModFiltParamsFxP<'a> {
    /// The amount of envelope modulation, from 0 (none) to 1 (the envelope
    /// will, at peak, fully open the filter)
    pub env_mod: &'a mut [ScalarFxP],
    /// The amount of velocity modulation, from 0 (none) to 1 (max velocity
    /// will fully open the filter)
    pub vel_mod: &'a mut [ScalarFxP],
    /// The amount of keyboard tracking, from 0 (none) to 1 (1:1)
    pub kbd: &'a mut [ScalarFxP],
    /// The cutoff frequency of the filter, as a MIDI note number
    pub cutoff: &'a mut [NoteFxP],
    /// The resonance of the filter, from 0 (none) to 1 (nearly self-resonant)
    pub resonance: &'a mut [ScalarFxP],
    /// The mix of the low-pass output of the filter
    pub low_mix: &'a mut [ScalarFxP],
    /// The mix of the band-pass output of the filter
    pub band_mix: &'a mut [ScalarFxP],
    /// The mix of the high-pass output of the filter
    pub high_mix: &'a mut [ScalarFxP],
}

impl<'a> MutModFiltParamsFxP<'a> {
    /// The length of this parameter pack, defined as the length of
    /// the shortest subslice
    pub fn len(&self) -> usize {
        min_size(&[
            self.env_mod.len(),
            self.vel_mod.len(),
            self.kbd.len(),
            self.cutoff.len(),
            self.resonance.len(),
            self.low_mix.len(),
            self.band_mix.len(),
            self.high_mix.len(),
        ])
    }
    /// True if any subslice is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> From<MutModFiltParamsFxP<'a>> for ModFiltParamsFxP<'a> {
    fn from(value: MutModFiltParamsFxP<'a>) -> Self {
        Self {
            env_mod: value.env_mod,
            vel_mod: value.vel_mod,
            kbd: value.kbd,
            cutoff: value.cutoff,
            resonance: value.resonance,
            low_mix: value.low_mix,
            band_mix: value.band_mix,
            high_mix: value.high_mix,
        }
    }
}

/// Wraps a [FiltFxP], with a series of additional parameters to make it easier
/// to use and with many more modulation options.  It takes parameters for
/// low, band, and high-pass gain, and mixes the outputs together, and adds
/// parameters to modulate the cutoff frequency with keyboard tracking,
/// envelope, and velocity modulation.
#[derive(Clone)]
pub struct ModFiltFxP {
    filter: FiltFxP,
    modbuf: BufferT<NoteFxP>,
    outbuf: BufferT<SampleFxP>,
}

impl ModFiltFxP {
    /// Constructor
    pub fn new() -> Self {
        Self {
            filter: Default::default(),
            modbuf: [NoteFxP::ZERO; STATIC_BUFFER_SIZE],
            outbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    /// Run the filter on the given input signal, taking into account all
    /// parameters as well as the provided envelope signal, voice MIDI note
    /// number signal, and velocity signal.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(
        &mut self,
        ctx: &ContextFxP,
        input: &[SampleFxP],
        env: &[ScalarFxP],
        note: &[NoteFxP],
        vel: &[ScalarFxP],
        params: ModFiltParamsFxP,
    ) -> &[SampleFxP] {
        let numsamples = min_size(&[input.len(), env.len(), note.len(), params.len()]);
        for i in 0..numsamples {
            // reinterpret the env_mod as a number from 0 to NOTE_MAX instead of 0 to 1,
            // then multiply by the envelope output:
            let envmod = apply_scalar_u(NoteFxP::from_bits(params.env_mod[i].to_bits()), env[i]);
            let velmod = apply_scalar_u(NoteFxP::from_bits(params.vel_mod[i].to_bits()), vel[i]);
            let kbdmod = apply_scalar_u(note[i], params.kbd[i]);
            // TODO: reuse outbuf?  Would require some _ugly_ casts to get around the type system...
            self.modbuf[i] = params.cutoff[i]
                .saturating_add(envmod)
                .saturating_add(velmod)
                .saturating_add(kbdmod);
        }
        //calculate filter output
        let filt_out = self.filter.process(
            ctx,
            input,
            FiltParamsFxP {
                cutoff: &self.modbuf[0..numsamples],
                resonance: params.resonance,
            },
        );
        //now mix the outputs
        for i in 0..numsamples {
            let low = params.low_mix[i].wide_mul_signed(filt_out.low[i]);
            let band = params.band_mix[i].wide_mul_signed(filt_out.band[i]);
            let high = params.high_mix[i].wide_mul_signed(filt_out.high[i]);
            self.outbuf[i] = SampleFxP::from_num(low.saturating_add(band).saturating_add(high));
        }
        &self.outbuf[0..numsamples]
    }
}

impl Default for ModFiltFxP {
    fn default() -> Self {
        Self::new()
    }
}
