//! This module contains a struct composing various devices together as a
//! single voice unit for a basic subtractive synthesizer.

use crate::context::{Context, ContextFxP};
use crate::devices::*;
use crate::{min_size, BufferT, STATIC_BUFFER_SIZE};
use crate::{NoteFxP, SampleFxP, ScalarFxP};

pub mod modulation;

/// This struct encapsulates a single voice unit, containing a single oscillator,
/// a single VCF (with modulation inputs and mixing of low/band/high pass outputs),
/// a VCA, and two envelopes (one for the VCA and one for the VCF).
///
/// This implementaiton uses fixed point logic.
pub struct VoiceFxP {
    osc1: MixOscFxP,
    osc2: MixOscFxP,
    ringmod: RingModFxP,
    filt: ModFiltFxP,
    env_amp: EnvFxP,
    env_filt: EnvFxP,
    vca: AmpFxP,
    modmatrix: modulation::ModMatrixFxP,

    vcabuf: BufferT<SampleFxP>,
}

impl VoiceFxP {
    /// Constructor
    pub fn new() -> Self {
        Self {
            vcabuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            osc1: Default::default(),
            osc2: Default::default(),
            ringmod: Default::default(),
            filt: Default::default(),
            env_amp: Default::default(),
            env_filt: Default::default(),
            vca: Default::default(),
            modmatrix: Default::default(),
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
        ctx: &ContextFxP,
        note: &[NoteFxP],
        gate: &[SampleFxP],
        vel: &[ScalarFxP],
        aftertouch: &[ScalarFxP],
        modwheel: &[ScalarFxP],
        sync: &mut [ScalarFxP],
        osc1_p: MixOscParamsFxP,
        osc2_p: MixOscParamsFxP,
        ring_p: RingModParamsFxP,
        filt_p: ModFiltParamsFxP,
        filt_env_p: EnvParamsFxP,
        amp_env_p: EnvParamsFxP,
        lfo1_p: LfoParamsFxP,
        lfo2_p: MutLfoParamsFxP,
        env1_p: EnvParamsFxP,
        env2_p: MutEnvParamsFxP,
    ) -> &[SampleFxP] {
        let numsamples = min_size(&[
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
        ]);
        let modparams = modulation::ModMatrixParamsFxP {
            velocity: vel,
            aftertouch,
            modwheel,
            lfo1_params: lfo1_p,
            lfo2_params: lfo2_p,
            env1_params: env1_p,
            env2_params: env2_p,
        };
        let modulation = self.modmatrix.process(ctx, gate, modparams);
        let osc1_out = self.osc1.process(
            ctx,
            &note[0..numsamples],
            osc1_p.with_sync(OscSync::Master(sync)),
        );
        let osc2_out = self.osc2.process(
            ctx,
            &note[0..numsamples],
            osc2_p.with_sync(OscSync::Slave(sync)),
        );
        let ring_mod_out = self.ringmod.process(ctx, osc1_out, osc2_out, ring_p);
        let filt_env_out = self.env_filt.process(ctx, &gate[0..numsamples], filt_env_p);
        let filt_out = self
            .filt
            .process(ctx, ring_mod_out, filt_env_out, note, vel, filt_p);
        let vca_env_out = self.env_amp.process(ctx, &gate[0..numsamples], amp_env_p);
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

/// This struct encapsulates a single voice unit, containing a single oscillator,
/// a single VCF (with modulation inputs and mixing of low/band/high pass outputs),
/// a VCA, and two envelopes (one for the VCA and one for the VCF).
///
/// This implementation uses floating point logic
pub struct Voice<Smp: Float> {
    osc1: MixOsc<Smp>,
    osc2: MixOsc<Smp>,
    ringmod: RingMod<Smp>,
    filt: ModFilt<Smp>,
    env_amp: Env<Smp>,
    env_filt: Env<Smp>,
    vca: Amp<Smp>,
}

impl<Smp: Float> Voice<Smp> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            osc1: MixOsc::new(),
            osc2: MixOsc::new(),
            ringmod: RingMod::new(),
            filt: ModFilt::new(),
            env_amp: Env::new(),
            env_filt: Env::new(),
            vca: Amp::new(),
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
        ctx: &Context<Smp>,
        note: &[Smp],
        gate: &[Smp],
        vel: &[Smp],
        sync: &mut [Smp],
        osc1_p: MixOscParams<Smp>,
        osc2_p: MixOscParams<Smp>,
        ring_p: RingModParams<Smp>,
        filt_p: ModFiltParams<Smp>,
        filt_env_p: EnvParams<Smp>,
        amp_env_p: EnvParams<Smp>,
        lfo1_p: LfoParams<Smp>,
        lfo2_p: MutLfoParams<Smp>,
        env1_p: EnvParams<Smp>,
        env2_p: MutEnvParams<Smp>,
    ) -> &[Smp] {
        let numsamples = min_size(&[
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
            lfo1_p.len(),
            lfo2_p.len(),
            env1_p.len(),
            env2_p.len(),
            STATIC_BUFFER_SIZE,
        ]);
        let osc1_out = self.osc1.process(
            ctx,
            &note[0..numsamples],
            osc1_p.with_sync(OscSync::Master(sync)),
        );
        let osc2_out = self.osc2.process(
            ctx,
            &note[0..numsamples],
            osc2_p.with_sync(OscSync::Slave(sync)),
        );
        let ring_mod_out = self.ringmod.process(ctx, osc1_out, osc2_out, ring_p);
        let filt_env_out = self.env_filt.process(ctx, &gate[0..numsamples], filt_env_p);
        let filt_out = self
            .filt
            .process(ctx, ring_mod_out, filt_env_out, note, vel, filt_p);
        let vca_env_out = self.env_amp.process(ctx, &gate[0..numsamples], amp_env_p);
        self.vca.process(filt_out, vca_env_out)
    }
}

impl<Smp: Float> Default for Voice<Smp> {
    fn default() -> Self {
        Self::new()
    }
}
