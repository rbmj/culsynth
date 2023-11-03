use super::*;
use fixedmath::apply_scalar_u;

pub struct ModFiltParams<'a, Smp> {
    pub env_mod: &'a [Smp],
    pub kbd: &'a [Smp],
    pub cutoff: &'a [Smp],
    pub resonance: &'a [Smp],
    pub low_mix: &'a [Smp],
    pub band_mix: &'a [Smp],
    pub high_mix: &'a [Smp],
}

impl<'a, Smp> ModFiltParams<'a, Smp> {
    pub fn len(&self) -> usize {
        *[
            self.env_mod.len(),
            self.kbd.len(),
            self.cutoff.len(),
            self.resonance.len(),
            self.low_mix.len(),
            self.band_mix.len(),
            self.high_mix.len(),
        ]
        .iter()
        .min()
        .unwrap()
    }
}

pub struct ModFilt<Smp> {
    filter: Filt<Smp>,
    outbuf: BufferT<Smp>,
}

impl<Smp: Float> ModFilt<Smp> {
    pub fn new() -> Self {
        Self {
            filter: Default::default(),
            outbuf: [Smp::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    pub fn process(
        &mut self,
        input: &[Smp],
        env: &[Smp],
        note: &[Smp],
        params: ModFiltParams<Smp>,
    ) -> &[Smp] {
        let numsamples = *[input.len(), env.len(), note.len(), params.len()]
            .iter()
            .min()
            .unwrap();
        for i in 0..numsamples {
            // use outbuf to hold the modulated filter cutoff
            self.outbuf[i] = Smp::NOTE_MAX * params.env_mod[i] * env[i];
            self.outbuf[i] = self.outbuf[i] + params.cutoff[i] + (params.kbd[i] * note[i]);
            // saturate the cutoff if it's higher than note_max
            if self.outbuf[i] > Smp::NOTE_MAX {
                self.outbuf[i] = Smp::NOTE_MAX;
            }
        }
        //calculate filter output
        let filt_out = self.filter.process(
            &input,
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

pub struct ModFiltParamsFxP<'a> {
    pub env_mod: &'a [ScalarFxP],
    pub kbd: &'a [ScalarFxP],
    pub cutoff: &'a [NoteFxP],
    pub resonance: &'a [ScalarFxP],
    pub low_mix: &'a [ScalarFxP],
    pub band_mix: &'a [ScalarFxP],
    pub high_mix: &'a [ScalarFxP],
}

impl<'a> ModFiltParamsFxP<'a> {
    pub fn len(&self) -> usize {
        *[
            self.env_mod.len(),
            self.kbd.len(),
            self.cutoff.len(),
            self.resonance.len(),
            self.low_mix.len(),
            self.band_mix.len(),
            self.high_mix.len(),
        ]
        .iter()
        .min()
        .unwrap()
    }
}

pub struct ModFiltFxP {
    filter: FiltFxP,
    modbuf: BufferT<NoteFxP>,
    outbuf: BufferT<SampleFxP>,
}

impl ModFiltFxP {
    pub fn new() -> Self {
        Self {
            filter: Default::default(),
            modbuf: [NoteFxP::ZERO; STATIC_BUFFER_SIZE],
            outbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    pub fn process(
        &mut self,
        input: &[SampleFxP],
        env: &[ScalarFxP],
        note: &[NoteFxP],
        params: ModFiltParamsFxP,
    ) -> &[SampleFxP] {
        let numsamples = *[input.len(), env.len(), note.len(), params.len()]
            .iter()
            .min()
            .unwrap();
        for i in 0..numsamples {
            // reinterpret the env_mod as a number from 0 to NOTE_MAX instead of 0 to 1,
            // then multiply by the envelope output:
            let envmod = apply_scalar_u(NoteFxP::from_bits(params.env_mod[i].to_bits()), env[i]);
            let kbdmod = apply_scalar_u(note[i], params.kbd[i]);
            // TODO: reuse outbuf?  Would require some _ugly_ casts to get around the type system...
            self.modbuf[i] = params.cutoff[i]
                .saturating_add(envmod)
                .saturating_add(kbdmod);
        }
        //calculate filter output
        let filt_out = self.filter.process(
            &input,
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
