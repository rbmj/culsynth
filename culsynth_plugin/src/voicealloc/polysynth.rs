use std::collections::VecDeque;

use super::*;
use culsynth::DspFormat;
use nih_plug::nih_error;
use rand::random;
use itertools::izip;

struct PolySynthVoice<T: DspFormat> {
    voice: Voice<T>,
    inputbuf: Box<[VoiceInput<i16>]>,
    gate: SampleFxP,
    vel: ScalarFxP,
    note: u8,
}

impl<T: DspFormat> PolySynthVoice<T> {
    fn new(sz: usize) -> Self {
        let mut inputbuf = Vec::<VoiceInput<i16>>::default();
        inputbuf.resize(sz, Default::default());
        Self {
            voice: Voice::new_with_seeds(random(), random()),
            inputbuf: inputbuf.into_boxed_slice(),
            note: 69, //A440
            gate: SampleFxP::ZERO,
            vel: ScalarFxP::ZERO,
        }
    }
}

pub struct PolySynth<T: DspFormat> {
    voices: Box<[PolySynthVoice<T>]>,
    active_voices: VecDeque<usize>,
    inactive_voices: VecDeque<usize>,
    outbuf: Box<[f32]>,
    inputbuf: Box<[VoiceChannelInput<i16>]>,
    pitch_bend_range: (fixed::types::I16F0, fixed::types::I16F0),
    pitch_bend: SignedNoteFxP,
    aftertouch: ScalarFxP,
    modwheel: ScalarFxP,
    index: usize,
    ctx: T::Context,
}

impl<T: DspFormat> PolySynth<T> {
    pub fn new(context: T::Context, bufsz: usize, num_voices: usize) -> Self {
        let voices = std::iter::repeat_with(|| PolySynthVoice::<T>::new(bufsz))
            .take(num_voices)
            .collect::<Box<[_]>>();
        let mut outbuf: Vec<f32> = Vec::default();
        let mut inputbuf: Vec<VoiceChannelInput<i16>> = Vec::default();
        outbuf.resize(bufsz, 0f32);
        inputbuf.resize(bufsz, Default::default());
        let mut active_voices = VecDeque::<usize>::new();
        let mut inactive_voices = VecDeque::<usize>::new();
        active_voices.reserve(voices.len());
        inactive_voices.reserve(voices.len());
        for (i, _) in voices.iter().enumerate() {
            inactive_voices.push_back(i);
        }
        Self {
            voices,
            active_voices,
            inactive_voices,
            outbuf: outbuf.into_boxed_slice(),
            inputbuf: inputbuf.into_boxed_slice(),
            pitch_bend: SignedNoteFxP::ZERO,
            pitch_bend_range: (2i16.into(), 2i16.into()),
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

impl<T: DspFormat> VoiceAllocator for PolySynth<T>
    where for<'a> ModMatrix<T>: From<&'a ModMatrix<i16>>,
        for<'a> VoiceInput<T>: From<&'a VoiceInput<i16>>,
        for<'a> VoiceChannelInput<T>: From<&'a VoiceChannelInput<i16>>,
        for<'a> VoiceParams<T>: From<&'a VoiceParams<i16>>
{
    fn sample_tick(&mut self) {
        self.inputbuf[self.index] = VoiceChannelInput {
            aftertouch: self.aftertouch,
            modwheel: self.modwheel,
        };
        for voice in self.voices.iter_mut() {
            voice.inputbuf[self.index] = VoiceInput {
                note: NoteFxP::from_num(voice.note).add_signed(self.pitch_bend),
                gate: voice.gate,
                velocity: voice.vel,
            };
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
        matrix: &ModMatrix<i16>,
        params: &[VoiceParams<i16>],
    ) -> &[f32] {
        for smp in self.outbuf.iter_mut() {
            *smp = 0f32;
        }
        let matrix = ModMatrix::<T>::from(matrix);
        for v in self.voices.iter_mut() {
            for (out, ch_input, input, param) in izip!(
                self.outbuf.iter_mut(),
                self.inputbuf[0..self.index].iter(),
                v.inputbuf.iter(),
                params.iter()
            ) {
                let x = v.voice.next(&self.ctx, &matrix, &input.into(), &ch_input.into(), param.into());
                *out += T::sample_to_float(x);
            }
        }
        let old_index = self.index;
        self.index = 0;
        &self.outbuf[0..std::cmp::min(params.len(), old_index)]
    }
    fn get_context(&self) -> &dyn GenericContext {
        <T::Context as culsynth::context::GetContext>::get_context(&self.ctx)
    }
    fn is_poly(&self) -> bool {
        true
    }
}
