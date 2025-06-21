use culsynth::devices::{EnvParams, LfoParams, MixOscParams, ModFiltParams, RingModParams};
use culsynth::devices::{LfoOptions, LfoWave, SyncedMixOscsParams};
use culsynth::voice::modulation::{ModDest, ModMatrix, ModSrc};
use culsynth::voice::VoiceParams;
use culsynth::{
    CoarseTuneFxP, EnvParamFxP, FineTuneFxP, IScalarFxP, LfoFreqFxP, NoteFxP, ScalarFxP,
    SignedNoteFxP,
};
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;

use std::sync::Arc;

use super::fixedparam::{
    new_fixed_param, new_fixed_param_env, new_fixed_param_freq, new_fixed_param_lfo,
    new_fixed_param_percent,
};

/// Contains all of the parameters for an oscillator within the plugin
#[derive(Params)]
pub struct OscPluginParams {
    #[id = "coarse"]
    pub coarse: IntParam,

    #[id = "fine"]
    pub fine: IntParam,

    #[id = "shape"]
    pub shape: IntParam,

    #[id = "sin"]
    pub sin: IntParam,

    #[id = "sq"]
    pub sq: IntParam,

    #[id = "tri"]
    pub tri: IntParam,

    #[id = "saw"]
    pub saw: IntParam,
}

impl OscPluginParams {
    pub(crate) fn tuning(&self) -> crate::Tuning {
        crate::Tuning {
            coarse: CoarseTuneFxP::from_bits(self.coarse.value() as i16),
            fine: FineTuneFxP::from_bits(self.fine.value() as i16),
        }
    }
}

impl Default for OscPluginParams {
    fn default() -> Self {
        Self {
            coarse: new_fixed_param("Coarse", CoarseTuneFxP::ZERO),
            fine: new_fixed_param("Fine", FineTuneFxP::ZERO),
            shape: new_fixed_param_percent("Shape", ScalarFxP::ZERO),
            sin: new_fixed_param_percent("Sin", ScalarFxP::ZERO),
            saw: new_fixed_param_percent("Saw", ScalarFxP::MAX),
            sq: new_fixed_param_percent("Square", ScalarFxP::ZERO),
            tri: new_fixed_param_percent("Triangle", ScalarFxP::ZERO),
        }
    }
}

impl From<&OscPluginParams> for MixOscParams<i16> {
    fn from(value: &OscPluginParams) -> Self {
        let coarse = CoarseTuneFxP::from_bits(value.coarse.smoothed.next() as i16);
        let fine = FineTuneFxP::from_bits(value.fine.smoothed.next() as i16);
        let tune = SignedNoteFxP::from_num(coarse) + SignedNoteFxP::from_num(fine);
        MixOscParams {
            tune,
            shape: ScalarFxP::from_bits(value.shape.smoothed.next() as u16),
            sin: ScalarFxP::from_bits(value.sin.smoothed.next() as u16),
            sq: ScalarFxP::from_bits(value.sq.smoothed.next() as u16),
            tri: ScalarFxP::from_bits(value.tri.smoothed.next() as u16),
            saw: ScalarFxP::from_bits(value.saw.smoothed.next() as u16),
        }
    }
}

/// Contains all of the parameters for an LFO within the plugin
#[derive(Params)]
pub struct LfoPluginParams {
    #[id = "freq"]
    pub rate: IntParam,

    #[id = "depth"]
    pub depth: IntParam,

    #[id = "wave"]
    pub wave: IntParam,

    #[id = "retrigger"]
    pub retrigger: BoolParam,

    #[id = "bipolar"]
    pub bipolar: BoolParam,
}

impl LfoPluginParams {
    fn new(name: &str) -> Self {
        Self {
            wave: IntParam::new(
                name.to_owned() + " Wave",
                LfoWave::Sine as i32,
                IntRange::Linear {
                    min: LfoWave::Sine as i32,
                    max: LfoWave::SampleGlide as i32,
                },
            ),
            rate: new_fixed_param_lfo(name.to_owned() + " Rate", LfoFreqFxP::ONE),
            depth: new_fixed_param_percent(name.to_owned() + " Depth", ScalarFxP::MAX),
            retrigger: BoolParam::new(name.to_owned() + " Retrigger", true),
            bipolar: BoolParam::new(name.to_owned() + " Bipolar", true),
        }
    }
}

impl From<&LfoPluginParams> for LfoOptions {
    fn from(param: &LfoPluginParams) -> Self {
        LfoOptions::new(
            LfoWave::try_from(param.wave.value() as u8).unwrap_or_default(),
            param.bipolar.value(),
            param.retrigger.value(),
        )
    }
}

impl From<&LfoPluginParams> for LfoParams<i16> {
    fn from(value: &LfoPluginParams) -> Self {
        LfoParams {
            freq: LfoFreqFxP::from_bits(value.rate.smoothed.next() as u16),
            depth: ScalarFxP::from_bits(value.depth.smoothed.next() as u16),
            opts: value.into(),
        }
    }
}

/// Contains all of the parameters for an oscillator within the plugin
#[derive(Params)]
pub struct RingModPluginParams {
    #[id = "vol_o1"]
    pub mix_a: IntParam,

    #[id = "vol_o2"]
    pub mix_b: IntParam,

    #[id = "ringmd"]
    pub mix_mod: IntParam,
}

impl Default for RingModPluginParams {
    fn default() -> Self {
        Self {
            mix_a: new_fixed_param_percent("Osc 1", ScalarFxP::MAX),
            mix_b: new_fixed_param_percent("Osc 2", ScalarFxP::ZERO),
            mix_mod: new_fixed_param_percent("Ring Mod", ScalarFxP::ZERO),
        }
    }
}

impl From<&RingModPluginParams> for RingModParams<i16> {
    fn from(value: &RingModPluginParams) -> Self {
        RingModParams {
            mix_a: ScalarFxP::from_bits(value.mix_a.smoothed.next() as u16),
            mix_b: ScalarFxP::from_bits(value.mix_b.smoothed.next() as u16),
            mix_mod: ScalarFxP::from_bits(value.mix_mod.smoothed.next() as u16),
        }
    }
}

/// Contains all of the parameters for a filter within the plugin
#[derive(Params)]
pub struct FiltPluginParams {
    #[id = "kbd"]
    pub kbd: IntParam,

    #[id = "vel"]
    pub vel: IntParam,

    #[id = "env"]
    pub env: IntParam,

    #[id = "cut"]
    pub cutoff: IntParam,

    #[id = "res"]
    pub res: IntParam,

    #[id = "low"]
    pub low: IntParam,

    #[id = "bnd"]
    pub band: IntParam,

    #[id = "hi"]
    pub high: IntParam,
}

impl Default for FiltPluginParams {
    fn default() -> Self {
        Self {
            env: new_fixed_param_percent("Filter Envelope Modulation", ScalarFxP::ZERO),
            kbd: new_fixed_param_percent("Filter Keyboard Tracking", ScalarFxP::ZERO),
            vel: new_fixed_param_percent("Filter Velocity Modulation", ScalarFxP::ZERO),
            cutoff: new_fixed_param_freq("Filter Cutoff", NoteFxP::lit("127")),
            res: new_fixed_param_percent("Filter Resonance", ScalarFxP::ZERO),
            low: new_fixed_param_percent("Filter Low Pass", ScalarFxP::MAX),
            band: new_fixed_param_percent("Filter Band Pass", ScalarFxP::ZERO),
            high: new_fixed_param_percent("Filter High Pass", ScalarFxP::ZERO),
        }
    }
}

impl From<&FiltPluginParams> for ModFiltParams<i16> {
    fn from(value: &FiltPluginParams) -> Self {
        ModFiltParams {
            env_mod: ScalarFxP::from_bits(value.env.smoothed.next() as u16),
            vel_mod: ScalarFxP::from_bits(value.vel.smoothed.next() as u16),
            kbd_tracking: ScalarFxP::from_bits(value.kbd.smoothed.next() as u16),
            cutoff: NoteFxP::from_bits(value.cutoff.smoothed.next() as u16),
            resonance: ScalarFxP::from_bits(value.res.smoothed.next() as u16),
            low_mix: ScalarFxP::from_bits(value.low.smoothed.next() as u16),
            band_mix: ScalarFxP::from_bits(value.band.smoothed.next() as u16),
            high_mix: ScalarFxP::from_bits(value.high.smoothed.next() as u16),
        }
    }
}

/// Contains all of the parameters for an envelope within the plugin
#[derive(Params)]
pub struct EnvPluginParams {
    #[id = "a"]
    pub a: IntParam,

    #[id = "d"]
    pub d: IntParam,

    #[id = "s"]
    pub s: IntParam,

    #[id = "r"]
    pub r: IntParam,
}

impl EnvPluginParams {
    fn new(name: &str) -> Self {
        Self {
            a: new_fixed_param_env(name.to_owned() + " Attack", EnvParamFxP::lit("0.1")),
            d: new_fixed_param_env(name.to_owned() + " Decay", EnvParamFxP::lit("0.1")),
            s: new_fixed_param_percent(name.to_owned() + " Sustain", ScalarFxP::MAX),
            r: new_fixed_param_env(name.to_owned() + " Release", EnvParamFxP::lit("0.1")),
        }
    }
}

impl From<&EnvPluginParams> for EnvParams<i16> {
    fn from(value: &EnvPluginParams) -> Self {
        EnvParams {
            attack: EnvParamFxP::from_bits(value.a.smoothed.next() as u16),
            decay: EnvParamFxP::from_bits(value.d.smoothed.next() as u16),
            sustain: ScalarFxP::from_bits(value.s.smoothed.next() as u16),
            release: EnvParamFxP::from_bits(value.r.smoothed.next() as u16),
        }
    }
}

#[derive(Params)]
struct ModMatrixEntry {
    #[id = "s"]
    pub slot: IntParam,
}

impl ModMatrixEntry {
    fn new(src: ModSrc, dest: ModDest) -> Self {
        Self {
            slot: new_fixed_param(
                format!("{} -> {}", src.to_str(), dest.to_str()),
                IScalarFxP::ZERO,
            ),
        }
    }
}

// This extra piece of indirection is necessary to make NIH happy
#[derive(Params)]
struct ModMatrixRow {
    #[nested(array)]
    pub entries: [ModMatrixEntry; ModDest::numel()],
}

#[derive(Params)]
pub struct ModMatrixPluginParams {
    #[nested(id_prefix = "MVel", group = "VelMod")]
    velocity: ModMatrixRow,
    #[nested(id_prefix = "MAft", group = "AftMod")]
    aftertouch: ModMatrixRow,
    #[nested(id_prefix = "MWhl", group = "WhlMod")]
    modwheel: ModMatrixRow,
    #[nested(id_prefix = "MEv1", group = "E1Mod")]
    env1: ModMatrixRow,
    #[nested(id_prefix = "MEv2", group = "E2Mod")]
    env2: ModMatrixRow,
    #[nested(id_prefix = "MLf1", group = "L1Mod")]
    lfo1: ModMatrixRow,
    #[nested(id_prefix = "MLf2", group = "L2Mod")]
    lfo2: ModMatrixRow,
}

impl Default for ModMatrixPluginParams {
    fn default() -> Self {
        Self::new()
    }
}

impl ModMatrixPluginParams {
    pub fn new() -> Self {
        Self {
            velocity: ModMatrixRow {
                entries: ModDest::ELEMENTS.map(|dest| ModMatrixEntry::new(ModSrc::Velocity, dest)),
            },
            aftertouch: ModMatrixRow {
                entries: ModDest::ELEMENTS
                    .map(|dest| ModMatrixEntry::new(ModSrc::Aftertouch, dest)),
            },
            modwheel: ModMatrixRow {
                entries: ModDest::ELEMENTS.map(|dest| ModMatrixEntry::new(ModSrc::ModWheel, dest)),
            },
            env1: ModMatrixRow {
                entries: ModDest::ELEMENTS.map(|dest| ModMatrixEntry::new(ModSrc::Env1, dest)),
            },
            env2: ModMatrixRow {
                entries: ModDest::ELEMENTS.map(|dest| ModMatrixEntry::new(ModSrc::Env2, dest)),
            },
            lfo1: ModMatrixRow {
                entries: ModDest::ELEMENTS.map(|dest| ModMatrixEntry::new(ModSrc::Lfo1, dest)),
            },
            lfo2: ModMatrixRow {
                entries: ModDest::ELEMENTS.map(|dest| ModMatrixEntry::new(ModSrc::Lfo2, dest)),
            },
        }
    }
    fn row(&self, src: ModSrc) -> &[ModMatrixEntry; ModDest::numel()] {
        match src {
            ModSrc::Velocity => &self.velocity.entries,
            ModSrc::Aftertouch => &self.aftertouch.entries,
            ModSrc::ModWheel => &self.modwheel.entries,
            ModSrc::Env1 => &self.env1.entries,
            ModSrc::Env2 => &self.env2.entries,
            ModSrc::Lfo1 => &self.lfo1.entries,
            ModSrc::Lfo2 => &self.lfo2.entries,
        }
    }
    pub fn entry(&self, src: ModSrc, dest: ModDest) -> &IntParam {
        &self.row(src)[dest as usize].slot
    }
}

impl From<&ModMatrixPluginParams> for ModMatrix<i16> {
    fn from(value: &ModMatrixPluginParams) -> Self {
        Self::from_fn(|src, dst| IScalarFxP::from_bits(value.entry(src, dst).value() as i16))
    }
}

/// Holds all of the plugin parameters
#[derive(Params)]
pub struct CulSynthParams {
    /// The editor state, saved together with the parameter state so the
    /// custom scaling can be restored.
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "osync"]
    pub osc_sync: BoolParam,

    #[nested(id_prefix = "o1", group = "osc1")]
    pub osc1: OscPluginParams,

    #[nested(id_prefix = "o2", group = "osc2")]
    pub osc2: OscPluginParams,

    #[nested(group = "ringmod")]
    pub ringmod: RingModPluginParams,

    #[nested(group = "filt")]
    pub filt: FiltPluginParams,

    #[nested(id_prefix = "envA", group = "envvca")]
    pub env_vca: EnvPluginParams,

    #[nested(id_prefix = "envF", group = "envvcf")]
    pub env_vcf: EnvPluginParams,

    #[nested(id_prefix = "lf1", group = "lfo1")]
    pub lfo1: LfoPluginParams,

    #[nested(id_prefix = "lf2", group = "lfo2")]
    pub lfo2: LfoPluginParams,

    #[nested(id_prefix = "env1", group = "envmd1")]
    pub env1: EnvPluginParams,

    #[nested(id_prefix = "env2", group = "envmd2")]
    pub env2: EnvPluginParams,

    #[nested(group = "Mod")]
    pub modmatrix: ModMatrixPluginParams,
}

impl CulSynthParams {
    pub fn bool_param_from_cc(&self, cc: wmidi::ControlFunction) -> Option<&BoolParam> {
        use culsynth::voice::cc;
        match cc {
            cc::LFO1_RETRIGGER => Some(&self.lfo1.retrigger),
            cc::LFO1_BIPOLAR => Some(&self.lfo1.bipolar),
            cc::LFO2_RETRIGGER => Some(&self.lfo2.retrigger),
            cc::LFO2_BIPOLAR => Some(&self.lfo2.bipolar),
            cc::OSC_SYNC => Some(&self.osc_sync),
            _ => None,
        }
    }
    pub fn int_param_from_cc(&self, cc: wmidi::ControlFunction) -> Option<&IntParam> {
        use culsynth::voice::cc;
        match cc {
            cc::OSC1_SIN => Some(&self.osc1.sin),
            cc::OSC1_SQ => Some(&self.osc1.sq),
            cc::OSC1_TRI => Some(&self.osc1.tri),
            cc::OSC1_SAW => Some(&self.osc1.saw),
            cc::RING_MIXA => Some(&self.ringmod.mix_a),
            cc::ENV_FILT_ATTACK => Some(&self.env_vcf.a),
            cc::ENV_FILT_DECAY => Some(&self.env_vcf.d),
            cc::ENV_FILT_SUSTAIN => Some(&self.env_vcf.s),
            cc::ENV_FILT_RELEASE => Some(&self.env_vcf.r),
            cc::OSC2_SIN => Some(&self.osc2.sin),
            cc::OSC2_SQ => Some(&self.osc2.sq),
            cc::OSC2_TRI => Some(&self.osc2.tri),
            cc::OSC2_SAW => Some(&self.osc2.saw),
            cc::RING_MIXB => Some(&self.ringmod.mix_b),
            cc::ENV_AMP_ATTACK => Some(&self.env_vca.a),
            cc::ENV_AMP_DECAY => Some(&self.env_vca.d),
            cc::ENV_AMP_SUSTAIN => Some(&self.env_vca.s),
            cc::ENV_AMP_RELEASE => Some(&self.env_vca.r),
            cc::FILT_CUTOFF => Some(&self.filt.cutoff),
            cc::FILT_RESONANCE => Some(&self.filt.res),
            cc::FILT_KBD => Some(&self.filt.kbd),
            cc::FILT_VEL => Some(&self.filt.vel),
            cc::FILT_ENV => Some(&self.filt.env),
            cc::FILT_LOW => Some(&self.filt.low),
            cc::FILT_BAND => Some(&self.filt.band),
            cc::FILT_HIGH => Some(&self.filt.high),
            cc::OSC1_SHAPE => Some(&self.osc1.shape),
            cc::RING_MIXMOD => Some(&self.ringmod.mix_mod),
            cc::OSC2_FINE => Some(&self.osc2.fine),
            cc::LFO1_RATE => Some(&self.lfo1.rate),
            cc::LFO1_DEPTH => Some(&self.lfo1.depth),
            cc::LFO1_WAVE => Some(&self.lfo1.wave),
            cc::LFO2_RATE => Some(&self.lfo2.rate),
            cc::LFO2_DEPTH => Some(&self.lfo2.depth),
            cc::LFO2_WAVE => Some(&self.lfo2.wave),
            cc::OSC2_SHAPE => Some(&self.osc2.shape),
            cc::OSC1_COARSE => Some(&self.osc1.coarse),
            cc::OSC1_FINE => Some(&self.osc1.fine),
            cc::OSC2_COARSE => Some(&self.osc2.coarse),
            cc::ENV_M1_ATTACK => Some(&self.env1.a),
            cc::ENV_M1_DECAY => Some(&self.env1.d),
            cc::ENV_M1_SUSTAIN => Some(&self.env1.s),
            cc::ENV_M1_RELEASE => Some(&self.env1.r),
            cc::ENV_M2_ATTACK => Some(&self.env2.a),
            cc::ENV_M2_DECAY => Some(&self.env2.d),
            cc::ENV_M2_SUSTAIN => Some(&self.env2.s),
            cc::ENV_M2_RELEASE => Some(&self.env2.r),
            _ => None,
        }
    }
}

impl Default for CulSynthParams {
    fn default() -> Self {
        Self {
            editor_state: crate::nih::editor::default_state(),
            osc_sync: BoolParam::new("Oscillator Sync", false),
            osc1: Default::default(),
            osc2: Default::default(),
            ringmod: Default::default(),
            filt: Default::default(),
            env_vca: EnvPluginParams::new("VCA Envelope"),
            env_vcf: EnvPluginParams::new("VCF Envelope"),
            lfo1: LfoPluginParams::new("LFO1"),
            lfo2: LfoPluginParams::new("LFO2"),
            env1: EnvPluginParams::new("Mod Envelope 1"),
            env2: EnvPluginParams::new("Mod Envelope 2"),
            modmatrix: ModMatrixPluginParams::new(),
        }
    }
}

impl From<&CulSynthParams> for VoiceParams<i16> {
    fn from(value: &CulSynthParams) -> Self {
        VoiceParams {
            oscs_p: SyncedMixOscsParams {
                primary: MixOscParams::from(&value.osc1),
                secondary: MixOscParams::from(&value.osc2),
                sync: value.osc_sync.value(),
            },
            ring_p: RingModParams::from(&value.ringmod),
            filt_p: ModFiltParams::from(&value.filt),
            filt_env_p: EnvParams::from(&value.env_vcf),
            amp_env_p: EnvParams::from(&value.env_vca),
            lfo1_p: LfoParams::from(&value.lfo1),
            lfo2_p: LfoParams::from(&value.lfo2),
            env1_p: EnvParams::from(&value.env1),
            env2_p: EnvParams::from(&value.env2),
        }
    }
}
