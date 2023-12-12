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

mod fixedmath;
mod util;

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
pub use voice::VoiceFxP;

const STATIC_BUFFER_SIZE: usize = 256;
type BufferT<T> = [T; STATIC_BUFFER_SIZE];

pub use fixedmath::midi_note_to_frequency;
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
pub type IScalarFxP = fixedmath::I1F15;

fn min_size(sizes: &[usize]) -> usize {
    *sizes.iter().min().unwrap_or(&0)
}

pub trait Fixed16: fixed::traits::Fixed {
    //none required currently
}

impl<N> Fixed16 for fixed::FixedI16<N>
where
    N: fixed::types::extra::Unsigned + fixed::types::extra::LeEqU16,
{
    //
}

impl<N> Fixed16 for fixed::FixedU16<N>
where
    N: fixed::types::extra::Unsigned + fixed::types::extra::LeEqU16,
{
    //
}
