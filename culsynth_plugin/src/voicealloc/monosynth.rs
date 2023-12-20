use super::*;
use rand::random;

/// A monosynth utilizing fixed point logic internally
#[derive(Default, Clone)]
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
            voice: VoiceFxP::new_with_seeds(random(), random()),
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
        self.velocity = ScalarFxP::from_bits((velocity as u16) << 9);
    }
    fn note_off(&mut self, note: u8, _velocity: u8) {
        if self.note == note {
            self.gate = SampleFxP::ZERO;
            //self.velocity = ScalarFxP::from_bits((velocity as u16) << 9);
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
                SignedNoteFxP::from_num(IScalarFxP::from_bits(v).wide_mul(self.pitch_range.0));
        } else {
            self.pitch_bend =
                SignedNoteFxP::from_num(IScalarFxP::from_bits(v).wide_mul(self.pitch_range.1));
        }
    }
    fn get_pitch_bend_range(&self) -> (i8, i8) {
        (
            self.pitch_range.0.to_num::<i8>(),
            self.pitch_range.1.to_num::<i8>(),
        )
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
        params: &mut PluginParamBufFxP,
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
                params.global.sync_mut(processed, self.index),
                params.osc1.params_mut(processed, self.index),
                params.osc2.params_mut(processed, self.index),
                params.ringmod.params_mut(processed, self.index),
                params.filt.params_mut(processed, self.index),
                params.env_filt.params_mut(processed, self.index),
                params.env_amp.params_mut(processed, self.index),
                params.lfo1.params(processed, self.index),
                params.lfo2.params_mut(processed, self.index),
                params.env1.params(processed, self.index),
                params.env2.params_mut(processed, self.index),
            );
            for smp in thisiter {
                self.outbuf[processed] = smp.unwrapped_shr(4).to_num::<f32>();
                processed += 1;
            }
        }
        self.index = 0;
        &self.outbuf[0..processed]
    }
    fn get_context(&self) -> &dyn GenericContext {
        &self.ctx
    }
    fn is_poly(&self) -> bool {
        false
    }
}

/// A monosynth utilizing floating point logic internally
#[derive(Default, Clone)]
pub struct MonoSynth {
    voice: Voice<f32>,
    params: PluginParamBuf,
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
        self.params.allocate(sz as u32);
        self.index = 0;
        self.note = 69f32; //A440, nice
        self.gate = 0f32;
        self.set_pitch_bend_range(2, 2);
    }
    fn sample_tick(&mut self) {
        self.notebuf[self.index] = self.note + self.pitch_bend;
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
    fn note_off(&mut self, note: u8, _velocity: u8) {
        if self.note == f32::from(note) {
            self.gate = 0f32;
            //self.velocity = f32::from(velocity) / 127f32;
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
        params: &mut PluginParamBufFxP,
    ) -> &[f32] {
        params.into_float(&mut self.params, self.index);
        let mut processed: usize = 0;
        let matrix = matrix_p.build_matrix_float();
        while processed < self.index {
            let thisiter = self.voice.process(
                &self.ctx,
                &matrix,
                &self.notebuf[processed..self.index],
                &self.gatebuf[processed..self.index],
                &self.velbuf[processed..self.index],
                &self.aftertouchbuf[processed..self.index],
                &self.modwheelbuf[processed..self.index],
                self.params.global.sync_mut(processed, self.index),
                self.params.osc1.params_mut(processed, self.index),
                self.params.osc2.params_mut(processed, self.index),
                self.params.ringmod.params_mut(processed, self.index),
                self.params.filt.params_mut(processed, self.index),
                self.params.env_filt.params_mut(processed, self.index),
                self.params.env_amp.params_mut(processed, self.index),
                self.params.lfo1.params(processed, self.index),
                self.params.lfo2.params_mut(processed, self.index),
                self.params.env1.params(processed, self.index),
                self.params.env2.params_mut(processed, self.index),
            );
            for smp in thisiter {
                self.outbuf[processed] = *smp / 16f32;
                processed += 1;
            }
        }
        self.index = 0;
        &self.outbuf[0..processed]
    }
    fn get_context(&self) -> &dyn GenericContext {
        &self.ctx
    }
    fn is_poly(&self) -> bool {
        false
    }
}
