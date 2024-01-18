//! This module contains a struct composing various devices together as a
//! single voice unit for a basic subtractive synthesizer.

use crate::{devices::*, DspFormat, DspFloat};

use self::modulation::{ModMatrix, ModSection};

pub mod modulation;

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
    /// The gate signal, as a Sample
    pub gate: T::Sample,
    /// The velocity this note was played with
    pub velocity: T::Scalar,

}

impl<T: DspFloat> From<&VoiceInput<i16>> for VoiceInput<T> {
    fn from(value: &VoiceInput<i16>) -> Self {
        Self {
            note: value.note.to_num(),
            gate: value.gate.to_num(),
            velocity: value.velocity.to_num()
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
    pub fn next(
        &mut self,
        ctx: &T::Context,
        matrix: &ModMatrix<T>,
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
