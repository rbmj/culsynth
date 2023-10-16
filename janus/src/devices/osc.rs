use crate::fixedmath::midi_note_to_frequency;
use crate::util::midi_note_pretty;

//
use super::*;

type PhaseFxP = fixedmath::I4F28;
type PhaseFxP_Trunc = fixedmath::I4F12;

struct Oscillator<smp> {
    sinbuf: BufferT<smp>,
    sqbuf: BufferT<smp>,
    tribuf: BufferT<smp>,
    phase: smp
}

impl<smp> Oscillator<smp> {
    fn process() {
        //smp_per_period = sample_rate / frequency;

    }
}

pub struct OscOutputFxP<'a> {
    sin: &'a [SampleFxP],
    sq: &'a [SampleFxP],
    tri: &'a [SampleFxP]
}

pub struct OscillatorFxP {
    sinbuf: BufferT<SampleFxP>,
    sqbuf: BufferT<SampleFxP>,
    tribuf: BufferT<SampleFxP>,
    phase: PhaseFxP
}

impl OscillatorFxP {
    pub fn create() -> OscillatorFxP {
        OscillatorFxP {
            sinbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            sqbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            tribuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            phase: PhaseFxP::ZERO
        }
    }
    pub fn process(&mut self, note: &[NoteFxP], shape: &[fixedmath::U0F16]) -> OscOutputFxP {
        let input_len = std::cmp::min(note.len(), shape.len());
        let numsamples = std::cmp::min(input_len, STATIC_BUFFER_SIZE);
        const FRAC_2_PI : fixedmath::U0F16 = fixedmath::U0F16::lit("0x0.a2fa");
        //FIXME: Variable sample rate?
        const SAMPLE_RATE : fixedmath::U16F0 = fixedmath::U16F0::lit("44100");
        // 4096*2*pi/44.1k
        const FRAC_4096_2PI_SR : fixedmath::U0F32 = fixedmath::U0F32::lit("0x0.9565925d");
        const SHAPE_CAP : fixedmath::U0F16 = fixedmath::U0F16::lit("0x0.FF00");
        for i in 0..numsamples {
            //generate waveforms (piecewise defined)
            let frac_2phase_pi = SampleFxP::from_num(PhaseFxP_Trunc::from_num(
                self.phase).wide_mul_unsigned(FRAC_2_PI));
            if self.phase < 0 {
                self.sqbuf[i] = SampleFxP::NEG_ONE;
                if self.phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {  // phase in [-pi, pi/2)
                    // sin(x) = -cos(x+pi/2)
                    self.sinbuf[i] = fixedmath::cos_fixed(
                        SampleFxP::from_num(self.phase + PhaseFxP::FRAC_PI_2))
                        .unwrapped_neg();
                    self.tribuf[i] = frac_2phase_pi.unwrapped_neg() - SampleFxP::lit("2");
                }
                else {  // phase in [-pi/2, 0)
                    self.sinbuf[i] = fixedmath::sin_fixed(SampleFxP::from_num(self.phase));
                    //triangle
                    self.tribuf[i] = frac_2phase_pi;
                }
            }
            else {
                self.sqbuf[i] = SampleFxP::ONE;
                if self.phase < PhaseFxP::FRAC_PI_2 { // phase in [0, pi/2)
                    self.sinbuf[i] = fixedmath::sin_fixed(SampleFxP::from_num(self.phase));
                    self.tribuf[i] = frac_2phase_pi;
                }
                else { // phase in [pi/2, pi)
                    // sin(x) = cos(x-pi/2)
                    self.sinbuf[i] = fixedmath::cos_fixed(SampleFxP::from_num(self.phase - PhaseFxP::FRAC_PI_2));
                    self.tribuf[i] = SampleFxP::lit("2") - frac_2phase_pi;
                }
            }
            //calculate the next phase
            let phase_per_sample = PhaseFxP::from_num(fixedmath::midi_note_to_frequency(note[i])
                .wide_mul(FRAC_4096_2PI_SR)
                .unwrapped_shr(12));
            let shape_capped = if shape[i] <= SHAPE_CAP { shape[i] } else { SHAPE_CAP };
            let mut adjustment = PhaseFxP::from_num(
                fixedmath::U1F15::from_num(phase_per_sample).wide_mul(shape_capped));
            if self.phase < PhaseFxP::ZERO {
                adjustment = adjustment.unwrapped_neg();
            }
            let phase_per_sample_adj = phase_per_sample + adjustment;
            self.phase += phase_per_sample_adj;
            if self.phase >= PhaseFxP::PI {
                self.phase -= PhaseFxP::TAU;
            }
        }
        OscOutputFxP {
            sin: &self.sinbuf[0..numsamples],
            tri: &self.tribuf[0..numsamples],
            sq: &self.sqbuf[0..numsamples]
        }    
    }
}

mod bindings {
    use super::*;

    #[no_mangle]
    pub extern "C" fn janus_osc_u16_new() -> *mut OscillatorFxP {
        Box::into_raw(Box::new(OscillatorFxP::create()))
    }

    #[no_mangle]
    pub extern "C" fn janus_osc_u16_free(p: *mut OscillatorFxP) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_osc_u16_process(
        p: *mut OscillatorFxP,
        samples: u32,
        note: *const u16,
        shape: *const u16,
        sin: *mut *const i16,
        tri: *mut *const i16,
        sq: *mut *const i16,
        offset: u32
    ) -> i32 {
        if p.is_null()
            || note.is_null()
            || shape.is_null()
            || sin.is_null()
            || tri.is_null()
            || sq.is_null()
        {
            return -1;
        }
        unsafe {
            let note_s = std::slice::from_raw_parts(
                note.offset(offset as isize).cast::<NoteFxP>(), samples as usize);
            let shape_s = std::slice::from_raw_parts(
                shape.offset(offset as isize).cast::<fixedmath::U0F16>(), samples as usize);
            let out = p.as_mut().unwrap().process(note_s, shape_s);
            *sin = out.sin.as_ptr().cast();
            *tri = out.tri.as_ptr().cast();
            *sq = out.sq.as_ptr().cast();
            out.sin.len() as i32
        }
    }
}