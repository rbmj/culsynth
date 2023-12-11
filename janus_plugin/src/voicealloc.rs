//! This module abstracts over the idea of sending notes and parameters to a synth
//! and having it "decide" how to handle the notes based on the polyphony mode,
//! selected logic form (fixed, float32, float64), etc.

use crate::parambuf::{
    EnvParamBuffer, FiltParamBuffer, GlobalParamBuffer, OscParamBuffer, RingModParamBuffer, LfoParamBuffer,
};
use crate::pluginparams::ModMatrixPluginParams;
use janus::context::{Context, ContextFxP, GenericContext};
use janus::voice::{Voice, VoiceFxP};
use janus::{NoteFxP, SampleFxP, ScalarFxP, SignedNoteFxP, IScalarFxP};

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
        matrix_p: &ModMatrixPluginParams,
        glob_p: &mut GlobalParamBuffer,
        o1p: &mut OscParamBuffer,
        o2p: &mut OscParamBuffer,
        rp: &mut RingModParamBuffer,
        fp: &mut FiltParamBuffer,
        efp: &mut EnvParamBuffer,
        eap: &mut EnvParamBuffer,
        l1p: &LfoParamBuffer,
        l2p: &mut LfoParamBuffer,
        e1p: &EnvParamBuffer,
        e2p: &mut EnvParamBuffer,
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
    aftertouchbuf: Vec<ScalarFxP>,
    modwheelbuf: Vec<ScalarFxP>,
    ctx: ContextFxP,
    index: usize,
    note: NoteFxP,
    gate: SampleFxP,
    velocity: ScalarFxP,
    aftertouch: ScalarFxP,
    modwheel: ScalarFxP,
    pitch_bend: SignedNoteFxP,
    pitch_range: (fixed::types::I16F0, fixed::types::I16F0),
}

impl MonoSynthFxP {
    pub fn new(ctx: ContextFxP) -> Self {
        Self {
            ctx,
            ..Default::default()
        }
    }
}

impl VoiceAllocator for MonoSynthFxP {
    fn initialize(&mut self, sz: usize) {
        self.outbuf.resize(sz, 0f32);
        self.notebuf.resize(sz, NoteFxP::ZERO);
        self.gatebuf.resize(sz, SampleFxP::ZERO);
        self.velbuf.resize(sz, ScalarFxP::ZERO);
        self.aftertouchbuf.resize(sz, ScalarFxP::ZERO);
        self.modwheelbuf.resize(sz, ScalarFxP::ZERO);
        self.index = 0;
        self.note = NoteFxP::lit("69"); //A440, nice
        self.gate = SampleFxP::ZERO;
        self.set_pitch_bend_range(2, 2);
    }
    fn sample_tick(&mut self) {
        self.notebuf[self.index] = self.note.add_signed(self.pitch_bend);
        self.gatebuf[self.index] = self.gate;
        self.velbuf[self.index] = self.velocity;
        self.aftertouchbuf[self.index] = self.aftertouch;
        self.modwheelbuf[self.index] = self.modwheel;
        self.index += 1;
    }
    fn note_on(&mut self, note: u8, velocity: u8) {
        self.note = NoteFxP::from_num(note);
        self.gate = SampleFxP::ONE;
        self.velocity = ScalarFxP::from_bits((velocity as u16) << 8);
    }
    fn note_off(&mut self, note: u8, velocity: u8) {
        if self.note == note {
            self.gate = SampleFxP::ZERO;
            self.velocity = ScalarFxP::from_bits((velocity as u16) << 8);
        }
    }
    fn aftertouch(&mut self, value: u8) {
        self.aftertouch = ScalarFxP::from_bits((value as u16) << 8);
    }
    fn modwheel(&mut self, value: u8) {
        self.modwheel = ScalarFxP::from_bits((value as u16) << 8);
    }
    fn pitch_bend(&mut self, v: i16) {
        if v < 0 {
            self.pitch_bend = SignedNoteFxP::from_num(
                IScalarFxP::from_bits(v).wide_mul(self.pitch_range.0),
            );
        } else {
            self.pitch_bend = SignedNoteFxP::from_num(
                IScalarFxP::from_bits(v).wide_mul(self.pitch_range.1),
            );
        }
    }
    fn get_pitch_bend_range(&self) -> (i8, i8) {
        (self.pitch_range.0.to_num::<i8>(), self.pitch_range.1.to_num::<i8>())
    }
    fn set_pitch_bend_range(&mut self, low: i8, high: i8) {
        self.pitch_range = (
            fixed::types::I16F0::from_num(low),
            fixed::types::I16F0::from_num(high),
        );
    }
    fn process(
        &mut self,
        matrix_p: &ModMatrixPluginParams,
        glob_p: &mut GlobalParamBuffer,
        osc1_p: &mut OscParamBuffer,
        osc2_p: &mut OscParamBuffer,
        ring_p: &mut RingModParamBuffer,
        filt_p: &mut FiltParamBuffer,
        filt_env_p: &mut EnvParamBuffer,
        amp_env_p: &mut EnvParamBuffer,
        lfo1_p: &LfoParamBuffer,
        lfo2_p: &mut LfoParamBuffer,
        env1_p: &EnvParamBuffer,
        env2_p: &mut EnvParamBuffer,
    ) -> &[f32] {
        let mut processed: usize = 0;
        let matrix = matrix_p.build_matrix();
        while processed < self.index {
            let thisiter = self.voice.process(
                &self.ctx,
                &matrix,
                &self.notebuf[processed..self.index],
                &self.gatebuf[processed..self.index],
                &self.velbuf[processed..self.index],
                &self.aftertouchbuf[processed..self.index],
                &self.modwheelbuf[processed..self.index],
                glob_p.sync(processed, self.index),
                osc1_p.params_mut(processed, self.index),
                osc2_p.params_mut(processed, self.index),
                ring_p.params_mut(processed, self.index),
                filt_p.params_mut(processed, self.index),
                filt_env_p.params_mut(processed, self.index),
                amp_env_p.params_mut(processed, self.index),
                lfo1_p.params(processed, self.index),
                lfo2_p.params_mut(processed, self.index),
                env1_p.params(processed, self.index),
                env2_p.params_mut(processed, self.index),
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
    aftertouchbuf: Vec<f32>,
    modwheelbuf: Vec<f32>,
    ctx: Context<f32>,
    index: usize,
    note: f32,
    gate: f32,
    velocity: f32,
    aftertouch: f32,
    modwheel: f32,
    pitch_bend: f32,
    pitch_bend_range: (f32, f32),
}

impl MonoSynth {
    pub fn new(ctx: Context<f32>) -> Self {
        Self {
            ctx,
            ..Default::default()
        }
    }
}

impl VoiceAllocator for MonoSynth {
    fn initialize(&mut self, sz: usize) {
        self.outbuf.resize(sz, 0f32);
        self.notebuf.resize(sz, 0f32);
        self.gatebuf.resize(sz, 0f32);
        self.velbuf.resize(sz, 0f32);
        self.aftertouchbuf.resize(sz, 0f32);
        self.modwheelbuf.resize(sz, 0f32);
        self.index = 0;
        self.note = 69f32; //A440, nice
        self.gate = 0f32;
        self.set_pitch_bend_range(2, 2);
    }
    fn sample_tick(&mut self) {
        self.notebuf[self.index] = self.note;
        self.gatebuf[self.index] = self.gate;
        self.velbuf[self.index] = self.velocity;
        self.aftertouchbuf[self.index] = self.aftertouch;
        self.modwheelbuf[self.index] = self.modwheel;
        self.index += 1;
    }
    fn note_on(&mut self, note: u8, velocity: u8) {
        self.note = f32::from(note);
        self.gate = 1f32;
        self.velocity = f32::from(velocity) / 127f32;
    }
    fn note_off(&mut self, note: u8, velocity: u8) {
        if self.note == f32::from(note) {
            self.gate = 0f32;
            self.velocity = f32::from(velocity) / 127f32;
        }
    }
    fn aftertouch(&mut self, value: u8) {
        self.aftertouch = f32::from(value) / 127f32;
    }
    fn modwheel(&mut self, value: u8) {
        self.modwheel = f32::from(value) / 127f32;
    }
    fn pitch_bend(&mut self, value: i16) {
        let val_float = (value as f32) / (i16::MAX as f32);
        if val_float < 0f32 {
            self.pitch_bend = self.pitch_bend_range.0 * val_float;
        } else {
            self.pitch_bend = self.pitch_bend_range.1 * val_float;
        }
    }
    fn get_pitch_bend_range(&self) -> (i8, i8) {
        (self.pitch_bend_range.0 as i8, self.pitch_bend_range.1 as i8)
    }
    fn set_pitch_bend_range(&mut self, low: i8, high: i8) {
        self.pitch_bend_range = (
            low as f32,
            high as f32,
        );
    }
    fn process(
        &mut self,
        matrix_p: &ModMatrixPluginParams,
        glob_p: &mut GlobalParamBuffer,
        osc1_p: &mut OscParamBuffer,
        osc2_p: &mut OscParamBuffer,
        ring_p: &mut RingModParamBuffer,
        filt_p: &mut FiltParamBuffer,
        filt_env_p: &mut EnvParamBuffer,
        amp_env_p: &mut EnvParamBuffer,
        lfo1_p: &LfoParamBuffer,
        lfo2_p: &mut LfoParamBuffer,
        env1_p: &EnvParamBuffer,
        env2_p: &mut EnvParamBuffer,
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
                lfo1_p.params_float(processed, self.index),
                lfo2_p.params_float_mut(processed, self.index),
                env1_p.params_float(processed, self.index),
                env2_p.params_float_mut(processed, self.index),
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
