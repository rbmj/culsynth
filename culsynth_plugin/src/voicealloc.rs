//! This module abstracts over the idea of sending notes and parameters to a synth
//! and having it "decide" how to handle the notes based on the polyphony mode,
//! selected logic form (fixed, float32, float64), etc.

use std::sync::mpsc::SyncSender;

use culsynth::context::GenericContext;
use culsynth::voice::modulation::ModMatrix;
use culsynth::voice::{Voice, VoiceChannelInput, VoiceInput, VoiceParams};
use culsynth::{IScalarFxP, NoteFxP, ScalarFxP, SignedNoteFxP};

use nih_plug::buffer::SamplesIter;
use nih_plug::context::process::ProcessContext;
use nih_plug::midi;

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
    /// Handle a MIDI control change message:
    fn handle_cc(&mut self, cc: u8, value: u8, dispatcher: &mut SyncSender<(u8, u8)>);
    /// Process the provided samples in the given process context and
    /// parameters.  If `matrix.is_some()`, update the mod matrix as well.
    fn process(
        &mut self,
        smps: SamplesIter,
        ctx: &mut dyn ProcessContext<CulSynthPlugin>,
        params: &CulSynthParams,
        dispatcher: &mut SyncSender<(u8, u8)>,
        mut matrix: Option<ModMatrix<i16>>,
    ) {
        let mut next_event = ctx.next_event();
        for (smpid, ch_smps) in smps.enumerate() {
            let params: VoiceParams<i16> = params.into();
            // Process MIDI events:
            while let Some(event) = next_event {
                if event.timing() > smpid as u32 {
                    break;
                }
                match event {
                    midi::NoteEvent::NoteOn { note, velocity, .. } => {
                        self.note_on(note, (velocity * 127f32) as u8);
                    }
                    midi::NoteEvent::NoteOff { note, velocity, .. } => {
                        self.note_off(note, (velocity * 127f32) as u8);
                    }
                    midi::NoteEvent::MidiCC { cc, value, .. } => {
                        self.handle_cc(cc, (value * 127f32) as u8, dispatcher);
                    }
                    midi::NoteEvent::MidiChannelPressure { pressure, .. } => {
                        self.aftertouch((pressure * 127f32) as u8);
                    }
                    midi::NoteEvent::MidiPitchBend { value, .. } => {
                        self.pitch_bend((((value - 0.5) * (i16::MAX as f32)) as i16) << 1);
                    }
                    _ => (),
                }
                next_event = ctx.next_event();
            }
            let out = self.next(&params, matrix.take().as_ref());
            for smp in ch_smps {
                *smp = out;
            }
        }
    }
}

mod monosynth;
pub use monosynth::MonoSynth;

mod polysynth;
pub use polysynth::PolySynth;

use crate::pluginparams::CulSynthParams;
use crate::CulSynthPlugin;
