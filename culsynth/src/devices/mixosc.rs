use super::*;

/// This wraps [Osc], combining the oscillator with a mixer for each of the
/// wave shapes and taking the gain of each wave as a parameter and providing
/// a pre-mixed output
#[derive(Clone)]
pub struct MixOsc<Smp> {
    outbuf: BufferT<Smp>,
    osc: Osc<Smp>,
}

/// A parameter pack for [MixOsc].  This is immutable except for `sync`.
pub struct MixOscParams<'a, Smp> {
    /// The tuning offset, in semitones offset from 12TET/A440
    pub tune: &'a [Smp],
    /// The oscillator shape parameter, ranging from 0-1 and representing the
    /// amount of phase distortion/pulse width ratio for this output
    pub shape: &'a [Smp],
    /// The sync input/output for this oscillator (see [OscSync])
    pub sync: OscSync<'a, Smp>,
    /// Sine wave gain
    pub sin: &'a [Smp],
    /// Square wave gain
    pub sq: &'a [Smp],
    /// Triangle wave gain
    pub tri: &'a [Smp],
    /// Sawtooth wave gain
    pub saw: &'a [Smp],
}

impl<'a, Smp> MixOscParams<'a, Smp> {
    /// The length of this parameter pack, defined as the length of the shortest
    /// subslice.  The length of [OscSync::Off] is effectively infinite.
    pub fn len(&self) -> usize {
        let x = min_size(&[
            self.tune.len(),
            self.shape.len(),
            self.sin.len(),
            self.sq.len(),
            self.tri.len(),
            self.saw.len(),
        ]);
        self.sync.len().map_or(x, |y| core::cmp::min(x, y))
    }
    /// This is empty if `self.len() == 0`
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Create a new parameter pack, replacing the sync parameter with the one
    /// that was supplied.  This is useful when generating parameter packs for
    /// two oscillators with [OscSync::Off] then modifying them by adding the
    /// sync input appropriate for each oscillator.
    pub fn with_sync(self, s: OscSync<'a, Smp>) -> Self {
        Self { sync: s, ..self }
    }
}

/// A mutable parameter pack for a [MixOsc] - see [MixOscParams]
pub struct MutMixOscParams<'a, Smp> {
    /// The tuning offset, in semitones offset from 12TET/A440
    pub tune: &'a mut [Smp],
    /// The oscillator shape parameter, ranging from 0-1 and representing the
    /// amount of phase distortion/pulse width ratio for this output
    pub shape: &'a mut [Smp],
    /// The sync input/output for this oscillator (see [OscSync])
    pub sync: OscSync<'a, Smp>,
    /// Sine wave gain
    pub sin: &'a mut [Smp],
    /// Square wave gain
    pub sq: &'a mut [Smp],
    /// Triangle wave gain
    pub tri: &'a mut [Smp],
    /// Sawtooth wave gain
    pub saw: &'a mut [Smp],
}

impl<'a, Smp> From<MutMixOscParams<'a, Smp>> for MixOscParams<'a, Smp> {
    fn from(value: MutMixOscParams<'a, Smp>) -> Self {
        Self {
            tune: value.tune,
            shape: value.shape,
            sync: value.sync,
            sin: value.sin,
            sq: value.sq,
            tri: value.tri,
            saw: value.saw,
        }
    }
}

impl<'a, Smp: Float> MutMixOscParams<'a, Smp> {
    /// The length of this parameter pack, defined as the length of the shortest
    /// subslice.  The length of [OscSync::Off] is effectively infinite.
    pub fn len(&self) -> usize {
        let x = min_size(&[
            self.tune.len(),
            self.shape.len(),
            self.sin.len(),
            self.sq.len(),
            self.tri.len(),
            self.saw.len(),
        ]);
        self.sync.len().map_or(x, |y| core::cmp::min(x, y))
    }
    /// This is empty if `self.len() == 0`
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Create a new parameter pack, replacing the sync parameter with the one
    /// that was supplied.  This is useful when generating parameter packs for
    /// two oscillators with [OscSync::Off] then modifying them by adding the
    /// sync input appropriate for each oscillator.
    pub fn with_sync(self, s: OscSync<'a, Smp>) -> Self {
        Self { sync: s, ..self }
    }
}

impl<Smp: Float> MixOsc<Smp> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            outbuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            osc: Default::default(),
        }
    }
    /// Run this oscillator with the supplied note signal and parameters
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(
        &mut self,
        ctx: &Context<Smp>,
        note: &[Smp],
        params: MixOscParams<Smp>,
    ) -> &[Smp] {
        let osc_out = self.osc.process(
            ctx,
            note,
            OscParams {
                tune: params.tune,
                shape: params.shape,
                sync: params.sync,
            },
        );
        let numsamples = osc_out.len();
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

/// This wraps [OscFxP], combining the oscillator with a mixer for each of the
/// wave shapes and taking the gain of each wave as a parameter and providing
/// a pre-mixed output
#[derive(Clone)]
pub struct MixOscFxP {
    outbuf: BufferT<SampleFxP>,
    osc: OscFxP,
}

/// A parameter pack for [MixOscFxP].  This is immutable except for `sync`.
pub struct MixOscParamsFxP<'a> {
    /// The tuning offset, in semitones offset from 12TET/A440
    pub tune: &'a [SignedNoteFxP],
    /// The oscillator shape parameter, ranging from 0-1 and representing the
    /// amount of phase distortion/pulse width ratio for this output
    pub shape: &'a [ScalarFxP],
    /// The sync input/output for this oscillator (see [OscSync])
    pub sync: OscSync<'a, ScalarFxP>,
    /// Sine wave gain
    pub sin: &'a [ScalarFxP],
    /// Square wave gain
    pub sq: &'a [ScalarFxP],
    /// Triangle wave gain
    pub tri: &'a [ScalarFxP],
    /// Sawtooth wave gain
    pub saw: &'a [ScalarFxP],
}

impl<'a> MixOscParamsFxP<'a> {
    /// The length of this parameter pack, defined as the length of the shortest
    /// subslice.  The length of [OscSync::Off] is effectively infinite.
    pub fn len(&self) -> usize {
        let x = min_size(&[
            self.tune.len(),
            self.shape.len(),
            self.sin.len(),
            self.sq.len(),
            self.tri.len(),
            self.saw.len(),
        ]);
        self.sync.len().map_or(x, |y| core::cmp::min(x, y))
    }
    /// This is empty if `self.len() == 0`
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Create a new parameter pack, replacing the sync parameter with the one
    /// that was supplied.  This is useful when generating parameter packs for
    /// two oscillators with [OscSync::Off] then modifying them by adding the
    /// sync input appropriate for each oscillator.
    pub fn with_sync(self, s: OscSync<'a, ScalarFxP>) -> Self {
        Self { sync: s, ..self }
    }
}

/// A mutable parameter pack for a [MixOscFxP] - see [MixOscParamsFxP]
pub struct MutMixOscParamsFxP<'a> {
    /// The tuning offset, in semitones offset from 12TET/A440
    pub tune: &'a mut [SignedNoteFxP],
    /// The oscillator shape parameter, ranging from 0-1 and representing the
    /// amount of phase distortion/pulse width ratio for this output
    pub shape: &'a mut [ScalarFxP],
    /// The sync input/output for this oscillator (see [OscSync])
    pub sync: OscSync<'a, ScalarFxP>,
    /// Sine wave gain
    pub sin: &'a mut [ScalarFxP],
    /// Square wave gain
    pub sq: &'a mut [ScalarFxP],
    /// Triangle wave gain
    pub tri: &'a mut [ScalarFxP],
    /// Sawtooth wave gain
    pub saw: &'a mut [ScalarFxP],
}

impl<'a> From<MutMixOscParamsFxP<'a>> for MixOscParamsFxP<'a> {
    fn from(value: MutMixOscParamsFxP<'a>) -> Self {
        Self {
            tune: value.tune,
            shape: value.shape,
            sync: value.sync,
            sin: value.sin,
            sq: value.sq,
            tri: value.tri,
            saw: value.saw,
        }
    }
}

impl<'a> MutMixOscParamsFxP<'a> {
    /// The length of this parameter pack, defined as the length of the shortest
    /// subslice.  The length of [OscSync::Off] is effectively infinite.
    pub fn len(&self) -> usize {
        let x = min_size(&[
            self.tune.len(),
            self.shape.len(),
            self.sin.len(),
            self.sq.len(),
            self.tri.len(),
            self.saw.len(),
        ]);
        self.sync.len().map_or(x, |y| core::cmp::min(x, y))
    }
    /// This is empty if `self.len() == 0`
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Create a new parameter pack, replacing the sync parameter with the one
    /// that was supplied.  This is useful when generating parameter packs for
    /// two oscillators with [OscSync::Off] then modifying them by adding the
    /// sync input appropriate for each oscillator.
    pub fn with_sync(self, s: OscSync<'a, ScalarFxP>) -> Self {
        Self { sync: s, ..self }
    }
}

impl MixOscFxP {
    /// Constructor
    pub fn new() -> Self {
        Self {
            outbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            osc: Default::default(),
        }
    }
    /// Run this oscillator with the supplied note signal and parameters
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(
        &mut self,
        ctx: &ContextFxP,
        note: &[NoteFxP],
        params: MixOscParamsFxP,
    ) -> &[SampleFxP] {
        let osc_out = self.osc.process(
            ctx,
            note,
            OscParamsFxP {
                tune: params.tune,
                shape: params.shape,
                sync: params.sync,
            },
        );
        let numsamples = osc_out.len();
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
