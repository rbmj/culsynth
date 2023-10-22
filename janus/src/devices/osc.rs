use super::*;

type PhaseFxP = fixedmath::I4F28;

pub struct Osc<Smp> {
    sinbuf: BufferT<Smp>,
    sqbuf: BufferT<Smp>,
    tribuf: BufferT<Smp>,
    sawbuf: BufferT<Smp>,
    phase: Smp
}

pub struct OscOutput<'a, Smp> {
    pub sin: &'a [Smp],
    pub sq: &'a [Smp],
    pub tri: &'a [Smp],
    pub saw: &'a [Smp]
}

pub struct OscParams<'a, Smp> {
    pub shape: &'a [Smp]
}


impl<Smp: Float> Osc<Smp> {
    pub fn new() -> Self {
        Self {
            sinbuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            sqbuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            tribuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            sawbuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            phase: Smp::zero()
        }
    }
    pub fn process(&mut self, note: &[Smp], params: OscParams<Smp>) -> OscOutput<Smp> {
        let shape = params.shape;
        let input_len = std::cmp::min(note.len(), shape.len());
        let numsamples = std::cmp::min(input_len, STATIC_BUFFER_SIZE);
        // We don't have to split sin up piecewise but we'll do it for symmetry with
        // the fixed point implementation and to make it easy to switch to an
        // approximation if performance dictates
        for i in 0..numsamples {
            //generate waveforms (piecewise defined)
            let frac_2phase_pi = self.phase * Smp::FRAC_2_PI();
            self.sawbuf[i] = frac_2phase_pi / (Smp::one() + Smp::one());
            if self.phase < Smp::ZERO {
                self.sqbuf[i] = Smp::one().neg();
                if self.phase < Smp::FRAC_PI_2().neg() {  // phase in [-pi, pi/2)
                    // sin(x) = -cos(x+pi/2)
                    // TODO: Use fast approximation?
                    self.sinbuf[i] = (self.phase + Smp::FRAC_PI_2()).cos().neg();
                    // Subtract (1+1) because traits :eyeroll:
                    self.tribuf[i] = frac_2phase_pi.neg() - Smp::TWO;
                }
                else {  // phase in [-pi/2, 0)
                    self.sinbuf[i] = self.phase.sin();
                    //triangle
                    self.tribuf[i] = frac_2phase_pi;
                }
            }
            else {
                self.sqbuf[i] = Smp::one();
                if self.phase < Smp::FRAC_PI_2() { // phase in [0, pi/2)
                    self.sinbuf[i] = self.phase.sin();
                    self.tribuf[i] = frac_2phase_pi;
                }
                else { // phase in [pi/2, pi)
                    // sin(x) = cos(x-pi/2)
                    self.sinbuf[i] = (self.phase - Smp::FRAC_PI_2()).cos();
                    self.tribuf[i] = Smp::TWO - frac_2phase_pi;
                }
            }
            let sample_rate = Smp::from(SAMPLE_RATE).unwrap();
            //calculate the next phase
            let phase_per_sample = midi_note_to_frequency(note[i])*Smp::TAU()/sample_rate;
            let shape_clip = Smp::from(0.9375).unwrap();
            let shp = if shape[i] < shape_clip { shape[i] } else { shape_clip };
            let phase_per_smp_adj = if self.phase < Smp::zero() {
                phase_per_sample * (Smp::ONE / (Smp::ONE + shp))
            }
            else {
                phase_per_sample * (Smp::ONE / (Smp::ONE - shp))
            };
            self.phase = self.phase + phase_per_smp_adj;
            if self.phase >= Smp::PI() {
                self.phase = self.phase - Smp::TAU();
            }
        }
        OscOutput {
            sin: &self.sinbuf[0..numsamples],
            tri: &self.tribuf[0..numsamples],
            sq: &self.sqbuf[0..numsamples],
            saw: &self.sawbuf[0..numsamples]
        }    
    }
}

impl<Smp: Float> Default for Osc<Smp> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OscOutputFxP<'a> {
    pub sin: &'a [SampleFxP],
    pub sq: &'a [SampleFxP],
    pub tri: &'a [SampleFxP],
    pub saw: &'a [SampleFxP]
}

pub struct OscParamsFxP<'a> {
    pub shape: &'a [ScalarFxP]
}

pub struct OscFxP {
    sinbuf: BufferT<SampleFxP>,
    sqbuf: BufferT<SampleFxP>,
    tribuf: BufferT<SampleFxP>,
    sawbuf: BufferT<SampleFxP>,
    phase: PhaseFxP
}

impl OscFxP {
    pub fn new() -> OscFxP {
        OscFxP {
            sinbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            sqbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            tribuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            sawbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            phase: PhaseFxP::ZERO
        }
    }
    pub fn process(&mut self, note: &[NoteFxP], params: OscParamsFxP) -> OscOutputFxP {
        let shape = params.shape;
        let input_len = std::cmp::min(note.len(), shape.len());
        let numsamples = std::cmp::min(input_len, STATIC_BUFFER_SIZE);
        const FRAC_2_PI : ScalarFxP = ScalarFxP::lit("0x0.a2fa");
        for i in 0..numsamples {
            //generate waveforms (piecewise defined)
            let frac_2phase_pi = SampleFxP::from_num(SampleFxP::from_num(
                self.phase).wide_mul_unsigned(FRAC_2_PI));
            //Sawtooth wave does not have to be piecewise-defined
            self.sawbuf[i] = frac_2phase_pi.unwrapped_shr(1);
            //All other functions are piecewise-defined:
            if self.phase < 0 {
                self.sqbuf[i] = SampleFxP::NEG_ONE;
                if self.phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {  // phase in [-pi, pi/2)
                    // Use the identity sin(x) = -cos(x+pi/2) since our taylor series
                    // approximations are centered about zero and this will be more accurate
                    self.sinbuf[i] = fixedmath::cos_fixed(
                        SampleFxP::from_num(self.phase + PhaseFxP::FRAC_PI_2))
                        .unwrapped_neg();
                    self.tribuf[i] = frac_2phase_pi.unwrapped_neg() - SampleFxP::lit("2");
                }
                else {  // phase in [-pi/2, 0)
                    self.sinbuf[i] = fixedmath::sin_fixed(SampleFxP::from_num(self.phase));
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
            let phase_per_sample = fixedmath::U4F28::from_num(fixedmath::midi_note_to_frequency(note[i])
                .wide_mul(FRAC_4096_2PI_SR)
                .unwrapped_shr(12));
            let phase_per_smp_adj = PhaseFxP::from_num(if self.phase < PhaseFxP::ZERO {
                let (x, s) = fixedmath::one_over_one_plus_highacc(clip_shape(shape[i]));
                fixedmath::scale_fixedfloat(phase_per_sample, x).unwrapped_shr(s)
            }
            else {
                fixedmath::scale_fixedfloat(phase_per_sample, one_over_one_minus_x(shape[i]))
            });
            //FIXME:  when shape is close to 1, tuning becomes inaccurate due to the incorrect phase
            //calculation at the transition between the two halfs of the waveform
            self.phase += phase_per_smp_adj;
            if self.phase >= PhaseFxP::PI {
                self.phase -= PhaseFxP::TAU;
            }
        }
        OscOutputFxP {
            sin: &self.sinbuf[0..numsamples],
            tri: &self.tribuf[0..numsamples],
            sq: &self.sqbuf[0..numsamples],
            saw: &self.sawbuf[0..numsamples]
        }    
    }
}

impl Default for OscFxP {
    fn default() -> Self {
        Self::new()
    }
}

fn clip_shape(x: ScalarFxP) -> ScalarFxP {
    const CLIP_MAX: ScalarFxP = ScalarFxP::lit("0x0.F");
    if x > CLIP_MAX {
        CLIP_MAX
    }
    else {
        x
    }
}

fn one_over_one_minus_x(x: ScalarFxP) -> USampleFxP {
    let x_bits = clip_shape(x).to_bits();
    // Table generated with python:
    //
    // table = [1/(1-(x/256.0)) for x in range(0,256)][:0xF1]
    // shifted = [int(x*256*16) for x in table]
    // shifted[-1] = shifted[-1] - 1 # Prevent overflow
    // hexvals = [hex(x) for x in shifted]
    // for i in range(len(hexvals)):
    //     val = hexvals[i]
    //     print('USampleFxP::lit("' + val[:3] + '.' + val[3:] + '"), ', end='')
    //     if i % 4 == 3:
    //         print('')
    const LOOKUP_TABLE : [USampleFxP; 0xF2] = [
        USampleFxP::lit("0x1.000"), USampleFxP::lit("0x1.010"), USampleFxP::lit("0x1.020"), USampleFxP::lit("0x1.030"),
        USampleFxP::lit("0x1.041"), USampleFxP::lit("0x1.051"), USampleFxP::lit("0x1.062"), USampleFxP::lit("0x1.073"),
        USampleFxP::lit("0x1.084"), USampleFxP::lit("0x1.095"), USampleFxP::lit("0x1.0a6"), USampleFxP::lit("0x1.0b7"),
        USampleFxP::lit("0x1.0c9"), USampleFxP::lit("0x1.0db"), USampleFxP::lit("0x1.0ec"), USampleFxP::lit("0x1.0fe"),
        USampleFxP::lit("0x1.111"), USampleFxP::lit("0x1.123"), USampleFxP::lit("0x1.135"), USampleFxP::lit("0x1.148"),
        USampleFxP::lit("0x1.15b"), USampleFxP::lit("0x1.16e"), USampleFxP::lit("0x1.181"), USampleFxP::lit("0x1.194"),
        USampleFxP::lit("0x1.1a7"), USampleFxP::lit("0x1.1bb"), USampleFxP::lit("0x1.1cf"), USampleFxP::lit("0x1.1e2"),
        USampleFxP::lit("0x1.1f7"), USampleFxP::lit("0x1.20b"), USampleFxP::lit("0x1.21f"), USampleFxP::lit("0x1.234"),
        USampleFxP::lit("0x1.249"), USampleFxP::lit("0x1.25e"), USampleFxP::lit("0x1.273"), USampleFxP::lit("0x1.288"),
        USampleFxP::lit("0x1.29e"), USampleFxP::lit("0x1.2b4"), USampleFxP::lit("0x1.2c9"), USampleFxP::lit("0x1.2e0"),
        USampleFxP::lit("0x1.2f6"), USampleFxP::lit("0x1.30d"), USampleFxP::lit("0x1.323"), USampleFxP::lit("0x1.33a"),
        USampleFxP::lit("0x1.352"), USampleFxP::lit("0x1.369"), USampleFxP::lit("0x1.381"), USampleFxP::lit("0x1.399"),
        USampleFxP::lit("0x1.3b1"), USampleFxP::lit("0x1.3c9"), USampleFxP::lit("0x1.3e2"), USampleFxP::lit("0x1.3fb"),
        USampleFxP::lit("0x1.414"), USampleFxP::lit("0x1.42d"), USampleFxP::lit("0x1.446"), USampleFxP::lit("0x1.460"),
        USampleFxP::lit("0x1.47a"), USampleFxP::lit("0x1.495"), USampleFxP::lit("0x1.4af"), USampleFxP::lit("0x1.4ca"),
        USampleFxP::lit("0x1.4e5"), USampleFxP::lit("0x1.501"), USampleFxP::lit("0x1.51d"), USampleFxP::lit("0x1.539"),
        USampleFxP::lit("0x1.555"), USampleFxP::lit("0x1.571"), USampleFxP::lit("0x1.58e"), USampleFxP::lit("0x1.5ac"),
        USampleFxP::lit("0x1.5c9"), USampleFxP::lit("0x1.5e7"), USampleFxP::lit("0x1.605"), USampleFxP::lit("0x1.623"),
        USampleFxP::lit("0x1.642"), USampleFxP::lit("0x1.661"), USampleFxP::lit("0x1.681"), USampleFxP::lit("0x1.6a1"),
        USampleFxP::lit("0x1.6c1"), USampleFxP::lit("0x1.6e1"), USampleFxP::lit("0x1.702"), USampleFxP::lit("0x1.724"),
        USampleFxP::lit("0x1.745"), USampleFxP::lit("0x1.767"), USampleFxP::lit("0x1.78a"), USampleFxP::lit("0x1.7ad"),
        USampleFxP::lit("0x1.7d0"), USampleFxP::lit("0x1.7f4"), USampleFxP::lit("0x1.818"), USampleFxP::lit("0x1.83c"),
        USampleFxP::lit("0x1.861"), USampleFxP::lit("0x1.886"), USampleFxP::lit("0x1.8ac"), USampleFxP::lit("0x1.8d3"),
        USampleFxP::lit("0x1.8f9"), USampleFxP::lit("0x1.920"), USampleFxP::lit("0x1.948"), USampleFxP::lit("0x1.970"),
        USampleFxP::lit("0x1.999"), USampleFxP::lit("0x1.9c2"), USampleFxP::lit("0x1.9ec"), USampleFxP::lit("0x1.a16"),
        USampleFxP::lit("0x1.a41"), USampleFxP::lit("0x1.a6d"), USampleFxP::lit("0x1.a98"), USampleFxP::lit("0x1.ac5"),
        USampleFxP::lit("0x1.af2"), USampleFxP::lit("0x1.b20"), USampleFxP::lit("0x1.b4e"), USampleFxP::lit("0x1.b7d"),
        USampleFxP::lit("0x1.bac"), USampleFxP::lit("0x1.bdd"), USampleFxP::lit("0x1.c0e"), USampleFxP::lit("0x1.c3f"),
        USampleFxP::lit("0x1.c71"), USampleFxP::lit("0x1.ca4"), USampleFxP::lit("0x1.cd8"), USampleFxP::lit("0x1.d0c"),
        USampleFxP::lit("0x1.d41"), USampleFxP::lit("0x1.d77"), USampleFxP::lit("0x1.dae"), USampleFxP::lit("0x1.de5"),
        USampleFxP::lit("0x1.e1e"), USampleFxP::lit("0x1.e57"), USampleFxP::lit("0x1.e91"), USampleFxP::lit("0x1.ecc"),
        USampleFxP::lit("0x1.f07"), USampleFxP::lit("0x1.f44"), USampleFxP::lit("0x1.f81"), USampleFxP::lit("0x1.fc0"),
        USampleFxP::lit("0x2.000"), USampleFxP::lit("0x2.040"), USampleFxP::lit("0x2.082"), USampleFxP::lit("0x2.0c4"),
        USampleFxP::lit("0x2.108"), USampleFxP::lit("0x2.14d"), USampleFxP::lit("0x2.192"), USampleFxP::lit("0x2.1d9"),
        USampleFxP::lit("0x2.222"), USampleFxP::lit("0x2.26b"), USampleFxP::lit("0x2.2b6"), USampleFxP::lit("0x2.302"),
        USampleFxP::lit("0x2.34f"), USampleFxP::lit("0x2.39e"), USampleFxP::lit("0x2.3ee"), USampleFxP::lit("0x2.43f"),
        USampleFxP::lit("0x2.492"), USampleFxP::lit("0x2.4e6"), USampleFxP::lit("0x2.53c"), USampleFxP::lit("0x2.593"),
        USampleFxP::lit("0x2.5ed"), USampleFxP::lit("0x2.647"), USampleFxP::lit("0x2.6a4"), USampleFxP::lit("0x2.702"),
        USampleFxP::lit("0x2.762"), USampleFxP::lit("0x2.7c4"), USampleFxP::lit("0x2.828"), USampleFxP::lit("0x2.88d"),
        USampleFxP::lit("0x2.8f5"), USampleFxP::lit("0x2.95f"), USampleFxP::lit("0x2.9cb"), USampleFxP::lit("0x2.a3a"),
        USampleFxP::lit("0x2.aaa"), USampleFxP::lit("0x2.b1d"), USampleFxP::lit("0x2.b93"), USampleFxP::lit("0x2.c0b"),
        USampleFxP::lit("0x2.c85"), USampleFxP::lit("0x2.d02"), USampleFxP::lit("0x2.d82"), USampleFxP::lit("0x2.e05"),
        USampleFxP::lit("0x2.e8b"), USampleFxP::lit("0x2.f14"), USampleFxP::lit("0x2.fa0"), USampleFxP::lit("0x3.030"),
        USampleFxP::lit("0x3.0c3"), USampleFxP::lit("0x3.159"), USampleFxP::lit("0x3.1f3"), USampleFxP::lit("0x3.291"),
        USampleFxP::lit("0x3.333"), USampleFxP::lit("0x3.3d9"), USampleFxP::lit("0x3.483"), USampleFxP::lit("0x3.531"),
        USampleFxP::lit("0x3.5e5"), USampleFxP::lit("0x3.69d"), USampleFxP::lit("0x3.759"), USampleFxP::lit("0x3.81c"),
        USampleFxP::lit("0x3.8e3"), USampleFxP::lit("0x3.9b0"), USampleFxP::lit("0x3.a83"), USampleFxP::lit("0x3.b5c"),
        USampleFxP::lit("0x3.c3c"), USampleFxP::lit("0x3.d22"), USampleFxP::lit("0x3.e0f"), USampleFxP::lit("0x3.f03"),
        USampleFxP::lit("0x4.000"), USampleFxP::lit("0x4.104"), USampleFxP::lit("0x4.210"), USampleFxP::lit("0x4.325"),
        USampleFxP::lit("0x4.444"), USampleFxP::lit("0x4.56c"), USampleFxP::lit("0x4.69e"), USampleFxP::lit("0x4.7dc"),
        USampleFxP::lit("0x4.924"), USampleFxP::lit("0x4.a79"), USampleFxP::lit("0x4.bda"), USampleFxP::lit("0x4.d48"),
        USampleFxP::lit("0x4.ec4"), USampleFxP::lit("0x5.050"), USampleFxP::lit("0x5.1eb"), USampleFxP::lit("0x5.397"),
        USampleFxP::lit("0x5.555"), USampleFxP::lit("0x5.726"), USampleFxP::lit("0x5.90b"), USampleFxP::lit("0x5.b05"),
        USampleFxP::lit("0x5.d17"), USampleFxP::lit("0x5.f41"), USampleFxP::lit("0x6.186"), USampleFxP::lit("0x6.3e7"),
        USampleFxP::lit("0x6.666"), USampleFxP::lit("0x6.906"), USampleFxP::lit("0x6.bca"), USampleFxP::lit("0x6.eb3"),
        USampleFxP::lit("0x7.1c7"), USampleFxP::lit("0x7.507"), USampleFxP::lit("0x7.878"), USampleFxP::lit("0x7.c1f"),
        USampleFxP::lit("0x8.000"), USampleFxP::lit("0x8.421"), USampleFxP::lit("0x8.888"), USampleFxP::lit("0x8.d3d"),
        USampleFxP::lit("0x9.249"), USampleFxP::lit("0x9.7b4"), USampleFxP::lit("0x9.d89"), USampleFxP::lit("0xa.3d7"),
        USampleFxP::lit("0xa.aaa"), USampleFxP::lit("0xb.216"), USampleFxP::lit("0xb.a2e"), USampleFxP::lit("0xc.30c"),
        USampleFxP::lit("0xc.ccc"), USampleFxP::lit("0xd.794"), USampleFxP::lit("0xe.38e"), USampleFxP::lit("0xf.0f0"),
        USampleFxP::lit("0xf.fff"), USampleFxP::lit("0xf.fff")]; //throw 2x maxs at the end to avoid out-of-bounds on CLIP_MAX
    let index = x_bits >> 8;
    let lookup_val = LOOKUP_TABLE[index as usize];
    let interp = (LOOKUP_TABLE[index as usize + 1] - lookup_val)
        .wide_mul(fixedmath::U8F8::from_bits(x_bits & 0xFF));
    lookup_val + USampleFxP::from_num(interp)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn shape_mod_calculation() {
        let mut rms_error_pos = 0f32;
        let mut rms_error_neg = 0f32;
        for i in 0..1024 {
            let x_wide = fixedmath::U16F16::from_num(i).unwrapped_shr(10);
            let x = clip_shape(ScalarFxP::from_num(x_wide));
            let xf = x.to_num::<f32>();
            let (x_pos_, shift) = fixedmath::one_over_one_plus_highacc(x);
            let x_pos = x_pos_.unwrapped_shr(shift);
            let xf_pos = 1f32 / (1f32 + xf);
            let x_neg = one_over_one_minus_x(x);
            let xf_neg = 1f32 / (1f32 - xf);
            let error_pos = (x_pos.to_num::<f32>() / xf_pos) - 1f32;
            let error_neg = (x_neg.to_num::<f32>() / xf_neg) - 1f32;
            rms_error_pos += error_pos*error_pos;
            rms_error_neg += error_neg*error_neg;
        }
        rms_error_pos = (rms_error_pos / 1024f32).sqrt();
        rms_error_neg = (rms_error_neg / 1024f32).sqrt();
        assert!(rms_error_pos < 0.01f32); //FIXME: Should probably have better thresholds
        assert!(rms_error_neg < 0.01f32);
    }
    #[test]
    fn shape_mod_nopanic() {
        for i in 0..(1<<16) {
            let x = ScalarFxP::from_bits(i as u16);
            let (_pos, _) = fixedmath::one_over_one_plus_highacc(x);
            let _neg = one_over_one_minus_x(x);
        }
    }
}

mod bindings {
    use super::*;

    #[no_mangle]
    pub extern "C" fn janus_osc_u16_new() -> *mut OscFxP {
        Box::into_raw(Box::new(OscFxP::new()))
    }

    #[no_mangle]
    pub extern "C" fn janus_osc_u16_free(p: *mut OscFxP) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_osc_u16_process(
        p: *mut OscFxP,
        samples: u32,
        note: *const u16,
        shape: *const u16,
        sin: *mut *const i16,
        tri: *mut *const i16,
        sq: *mut *const i16,
        saw: *mut *const i16,
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
            let params = OscParamsFxP{ shape: shape_s };
            let out = p.as_mut().unwrap().process(note_s, params);
            *sin = out.sin.as_ptr().cast();
            *tri = out.tri.as_ptr().cast();
            *sq = out.sq.as_ptr().cast();
            *saw = out.saw.as_ptr().cast();
            out.sin.len() as i32
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_osc_f32_new() -> *mut Osc<f32> {
        Box::into_raw(Box::new(Osc::<f32>::new()))
    }

    #[no_mangle]
    pub extern "C" fn janus_osc_f32_free(p: *mut Osc<f32>) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_osc_f32_process(
        p: *mut Osc<f32>,
        samples: u32,
        note: *const f32,
        shape: *const f32,
        sin: *mut *const f32,
        tri: *mut *const f32,
        sq: *mut *const f32,
        saw: *mut *const f32,
        offset: u32
    ) -> i32 {
        if p.is_null()
            || note.is_null()
            || shape.is_null()
            || sin.is_null()
            || tri.is_null()
            || sq.is_null()
            || saw.is_null()
        {
            return -1;
        }
        unsafe {
            let note_s = std::slice::from_raw_parts(
                note.offset(offset as isize), samples as usize);
            let shape_s = std::slice::from_raw_parts(
                shape.offset(offset as isize), samples as usize);
            let params = OscParams::<f32> { shape: shape_s };
            let out = p.as_mut().unwrap().process(note_s, params);
            *sin = out.sin.as_ptr().cast();
            *tri = out.tri.as_ptr().cast();
            *sq = out.sq.as_ptr().cast();
            *saw = out.saw.as_ptr().cast();
            out.sin.len() as i32
        }
    }
}