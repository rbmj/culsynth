use std::collections::VecDeque;

use super::*;
use culsynth::DspFormat;
use nih_plug::nih_error;
use rand::random;

struct PolySynthVoice<T: DspFormat> {
    voice: Voice<T>,
    vel: ScalarFxP,
    note: NoteFxP,
    gate: bool,
}

impl<T: DspFormat> PolySynthVoice<T> {
    fn new() -> Self {
        Self {
            voice: Voice::new_with_seeds(random(), random()),
            note: NoteFxP::from_num(69), //A440
            gate: false,
            vel: ScalarFxP::ZERO,
        }
    }
}

pub struct PolySynth<T: DspFormat> {
    voices: Box<[PolySynthVoice<T>]>,
    matrix: ModMatrix<T>,
    active_voices: VecDeque<usize>,
    inactive_voices: VecDeque<usize>,
    pitch_bend_range: (fixed::types::I16F0, fixed::types::I16F0),
    pitch_bend: SignedNoteFxP,
    aftertouch: ScalarFxP,
    modwheel: ScalarFxP,
    ctx: T::Context,
}

impl<T: DspFormat> PolySynth<T> {
    pub fn new(context: T::Context, num_voices: usize) -> Self {
        let voices = std::iter::repeat_with(|| PolySynthVoice::<T>::new())
            .take(num_voices)
            .collect::<Box<[_]>>();
        let mut active_voices = VecDeque::<usize>::new();
        let mut inactive_voices = VecDeque::<usize>::new();
        active_voices.reserve(voices.len());
        inactive_voices.reserve(voices.len());
        for (i, _) in voices.iter().enumerate() {
            inactive_voices.push_back(i);
        }
        Self {
            voices,
            matrix: Default::default(),
            active_voices,
            inactive_voices,
            pitch_bend: SignedNoteFxP::ZERO,
            pitch_bend_range: (2i16.into(), 2i16.into()),
            aftertouch: ScalarFxP::ZERO,
            modwheel: ScalarFxP::ZERO,
            ctx: context,
        }
    }
    fn note_on_i(&mut self, voice_index: usize, note: u8, vel: u8) {
        self.active_voices.push_back(voice_index);
        let voice = &mut self.voices[voice_index];
        voice.note = NoteFxP::from_num(note);
        voice.vel = ScalarFxP::from_bits((vel as u16) << 9);
        voice.gate = true;
    }
}

impl<T: DspFormat> VoiceAllocator for PolySynth<T>
where
    for<'a> ModMatrix<T>: From<&'a ModMatrix<i16>>,
    for<'a> VoiceInput<T>: From<&'a VoiceInput<i16>>,
    for<'a> VoiceChannelInput<T>: From<&'a VoiceChannelInput<i16>>,
    for<'a> VoiceParams<T>: From<&'a VoiceParams<i16>>,
{
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
            self.voices[*vox_idx].gate = false;
            self.active_voices.remove(act_idx);
        }
    }
    fn aftertouch(&mut self, value: u8) {
        self.aftertouch = ScalarFxP::from_bits((value as u16) << 9);
    }
    fn handle_cc(&mut self, cc: u8, value: u8, dispatcher: &mut SyncSender<(u8, u8)>) {
        match cc {
            midi::control_change::MODULATION_MSB => {
                self.modwheel = ScalarFxP::from_bits((value as u16) << 9);
            }
            midi::control_change::MODULATION_LSB => {
                self.modwheel |= ScalarFxP::from_bits((value as u16) << 2);
            }
            _ => {
                let _ = dispatcher.try_send((cc, value));
            }
        }
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
    fn next(&mut self, params: &VoiceParams<i16>, matrix: Option<&ModMatrix<i16>>) -> f32 {
        let mut out = 0f32;
        // Handle matrix conversion into a different format, if required
        let matrix_param = if let Some(matrix) = matrix {
            self.matrix = matrix.into();
            Some(&self.matrix)
        } else {
            None
        };
        let ch_in = &VoiceChannelInput::<i16> {
            aftertouch: self.aftertouch,
            modwheel: self.modwheel,
        };
        for v in self.voices.iter_mut() {
            let input = &VoiceInput::<i16> {
                note: v.note.add_signed(self.pitch_bend),
                gate: v.gate,
                velocity: v.vel,
            };
            out += T::sample_to_float(v.voice.next(
                &self.ctx,
                matrix_param,
                &input.into(),
                &ch_in.into(),
                params.into(),
            ));
        }
        // Signal is a hair hot (0dB), so attenuate it just a bit...
        out / 8.
    }
    fn get_context(&self) -> &dyn GenericContext {
        <T::Context as culsynth::context::GetContext>::get_context(&self.ctx)
    }
    fn is_poly(&self) -> bool {
        true
    }
}
