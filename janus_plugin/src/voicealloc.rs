//! This module abstracts over the idea of sending notes and parameters to a synth
//! and having it "decide" how to handle the notes based on the polyphony mode,
//! selected logic form (fixed, float32, float64), etc.

use crate::parambuf::{
    EnvParamBuffer, FiltParamBuffer, GlobalParamBuffer, OscParamBuffer, RingModParamBuffer,
};
use janus::context::{Context, ContextFxP, GenericContext};
use janus::voice::{Voice, VoiceFxP};
use janus::{NoteFxP, SampleFxP, ScalarFxP};
use nih_plug::nih_trace;

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
    /// If `self.get_context().is_fixed_point()` then callers must call
    /// `conv_float()` on all parameter buffers before calling this function.
    ///
    /// After calling this function, the internal index will be reset back
    /// to the beginning of the buffer (see [VoiceAllocator::sample_tick]).
    fn process(
        &mut self,
        glob_p: &mut GlobalParamBuffer,
        o1p: &OscParamBuffer,
        o2p: &OscParamBuffer,
        rp: &RingModParamBuffer,
        fp: &FiltParamBuffer,
        efp: &EnvParamBuffer,
        eap: &EnvParamBuffer,
    ) -> &[f32];
    /// Get the process context for this voice allocator.
    fn get_context(&self) -> &dyn GenericContext;
}

/// A monosynth utilizing fixed point logic internally
#[derive(Default)]
pub struct MonoSynthFxP {
    voice: VoiceFxP,
    outbuf: Vec<f32>,
    notebuf: Vec<NoteFxP>,
    gatebuf: Vec<SampleFxP>,
    velbuf: Vec<ScalarFxP>,
    ctx: ContextFxP,
    index: usize,
    note: NoteFxP,
    gate: SampleFxP,
    velocity: ScalarFxP,
}

impl MonoSynthFxP {
    pub fn new(ctx: ContextFxP) -> Self {
        let mut ret = Self::default();
        ret.ctx = ctx;
        ret
    }
}

impl VoiceAllocator for MonoSynthFxP {
    fn initialize(&mut self, sz: usize) {
        self.outbuf.resize(sz, 0f32);
        self.notebuf.resize(sz, NoteFxP::ZERO);
        self.gatebuf.resize(sz, SampleFxP::ZERO);
        self.velbuf.resize(sz, ScalarFxP::ZERO);
        self.index = 0;
        self.note = NoteFxP::lit("69"); //A440, nice
        self.gate = SampleFxP::ZERO;
    }
    fn sample_tick(&mut self) {
        self.notebuf[self.index] = self.note;
        self.gatebuf[self.index] = self.gate;
        self.velbuf[self.index] = self.velocity;
        self.index += 1;
    }
    fn note_on(&mut self, note: u8, velocity: u8) {
        nih_trace!("Received Note On");
        self.note = NoteFxP::from_num(note);
        self.gate = SampleFxP::ONE;
        self.velocity = ScalarFxP::from_bits((velocity as u16) << 8);
    }
    fn note_off(&mut self, _note: u8, velocity: u8) {
        nih_trace!("Received Note Off");
        self.gate = SampleFxP::ZERO;
        self.velocity = ScalarFxP::from_bits((velocity as u16) << 8);
    }
    fn process(
        &mut self,
        glob_p: &mut GlobalParamBuffer,
        osc1_p: &OscParamBuffer,
        osc2_p: &OscParamBuffer,
        ring_p: &RingModParamBuffer,
        filt_p: &FiltParamBuffer,
        filt_env_p: &EnvParamBuffer,
        amp_env_p: &EnvParamBuffer,
    ) -> &[f32] {
        let mut processed: usize = 0;
        while processed < self.index {
            let thisiter = self.voice.process(
                &self.ctx,
                &self.notebuf[processed..self.index],
                &self.gatebuf[processed..self.index],
                &self.velbuf[processed..self.index],
                glob_p.sync(processed, self.index),
                osc1_p.params(processed, self.index),
                osc2_p.params(processed, self.index),
                ring_p.params(processed, self.index),
                filt_p.params(processed, self.index),
                filt_env_p.params(processed, self.index),
                amp_env_p.params(processed, self.index),
            );
            for smp in thisiter {
                self.outbuf[processed] = smp.to_num::<f32>();
                processed += 1;
            }
        }
        self.index = 0;
        &self.outbuf[0..processed]
    }
    fn get_context(&self) -> &dyn GenericContext {
        &self.ctx
    }
}

/// A monosynth utilizing floating point logic internally
#[derive(Default)]
pub struct MonoSynth {
    voice: Voice<f32>,
    outbuf: Vec<f32>,
    notebuf: Vec<f32>,
    gatebuf: Vec<f32>,
    velbuf: Vec<f32>,
    ctx: Context<f32>,
    index: usize,
    note: f32,
    gate: f32,
    velocity: f32,
}

impl MonoSynth {
    pub fn new(ctx: Context<f32>) -> Self {
        let mut ret = Self::default();
        ret.ctx = ctx;
        ret
    }
}

impl VoiceAllocator for MonoSynth {
    fn initialize(&mut self, sz: usize) {
        self.outbuf.resize(sz, 0f32);
        self.notebuf.resize(sz, 0f32);
        self.gatebuf.resize(sz, 0f32);
        self.velbuf.resize(sz, 0f32);
        self.index = 0;
        self.note = 69f32; //A440, nice
        self.gate = 0f32;
    }
    fn sample_tick(&mut self) {
        self.notebuf[self.index] = self.note;
        self.gatebuf[self.index] = self.gate;
        self.velbuf[self.index] = self.velocity;
        self.index += 1;
    }
    fn note_on(&mut self, note: u8, velocity: u8) {
        nih_trace!("Received Note On");
        self.note = f32::from(note);
        self.gate = 1f32;
        self.velocity = f32::from((velocity as u16) << 8);
    }
    fn note_off(&mut self, _note: u8, velocity: u8) {
        nih_trace!("Received Note Off");
        self.gate = 0f32;
        self.velocity = f32::from((velocity as u16) << 8);
    }
    fn process(
        &mut self,
        glob_p: &mut GlobalParamBuffer,
        osc1_p: &OscParamBuffer,
        osc2_p: &OscParamBuffer,
        ring_p: &RingModParamBuffer,
        filt_p: &FiltParamBuffer,
        filt_env_p: &EnvParamBuffer,
        amp_env_p: &EnvParamBuffer,
    ) -> &[f32] {
        let mut processed: usize = 0;
        while processed < self.index {
            let thisiter = self.voice.process(
                &self.ctx,
                &self.notebuf[processed..self.index],
                &self.gatebuf[processed..self.index],
                &self.velbuf[processed..self.index],
                glob_p.sync_float(processed, self.index),
                osc1_p.params_float(processed, self.index),
                osc2_p.params_float(processed, self.index),
                ring_p.params_float(processed, self.index),
                filt_p.params_float(processed, self.index),
                filt_env_p.params_float(processed, self.index),
                amp_env_p.params_float(processed, self.index),
            );
            for smp in thisiter {
                self.outbuf[processed] = *smp;
                processed += 1;
            }
        }
        self.index = 0;
        &self.outbuf[0..processed]
    }
    fn get_context(&self) -> &dyn GenericContext {
        &self.ctx
    }
}
