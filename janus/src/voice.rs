//! This module contains a struct composing various devices together as a
//! single voice unit for a basic subtractive synthesizer.

use super::{BufferT, STATIC_BUFFER_SIZE};
use super::{NoteFxP, SampleFxP, ScalarFxP};
use crate::devices::*;

/// This struct encapsulates a single voice unit, containing a single oscillator,
/// a single VCF (with modulation inputs and mixing of low/band/high pass outputs),
/// a VCA, and two envelopes (one for the VCA and one for the VCF).
pub struct VoiceFxP {
    osc1: MixOscFxP,
    osc2: MixOscFxP,
    ringmod: RingModFxP,
    filt: ModFiltFxP,
    env_amp: EnvFxP,
    env_filt: EnvFxP,
    vca: AmpFxP,

    vcabuf: BufferT<SampleFxP>,
}

impl VoiceFxP {
    /// Constructor
    pub fn new() -> Self {
        Self {
            osc1: MixOscFxP::new(),
            osc2: MixOscFxP::new(),
            ringmod: RingModFxP::new(),
            filt: ModFiltFxP::new(),
            env_amp: EnvFxP::new(),
            env_filt: EnvFxP::new(),
            vca: AmpFxP::new(),
            vcabuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    /// Process the note/gate inputs, passing the parameters to the relevant
    /// components of the voice unit, and return a reference to an internal
    /// buffer containing the output sample data.
    /// 
    /// The syncbuf should be set non-zero for any sample where oscillator sync
    /// is enabled, or zero if sync is disabled.  This function will clobber the
    /// `sync` buffer unless it is zero for all samples.
    /// 
    /// `osc1_p.sync` and `osc2_p.sync` may be set to `OscSync::Off`, and this
    /// will internally set osc1 to be the master and osc2 to be the slave.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(
        &mut self,
        note: &[NoteFxP],
        gate: &[SampleFxP],
        vel: &[ScalarFxP],
        sync: &mut [ScalarFxP],
        osc1_p: MixOscParamsFxP,
        osc2_p: MixOscParamsFxP,
        ring_p: RingModParamsFxP,
        filt_p: ModFiltParamsFxP,
        filt_env_p: EnvParamsFxP,
        amp_env_p: EnvParamsFxP,
    ) -> &[SampleFxP] {
        let numsamples = *[
            note.len(),
            gate.len(),
            vel.len(),
            sync.len(),
            osc1_p.len(),
            osc2_p.len(),
            ring_p.len(),
            filt_p.len(),
            filt_env_p.len(),
            amp_env_p.len(),
            STATIC_BUFFER_SIZE,
        ]
        .iter()
        .min()
        .unwrap_or(&0);
        let osc1_out = self.osc1.process(
            &note[0..numsamples],
            osc1_p.with_sync(OscSync::Master(sync)),
        );
        let osc2_out = self.osc2.process(
            &note[0..numsamples], 
            osc2_p.with_sync(OscSync::Slave(sync)),
        );
        let ring_mod_out = self.ringmod.process(osc1_out, osc2_out, ring_p);
        let filt_env_out = self.env_filt.process(&gate[0..numsamples], filt_env_p);
        let filt_out = self.filt.process(ring_mod_out, filt_env_out, note, vel, filt_p);
        let vca_env_out = self.env_amp.process(&gate[0..numsamples], amp_env_p);
        for i in 0..numsamples {
            self.vcabuf[i] = SampleFxP::from_num(vca_env_out[i]);
        }
        self.vca.process(filt_out, &self.vcabuf[0..numsamples])
    }
}

impl Default for VoiceFxP {
    fn default() -> Self {
        Self::new()
    }
}
