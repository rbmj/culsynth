//! This module contains definitions of several different DSP primitives.

mod amp;
mod env;
mod filt;
mod lfo;
mod mixosc;
mod modfilt;
mod osc;
mod ringmod;

use super::{
    fixedmath, EnvParamFxP, LfoFreqFxP, NoteFxP, SampleFxP, ScalarFxP, SignedNoteFxP, USampleFxP,
};

use super::context::Context;
use super::context::ContextFxP;
use super::min_size;
use super::BufferT;
use super::STATIC_BUFFER_SIZE;

/// Types must implement this trait to instantiate any of the generic devices
/// in this module.  Implementations are provided for `f32` and `f64`.
pub trait Float: num_traits::Float + num_traits::FloatConst + From<u16> {
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const THREE: Self;
    const ONE_HALF: Self;
    const POINT_NINE_EIGHT: Self;
    const RES_MAX: Self;
    const NOTE_MAX: Self;
    const SHAPE_CLIP: Self;
    fn from_u16(x: u16) -> Self {
        <Self as From<u16>>::from(x)
    }
}

impl Float for f32 {
    const ZERO: f32 = 0.0f32;
    const ONE: f32 = 1.0f32;
    const TWO: f32 = 2.0f32;
    const THREE: f32 = 3.0f32;
    const ONE_HALF: f32 = 0.5f32;
    const POINT_NINE_EIGHT: f32 = 0.98f32;
    const RES_MAX: f32 = 0xF000 as f32 / 0xFFFF as f32;
    const NOTE_MAX: f32 = 127.0f32 * (0xFFFF as f32 / 0x10000 as f32);
    const SHAPE_CLIP: f32 = 0.9375f32;
}

impl Float for f64 {
    const ZERO: f64 = 0.0f64;
    const ONE: f64 = 1.0f64;
    const TWO: f64 = 2.0f64;
    const THREE: f64 = 3.0f64;
    const ONE_HALF: f64 = 0.5f64;
    const POINT_NINE_EIGHT: f64 = 0.98f64;
    const RES_MAX: f64 = 0xF000 as f64 / 0xFFFF as f64;
    const NOTE_MAX: f64 = 127.0f64 * (0xFFFF as f64 / 0x10000 as f64);
    const SHAPE_CLIP: f64 = 0.9375f64;
}

/// Converts a MIDI note number to a frequency
fn midi_note_to_frequency<T: Float>(note: T) -> T {
    let c69 = T::from_u16(69);
    let c12 = T::from_u16(12);
    let c440 = T::from_u16(440);
    c440 * ((note - c69) / c12).exp2()
}

pub use amp::{Amp, AmpFxP};
pub use env::{Env, EnvFxP, EnvParams, EnvParamsFxP};
pub use filt::{Filt, FiltFxP, FiltOutput, FiltOutputFxP, FiltParams, FiltParamsFxP};
pub use lfo::{Lfo, LfoFxP, LfoParam, LfoWave};
pub use mixosc::{MixOsc, MixOscFxP, MixOscParams, MixOscParamsFxP};
pub use modfilt::{ModFilt, ModFiltFxP, ModFiltParams, ModFiltParamsFxP};
pub use osc::{Osc, OscFxP, OscOutput, OscParams, OscParamsFxP, OscSync};
pub use ringmod::{RingMod, RingModFxP, RingModParams, RingModParamsFxP};
