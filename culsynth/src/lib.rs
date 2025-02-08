//! This crate contains all of the DSP logic for the synthesizer.  It is designed
//! to be `no_std` friendly (though it has not yet been build/tested in this config)
//! and all of the API and algorithms are designed to be implemented using both
//! floating-point logic and fixed point logic.  The fixed point logic additionally
//! does not use division and minimizes the use of 32 bit widening multiplies
//! (that is, with a 64 bit result) to the maximum extent possible for speed on
//! embedded platforms without native hardware support for these primitives.
//!
//! Most of the relevant code for users can be found in the [devices] module.
//!
//! This crate uses the (somewhat regrettably hungarian-style) convention of
//! having all fixed-point structs and traits be the same as their floating-point
//! counterparts with the FxP suffix to denote fixed point operation.  This is
//! used instead of implementing it as generics implementations on `u16` to
//! preserve information about the location of the decimal place within the type
//! system, but does require some duplication of code throughout the crate.

#![no_std]
#![warn(missing_docs)]

use fixed::{traits::Fixed, FixedI32};

mod fixedmath;
mod float_approx;
pub mod util;

/// True if using libm for floating-point math, false if using internal
/// approximation functions
pub const USE_LIBM: bool = cfg!(feature = "libm");

/// This module contains `u8` constants for MIDI note numbers, using standard
/// musical notation.  For example, `midi_const::Db4` is the note a semitone
/// above middle C, and `midi_const::A4 == 69u8` is A440.
///
/// Note that these are constants but use a lowercase `b` to denote flats for
/// visual similarity with musical notation, breaking the convention for `const`
/// values in Rust.  No constants are provided for sharps - use the enharmonic
/// flat.
pub mod midi_const;

pub mod context;
pub mod devices;

pub mod voice;

pub use fixedmath::midi_note_to_frequency;
pub use fixedmath::Frequency as FrequencyFxP;
pub use fixedmath::Note as NoteFxP;
pub use fixedmath::Sample as SampleFxP;
pub use fixedmath::Scalar as ScalarFxP;
pub use fixedmath::SignedNote as SignedNoteFxP;
pub use fixedmath::USample as USampleFxP;
/// An envelope rise/fall time parameter, represented in seconds as an unsigned
/// 16 bit fixed point number with 13 fractional bits and 3 integral bits.  This
/// yields a range of 0 to 8 seconds - though as implemented this timing is not
/// precisely accurate (see [devices::EnvParamsFxP])
pub type EnvParamFxP = fixedmath::U3F13;
/// A frequency parameter for a LFO, in Hertz
///
/// TODO: should this be a newtype?
pub type LfoFreqFxP = fixedmath::U7F9;
/// A signed value in the range `[-1, 1)` - used where we need a signed version
/// of a [ScalarFxP]
pub type IScalarFxP = fixedmath::I1F15;
/// A 32-bit fixed point number representing a sinusoid's phase.  
type PhaseFxP = fixedmath::I4F28;
/// A 16 bit fixed point number representing a tuning offset from -32 to +32
/// semitones
pub type CoarseTuneFxP = fixedmath::I6F10;
/// A 16 bit fixed point number representing a tuning offset from -2 to +2
/// semitones
pub type FineTuneFxP = fixedmath::I3F13;

mod fixed_traits;
pub use fixed_traits::Fixed16;

mod float_traits;
pub use float_traits::Float;

mod dsp_format;
pub use dsp_format::{DspFloat, DspFormat, DspFormatBase, DspType};

type WideSampleFxP = FixedI32<<SampleFxP as Fixed>::Frac>;
