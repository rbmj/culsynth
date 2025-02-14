//! This module abstracts over the idea of sending notes and parameters to a synth
//! and having it "decide" how to handle the notes based on the polyphony mode,
//! selected logic form (fixed, float32, float64), etc.

use culsynth::context::GenericContext;
use culsynth::voice::modulation::ModMatrix;
use culsynth::voice::{Voice, VoiceChannelInput, VoiceInput, VoiceParams};
use culsynth::{IScalarFxP, NoteFxP, ScalarFxP, SignedNoteFxP};

use wmidi::MidiMessage;

/// This trait is the main abstraction for this module - the plugin may send it
/// note on/off events and it will assign those events to voices, stealing if
/// required (or always, in the case of a monosynth).
pub trait VoiceAllocator: Send {
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
    /// Get the next sample
    fn next(&mut self, params: &VoiceParams<i16>, matrix: Option<&ModMatrix<i16>>) -> f32;
    /// Get the process context for this voice allocator.
    fn get_context(&self) -> &dyn GenericContext;
    /// Is this Voice Allocator polyphonic?
    fn is_poly(&self) -> bool;
    /// Get the MIDI channel associated with this VoiceAllocator, or None for all channels
    fn get_channel(&self) -> Option<wmidi::Channel>;
    /// Handle a MIDI control change message:
    fn handle_cc(&mut self, cc: wmidi::ControlFunction, value: u8);
    fn handle_midi(&mut self, msg: MidiMessage) {
        if let (Some(my_ch), Some(msg_ch)) = (self.get_channel(), msg.channel()) {
            if my_ch != msg_ch {
                return;
            }
        }
        match msg {
            MidiMessage::NoteOn(_, note, velocity) => self.note_on(note.into(), velocity.into()),
            MidiMessage::NoteOff(_, note, velocity) => self.note_off(note.into(), velocity.into()),
            MidiMessage::ChannelPressure(_, velocity) => self.aftertouch(velocity.into()),
            MidiMessage::PitchBendChange(_, value) => {
                let bend: u16 = value.into();
                self.pitch_bend((bend as i32 - 8192i32) as i16)
            }
            MidiMessage::ControlChange(_, cc, value) => self.handle_cc(cc, value.into()),
            _ => (),
        }
    }
}

mod monosynth;
pub use monosynth::MonoSynth;

mod polysynth;
pub use polysynth::PolySynth;
