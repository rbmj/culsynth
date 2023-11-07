use super::*;

pub struct MixOsc<Smp> {
    outbuf: BufferT<Smp>,
    osc: Osc<Smp>,
}

pub struct MixOscParams<'a, Smp> {
    pub tune: &'a [Smp],
    pub shape: &'a [Smp],
    pub sin: &'a [Smp],
    pub sq: &'a [Smp],
    pub tri: &'a [Smp],
    pub saw: &'a [Smp],
}

impl<'a, Smp> MixOscParams<'a, Smp> {
    pub fn len(&self) -> usize {
        *[
            self.tune.len(),
            self.shape.len(),
            self.sin.len(),
            self.sq.len(),
            self.tri.len(),
            self.saw.len(),
        ]
        .iter()
        .min()
        .unwrap_or(&0)
    }
}

impl<Smp: Float> MixOsc<Smp> {
    pub fn new() -> Self {
        Self {
            outbuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            osc: Default::default(),
        }
    }
    pub fn process(&mut self, note: &[Smp], params: MixOscParams<Smp>, sync: OscSync<Smp>) -> &[Smp] {
        let numsamples = std::cmp::min(
            std::cmp::min(STATIC_BUFFER_SIZE, sync.len().unwrap_or(STATIC_BUFFER_SIZE)),
            std::cmp::min(note.len(), params.len()),
        );
        let osc_out = self.osc.process(
            &note[0..numsamples],
            OscParams {
                tune: params.tune,
                shape: params.shape,
            },
            sync,
        );
        for i in 0..numsamples {
            self.outbuf[i] = (osc_out.sin[i] * params.sin[i])
                + (osc_out.sq[i] * params.sq[i])
                + (osc_out.tri[i] * params.tri[i])
                + (osc_out.saw[i] * params.saw[i]);
        }
        &self.outbuf[0..numsamples]
    }
}

impl<Smp: Float> Default for MixOsc<Smp> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MixOscFxP {
    outbuf: BufferT<SampleFxP>,
    osc: OscFxP,
}

pub struct MixOscParamsFxP<'a> {
    pub tune: &'a [SignedNoteFxP],
    pub shape: &'a [ScalarFxP],
    pub sin: &'a [ScalarFxP],
    pub sq: &'a [ScalarFxP],
    pub tri: &'a [ScalarFxP],
    pub saw: &'a [ScalarFxP],
}

impl<'a> MixOscParamsFxP<'a> {
    pub fn len(&self) -> usize {
        *[
            self.tune.len(),
            self.shape.len(),
            self.sin.len(),
            self.sq.len(),
            self.tri.len(),
            self.saw.len(),
        ]
        .iter()
        .min()
        .unwrap()
    }
}

impl MixOscFxP {
    pub fn new() -> Self {
        Self {
            outbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            osc: Default::default(),
        }
    }
    pub fn process(&mut self, note: &[NoteFxP], params: MixOscParamsFxP, sync: OscSync<ScalarFxP>) -> &[SampleFxP] {
        let numsamples = std::cmp::min(
            std::cmp::min(STATIC_BUFFER_SIZE, sync.len().unwrap_or(STATIC_BUFFER_SIZE)),
            std::cmp::min(note.len(), params.len()),
        );
        let osc_out = self.osc.process(
            &note[0..numsamples],
            OscParamsFxP {
                tune: params.tune,
                shape: params.shape,
            },
            sync,
        );
        for i in 0..numsamples {
            let sin = osc_out.sin[i].wide_mul_unsigned(params.sin[i]);
            let sq = osc_out.sq[i].wide_mul_unsigned(params.sq[i]);
            let tri = osc_out.tri[i].wide_mul_unsigned(params.tri[i]);
            let saw = osc_out.saw[i].wide_mul_unsigned(params.saw[i]);
            self.outbuf[i] = SampleFxP::from_num(sin + sq + tri + saw);
        }
        &self.outbuf[0..numsamples]
    }
}

impl Default for MixOscFxP {
    fn default() -> Self {
        Self::new()
    }
}
