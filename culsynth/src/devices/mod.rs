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

pub(crate) use osc::PhaseFxP;

static U16_ZEROBUF: [u16; STATIC_BUFFER_SIZE] = [0u16; STATIC_BUFFER_SIZE];

/// Get a slice to a buffer of [STATIC_BUFFER_SIZE] fixed-point zero values
/// for any 16-bit fixed point type
pub fn fixed_zerobuf<T: Fixed16>() -> &'static [T] {
    // Fixed is #[repr(transparent)], so this is valid:
    unsafe { core::mem::transmute(&U16_ZEROBUF[0..STATIC_BUFFER_SIZE]) }
}

#[cfg(not(feature = "libm"))]
use num_traits::float::FloatCore as NumTraitsFloat;
#[cfg(feature = "libm")]
use num_traits::Float as NumTraitsFloat;

/// Types must implement this trait to instantiate any of the generic devices
/// in this module.  Implementations are provided for `f32` and `f64`.
pub trait Float: NumTraitsFloat + From<u16> + Default + Copy {
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
    /// 0.1
    const POINT_ONE: Self;
    /// 0.98
    const POINT_NINE_EIGHT: Self;
    /// 0xF000 / 0xFFFF
    const RES_MAX: Self;
    /// 127 * (0xFFFF / 0x10000)
    const NOTE_MAX: Self;
    /// 0.9375
    const SHAPE_CLIP: Self;
    /// 2 / pi
    const FRAC_2_PI: Self;
    /// pi / 2
    const FRAC_PI_2: Self;
    /// pi
    const PI: Self;
    /// 2*pi
    const TAU: Self;
    /// Creates a value of this type from a u16.  Functionality provided by
    /// the trait (uses the `From<u16>` implementation)
    fn from_u16(x: u16) -> Self {
        <Self as From<u16>>::from(x)
    }
    /// Return a buffer of zeros
    fn zerobuf<'a>() -> &'a [Self; STATIC_BUFFER_SIZE];
    /// Returns the sine of self
    fn fsin(self) -> Self;
    /// Returns the cosine of self
    fn fcos(self) -> Self;
    /// Returns the tangent of self
    fn ftan(self) -> Self;
    /// Convert a MIDI note number to a frequency
    fn midi_to_freq(self) -> Self;
}

static F32_ZEROBUF: [f32; STATIC_BUFFER_SIZE] = [0f32; STATIC_BUFFER_SIZE];
impl Float for f32 {
    const ZERO: f32 = 0.0f32;
    const ONE: f32 = 1.0f32;
    const TWO: f32 = 2.0f32;
    const THREE: f32 = 3.0f32;
    const ONE_HALF: f32 = 0.5f32;
    const POINT_ONE: f32 = 0.1f32;
    const POINT_NINE_EIGHT: f32 = 0.98f32;
    const FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;
    const FRAC_2_PI: f32 = core::f32::consts::FRAC_2_PI;
    const PI: f32 = core::f32::consts::PI;
    const TAU: f32 = core::f32::consts::TAU;
    const RES_MAX: f32 = 0xF000 as f32 / 0xFFFF as f32;
    const NOTE_MAX: f32 = 127.0f32 * (0xFFFF as f32 / 0x10000 as f32);
    const SHAPE_CLIP: f32 = 0.9375f32;
    fn zerobuf<'a>() -> &'a [Self; STATIC_BUFFER_SIZE] {
        &F32_ZEROBUF
    }
    fn fsin(self) -> Self {
        #[cfg(not(feature = "libm"))]
        let ret = crate::float_approx::sin_approx(self);
        #[cfg(feature = "libm")]
        let ret = <Self as NumTraitsFloat>::sin(self);
        ret
    }
    fn fcos(self) -> Self {
        #[cfg(not(feature = "libm"))]
        let ret = crate::float_approx::cos_approx(self);
        #[cfg(feature = "libm")]
        let ret = <Self as NumTraitsFloat>::cos(self);
        ret
    }
    fn ftan(self) -> Self {
        #[cfg(not(feature = "libm"))]
        let ret = crate::float_approx::tan_approx(self);
        #[cfg(feature = "libm")]
        let ret = <Self as NumTraitsFloat>::tan(self);
        ret
    }
    fn midi_to_freq(self) -> Self {
        #[cfg(not(feature = "libm"))]
        let ret = crate::float_approx::midi_note_to_frequency(self);
        #[cfg(feature = "libm")]
        let ret = 440f32 * ((self - 69f32) / 12f32).exp2();
        ret
    }
}

static F64_ZEROBUF: [f64; STATIC_BUFFER_SIZE] = [0f64; STATIC_BUFFER_SIZE];
impl Float for f64 {
    const ZERO: f64 = 0.0f64;
    const ONE: f64 = 1.0f64;
    const TWO: f64 = 2.0f64;
    const THREE: f64 = 3.0f64;
    const ONE_HALF: f64 = 0.5f64;
    const POINT_ONE: f64 = 0.1f64;
    const POINT_NINE_EIGHT: f64 = 0.98f64;
    const RES_MAX: f64 = 0xF000 as f64 / 0xFFFF as f64;
    const FRAC_PI_2: f64 = core::f64::consts::FRAC_PI_2;
    const FRAC_2_PI: f64 = core::f64::consts::FRAC_2_PI;
    const PI: f64 = core::f64::consts::PI;
    const TAU: f64 = core::f64::consts::TAU;
    const NOTE_MAX: f64 = 127.0f64 * (0xFFFF as f64 / 0x10000 as f64);
    const SHAPE_CLIP: f64 = 0.9375f64;
    fn zerobuf<'a>() -> &'a [Self; STATIC_BUFFER_SIZE] {
        &F64_ZEROBUF
    }
    fn fsin(self) -> Self {
        #[cfg(not(feature = "libm"))]
        let ret = crate::float_approx::sin_approx(self);
        #[cfg(feature = "libm")]
        let ret = <Self as NumTraitsFloat>::sin(self);
        ret
    }
    fn fcos(self) -> Self {
        #[cfg(not(feature = "libm"))]
        let ret = crate::float_approx::cos_approx(self);
        #[cfg(feature = "libm")]
        let ret = <Self as NumTraitsFloat>::cos(self);
        ret
    }
    fn ftan(self) -> Self {
        #[cfg(not(feature = "libm"))]
        let ret = crate::float_approx::tan_approx(self);
        #[cfg(feature = "libm")]
        let ret = <Self as NumTraitsFloat>::tan(self);
        ret
    }
    fn midi_to_freq(self) -> Self {
        #[cfg(not(feature = "libm"))]
        let ret = crate::float_approx::midi_note_to_frequency(self);
        #[cfg(feature = "libm")]
        let ret = 440f64 * ((self - 69f64) / 12f64).exp2();
        ret
    }
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
