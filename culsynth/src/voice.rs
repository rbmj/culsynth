//! This module contains a struct composing various devices together as a
//! single voice unit for a basic subtractive synthesizer.

use crate::{devices::*, DspFloat, DspFormat, Fixed16};

use self::modulation::{ModMatrix, ModSection};

pub mod cc;
pub mod modulation;
pub mod nrpn;

/// A parameter pack for a [Voice]
#[derive(Clone, Default)]
pub struct VoiceParams<T: DspFormat> {
    /// Oscillator section parameters
    pub oscs_p: SyncedMixOscsParams<T>,
    /// Ring-Mod
    pub ring_p: RingModParams<T>,
    /// Filter
    pub filt_p: ModFiltParams<T>,
    /// VCF Envelope
    pub filt_env_p: EnvParams<T>,
    /// VCA Envelope
    pub amp_env_p: EnvParams<T>,
    /// LFO1
    pub lfo1_p: LfoParams<T>,
    /// LFO2
    pub lfo2_p: LfoParams<T>,
    /// Modulation Envelope 1
    pub env1_p: EnvParams<T>,
    /// Modulation Envelope 2
    pub env2_p: EnvParams<T>,
}

impl VoiceParams<i16> {
    /// Apply the given MIDI control change to this parameter pack
    pub fn apply_cc(&mut self, cc: wmidi::ControlFunction, value: wmidi::U7) {
        match cc {
            cc::OSC1_SIN => self.oscs_p.primary.sin.set_from_u7(value),
            cc::OSC1_SQ => self.oscs_p.primary.sq.set_from_u7(value),
            cc::OSC1_TRI => self.oscs_p.primary.tri.set_from_u7(value),
            cc::OSC1_SAW => self.oscs_p.primary.saw.set_from_u7(value),
            cc::RING_MIXA => self.ring_p.mix_a.set_from_u7(value),

            cc::ENV_FILT_ATTACK => self.filt_env_p.attack.set_from_u7(value),
            cc::ENV_FILT_DECAY => self.filt_env_p.decay.set_from_u7(value),
            cc::ENV_FILT_SUSTAIN => self.filt_env_p.sustain.set_from_u7(value),
            cc::ENV_FILT_RELEASE => self.filt_env_p.release.set_from_u7(value),

            cc::OSC2_SIN => self.oscs_p.secondary.sin.set_from_u7(value),
            cc::OSC2_SQ => self.oscs_p.secondary.sq.set_from_u7(value),
            cc::OSC2_TRI => self.oscs_p.secondary.tri.set_from_u7(value),
            cc::OSC2_SAW => self.oscs_p.secondary.saw.set_from_u7(value),
            cc::RING_MIXB => self.ring_p.mix_b.set_from_u7(value),

            cc::ENV_AMP_ATTACK => self.amp_env_p.attack.set_from_u7(value),
            cc::ENV_AMP_DECAY => self.amp_env_p.decay.set_from_u7(value),
            cc::ENV_AMP_SUSTAIN => self.amp_env_p.sustain.set_from_u7(value),
            cc::ENV_AMP_RELEASE => self.amp_env_p.release.set_from_u7(value),

            cc::FILT_CUTOFF => self.filt_p.cutoff.set_from_u7(value),
            cc::FILT_RESONANCE => self.filt_p.resonance.set_from_u7(value),
            cc::FILT_KBD => self.filt_p.kbd_tracking.set_from_u7(value),
            cc::FILT_VEL => self.filt_p.vel_mod.set_from_u7(value),
            cc::FILT_ENV => self.filt_p.env_mod.set_from_u7(value),
            cc::FILT_LOW => self.filt_p.low_mix.set_from_u7(value),
            cc::FILT_BAND => self.filt_p.band_mix.set_from_u7(value),
            cc::FILT_HIGH => self.filt_p.high_mix.set_from_u7(value),
            cc::OSC1_SHAPE => self.oscs_p.primary.shape.set_from_u7(value),

            cc::RING_MIXMOD => self.ring_p.mix_mod.set_from_u7(value),
            cc::OSC2_FINE => {}
            cc::LFO1_RATE => self.lfo1_p.freq.set_from_u7(value),
            cc::LFO1_DEPTH => self.lfo1_p.depth.set_from_u7(value),
            cc::LFO1_WAVE => {
                LfoWave::new_from_u8(value.into()).map(|w| self.lfo1_p.opts.set_wave(w));
            }
            cc::LFO2_RATE => self.lfo2_p.freq.set_from_u7(value),
            cc::LFO2_DEPTH => self.lfo2_p.depth.set_from_u7(value),
            cc::LFO2_WAVE => {
                LfoWave::new_from_u8(value.into()).map(|w| self.lfo2_p.opts.set_wave(w));
            }
            cc::OSC2_SHAPE => self.oscs_p.secondary.shape.set_from_u7(value),

            cc::LFO1_RETRIGGER => self.lfo1_p.opts.set_retrigger(value > cc::CC_SIGNED_ZERO),
            cc::LFO1_BIPOLAR => self.lfo1_p.opts.set_bipolar(value > cc::CC_SIGNED_ZERO),
            cc::LFO2_RETRIGGER => self.lfo2_p.opts.set_retrigger(value > cc::CC_SIGNED_ZERO),
            cc::LFO2_BIPOLAR => self.lfo2_p.opts.set_bipolar(value > cc::CC_SIGNED_ZERO),
            cc::OSC_SYNC => self.oscs_p.sync = value > cc::CC_SIGNED_ZERO,

            cc::OSC1_COARSE => {}
            cc::OSC1_FINE => {}
            cc::OSC2_COARSE => {}

            cc::ENV_M1_ATTACK => self.env1_p.attack.set_from_u7(value),
            cc::ENV_M1_DECAY => self.env1_p.decay.set_from_u7(value),
            cc::ENV_M1_SUSTAIN => self.env1_p.sustain.set_from_u7(value),
            cc::ENV_M1_RELEASE => self.env1_p.release.set_from_u7(value),

            cc::ENV_M2_ATTACK => self.env2_p.attack.set_from_u7(value),
            cc::ENV_M2_DECAY => self.env2_p.decay.set_from_u7(value),
            cc::ENV_M2_SUSTAIN => self.env2_p.sustain.set_from_u7(value),
            cc::ENV_M2_RELEASE => self.env2_p.release.set_from_u7(value),
            _ => {}
        }
    }
}

impl<T: DspFloat> From<&VoiceParams<i16>> for VoiceParams<T> {
    fn from(value: &VoiceParams<i16>) -> Self {
        Self {
            oscs_p: (&value.oscs_p).into(),
            ring_p: (&value.ring_p).into(),
            filt_p: (&value.filt_p).into(),
            filt_env_p: (&value.filt_env_p).into(),
            amp_env_p: (&value.amp_env_p).into(),
            lfo1_p: (&value.lfo1_p).into(),
            lfo2_p: (&value.lfo2_p).into(),
            env1_p: (&value.env1_p).into(),
            env2_p: (&value.env2_p).into(),
        }
    }
}

impl From<&VoiceParams<i16>> for VoiceParams<i16> {
    fn from(value: &VoiceParams<i16>) -> Self {
        value.clone()
    }
}

/// Inputs for a [Voice] that are note-specific
#[derive(Clone, Default)]
pub struct VoiceInput<T: DspFormat> {
    /// The note itself, as a MIDI note number
    pub note: T::Note,
    /// The velocity this note was played with
    pub velocity: T::Scalar,
    /// The gate signal
    pub gate: bool,
}

impl<T: DspFloat> From<&VoiceInput<i16>> for VoiceInput<T> {
    fn from(value: &VoiceInput<i16>) -> Self {
        Self {
            note: value.note.to_num(),
            gate: value.gate,
            velocity: value.velocity.to_num(),
        }
    }
}

impl From<&VoiceInput<i16>> for VoiceInput<i16> {
    fn from(value: &VoiceInput<i16>) -> Self {
        value.clone()
    }
}

/// Channel-wide (i.e. affecting all notes) inputs for a given [Voice]
#[derive(Clone, Default)]
pub struct VoiceChannelInput<T: DspFormat> {
    /// Aftertouch (e.g. for a MIDI Channel Pressure Message)
    pub aftertouch: T::Scalar,
    /// Modulation Wheel (MIDI CC #1)
    pub modwheel: T::Scalar,
}

impl<T: DspFloat> From<&VoiceChannelInput<i16>> for VoiceChannelInput<T> {
    fn from(value: &VoiceChannelInput<i16>) -> Self {
        Self {
            aftertouch: value.aftertouch.to_num(),
            modwheel: value.modwheel.to_num(),
        }
    }
}

impl From<&VoiceChannelInput<i16>> for VoiceChannelInput<i16> {
    fn from(value: &VoiceChannelInput<i16>) -> Self {
        value.clone()
    }
}

/// This struct encapsulates a single voice unit, containing a single oscillator,
/// a single VCF (with modulation inputs and mixing of low/band/high pass outputs),
/// a VCA, and two envelopes (one for the VCA and one for the VCF).
#[derive(Clone, Default)]
pub struct Voice<T: DspFormat> {
    oscs: SyncedMixOscs<T>,
    ringmod: RingMod<T>,
    filt: ModFilt<T>,
    env_amp: Env<T>,
    env_filt: Env<T>,
    vca: Amp<T>,
    modsection: ModSection<T>,
}

impl<T: DspFormat> Voice<T> {
    /// Constructor
    pub fn new() -> Self {
        Default::default()
    }
    /// Constructor
    pub fn new_with_seeds(seeda: u64, seedb: u64) -> Self {
        Self {
            modsection: ModSection::new_with_seeds(seeda, seedb),
            ..Default::default()
        }
    }
    /// Get the next sample from this voice.
    ///
    /// If matrix is not `None`, this will update the internal modulation
    /// matrix - otherwise, this will reuse the last modulation matrix.  It
    /// is more efficient to set this to None than to pass the same
    /// mod matrix twice in a row.
    pub fn next(
        &mut self,
        ctx: &T::Context,
        matrix: Option<&ModMatrix<T>>,
        input: &VoiceInput<T>,
        ch_input: &VoiceChannelInput<T>,
        mut params: VoiceParams<T>,
    ) -> T::Sample {
        // Build the ModMatrix
        let modparams = modulation::ModSectionParams::<T> {
            velocity: input.velocity,
            aftertouch: ch_input.aftertouch,
            modwheel: ch_input.modwheel,
            lfo1_params: params.lfo1_p,
            lfo2_params: params.lfo2_p,
            env1_params: params.env1_p,
            env2_params: params.env2_p,
        };
        let m = self.modsection.next(ctx, input.gate, modparams, matrix);
        // Modulate all the parameters
        m.modulate_mix_osc(&mut params.oscs_p.primary, &modulation::OSC1_MOD_DEST);
        m.modulate_mix_osc(&mut params.oscs_p.secondary, &modulation::OSC2_MOD_DEST);
        m.modulate_ring(&mut params.ring_p);
        m.modulate_env(&mut params.filt_env_p, &modulation::ENV_FILT_MOD_DEST);
        m.modulate_env(&mut params.amp_env_p, &modulation::ENV_AMP_MOD_DEST);
        m.modulate_mod_filt(&mut params.filt_p);

        let oscs_out = self.oscs.next(ctx, input.note, params.oscs_p);

        let ring_mod_out = self.ringmod.next(
            ctx,
            RingModInput {
                signal_a: oscs_out.primary,
                signal_b: oscs_out.secondary,
            },
            params.ring_p,
        );

        let filt_env_out = self.env_filt.next(ctx, input.gate, params.filt_env_p);
        let filt_out = self.filt.next(
            ctx,
            ModFiltInput {
                signal: ring_mod_out,
                env: filt_env_out,
                kbd: input.note,
                vel: input.velocity,
            },
            params.filt_p,
        );
        let vca_env_out = self.env_amp.next(ctx, input.gate, params.amp_env_p);
        self.vca.next(ctx, filt_out, vca_env_out)
    }
}
