use super::*;
use rand::random;

use culsynth::{voice::VoiceInput, DspFormat};

/// A monosynth utilizing fixed point logic internally
#[derive(Default, Clone)]
pub struct MonoSynth<T: DspFormat> {
    voice: Voice<T>,
    matrix: ModMatrix<T>,
    ctx: T::Context,
    pitch_range: (fixed::types::I16F0, fixed::types::I16F0),
    pitch_bend: SignedNoteFxP,
    note: NoteFxP,
    velocity: ScalarFxP,
    aftertouch: ScalarFxP,
    modwheel: ScalarFxP,
    gate: bool,
}

impl<T: DspFormat> MonoSynth<T> {
    pub fn new(ctx: T::Context) -> Self {
        Self {
            voice: Voice::new_with_seeds(random(), random()),
            matrix: Default::default(),
            ctx,
            note: NoteFxP::lit("69"), //A440, nice
            gate: false,
            velocity: ScalarFxP::ZERO,
            aftertouch: ScalarFxP::ZERO,
            modwheel: ScalarFxP::ZERO,
            pitch_bend: SignedNoteFxP::ZERO,
            pitch_range: (2i16.into(), 2i16.into()),
        }
    }
}

impl<T: DspFormat> VoiceAllocator for MonoSynth<T>
where
    for<'a> ModMatrix<T>: From<&'a ModMatrix<i16>>,
    for<'a> VoiceInput<T>: From<&'a VoiceInput<i16>>,
    for<'a> VoiceChannelInput<T>: From<&'a VoiceChannelInput<i16>>,
    for<'a> VoiceParams<T>: From<&'a VoiceParams<i16>>,
{
    fn note_on(&mut self, note: u8, velocity: u8) {
        self.note = NoteFxP::from_num(note);
        self.gate = true;
        self.velocity = ScalarFxP::from_bits((velocity as u16) << 9);
    }
    fn note_off(&mut self, note: u8, _velocity: u8) {
        if self.note == note {
            self.gate = false;
            //self.velocity = ScalarFxP::from_bits((velocity as u16) << 9);
        }
    }
    fn get_channel(&self) -> Option<wmidi::Channel> {
        None //TODO
    }
    fn aftertouch(&mut self, value: u8) {
        self.aftertouch = ScalarFxP::from_bits((value as u16) << 9);
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
    fn next(&mut self, params: &VoiceParams<i16>, matrix: Option<&ModMatrix<i16>>) -> f32 {
        let ch_input = &VoiceChannelInput::<i16> {
            aftertouch: self.aftertouch,
            modwheel: self.modwheel,
        };
        let input = &VoiceInput::<i16> {
            note: self.note.add_signed(self.pitch_bend),
            gate: self.gate,
            velocity: self.velocity,
        };
        // Handle matrix conversion, if required
        let matrix_param = if let Some(matrix) = matrix {
            self.matrix = matrix.into();
            Some(&self.matrix)
        } else {
            None
        };
        T::sample_to_float(self.voice.next(
            &self.ctx,
            matrix_param,
            &input.into(),
            &ch_input.into(),
            params.into(),
        )) / 4. //Rescale from 0dB to -6dB to avoid DAWs going into the red
    }
    fn get_context(&self) -> &dyn GenericContext {
        <T::Context as culsynth::context::GetContext>::get_context(&self.ctx)
    }
    fn is_poly(&self) -> bool {
        false
    }
    fn handle_cc(
        &mut self,
        cc: wmidi::ControlFunction,
        value: u8,
        dispatcher: &mut dyn MidiCcHandler,
    ) {
        match cc {
            wmidi::ControlFunction::MODULATION_WHEEL => {
                self.modwheel = ScalarFxP::from_bits((value as u16) << 9);
            }
            wmidi::ControlFunction::MODULATION_WHEEL_LSB => {
                self.modwheel |= ScalarFxP::from_bits((value as u16) << 2);
            }
            _ => {
                let _ = dispatcher.handle_cc(cc, value);
            }
        }
    }
}
