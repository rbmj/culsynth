use crate::fixedmath::midi_note_to_frequency;
use crate::util::midi_note_pretty;

//
use super::BufferT;
use super::SampleFxP;
use super::fixedmath;

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

struct OscOutputFxP {
    sin: &[SampleFxP],
    sq: &[SampleFxP],
    tri: &[SampleFxP]
}

struct OscillatorFxP {
    sinbuf: BufferT<SampleFxP>,
    sqbuf: BufferT<SampleFxP>,
    tribuf: BufferT<SampleFxP>,
    phase: PhaseFxP
}

impl OscillatorFxP {
    pub fn process(&mut self, note: &[fixedmath::U7F9], shape: &[fixedmath::U0F16]) -> usize {
        let numsamples = note.len();
        assert!(numsamples == shape.len());
        const FRAC_2_PI : fixedmath::U0F16 = fixedmath::U0F16::lit("0x0.a2fa");
        const SAMPLE_RATE : fixedmath::U16F0 = fixedmath::U16F0::lit("44100");
        // 2048*2*pi/44.1k
        const FRAC_2048_2PI_SR : fixedmath::U0F16 = fixedmath::U0F32::lit("0x0.9565925d");
        const SHAPE_CAP : fixedmath::U0F16 = fixedmath::U0F16::lit("0x0.FF00");
        for i in 0..numsamples {
            //generate waveforms (piecewise defined)
            let frac_2phase_pi = PhaseFxP_Trunc::from_num(self.phase)
                .wide_mul_unsigned(FRAC_2_PI);
            if phase < 0 {
                self.sqbuf[i] = -1;
                if phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {  // phase in [-pi, pi/2)
                    // sin(x) = -cos(x+pi/2)
                    self.sinbuf[i] = cos_fixed(self.phase + PhaseFxP::FRAC_PI_2)
                        .unwrapped_neg();
                    self.tribuf[i] = frac_2phase_pi.unwrapped_neg() - 2;
                }
                else {  // phase in [-pi/2, 0)
                    self.sinbuf[i] = sin_fixed(self.phase);
                    //triangle
                    self.tribuf[i] = frac_2phase_pi;
                }
            }
            else {
                self.sqbuf[i] = 1;
                if phase < PhaseFxP::FRAC_PI_2 { // phase in [0, pi/2)
                    self.sinbuf[i] = sin_fixed(self.phase);
                    self.tribuf[i] = frac_2phase_pi;
                }
                else { // phase in [pi/2, pi)
                    // sin(x) = cos(x-pi/2)
                    self.sinbuf[i] = cos_fixed(self.phase - PhaseFxP::FRAC_PI_2);
                    self.tribuf[i] = 2 - frac_2phase_pi;
                }
            }
            //calculate the next phase
            let phase_per_sample = midi_note_to_frequency(note)
                .wide_mul(FRAC_2048_2PI_SR)
                .unwrapped_shr(12).to_fixed::<PhaseFxP>();
            let shape_capped = if shape[i] <= SHAPE_CAP { shape[i] } else { SHAPE_CAP };
            let mut adjustment = fixedmath::I4F12::from_num(phase_per_sample)
                .wide_mul(shape_capped);
            if self.phase > PhaseFxP::PI {
                adjustment = adjustment.unwrapped_neg();
            }
            let phase_per_sample_adj = phase_per_sample + adjustment;
            self.phase += phase_per_sample_adj;
            if self.phase >= PhaseFxP::PI {
                self.phase -= PhaseFxP::PI;
            }
        }
        OscOutputFxP {
            sin: &self.sinbuf[0..numsamples],
            tri: &self.tribuf[0..numsamples],
            sq: &self.sqbuf[0..numsamples]
        }    
    }
}