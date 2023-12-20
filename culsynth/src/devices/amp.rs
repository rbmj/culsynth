use super::*;

/// A floating-point VCA
#[derive(Clone)]
pub struct Amp<Smp> {
    outbuf: BufferT<Smp>,
}

impl<Smp: Float> Amp<Smp> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            outbuf: [Smp::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    /// Process the input signal and apply the VCA gain to it
    pub fn process(&mut self, signal: &[Smp], gain: &[Smp]) -> &[Smp] {
        let numsamples = min_size(&[signal.len(), gain.len(), STATIC_BUFFER_SIZE]);
        for i in 0..numsamples {
            self.outbuf[i] = signal[i] * gain[i];
        }
        &self.outbuf[0..numsamples]
    }
}

impl<Smp: Float> Default for Amp<Smp> {
    fn default() -> Self {
        Self::new()
    }
}

/// A fixed-point VCA
#[derive(Clone)]
pub struct AmpFxP {
    outbuf: BufferT<SampleFxP>,
}

impl AmpFxP {
    /// Constructor
    pub fn new() -> Self {
        Self {
            outbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    /// Process the input signal and apply the VCA gain supplied, then return the result
    pub fn process(&mut self, signal: &[SampleFxP], gain: &[SampleFxP]) -> &[SampleFxP] {
        let numsamples = min_size(&[signal.len(), gain.len(), STATIC_BUFFER_SIZE]);
        for i in 0..numsamples {
            self.outbuf[i] = signal[i].saturating_mul(gain[i]);
        }
        &self.outbuf[0..numsamples]
    }
}

impl Default for AmpFxP {
    fn default() -> Self {
        Self::new()
    }
}
