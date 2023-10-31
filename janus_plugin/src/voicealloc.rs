//! This module abstracts over the idea of sending notes and parameters to a synth
//! and having it "decide" how to handle the notes based on the polyphony mode,
//! selected logic form (fixed, float32, float64), etc.

use janus::{NoteFxP, SampleFxP, VoiceFxP};
use crate::parambuf::{EnvParamBuffer, FiltParamBuffer, OscParamBuffer};

/// This trait is the main abstraction for this module - the plugin may send it
/// note on/off events and it will assign those events to voices, stealing if
/// required (or always, in the case of a monosynth).
pub trait VoiceAllocator: Send {
    /// Initialize the voice allocator and allocate an internal buffer to
    /// handle up to `sz` samples.
    fn initialize(&mut self, sz: usize);
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
    /// Process all of the note on/off events within the buffer, taking the
    /// parameter buffers as input and returning a reference to an internal
    /// buffer holding the corresponding audio sample output
    /// 
    /// After calling this function, the internal index will be reset back
    /// to the beginning of the buffer (see [VoiceAllocator::sample_tick]).
    fn process(
        &mut self,
        op: &OscParamBuffer,
        fp: &FiltParamBuffer,
        efp: &EnvParamBuffer,
        eap: &EnvParamBuffer,
    ) -> &[f32];
    /// Is this a fixed point logic voice allocator?  If this function returns
    /// false, callers must call `conv_float()` on all parameter buffers before
    /// calling [VoiceAllocator::process].
    fn is_fixed_point(&self) -> bool;
}

/// A monosynth utilizing fixed point logic internally
#[derive(Default)]
pub struct MonoSynthFxP {
    voice: VoiceFxP,
    outbuf: Vec<f32>,
    notebuf: Vec<NoteFxP>,
    gatebuf: Vec<SampleFxP>,
    note: NoteFxP,
    index: usize,
    gate: SampleFxP,
}

impl MonoSynthFxP {
    pub fn new() -> Self {
        Default::default()
    }
}

impl VoiceAllocator for MonoSynthFxP {
    fn initialize(&mut self, sz: usize) {
        self.outbuf.resize(sz, 0f32);
        self.notebuf.resize(sz, NoteFxP::ZERO);
        self.gatebuf.resize(sz, SampleFxP::ZERO);
        self.index = 0;
        self.note = NoteFxP::lit("69"); //A440, nice
        self.gate = SampleFxP::ZERO;
    }
    fn sample_tick(&mut self) {
        self.notebuf[self.index] = self.note;
        self.gatebuf[self.index] = self.gate;
        self.index += 1;
    }
    fn note_on(&mut self, note: u8, _velocity: u8) {
        self.note = NoteFxP::from_num(note);
        self.gate = SampleFxP::ONE;
    }
    fn note_off(&mut self, _note: u8, _velocity: u8) {
        self.gate = SampleFxP::ZERO;
    }
    fn process(
        &mut self,
        op: &OscParamBuffer,
        fp: &FiltParamBuffer,
        efp: &EnvParamBuffer,
        eap: &EnvParamBuffer,
    ) -> &[f32] {
        let mut processed: usize = 0;
        while processed < self.index {
            let thisiter = self.voice.process(
                &self.notebuf[processed..self.index],
                &self.gatebuf[processed..self.index],
                op.params(processed, self.index),
                fp.params(processed, self.index),
                efp.params(processed, self.index),
                eap.params(processed, self.index),
            );
            for smp in thisiter {
                self.outbuf[processed] = smp.to_num::<f32>();
                processed += 1;
            }
        }
        self.index = 0;
        &self.outbuf[0..processed]
    }
    fn is_fixed_point(&self) -> bool {
        true
    }
}
