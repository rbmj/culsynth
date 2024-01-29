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

pub const OSC1_SIN: u8 = 80;
pub const OSC1_SQ: u8 = 81;
pub const OSC1_TRI: u8 = 82;
pub const OSC1_SAW: u8 = 83;
pub const RING_MIXA: u8 = 85;

pub const ENV_FILT_ATTACK: u8 = 73;
pub const ENV_FILT_DECAY: u8 = 75;
pub const ENV_FILT_SUSTAIN: u8 = 79;
pub const ENV_FILT_RELEASE: u8 = 72;

pub const OSC2_SIN: u8 = 87;
pub const OSC2_SQ: u8 = 88;
pub const OSC2_TRI: u8 = 89;
pub const OSC2_SAW: u8 = 90;
pub const RING_MIXB: u8 = 92;

pub const ENV_AMP_ATTACK: u8 = 67;
pub const ENV_AMP_DECAY: u8 = 68;
pub const ENV_AMP_SUSTAIN: u8 = 69;
pub const ENV_AMP_RELEASE: u8 = 70;

pub const FILT_CUTOFF: u8 = 74;
pub const FILT_RESONANCE: u8 = 71;
pub const FILT_KBD: u8 = 76;
pub const FILT_VEL: u8 = 77;
pub const FILT_ENV: u8 = 93;
pub const FILT_LOW: u8 = 18;
pub const FILT_BAND: u8 = 19;
pub const FILT_HIGH: u8 = 16;
pub const OSC1_SHAPE: u8 = 17;

pub const RING_MIXMOD: u8 = 35;
pub const OSC2_FINE: u8 = 36;
pub const LFO1_RATE: u8 = 37;
pub const LFO1_DEPTH: u8 = 38;
pub const LFO1_WAVE: u8 = 39;
pub const LFO2_RATE: u8 = 40;
pub const LFO2_DEPTH: u8 = 41;
pub const LFO2_WAVE: u8 = 42;
pub const OSC2_SHAPE: u8 = 43;

pub const LFO1_RETRIGGER: u8 = 22;
pub const LFO1_BIPOLAR: u8 = 23;
pub const LFO2_RETRIGGER: u8 = 24;
pub const LFO2_BIPOLAR: u8 = 25;
pub const OSC_SYNC: u8 = 26;
