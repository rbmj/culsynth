//! This module contains definitions of several different DSP primitives.

mod amp;
mod env;
mod filt;
mod lfo;
mod mixosc;
mod modfilt;
mod osc;
mod ringmod;

use crate::context::{Context, ContextFxP};
use crate::{
    fixedmath, EnvParamFxP, LfoFreqFxP, NoteFxP, SampleFxP, ScalarFxP, SignedNoteFxP, USampleFxP,
};
use crate::{min_size, BufferT, Fixed16, STATIC_BUFFER_SIZE};

static U16_ZEROBUF: [u16; STATIC_BUFFER_SIZE] = [0u16; STATIC_BUFFER_SIZE];

/// Get a slice to a buffer of [STATIC_BUFFER_SIZE] fixed-point zero values
/// for any 16-bit fixed point type
pub fn fixed_zerobuf<T: Fixed16>() -> &'static [T] {
    // Fixed is #[repr(transparent)], so this is valid:
    unsafe { core::mem::transmute(&U16_ZEROBUF[0..STATIC_BUFFER_SIZE]) }
}

/// Types must implement this trait to instantiate any of the generic devices
/// in this module.  Implementations are provided for `f32` and `f64`.
pub trait Float: num_traits::Float + num_traits::FloatConst + From<u16> + Default + Copy {
    /// 0
    const ZERO: Self;
    /// 1
    const ONE: Self;
    /// 2
    const TWO: Self;
    /// 3
    const THREE: Self;
    /// 1/2
    const ONE_HALF: Self;
    /// 0.98
    const POINT_NINE_EIGHT: Self;
    /// 0xF000 / 0xFFFF
    const RES_MAX: Self;
    /// 127 * (0xFFFF / 0x10000)
    const NOTE_MAX: Self;
    /// 0.9375
    const SHAPE_CLIP: Self;
    /// Creates a value of this type from a u16.  Functionality provided by
    /// the trait (uses the `From<u16>` implementation)
    fn from_u16(x: u16) -> Self {
        <Self as From<u16>>::from(x)
    }
    /// Return a buffer of zeros
    fn zerobuf<'a>() -> &'a [Self; STATIC_BUFFER_SIZE];
}

static F32_ZEROBUF: [f32; STATIC_BUFFER_SIZE] = [0f32; STATIC_BUFFER_SIZE];
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
    fn zerobuf<'a>() -> &'a [Self; STATIC_BUFFER_SIZE] {
        &F32_ZEROBUF
    }
}

static F64_ZEROBUF: [f64; STATIC_BUFFER_SIZE] = [0f64; STATIC_BUFFER_SIZE];
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
    fn zerobuf<'a>() -> &'a [Self; STATIC_BUFFER_SIZE] {
        &F64_ZEROBUF
    }
}

/// Converts a MIDI note number to a frequency
fn midi_note_to_frequency<T: Float>(note: T) -> T {
    let c69 = T::from_u16(69);
    let c12 = T::from_u16(12);
    let c440 = T::from_u16(440);
    c440 * ((note - c69) / c12).exp2()
}

pub use amp::{Amp, AmpFxP};
pub use env::{Env, EnvFxP, EnvParams, EnvParamsFxP, MutEnvParams, MutEnvParamsFxP};
pub use filt::{
    Filt, FiltFxP, FiltOutput, FiltOutputFxP, FiltParams, FiltParamsFxP, MutFiltParamsFxP,
};
pub use lfo::{
    Lfo, LfoFxP, LfoOptions, LfoParams, LfoParamsFxP, LfoWave, MutLfoParams, MutLfoParamsFxP,
};
pub use mixosc::{
    MixOsc, MixOscFxP, MixOscParams, MixOscParamsFxP, MutMixOscParams, MutMixOscParamsFxP,
};
pub use modfilt::{
    ModFilt, ModFiltFxP, ModFiltParams, ModFiltParamsFxP, MutModFiltParams, MutModFiltParamsFxP,
};
pub use osc::{Osc, OscFxP, OscOutput, OscParams, OscParamsFxP, OscSync};
pub use ringmod::{
    MutRingModParams, MutRingModParamsFxP, RingMod, RingModFxP, RingModParams, RingModParamsFxP,
};
