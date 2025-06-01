//! Contains constant definitions for mapping MIDI CCs to voice parameters
//!
//! These are based on the default MIDI CC assignments for the Arturia KeyLab,
//! so users of that keyboard will have an intuitive assignment of CCs when
//! using the default user preset.
//!
//! Bank 1:
//!  - Knobs: Filter Params (in order) + Osc 1 Shape.
//!  - Faders: VCF Envelope A/D/S/R + Osc 1 Sin/Sq/Tri/Saw/Vol
//! Bank 2:
//!  - Knobs: Ringmod Mix + Osc 2 Fine + LFOs Rate/Depth/Wave + Osc 2 Shape
//!  - Faders: VCA Envelope A/D/S/R + Osc 2  Sin/Sq/Tri/Saw/Vol
//! Buttons:
//!  - LFO1 Retrigger/Bipolar, LFO2 Retrigger/Bipolar, Osc Sync

#![allow(missing_docs)]

use wmidi::{ControlFunction, U7};

use super::modulation::ModDest;

pub const CC_SIGNED_ZERO: wmidi::U7 = U7::from_u8_lossy(64);

pub const OSC1_SIN: ControlFunction = ControlFunction(U7::from_u8_lossy(80));
pub const OSC1_SQ: ControlFunction = ControlFunction(U7::from_u8_lossy(81));
pub const OSC1_TRI: ControlFunction = ControlFunction(U7::from_u8_lossy(82));
pub const OSC1_SAW: ControlFunction = ControlFunction(U7::from_u8_lossy(83));
pub const RING_MIXA: ControlFunction = ControlFunction(U7::from_u8_lossy(85));

pub const ENV_FILT_ATTACK: ControlFunction = ControlFunction(U7::from_u8_lossy(73));
pub const ENV_FILT_DECAY: ControlFunction = ControlFunction(U7::from_u8_lossy(75));
pub const ENV_FILT_SUSTAIN: ControlFunction = ControlFunction(U7::from_u8_lossy(79));
pub const ENV_FILT_RELEASE: ControlFunction = ControlFunction(U7::from_u8_lossy(72));

pub const OSC2_SIN: ControlFunction = ControlFunction(U7::from_u8_lossy(87));
pub const OSC2_SQ: ControlFunction = ControlFunction(U7::from_u8_lossy(88));
pub const OSC2_TRI: ControlFunction = ControlFunction(U7::from_u8_lossy(89));
pub const OSC2_SAW: ControlFunction = ControlFunction(U7::from_u8_lossy(90));
pub const RING_MIXB: ControlFunction = ControlFunction(U7::from_u8_lossy(92));

pub const ENV_AMP_ATTACK: ControlFunction = ControlFunction(U7::from_u8_lossy(67));
pub const ENV_AMP_DECAY: ControlFunction = ControlFunction(U7::from_u8_lossy(68));
pub const ENV_AMP_SUSTAIN: ControlFunction = ControlFunction(U7::from_u8_lossy(69));
pub const ENV_AMP_RELEASE: ControlFunction = ControlFunction(U7::from_u8_lossy(70));

pub const FILT_CUTOFF: ControlFunction = ControlFunction(U7::from_u8_lossy(74));
pub const FILT_RESONANCE: ControlFunction = ControlFunction(U7::from_u8_lossy(71));
pub const FILT_KBD: ControlFunction = ControlFunction(U7::from_u8_lossy(76));
pub const FILT_VEL: ControlFunction = ControlFunction(U7::from_u8_lossy(77));
pub const FILT_ENV: ControlFunction = ControlFunction(U7::from_u8_lossy(93));
pub const FILT_LOW: ControlFunction = ControlFunction(U7::from_u8_lossy(18));
pub const FILT_BAND: ControlFunction = ControlFunction(U7::from_u8_lossy(19));
pub const FILT_HIGH: ControlFunction = ControlFunction(U7::from_u8_lossy(16));
pub const OSC1_SHAPE: ControlFunction = ControlFunction(U7::from_u8_lossy(17));

pub const RING_MIXMOD: ControlFunction = ControlFunction(U7::from_u8_lossy(35));
pub const OSC2_FINE: ControlFunction = ControlFunction(U7::from_u8_lossy(36));
pub const LFO1_RATE: ControlFunction = ControlFunction(U7::from_u8_lossy(37));
pub const LFO1_DEPTH: ControlFunction = ControlFunction(U7::from_u8_lossy(31));
pub const LFO1_WAVE: ControlFunction = ControlFunction(U7::from_u8_lossy(39));
pub const LFO2_RATE: ControlFunction = ControlFunction(U7::from_u8_lossy(40));
pub const LFO2_DEPTH: ControlFunction = ControlFunction(U7::from_u8_lossy(41));
pub const LFO2_WAVE: ControlFunction = ControlFunction(U7::from_u8_lossy(42));
pub const OSC2_SHAPE: ControlFunction = ControlFunction(U7::from_u8_lossy(43));

pub const LFO1_RETRIGGER: ControlFunction = ControlFunction(U7::from_u8_lossy(22));
pub const LFO1_BIPOLAR: ControlFunction = ControlFunction(U7::from_u8_lossy(23));
pub const LFO2_RETRIGGER: ControlFunction = ControlFunction(U7::from_u8_lossy(24));
pub const LFO2_BIPOLAR: ControlFunction = ControlFunction(U7::from_u8_lossy(25));
pub const OSC_SYNC: ControlFunction = ControlFunction(U7::from_u8_lossy(26));

pub const OSC1_COARSE: ControlFunction = ControlFunction(U7::from_u8_lossy(14));
pub const OSC1_FINE: ControlFunction = ControlFunction(U7::from_u8_lossy(15));
pub const OSC2_COARSE: ControlFunction = ControlFunction(U7::from_u8_lossy(21));

pub const ENV_M1_ATTACK: ControlFunction = ControlFunction(U7::from_u8_lossy(102));
pub const ENV_M1_DECAY: ControlFunction = ControlFunction(U7::from_u8_lossy(103));
pub const ENV_M1_SUSTAIN: ControlFunction = ControlFunction(U7::from_u8_lossy(104));
pub const ENV_M1_RELEASE: ControlFunction = ControlFunction(U7::from_u8_lossy(105));

pub const ENV_M2_ATTACK: ControlFunction = ControlFunction(U7::from_u8_lossy(106));
pub const ENV_M2_DECAY: ControlFunction = ControlFunction(U7::from_u8_lossy(107));
pub const ENV_M2_SUSTAIN: ControlFunction = ControlFunction(U7::from_u8_lossy(108));
pub const ENV_M2_RELEASE: ControlFunction = ControlFunction(U7::from_u8_lossy(109));

pub struct OscCCs {
    pub sin: ControlFunction,
    pub mod_sin: ModDest,
    pub sq: ControlFunction,
    pub mod_sq: ModDest,
    pub tri: ControlFunction,
    pub mod_tri: ModDest,
    pub saw: ControlFunction,
    pub mod_saw: ModDest,
    pub shape: ControlFunction,
    pub mod_shape: ModDest,
    pub coarse: ControlFunction,
    pub mod_coarse: ModDest,
    pub fine: ControlFunction,
    pub mod_fine: ModDest,
}

pub const OSC1_CC_ALL: OscCCs = OscCCs {
    sin: OSC1_SIN,
    mod_sin: ModDest::Osc1Sin,
    sq: OSC1_SQ,
    mod_sq: ModDest::Osc1Sq,
    tri: OSC1_TRI,
    mod_tri: ModDest::Osc1Tri,
    saw: OSC1_SAW,
    mod_saw: ModDest::Osc1Saw,
    shape: OSC1_SHAPE,
    mod_shape: ModDest::Osc1Shape,
    coarse: OSC1_COARSE,
    mod_coarse: ModDest::Osc1Coarse,
    fine: OSC1_FINE,
    mod_fine: ModDest::Osc1Fine,
};

pub const OSC2_CC_ALL: OscCCs = OscCCs {
    sin: OSC2_SIN,
    mod_sin: ModDest::Osc2Sin,
    sq: OSC2_SQ,
    mod_sq: ModDest::Osc2Sq,
    tri: OSC2_TRI,
    mod_tri: ModDest::Osc2Tri,
    saw: OSC2_SAW,
    mod_saw: ModDest::Osc2Saw,
    shape: OSC2_SHAPE,
    mod_shape: ModDest::Osc2Shape,
    coarse: OSC2_COARSE,
    mod_coarse: ModDest::Osc2Coarse,
    fine: OSC2_FINE,
    mod_fine: ModDest::Osc2Fine,
};

pub struct EnvCCs {
    pub attack: ControlFunction,
    pub mod_attack: ModDest,
    pub decay: ControlFunction,
    pub mod_decay: ModDest,
    pub sustain: ControlFunction,
    pub mod_sustain: ModDest,
    pub release: ControlFunction,
    pub mod_release: ModDest,
}

pub const ENV_AMP_CCS_ALL: EnvCCs = EnvCCs {
    attack: ENV_AMP_ATTACK,
    mod_attack: ModDest::EnvAmpA,
    decay: ENV_AMP_DECAY,
    mod_decay: ModDest::EnvAmpD,
    sustain: ENV_AMP_SUSTAIN,
    mod_sustain: ModDest::EnvAmpS,
    release: ENV_AMP_RELEASE,
    mod_release: ModDest::EnvAmpR,
};

pub const ENV_FILT_CCS_ALL: EnvCCs = EnvCCs {
    attack: ENV_FILT_ATTACK,
    mod_attack: ModDest::EnvFiltA,
    decay: ENV_FILT_DECAY,
    mod_decay: ModDest::EnvFiltD,
    sustain: ENV_FILT_SUSTAIN,
    mod_sustain: ModDest::EnvFiltS,
    release: ENV_FILT_RELEASE,
    mod_release: ModDest::EnvFiltR,
};

pub const ENV_M1_CCS_ALL: EnvCCs = EnvCCs {
    attack: ENV_M1_ATTACK,
    mod_attack: ModDest::Env1A,
    decay: ENV_M1_DECAY,
    mod_decay: ModDest::Env1D,
    sustain: ENV_M1_SUSTAIN,
    mod_sustain: ModDest::Env1S,
    release: ENV_M1_RELEASE,
    mod_release: ModDest::Env1R,
};

pub const ENV_M2_CCS_ALL: EnvCCs = EnvCCs {
    attack: ENV_M2_ATTACK,
    mod_attack: ModDest::Env2A,
    decay: ENV_M2_DECAY,
    mod_decay: ModDest::Env2D,
    sustain: ENV_M2_SUSTAIN,
    mod_sustain: ModDest::Env2S,
    release: ENV_M2_RELEASE,
    mod_release: ModDest::Env2R,
};

pub struct FiltCCs {
    pub cutoff: ControlFunction,
    pub mod_cutoff: ModDest,
    pub resonance: ControlFunction,
    pub mod_resonance: ModDest,
    pub kbd: ControlFunction,
    pub mod_kbd: ModDest,
    pub vel: ControlFunction,
    pub mod_vel: ModDest,
    pub env: ControlFunction,
    pub mod_env: ModDest,
    pub low: ControlFunction,
    pub mod_low: ModDest,
    pub band: ControlFunction,
    pub mod_band: ModDest,
    pub high: ControlFunction,
    pub mod_high: ModDest,
}

pub const FILT_CCS_ALL: FiltCCs = FiltCCs {
    cutoff: FILT_CUTOFF,
    mod_cutoff: ModDest::FiltCutoff,
    resonance: FILT_RESONANCE,
    mod_resonance: ModDest::FiltRes,
    kbd: FILT_KBD,
    mod_kbd: ModDest::FiltKbd,
    vel: FILT_VEL,
    mod_vel: ModDest::FiltVel,
    env: FILT_ENV,
    mod_env: ModDest::FiltEnv,
    low: FILT_LOW,
    mod_low: ModDest::FiltLow,
    band: FILT_BAND,
    mod_band: ModDest::FiltBand,
    high: FILT_HIGH,
    mod_high: ModDest::FiltHigh,
};

pub struct LfoCCs {
    pub rate: ControlFunction,
    pub mod_rate: ModDest,
    pub depth: ControlFunction,
    pub mod_depth: ModDest,
    pub wave: ControlFunction,
    pub retrigger: ControlFunction,
    pub bipolar: ControlFunction,
}

pub const LFO1_CCS_ALL: LfoCCs = LfoCCs {
    rate: LFO1_RATE,
    mod_rate: ModDest::Lfo1Rate,
    depth: LFO1_DEPTH,
    mod_depth: ModDest::Lfo1Depth,
    wave: LFO1_WAVE,
    retrigger: LFO1_RETRIGGER,
    bipolar: LFO1_BIPOLAR,
};

pub const LFO2_CCS_ALL: LfoCCs = LfoCCs {
    rate: LFO2_RATE,
    mod_rate: ModDest::Lfo2Rate,
    depth: LFO2_DEPTH,
    mod_depth: ModDest::Lfo2Depth,
    wave: LFO2_WAVE,
    retrigger: LFO2_RETRIGGER,
    bipolar: LFO2_BIPOLAR,
};

pub struct RingModCCs {
    pub mix_a: ControlFunction,
    pub mod_mix_a: ModDest,
    pub mix_b: ControlFunction,
    pub mod_mix_b: ModDest,
    pub mix_mod: ControlFunction,
    pub mod_mix_mod: ModDest,
}

pub const RING_CCS_ALL: RingModCCs = RingModCCs {
    mix_a: RING_MIXA,
    mod_mix_a: ModDest::RingOsc1,
    mix_b: RING_MIXB,
    mod_mix_b: ModDest::RingOsc2,
    mix_mod: RING_MIXMOD,
    mod_mix_mod: ModDest::RingMod,
};
