use std::collections::VecDeque;

use super::*;
use nih_plug::{nih_error, nih_trace};
use rand::random;

struct PolySynthVoice {
    voice: Voice<f32>,
    notebuf: Vec<f32>,
    gatebuf: Vec<f32>,
    velbuf: Vec<f32>,
    gate: f32,
    vel: f32,
    note: u8,
}

impl PolySynthVoice {
    fn new() -> Self {
        Self {
            voice: Voice::new_with_seeds(random(), random()),
            notebuf: Vec::default(),
            gatebuf: Vec::default(),
            velbuf: Vec::default(),
            note: 69, //A440
            gate: 0f32,
            vel: 0f32,
        }
    }
    fn initialize(&mut self, sz: usize) {
        self.notebuf.resize(sz, 0f32);
        self.gatebuf.resize(sz, 0f32);
        self.velbuf.resize(sz, 0f32);
        self.note = 69;
        self.gate = 0f32;
        self.vel = 0f32;
    }
}

impl Default for PolySynthVoice {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PolySynth {
    voices: Vec<PolySynthVoice>,
    active_voices: VecDeque<usize>,
    inactive_voices: VecDeque<usize>,
    outbuf: Vec<f32>,
    aftertouchbuf: Vec<f32>,
    modwheelbuf: Vec<f32>,
    pitch_bend_range: (f32, f32),
    pitch_bend: f32,
    aftertouch: f32,
    modwheel: f32,
    index: usize,
    ctx: Context<f32>,
}

impl PolySynth {
    pub fn new(num_voices: usize, context: Context<f32>) -> Self {
        Self {
            voices: std::iter::repeat_with(|| PolySynthVoice::new())
                .take(num_voices)
                .collect(),
            active_voices: VecDeque::new(),
            inactive_voices: VecDeque::new(),
            outbuf: Vec::default(),
            aftertouchbuf: Vec::default(),
            modwheelbuf: Vec::default(),
            aftertouch: 0f32,
            modwheel: 0f32,
            pitch_bend: 0f32,
            pitch_bend_range: (2f32, 2f32),
            index: 0,
            ctx: context,
        }
    }
    fn note_on_i(&mut self, voice_index: usize, note: u8, vel: u8) {
        nih_trace!("\tAssigned Voice #{}", voice_index);
        self.active_voices.push_back(voice_index);
        let voice = &mut self.voices[voice_index];
        voice.note = note;
        voice.vel = f32::from(vel) / 127f32;
        voice.gate = 1f32;
    }
}

impl VoiceAllocator for PolySynth {
    fn initialize(&mut self, sz: usize) {
        self.active_voices.reserve(sz);
        self.active_voices.clear();
        self.inactive_voices.reserve(sz);
        self.inactive_voices.clear();
        self.outbuf.resize(sz, 0f32);
        self.aftertouchbuf.resize(sz, 0f32);
        self.modwheelbuf.resize(sz, 0f32);
        self.index = 0;
        self.set_pitch_bend_range(2, 2);
        for (i, voice) in self.voices.iter_mut().enumerate() {
            voice.initialize(sz);
            self.inactive_voices.push_back(i);
        }
    }
    fn sample_tick(&mut self) {
        self.aftertouchbuf[self.index] = self.aftertouch;
        self.modwheelbuf[self.index] = self.modwheel;
        for voice in self.voices.iter_mut() {
            voice.notebuf[self.index] = f32::from(voice.note) + self.pitch_bend;
            voice.gatebuf[self.index] = voice.gate;
            voice.velbuf[self.index] = voice.vel;
        }
        self.index += 1;
    }
    fn note_on(&mut self, note: u8, velocity: u8) {
        nih_trace!("Poly Note On: {}", note);
        if let Some(i) = self.inactive_voices.pop_front() {
            self.note_on_i(i, note, velocity);
        } else if let Some(i) = self.active_voices.pop_front() {
            self.note_on_i(i, note, velocity);
        } else {
            nih_error!("Unable to steal voice");
        }
    }
    fn note_off(&mut self, note: u8, _velocity: u8) {
        nih_trace!("Poly Note Off: {}", note);
        if let Some((act_idx, vox_idx)) = self
            .active_voices
            .iter()
            .enumerate()
            .find(|(_, idx)| self.voices[**idx].note == note)
        {
            nih_trace!("\tVoice {} Off", *vox_idx);
            self.inactive_voices.push_back(*vox_idx);
            self.voices[*vox_idx].gate = 0f32;
            self.active_voices.remove(act_idx);
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
        self.pitch_bend_range = (low as f32, high as f32);
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
        let matrix = matrix_p.build_matrix_float();
        for smp in self.outbuf.iter_mut() {
            *smp = 0f32;
        }
        for voice in self.voices.iter_mut() {
            let mut processed: usize = 0;
            while processed < self.index {
                let thisiter = voice.voice.process(
                    &self.ctx,
                    &matrix,
                    &voice.notebuf[processed..self.index],
                    &voice.gatebuf[processed..self.index],
                    &voice.velbuf[processed..self.index],
                    &self.aftertouchbuf[processed..self.index],
                    &self.modwheelbuf[processed..self.index],
                    glob_p.sync_float(processed, self.index),
                    osc1_p.params_float_mut(processed, self.index),
                    osc2_p.params_float_mut(processed, self.index),
                    ring_p.params_float_mut(processed, self.index),
                    filt_p.params_float_mut(processed, self.index),
                    filt_env_p.params_float_mut(processed, self.index),
                    amp_env_p.params_float_mut(processed, self.index),
                    lfo1_p.params_float(processed, self.index),
                    lfo2_p.params_float_mut(processed, self.index),
                    env1_p.params_float(processed, self.index),
                    env2_p.params_float_mut(processed, self.index),
                );
                for smp in thisiter {
                    self.outbuf[processed] += *smp;
                    processed += 1;
                }
            }
        }
        let oldindex = self.index;
        self.index = 0;
        &self.outbuf[0..oldindex]
    }
    fn get_context(&self) -> &dyn GenericContext {
        &self.ctx
    }
    fn is_poly(&self) -> bool {
        true
    }
}

struct PolySynthVoiceFxP {
    voice: VoiceFxP,
    notebuf: Vec<NoteFxP>,
    gatebuf: Vec<SampleFxP>,
    velbuf: Vec<ScalarFxP>,
    gate: SampleFxP,
    vel: ScalarFxP,
    note: u8,
}

impl PolySynthVoiceFxP {
    fn new() -> Self {
        Self {
            voice: VoiceFxP::new_with_seeds(random(), random()),
            notebuf: Vec::default(),
            gatebuf: Vec::default(),
            velbuf: Vec::default(),
            note: 69, //A440
            gate: SampleFxP::ZERO,
            vel: ScalarFxP::ZERO,
        }
    }
    fn initialize(&mut self, sz: usize) {
        self.notebuf.resize(sz, NoteFxP::ZERO);
        self.gatebuf.resize(sz, SampleFxP::ZERO);
        self.velbuf.resize(sz, ScalarFxP::ZERO);
        self.note = 69;
        self.gate = SampleFxP::ZERO;
        self.vel = ScalarFxP::ZERO;
    }
}

pub struct PolySynthFxP {
    voices: Vec<PolySynthVoiceFxP>,
    active_voices: VecDeque<usize>,
    inactive_voices: VecDeque<usize>,
    outbuf: Vec<f32>,
    aftertouchbuf: Vec<ScalarFxP>,
    modwheelbuf: Vec<ScalarFxP>,
    pitch_bend_range: (fixed::types::I16F0, fixed::types::I16F0),
    pitch_bend: SignedNoteFxP,
    aftertouch: ScalarFxP,
    modwheel: ScalarFxP,
    index: usize,
    ctx: ContextFxP,
}

impl PolySynthFxP {
    pub fn new(num_voices: usize, context: ContextFxP) -> Self {
        // FIXME: this clone()s everything, which results in the same S&H
        // random values across all voices :(
        Self {
            voices: std::iter::repeat_with(|| PolySynthVoiceFxP::new())
                .take(num_voices)
                .collect(),
            active_voices: VecDeque::new(),
            inactive_voices: VecDeque::new(),
            outbuf: Vec::new(),
            aftertouchbuf: Vec::new(),
            modwheelbuf: Vec::new(),
            pitch_bend: SignedNoteFxP::ZERO,
            pitch_bend_range: Default::default(),
            aftertouch: ScalarFxP::ZERO,
            modwheel: ScalarFxP::ZERO,
            index: 0,
            ctx: context,
        }
    }
    fn note_on_i(&mut self, voice_index: usize, note: u8, vel: u8) {
        self.active_voices.push_back(voice_index);
        let voice = &mut self.voices[voice_index];
        voice.note = note;
        voice.vel = ScalarFxP::from_bits((vel as u16) << 9);
        voice.gate = SampleFxP::ONE;
    }
}

impl VoiceAllocator for PolySynthFxP {
    fn initialize(&mut self, sz: usize) {
        self.active_voices.reserve(sz);
        self.inactive_voices.reserve(sz);
        self.outbuf.resize(sz, 0f32);
        self.aftertouchbuf.resize(sz, ScalarFxP::ZERO);
        self.modwheelbuf.resize(sz, ScalarFxP::ZERO);
        self.index = 0;
        self.set_pitch_bend_range(2, 2);
        for (i, voice) in self.voices.iter_mut().enumerate() {
            voice.initialize(sz);
            self.inactive_voices.push_back(i);
        }
    }
    fn sample_tick(&mut self) {
        self.aftertouchbuf[self.index] = self.aftertouch;
        self.modwheelbuf[self.index] = self.modwheel;
        for voice in self.voices.iter_mut() {
            voice.notebuf[self.index] = NoteFxP::from_num(voice.note).add_signed(self.pitch_bend);
            voice.gatebuf[self.index] = voice.gate;
            voice.velbuf[self.index] = voice.vel;
        }
        self.index += 1;
    }
    fn note_on(&mut self, note: u8, velocity: u8) {
        if let Some(i) = self.inactive_voices.pop_front() {
            self.note_on_i(i, note, velocity);
        } else if let Some(i) = self.active_voices.pop_front() {
            self.note_on_i(i, note, velocity);
        } else {
            nih_error!("Unable to steal voice");
        }
    }
    fn note_off(&mut self, note: u8, _velocity: u8) {
        if let Some((act_idx, vox_idx)) = self
            .active_voices
            .iter()
            .enumerate()
            .find(|(_, idx)| self.voices[**idx].note == note)
        {
            self.inactive_voices.push_back(*vox_idx);
            self.voices[*vox_idx].gate = SampleFxP::ZERO;
            self.active_voices.remove(act_idx);
        }
    }
    fn aftertouch(&mut self, value: u8) {
        self.aftertouch = ScalarFxP::from_bits((value as u16) << 9);
    }
    fn modwheel(&mut self, value: u8) {
        self.modwheel = ScalarFxP::from_bits((value as u16) << 9);
    }
    fn pitch_bend(&mut self, v: i16) {
        if v < 0 {
            self.pitch_bend =
                SignedNoteFxP::from_num(IScalarFxP::from_bits(v).wide_mul(self.pitch_bend_range.0));
        } else {
            self.pitch_bend =
                SignedNoteFxP::from_num(IScalarFxP::from_bits(v).wide_mul(self.pitch_bend_range.1));
        }
    }
    fn get_pitch_bend_range(&self) -> (i8, i8) {
        (
            self.pitch_bend_range.0.to_num::<i8>(),
            self.pitch_bend_range.1.to_num::<i8>(),
        )
    }
    fn set_pitch_bend_range(&mut self, low: i8, high: i8) {
        self.pitch_bend_range = (
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
        let matrix = matrix_p.build_matrix();
        for smp in self.outbuf.iter_mut() {
            *smp = 0f32;
        }
        for voice in self.voices.iter_mut() {
            let mut processed: usize = 0;
            while processed < self.index {
                let thisiter = voice.voice.process(
                    &self.ctx,
                    &matrix,
                    &voice.notebuf[processed..self.index],
                    &voice.gatebuf[processed..self.index],
                    &voice.velbuf[processed..self.index],
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
                    self.outbuf[processed] += smp.to_num::<f32>();
                    processed += 1;
                }
            }
        }
        let old_index = self.index;
        self.index = 0;
        &self.outbuf[0..old_index]
    }
    fn get_context(&self) -> &dyn GenericContext {
        &self.ctx
    }
    fn is_poly(&self) -> bool {
        true
    }
}
