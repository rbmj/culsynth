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
pub const LFO1_DEPTH: ControlFunction = ControlFunction(U7::from_u8_lossy(38));
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
    pub sq: ControlFunction,
    pub tri: ControlFunction,
    pub saw: ControlFunction,
    pub shape: ControlFunction,
    pub coarse: ControlFunction,
    pub fine: ControlFunction,
}

pub const OSC1_CC_ALL: OscCCs = OscCCs {
    sin: OSC1_SIN,
    sq: OSC1_SQ,
    tri: OSC1_TRI,
    saw: OSC1_SAW,
    shape: OSC1_SHAPE,
    coarse: OSC1_COARSE,
    fine: OSC1_FINE,
};

pub const OSC2_CC_ALL: OscCCs = OscCCs {
    sin: OSC2_SIN,
    sq: OSC2_SQ,
    tri: OSC2_TRI,
    saw: OSC2_SAW,
    shape: OSC2_SHAPE,
    coarse: OSC2_COARSE,
    fine: OSC2_FINE,
};

pub struct EnvCCs {
    pub attack: ControlFunction,
    pub decay: ControlFunction,
    pub sustain: ControlFunction,
    pub release: ControlFunction,
}

pub const ENV_AMP_CCS_ALL: EnvCCs = EnvCCs {
    attack: ENV_AMP_ATTACK,
    decay: ENV_AMP_DECAY,
    sustain: ENV_AMP_SUSTAIN,
    release: ENV_AMP_RELEASE,
};

pub const ENV_FILT_CCS_ALL: EnvCCs = EnvCCs {
    attack: ENV_FILT_ATTACK,
    decay: ENV_FILT_DECAY,
    sustain: ENV_FILT_SUSTAIN,
    release: ENV_FILT_RELEASE,
};

pub const ENV_M1_CCS_ALL: EnvCCs = EnvCCs {
    attack: ENV_M1_ATTACK,
    decay: ENV_M1_DECAY,
    sustain: ENV_M1_SUSTAIN,
    release: ENV_M1_RELEASE,
};

pub const ENV_M2_CCS_ALL: EnvCCs = EnvCCs {
    attack: ENV_M2_ATTACK,
    decay: ENV_M2_DECAY,
    sustain: ENV_M2_SUSTAIN,
    release: ENV_M2_RELEASE,
};

pub struct FiltCCs {
    pub cutoff: ControlFunction,
    pub resonance: ControlFunction,
    pub kbd: ControlFunction,
    pub vel: ControlFunction,
    pub env: ControlFunction,
    pub low: ControlFunction,
    pub band: ControlFunction,
    pub high: ControlFunction,
}

pub const FILT_CCS_ALL: FiltCCs = FiltCCs {
    cutoff: FILT_CUTOFF,
    resonance: FILT_RESONANCE,
    kbd: FILT_KBD,
    vel: FILT_VEL,
    env: FILT_ENV,
    low: FILT_LOW,
    band: FILT_BAND,
    high: FILT_HIGH,
};

pub struct LfoCCs {
    pub rate: ControlFunction,
    pub depth: ControlFunction,
    pub wave: ControlFunction,
    pub retrigger: ControlFunction,
    pub bipolar: ControlFunction,
}

pub const LFO1_CCS_ALL: LfoCCs = LfoCCs {
    rate: LFO1_RATE,
    depth: LFO1_DEPTH,
    wave: LFO1_WAVE,
    retrigger: LFO1_RETRIGGER,
    bipolar: LFO1_BIPOLAR,
};

pub const LFO2_CCS_ALL: LfoCCs = LfoCCs {
    rate: LFO2_RATE,
    depth: LFO2_DEPTH,
    wave: LFO2_WAVE,
    retrigger: LFO2_RETRIGGER,
    bipolar: LFO2_BIPOLAR,
};

pub struct RingModCCs {
    pub mix_a: ControlFunction,
    pub mix_b: ControlFunction,
    pub mix_mod: ControlFunction,
}

pub const RING_CCS_ALL: RingModCCs = RingModCCs {
    mix_a: RING_MIXA,
    mix_b: RING_MIXB,
    mix_mod: RING_MIXMOD,
};

pub fn modmatrix_nrpn_lsb(src: crate::voice::modulation::ModSrc, slot: usize) -> u8 {
    let src = src as u8;
    let slot = slot as u8;
    let lsb = ((src & 0xF) | (slot << 4)) & 0x7F;
    lsb
}
