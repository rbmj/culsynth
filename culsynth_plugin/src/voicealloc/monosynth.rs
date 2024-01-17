use itertools::izip;

use super::*;
use rand::random;

use culsynth::{voice::VoiceInput, DspFormat};

/// A monosynth utilizing fixed point logic internally
#[derive(Default, Clone)]
pub struct MonoSynth<T: DspFormat> {
    voice: Voice<T>,
    outbuf: Box<[f32]>,
    ch_input_buf: Box<[VoiceChannelInput<i16>]>,
    input_buf: Box<[VoiceInput<i16>]>,
    ctx: T::Context,
    index: usize,
    note: NoteFxP,
    gate: SampleFxP,
    velocity: ScalarFxP,
    aftertouch: ScalarFxP,
    modwheel: ScalarFxP,
    pitch_bend: SignedNoteFxP,
    pitch_range: (fixed::types::I16F0, fixed::types::I16F0),
}

impl<T: DspFormat> MonoSynth<T> {
    pub fn new(ctx: T::Context, sz: usize) -> Self {
        let mut outbuf: Vec<f32> = Vec::default();
        let mut inputbuf: Vec<VoiceInput<i16>> = Vec::default();
        let mut chinputbuf: Vec<VoiceChannelInput<i16>> = Vec::default();
        outbuf.resize(sz, 0f32);
        inputbuf.resize(sz, Default::default());
        chinputbuf.resize(sz, Default::default());
        Self {
            voice: Voice::new_with_seeds(random(), random()),
            outbuf: outbuf.into_boxed_slice(),
            input_buf: inputbuf.into_boxed_slice(),
            ch_input_buf: chinputbuf.into_boxed_slice(),
            ctx,
            index: 0,
            note: NoteFxP::lit("69"), //A440, nice
            gate: SampleFxP::ZERO,
            velocity: ScalarFxP::ZERO,
            aftertouch: ScalarFxP::ZERO,
            modwheel: ScalarFxP::ZERO,
            pitch_bend: SignedNoteFxP::ZERO,
            pitch_range: (2i16.into(), 2i16.into()),
        }
    }
}

impl<T: DspFormat> VoiceAllocator for MonoSynth<T>
    where for<'a> ModMatrix<T>: From<&'a ModMatrix<i16>>,
        for<'a> VoiceInput<T>: From<&'a VoiceInput<i16>>,
        for<'a> VoiceChannelInput<T>: From<&'a VoiceChannelInput<i16>>,
        for<'a> VoiceParams<T>: From<&'a VoiceParams<i16>>
{
    fn sample_tick(&mut self) {
        self.ch_input_buf[self.index] = VoiceChannelInput {
            aftertouch: self.aftertouch,
            modwheel: self.modwheel,
        };
        self.input_buf[self.index] = VoiceInput {
            note: self.note.add_signed(self.pitch_bend),
            gate: self.gate,
            velocity: self.velocity,

        };
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
        matrix: &ModMatrix<i16>,
        params: &[VoiceParams<i16>],
    ) -> &[f32] {
        let mut processed: usize = 0;
        let matrix: ModMatrix<T> = matrix.into();
        for (out, ch_input, input, param) in izip!(
            self.outbuf.iter_mut(),
            self.ch_input_buf[0..self.index].iter(),
            self.input_buf.iter(),
            params.iter()
        ) {
            let x = self.voice.next(&self.ctx, &matrix, &input.into(), &ch_input.into(), param.into());
            *out = T::sample_to_float(x);
            processed += 1;
        }
        self.index = 0;
        &self.outbuf[0..processed]
    }
    fn get_context(&self) -> &dyn GenericContext {
        <T::Context as culsynth::context::GetContext>::get_context(&self.ctx)
    }
    fn is_poly(&self) -> bool {
        false
    }
}
