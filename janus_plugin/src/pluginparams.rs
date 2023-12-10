use janus::devices::{LfoOptions, LfoWave};
use janus::{EnvParamFxP, NoteFxP, ScalarFxP, LfoFreqFxP};
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;

use std::sync::Arc;

use crate::fixedparam::{new_fixed_param, new_fixed_param_freq, new_fixed_param_percent};

/// Contains all of the parameters for an oscillator within the plugin
#[derive(Params)]
pub struct OscPluginParams {
    /// Course tuning: -32 to +32 semitones
    #[id = "course"]
    pub course: IntParam,

    /// Fine tuning: -1024 to 1024 mapping to -2 to +2 semitones
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

impl Default for OscPluginParams {
    fn default() -> Self {
        Self {
            course: IntParam::new("Course", 0, IntRange::Linear{ min: -32, max: 32}),
            fine: IntParam::new("Fine", 0, IntRange::Linear{ min: -1024, max: 1024}),
            shape: new_fixed_param("Shape", ScalarFxP::ZERO),
            sin: new_fixed_param_percent("Sin", ScalarFxP::ZERO),
            saw: new_fixed_param_percent("Saw", ScalarFxP::MAX),
            sq: new_fixed_param_percent("Square", ScalarFxP::ZERO),
            tri: new_fixed_param_percent("Triangle", ScalarFxP::ZERO),
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
            rate: new_fixed_param(name.to_owned() + " Rate", LfoFreqFxP::ONE),
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
            a: new_fixed_param(name.to_owned() + " Attack", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
            d: new_fixed_param(name.to_owned() + " Decay", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
            s: new_fixed_param_percent(name.to_owned() + " Sustain", ScalarFxP::MAX),
            r: new_fixed_param(name.to_owned() + " Release", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
        }
    }
}

/// Holds all of the plugin parameters
#[derive(Params)]
pub struct JanusParams {
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
}

impl Default for JanusParams {
    fn default() -> Self {
        Self {
            editor_state: crate::editor::default_state(),
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
        }
    }
}
