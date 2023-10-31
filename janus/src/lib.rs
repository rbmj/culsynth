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

mod fixedmath;
pub mod util;

/// This module contains `u8` constants for MIDI note numbers, using standard
/// musical notation.  For example, `midi_const::Db4` is the note a semitone
/// above middle C, and `midi_const::A4 == 69u8` is A440.
/// 
/// Note that these are constants but use a lowercase `b` to denote flats for
/// visual similarity with musical notation, breaking the convention for `const`
/// values in Rust.  No constants are provided for sharps - use the enharmonic
/// flat.
pub mod midi_const;

pub mod devices;

pub mod voice;
pub use voice::VoiceFxP;

const STATIC_BUFFER_SIZE: usize = 256;
type BufferT<T> = [T; STATIC_BUFFER_SIZE];

pub use fixedmath::midi_note_to_frequency;
pub use fixedmath::Note as NoteFxP;
pub use fixedmath::Sample as SampleFxP;
pub use fixedmath::Scalar as ScalarFxP;
pub use fixedmath::USample as USampleFxP;
pub use fixedmath::U3F13 as EnvParamFxP;
