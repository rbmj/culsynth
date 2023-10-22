use janus::{NoteFxP, SampleFxP, VoiceFxP};

use crate::parambuf::{OscParamBuffer, FiltParamBuffer, EnvParamBuffer};

pub trait VoiceAllocator {
    fn initialize(&mut self, sz: usize);
    fn sample_tick(&mut self);
    fn note_on(&mut self, n: u8, v: u8);
    fn note_off(&mut self, n: u8, v: u8);
    fn process(&mut self, op: &OscParamBuffer, fp: &FiltParamBuffer, eap: &EnvParamBuffer) -> &[f32];
    fn is_fixed_point(&self) -> bool;
}

#[derive(Default)]
pub struct MonoSynthFxP {
    voice: VoiceFxP,
    outbuf: Vec<f32>,
    notebuf: Vec<NoteFxP>,
    gatebuf: Vec<SampleFxP>,
    note: NoteFxP,
    index: usize,
    gate: SampleFxP
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
    fn process(&mut self, op: &OscParamBuffer, fp: &FiltParamBuffer, eap: &EnvParamBuffer) -> &[f32] {
        let mut processed: usize = 0;
        while processed < self.index {
            let thisiter = self.voice.process(&self.notebuf[processed..self.index], 
                &self.gatebuf[processed..self.index],
                op.params_fxp(processed, self.index),
                fp.params_fxp(processed, self.index),
                eap.params_fxp(processed, self.index));
            for i in 0..thisiter.len() {
                self.outbuf[processed] = thisiter[i].to_num::<f32>();
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