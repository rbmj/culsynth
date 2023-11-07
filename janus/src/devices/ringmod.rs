use super::*;
use crate::fixedmath::{apply_scalar_i, widen_i};

/// A basic ring modulator with parameters to mix either of the original signals
/// in with the output.
pub struct RingMod<Smp> {
    outbuf: BufferT<Smp>,
}

/// Parameters for a [RingMod]
pub struct RingModParams<'a, Smp> {
    /// The volume of the output modulation signal
    pub mix_out: &'a [Smp],
    /// The volume of the input signal a mixed back in
    pub mix_a: &'a [Smp],
    /// The volume of the input signal b mixed back in
    pub mix_b: &'a [Smp],
}

impl<'a, Smp> RingModParams<'a, Smp> {
    /// The length of the input parameters, defined as the length of the shortest
    /// input slice.
    pub fn len(&self) -> usize {
        std::cmp::min(
            std::cmp::min(self.mix_a.len(), self.mix_b.len()),
            self.mix_out.len(),
        )
    }
}

impl<Smp: Float> RingMod<Smp> {
    pub fn new() -> Self {
        Self {
            outbuf: [Smp::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    /// Run the ring modulator on the provided input signals and mix the result
    /// back in with the input signals according to the provided parameters.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(&mut self, a: &[Smp], b: &[Smp], params: RingModParams<Smp>) -> &[Smp] {
        let numsamples = std::cmp::min(
            std::cmp::min(params.len(), STATIC_BUFFER_SIZE),
            std::cmp::min(a.len(), b.len()),
        );
        for i in 0..numsamples {
            let out = a[i] * b[i];
            self.outbuf[i] =
                out * params.mix_out[i] + a[i] * params.mix_a[i] + b[i] * params.mix_b[i];
        }
        &self.outbuf[0..numsamples]
    }
}

impl<Smp: Float> Default for RingMod<Smp> {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameters for a [RingModFxP]
pub struct RingModParamsFxP<'a> {
    /// The volume of the output modulation signal
    pub mix_out: &'a [ScalarFxP],
    /// The volume of the input signal a mixed back in
    pub mix_a: &'a [ScalarFxP],
    /// The volume of the input signal b mixed back in
    pub mix_b: &'a [ScalarFxP],
}

impl<'a> RingModParamsFxP<'a> {
    /// The length of the input parameters, defined as the length of the shortest
    /// input slice.
    pub fn len(&self) -> usize {
        std::cmp::min(
            std::cmp::min(self.mix_a.len(), self.mix_b.len()),
            self.mix_out.len(),
        )
    }
}

/// A basic ring modulator with parameters to mix either of the original signals
/// in with the output using fixed-point logic.
pub struct RingModFxP {
    outbuf: BufferT<SampleFxP>,
}

impl RingModFxP {
    /// Constructor
    pub fn new() -> Self {
        Self {
            outbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    /// Run the ring modulator on the provided input signals and mix the result
    /// back in with the input signals according to the provided parameters.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(
        &mut self,
        a: &[SampleFxP],
        b: &[SampleFxP],
        params: RingModParamsFxP,
    ) -> &[SampleFxP] {
        let numsamples = std::cmp::min(
            std::cmp::min(params.len(), STATIC_BUFFER_SIZE),
            std::cmp::min(a.len(), b.len()),
        );
        for i in 0..numsamples {
            let out = SampleFxP::saturating_from_num(a[i].wide_mul(b[i]));
            let mixed_32bits = widen_i(apply_scalar_i(out, params.mix_out[i]))
                + widen_i(apply_scalar_i(a[i], params.mix_a[i]))
                + widen_i(apply_scalar_i(b[i], params.mix_b[i]));
            self.outbuf[i] = SampleFxP::saturating_from_num(mixed_32bits);
        }
        &self.outbuf[0..numsamples]
    }
}

impl Default for RingModFxP {
    fn default() -> Self {
        Self::new()
    }
}