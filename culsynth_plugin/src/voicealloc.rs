//! This module abstracts over the idea of sending notes and parameters to a synth
//! and having it "decide" how to handle the notes based on the polyphony mode,
//! selected logic form (fixed, float32, float64), etc.

use culsynth::context::GenericContext;
use culsynth::voice::{Voice, VoiceParams, VoiceInput, VoiceChannelInput};
use culsynth::voice::modulation::ModMatrix;
use culsynth::{IScalarFxP, NoteFxP, SampleFxP, ScalarFxP, SignedNoteFxP};

/// This trait is the main abstraction for this module - the plugin may send it
/// note on/off events and it will assign those events to voices, stealing if
/// required (or always, in the case of a monosynth).
pub trait VoiceAllocator: Send {
    /// Increment the (hidden) internal index into the allocator's buffer
    fn sample_tick(&mut self);
    /// For the sample at the current index (see [VoiceAllocator::sample_tick]),
    /// process a 'note on' event with MIDI note number `n` and velocity `v`
    fn note_on(&mut self, n: u8, v: u8);
    /// For the sample at the current index (see [VoiceAllocator::sample_tick]),
    /// process a 'note off' event with MIDI note number `n` and velocity `v`
    ///
    /// Note:  Most implementations will ignore note off velocity
    fn note_off(&mut self, n: u8, v: u8);
    /// For the sample at the current index (see [VoiceAllocator::sample_tick]),
    /// process a change in the aftertouch value
    fn aftertouch(&mut self, v: u8);
    /// For the sample at the current index (see [VoiceAllocator::sample_tick]),
    /// process a change in modwheel value
    fn modwheel(&mut self, v: u8);
    /// For the sample at the current index (see [VoiceAllocator::sample_tick]),
    /// process a change in pitch bend value
    fn pitch_bend(&mut self, v: i16);
    /// Get the current pitch bend range, in semitones
    fn get_pitch_bend_range(&self) -> (i8, i8);
    /// Set the current pitch bend range, in semitones
    ///
    /// Both arguments should generally be positive.  For example,
    /// `set_pitch_bend_range(2, 2)` will set the pitch wheel to bend up/down
    /// a whole step.
    fn set_pitch_bend_range(&mut self, low: i8, high: i8);
    /// Process all of the note on/off events within the buffer, taking the
    /// parameter buffers as input and returning a reference to an internal
    /// buffer holding the corresponding audio sample output
    ///
    /// If `self.get_context().is_fixed_point()` then callers must call
    /// `conv_float()` on all parameter buffers before calling this function.
    ///
    /// After calling this function, the internal index will be reset back
    /// to the beginning of the buffer (see [VoiceAllocator::sample_tick]).
    fn process(
        &mut self,
        matrix: &ModMatrix<i16>,
        params: &[VoiceParams<i16>],
    ) -> &[f32];
    /// Get the process context for this voice allocator.
    fn get_context(&self) -> &dyn GenericContext;
    /// Is this Voice Allocator polyphonic?
    fn is_poly(&self) -> bool;
}

mod monosynth;
pub use monosynth::{MonoSynth};

mod polysynth;
pub use polysynth::{PolySynth};
