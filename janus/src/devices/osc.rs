use super::*;
use fixedmath::apply_scalar_i;

type PhaseFxP = fixedmath::I4F28;

/// A floating point Oscillator providing Sine, Square, Sawtooth, and Triangle outputs.
pub struct Osc<Smp> {
    sinbuf: BufferT<Smp>,
    sqbuf: BufferT<Smp>,
    tribuf: BufferT<Smp>,
    sawbuf: BufferT<Smp>,
    phase: Smp,
}

/// A struct wrapping the various output signals of an [Osc].  All signals
/// are in phase with each other.
pub struct OscOutput<'a, Smp> {
    /// Sine Wave
    pub sin: &'a [Smp],
    /// Square Wave
    pub sq: &'a [Smp],
    /// Triangle Wave
    pub tri: &'a [Smp],
    /// Sawtooth Wave
    pub saw: &'a [Smp],
}

/// A wrapper struct for passing parameters into the floating-point [Osc].
///
/// Currently there is only one parameter, but it is wrapped for API consistency
pub struct OscParams<'a, Smp> {
    pub shape: &'a [Smp],
}

impl<'a, Smp> OscParams<'a, Smp> {
    /// The length of the input parameters.
    ///
    /// As shape is the only parameter, this is the same as `.shape.len()`
    pub fn len(&self) -> usize {
        self.shape.len()
    }
}

impl<Smp: Float> Osc<Smp> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            sinbuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            sqbuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            tribuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            sawbuf: [Smp::zero(); STATIC_BUFFER_SIZE],
            phase: Smp::zero(),
        }
    }
    /// Run the oscillator, using the given note signal and parameters.
    ///
    /// `note` is formatted as a MIDI note number (floating point allowed for bends, etc.)
    ///
    /// Note: The output slices from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
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
                if self.phase < Smp::FRAC_PI_2().neg() {
                    // phase in [-pi, pi/2)
                    // sin(x) = -cos(x+pi/2)
                    // TODO: Use fast approximation?
                    self.sinbuf[i] = (self.phase + Smp::FRAC_PI_2()).cos().neg();
                    // Subtract (1+1) because traits :eyeroll:
                    self.tribuf[i] = frac_2phase_pi.neg() - Smp::TWO;
                } else {
                    // phase in [-pi/2, 0)
                    self.sinbuf[i] = self.phase.sin();
                    //triangle
                    self.tribuf[i] = frac_2phase_pi;
                }
            } else {
                self.sqbuf[i] = Smp::one();
                if self.phase < Smp::FRAC_PI_2() {
                    // phase in [0, pi/2)
                    self.sinbuf[i] = self.phase.sin();
                    self.tribuf[i] = frac_2phase_pi;
                } else {
                    // phase in [pi/2, pi)
                    // sin(x) = cos(x-pi/2)
                    self.sinbuf[i] = (self.phase - Smp::FRAC_PI_2()).cos();
                    self.tribuf[i] = Smp::TWO - frac_2phase_pi;
                }
            }
            let sample_rate = Smp::from(SAMPLE_RATE).unwrap();
            //calculate the next phase
            let phase_per_sample = midi_note_to_frequency(note[i]) * Smp::TAU() / sample_rate;
            let shape_clip = Smp::from(0.9375).unwrap();
            let shp = if shape[i] < shape_clip {
                shape[i]
            } else {
                shape_clip
            };
            let phase_per_smp_adj = if self.phase < Smp::zero() {
                phase_per_sample * (Smp::ONE / (Smp::ONE + shp))
            } else {
                phase_per_sample * (Smp::ONE / (Smp::ONE - shp))
            };
            let old_phase = self.phase;
            self.phase = old_phase + phase_per_smp_adj;
            // make sure we calculate the correct new phase on transitions for assymmetric waves:
            // check if we've crossed from negative to positive phase
            if old_phase < Smp::ZERO && self.phase > Smp::ZERO && shp != Smp::ZERO {
                // need to multiply residual phase i.e. (phase - 0) by (1+k)/(1-k)
                // where k is the shape, so no work required if shape is 0
                self.phase = self.phase * (Smp::ONE + shp) / (Smp::ONE + shp);
            }
            // Check if we've crossed from positive phase back to negative:
            if self.phase >= Smp::PI() {
                // if we're a symmetric wave this is as simple as just subtract 2pi
                if shp == Smp::ZERO {
                    self.phase = self.phase - Smp::TAU();
                } else {
                    // if assymmetric we have to multiply residual phase i.e. phase - pi
                    // by (1-k)/(1+k) where k is the shape:
                    let delta = (self.phase - Smp::PI()) * (Smp::ONE - shp) / (Smp::ONE + shp);
                    // add new change in phase to our baseline, -pi:
                    self.phase = delta - Smp::PI();
                }
            }
        }
        OscOutput {
            sin: &self.sinbuf[0..numsamples],
            tri: &self.tribuf[0..numsamples],
            sq: &self.sqbuf[0..numsamples],
            saw: &self.sawbuf[0..numsamples],
        }
    }
}

impl<Smp: Float> Default for Osc<Smp> {
    fn default() -> Self {
        Self::new()
    }
}

/// A struct wrapping the output of a fixed point oscillator (see [OscFxP])
///
/// All signals are in phase with each other.
pub struct OscOutputFxP<'a> {
    /// Sine wave output
    pub sin: &'a [SampleFxP],
    /// Square wave output
    pub sq: &'a [SampleFxP],
    /// Triangle wave output
    pub tri: &'a [SampleFxP],
    /// Sawtooth wave output
    pub saw: &'a [SampleFxP],
}

/// A wrapper struct for passing parameters to an [OscFxP].  Currently there
/// is only one parameter, but it is wrapped for API consistency.
pub struct OscParamsFxP<'a> {
    pub shape: &'a [ScalarFxP],
}

impl<'a> OscParamsFxP<'a> {
    pub fn len(&self) -> usize {
        self.shape.len()
    }
}

/// A fixed-point oscillator providing sine, square, triangle, and sawtooth waves.
pub struct OscFxP {
    sinbuf: BufferT<SampleFxP>,
    sqbuf: BufferT<SampleFxP>,
    tribuf: BufferT<SampleFxP>,
    sawbuf: BufferT<SampleFxP>,
    phase: PhaseFxP,
}

impl OscFxP {
    /// Constructor
    pub fn new() -> OscFxP {
        OscFxP {
            sinbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            sqbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            tribuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            sawbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            phase: PhaseFxP::ZERO,
        }
    }
    /// Generate waves based on the `note` control signal and parameters.
    ///
    /// See the definition of [NoteFxP] for further information.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(&mut self, note: &[NoteFxP], params: OscParamsFxP) -> OscOutputFxP {
        let shape = params.shape;
        let input_len = std::cmp::min(note.len(), shape.len());
        let numsamples = std::cmp::min(input_len, STATIC_BUFFER_SIZE);
        const FRAC_2_PI: ScalarFxP = ScalarFxP::lit("0x0.a2fa");
        for i in 0..numsamples {
            //generate waveforms (piecewise defined)
            let frac_2phase_pi =
                apply_scalar_i(SampleFxP::from_num(self.phase), FRAC_2_PI);
            //Sawtooth wave does not have to be piecewise-defined
            self.sawbuf[i] = frac_2phase_pi.unwrapped_shr(1);
            //All other functions are piecewise-defined:
            if self.phase < 0 {
                self.sqbuf[i] = SampleFxP::NEG_ONE;
                if self.phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {
                    // phase in [-pi, pi/2)
                    // Use the identity sin(x) = -cos(x+pi/2) since our taylor series
                    // approximations are centered about zero and this will be more accurate
                    self.sinbuf[i] =
                        fixedmath::cos_fixed(SampleFxP::from_num(self.phase + PhaseFxP::FRAC_PI_2))
                            .unwrapped_neg();
                    self.tribuf[i] = frac_2phase_pi.unwrapped_neg() - SampleFxP::lit("2");
                } else {
                    // phase in [-pi/2, 0)
                    self.sinbuf[i] = fixedmath::sin_fixed(SampleFxP::from_num(self.phase));
                    self.tribuf[i] = frac_2phase_pi;
                }
            } else {
                self.sqbuf[i] = SampleFxP::ONE;
                if self.phase < PhaseFxP::FRAC_PI_2 {
                    // phase in [0, pi/2)
                    self.sinbuf[i] = fixedmath::sin_fixed(SampleFxP::from_num(self.phase));
                    self.tribuf[i] = frac_2phase_pi;
                } else {
                    // phase in [pi/2, pi)
                    // sin(x) = cos(x-pi/2)
                    self.sinbuf[i] =
                        fixedmath::cos_fixed(SampleFxP::from_num(self.phase - PhaseFxP::FRAC_PI_2));
                    self.tribuf[i] = SampleFxP::lit("2") - frac_2phase_pi;
                }
            }
            // we need to divide by 2^12 here, but we're increasing the fractional part by 10
            // bits so we'll only actually shift by 2 places and then use a bitcast for the
            // remaining logical 10 bits:
            let phase_per_sample = fixedmath::U4F28::from_bits(
                fixedmath::scale_fixedfloat(
                    fixedmath::midi_note_to_frequency(note[i]),
                    FRAC_4096_2PI_SR,
                ).unwrapped_shr(2).to_bits()
            );
            //let phase_per_sample = fixedmath::U4F28::from_num(
            //    fixedmath::midi_note_to_frequency(note[i])
            //        .wide_mul(FRAC_4096_2PI_SR)
            //        .unwrapped_shr(12),
            //);
            let phase_per_smp_adj = PhaseFxP::from_num(if self.phase < PhaseFxP::ZERO {
                let (x, s) = fixedmath::one_over_one_plus_highacc(clip_shape(shape[i]));
                fixedmath::scale_fixedfloat(phase_per_sample, x).unwrapped_shr(s)
            } else {
                fixedmath::scale_fixedfloat(phase_per_sample, one_over_one_minus_x(shape[i]))
            });
            let old_phase = self.phase;
            self.phase += phase_per_smp_adj;
            // check if we've crossed from negative to positive phase
            if old_phase < PhaseFxP::ZERO
                && self.phase > PhaseFxP::ZERO
                && shape[i] != ScalarFxP::ZERO
            {
                // need to multiply residual phase i.e. (phase - 0) by (1+k)/(1-k)
                // where k is the shape, so no work required if shape is 0
                let scaled = fixedmath::scale_fixedfloat(
                    fixedmath::U4F28::from_num(self.phase),
                    one_over_one_minus_x(shape[i]),
                );
                let one_plus_shape =
                    fixedmath::U1F15::from_num(clip_shape(shape[i])) + fixedmath::U1F15::ONE;
                self.phase =
                    PhaseFxP::from_num(fixedmath::scale_fixedfloat(scaled, one_plus_shape));
            }
            // Check if we've crossed from positive phase back to negative:
            if self.phase >= PhaseFxP::PI {
                // if we're a symmetric wave this is as simple as just subtract 2pi
                if shape[i] == ScalarFxP::ZERO {
                    self.phase -= PhaseFxP::TAU;
                } else {
                    // if assymmetric we have to multiply residual phase i.e. phase - pi
                    // by (1-k)/(1+k) where k is the shape:
                    let one_minus_shape =
                        (ScalarFxP::MAX - clip_shape(shape[i])) + ScalarFxP::DELTA;
                    // scaled = residual_phase * (1-k)
                    let scaled = fixedmath::scale_fixedfloat(
                        fixedmath::U4F28::from_num(self.phase - PhaseFxP::PI),
                        one_minus_shape,
                    );
                    // new change in phase = scaled * 1/(1 + k)
                    let (x, s) = fixedmath::one_over_one_plus_highacc(clip_shape(shape[i]));
                    let delta = fixedmath::scale_fixedfloat(scaled, x).unwrapped_shr(s);
                    // add new change in phase to our baseline, -pi:
                    self.phase = PhaseFxP::from_num(delta) - PhaseFxP::PI;
                }
            }
        }
        OscOutputFxP {
            sin: &self.sinbuf[0..numsamples],
            tri: &self.tribuf[0..numsamples],
            sq: &self.sqbuf[0..numsamples],
            saw: &self.sawbuf[0..numsamples],
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
    } else {
        x
    }
}

fn one_over_one_minus_x(x: ScalarFxP) -> USampleFxP {
    let x_bits = clip_shape(x).to_bits();
    // Provide a local function alias for brevity:
    const fn lit(x: &str) -> USampleFxP {
        USampleFxP::lit(x)
    }
    // Table generated with python:
    //
    // table = [1/(1-(x/256.0)) for x in range(0,256)][:0xF1]
    // shifted = [int(x*256*16) for x in table]
    // shifted[-1] = shifted[-1] - 1 # Prevent overflow
    // hexvals = [hex(x) for x in shifted]
    // for i in range(len(hexvals)):
    //     val = hexvals[i]
    //     print('lit("' + val[:3] + '.' + val[3:] + '"), ', end='')
    //     if i % 4 == 3:
    //         print('')
    #[rustfmt::skip]
    const LOOKUP_TABLE: [USampleFxP; 0xF2] = [
        lit("0x1.000"), lit("0x1.010"), lit("0x1.020"), lit("0x1.030"),
        lit("0x1.041"), lit("0x1.051"), lit("0x1.062"), lit("0x1.073"),
        lit("0x1.084"), lit("0x1.095"), lit("0x1.0a6"), lit("0x1.0b7"),
        lit("0x1.0c9"), lit("0x1.0db"), lit("0x1.0ec"), lit("0x1.0fe"),
        lit("0x1.111"), lit("0x1.123"), lit("0x1.135"), lit("0x1.148"),
        lit("0x1.15b"), lit("0x1.16e"), lit("0x1.181"), lit("0x1.194"),
        lit("0x1.1a7"), lit("0x1.1bb"), lit("0x1.1cf"), lit("0x1.1e2"),
        lit("0x1.1f7"), lit("0x1.20b"), lit("0x1.21f"), lit("0x1.234"),
        lit("0x1.249"), lit("0x1.25e"), lit("0x1.273"), lit("0x1.288"),
        lit("0x1.29e"), lit("0x1.2b4"), lit("0x1.2c9"), lit("0x1.2e0"),
        lit("0x1.2f6"), lit("0x1.30d"), lit("0x1.323"), lit("0x1.33a"),
        lit("0x1.352"), lit("0x1.369"), lit("0x1.381"), lit("0x1.399"),
        lit("0x1.3b1"), lit("0x1.3c9"), lit("0x1.3e2"), lit("0x1.3fb"),
        lit("0x1.414"), lit("0x1.42d"), lit("0x1.446"), lit("0x1.460"),
        lit("0x1.47a"), lit("0x1.495"), lit("0x1.4af"), lit("0x1.4ca"),
        lit("0x1.4e5"), lit("0x1.501"), lit("0x1.51d"), lit("0x1.539"),
        lit("0x1.555"), lit("0x1.571"), lit("0x1.58e"), lit("0x1.5ac"),
        lit("0x1.5c9"), lit("0x1.5e7"), lit("0x1.605"), lit("0x1.623"),
        lit("0x1.642"), lit("0x1.661"), lit("0x1.681"), lit("0x1.6a1"),
        lit("0x1.6c1"), lit("0x1.6e1"), lit("0x1.702"), lit("0x1.724"),
        lit("0x1.745"), lit("0x1.767"), lit("0x1.78a"), lit("0x1.7ad"),
        lit("0x1.7d0"), lit("0x1.7f4"), lit("0x1.818"), lit("0x1.83c"),
        lit("0x1.861"), lit("0x1.886"), lit("0x1.8ac"), lit("0x1.8d3"),
        lit("0x1.8f9"), lit("0x1.920"), lit("0x1.948"), lit("0x1.970"),
        lit("0x1.999"), lit("0x1.9c2"), lit("0x1.9ec"), lit("0x1.a16"),
        lit("0x1.a41"), lit("0x1.a6d"), lit("0x1.a98"), lit("0x1.ac5"),
        lit("0x1.af2"), lit("0x1.b20"), lit("0x1.b4e"), lit("0x1.b7d"),
        lit("0x1.bac"), lit("0x1.bdd"), lit("0x1.c0e"), lit("0x1.c3f"),
        lit("0x1.c71"), lit("0x1.ca4"), lit("0x1.cd8"), lit("0x1.d0c"),
        lit("0x1.d41"), lit("0x1.d77"), lit("0x1.dae"), lit("0x1.de5"),
        lit("0x1.e1e"), lit("0x1.e57"), lit("0x1.e91"), lit("0x1.ecc"),
        lit("0x1.f07"), lit("0x1.f44"), lit("0x1.f81"), lit("0x1.fc0"),
        lit("0x2.000"), lit("0x2.040"), lit("0x2.082"), lit("0x2.0c4"),
        lit("0x2.108"), lit("0x2.14d"), lit("0x2.192"), lit("0x2.1d9"),
        lit("0x2.222"), lit("0x2.26b"), lit("0x2.2b6"), lit("0x2.302"),
        lit("0x2.34f"), lit("0x2.39e"), lit("0x2.3ee"), lit("0x2.43f"),
        lit("0x2.492"), lit("0x2.4e6"), lit("0x2.53c"), lit("0x2.593"),
        lit("0x2.5ed"), lit("0x2.647"), lit("0x2.6a4"), lit("0x2.702"),
        lit("0x2.762"), lit("0x2.7c4"), lit("0x2.828"), lit("0x2.88d"),
        lit("0x2.8f5"), lit("0x2.95f"), lit("0x2.9cb"), lit("0x2.a3a"),
        lit("0x2.aaa"), lit("0x2.b1d"), lit("0x2.b93"), lit("0x2.c0b"),
        lit("0x2.c85"), lit("0x2.d02"), lit("0x2.d82"), lit("0x2.e05"),
        lit("0x2.e8b"), lit("0x2.f14"), lit("0x2.fa0"), lit("0x3.030"),
        lit("0x3.0c3"), lit("0x3.159"), lit("0x3.1f3"), lit("0x3.291"),
        lit("0x3.333"), lit("0x3.3d9"), lit("0x3.483"), lit("0x3.531"),
        lit("0x3.5e5"), lit("0x3.69d"), lit("0x3.759"), lit("0x3.81c"),
        lit("0x3.8e3"), lit("0x3.9b0"), lit("0x3.a83"), lit("0x3.b5c"),
        lit("0x3.c3c"), lit("0x3.d22"), lit("0x3.e0f"), lit("0x3.f03"),
        lit("0x4.000"), lit("0x4.104"), lit("0x4.210"), lit("0x4.325"),
        lit("0x4.444"), lit("0x4.56c"), lit("0x4.69e"), lit("0x4.7dc"),
        lit("0x4.924"), lit("0x4.a79"), lit("0x4.bda"), lit("0x4.d48"),
        lit("0x4.ec4"), lit("0x5.050"), lit("0x5.1eb"), lit("0x5.397"),
        lit("0x5.555"), lit("0x5.726"), lit("0x5.90b"), lit("0x5.b05"),
        lit("0x5.d17"), lit("0x5.f41"), lit("0x6.186"), lit("0x6.3e7"),
        lit("0x6.666"), lit("0x6.906"), lit("0x6.bca"), lit("0x6.eb3"),
        lit("0x7.1c7"), lit("0x7.507"), lit("0x7.878"), lit("0x7.c1f"),
        lit("0x8.000"), lit("0x8.421"), lit("0x8.888"), lit("0x8.d3d"),
        lit("0x9.249"), lit("0x9.7b4"), lit("0x9.d89"), lit("0xa.3d7"),
        lit("0xa.aaa"), lit("0xb.216"), lit("0xb.a2e"), lit("0xc.30c"),
        lit("0xc.ccc"), lit("0xd.794"), lit("0xe.38e"), lit("0xf.0f0"),
        lit("0xf.fff"), lit("0xf.fff") //throw 2x maxs at the end to avoid out-of-bounds on CLIP_MAX
    ];
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
            rms_error_pos += error_pos * error_pos;
            rms_error_neg += error_neg * error_neg;
        }
        rms_error_pos = (rms_error_pos / 1024f32).sqrt();
        rms_error_neg = (rms_error_neg / 1024f32).sqrt();
        assert!(rms_error_pos < 0.01f32); //FIXME: Should probably have better thresholds
        assert!(rms_error_neg < 0.01f32);
    }
    #[test]
    fn shape_mod_nopanic() {
        for i in 0..(1 << 16) {
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
        offset: u32,
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
                note.offset(offset as isize).cast::<NoteFxP>(),
                samples as usize,
            );
            let shape_s = std::slice::from_raw_parts(
                shape.offset(offset as isize).cast::<fixedmath::U0F16>(),
                samples as usize,
            );
            let params = OscParamsFxP { shape: shape_s };
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
        offset: u32,
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
            let note_s = std::slice::from_raw_parts(note.offset(offset as isize), samples as usize);
            let shape_s =
                std::slice::from_raw_parts(shape.offset(offset as isize), samples as usize);
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
