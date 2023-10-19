use super::*;


pub struct FiltOutput<'a, Smp> {
    low: &'a [Smp],
    band: &'a [Smp],
    high: &'a [Smp]
}

pub struct Filt<Smp> {
    low : BufferT<Smp>,
    band : BufferT<Smp>,
    high : BufferT<Smp>,
    input_z: (Smp, Smp),
    low_z: (Smp, Smp),
    band_z: (Smp, Smp),
    high_z: (Smp, Smp),
}

impl<Smp: Float> Filt<Smp> {
    pub fn create() -> Self {
        Self {
            low: [Smp::ZERO; STATIC_BUFFER_SIZE],
            band: [Smp::ZERO; STATIC_BUFFER_SIZE],
            high: [Smp::ZERO; STATIC_BUFFER_SIZE],
            input_z: (Smp::ZERO, Smp::ZERO),
            low_z: (Smp::ZERO, Smp::ZERO),
            band_z: (Smp::ZERO, Smp::ZERO),
            high_z: (Smp::ZERO, Smp::ZERO)
        }
    }
    pub fn process(&mut self, input: &[Smp], cutoff: &[Smp], resonance: &[Smp]) -> FiltOutput<Smp> {
        let numsamples = std::cmp::min(
            std::cmp::min(input.len(), cutoff.len()),
            std::cmp::min(resonance.len(), STATIC_BUFFER_SIZE));
        for i in 0..numsamples {
            // omega_d = omega_c*Ts/2 = 2pi*f_c / (2*SR) = f_c*2pi*2^12 / (SR*2^13)
            let f_c = midi_note_to_frequency(cutoff[i]);
            let omega_d = Smp::PI()*f_c / Smp::from(SAMPLE_RATE).unwrap();
            let omega_p = Smp::tan(omega_d);
            let omega_p_2 = omega_p * omega_p;
            let omega_p_r = omega_p * resonance[i];
            let omega_p_2_plus1 = omega_p_2 + Smp::ONE;
            let quad_term = omega_p_2_plus1 - (Smp::TWO * omega_p_r);
            // linear term is 2*omega_p^2 - 2 = 2 * (omega_p^2 - 1)
            let linear_term = Smp::TWO * (omega_p_2 - Smp::ONE);
            let denom = omega_p_2_plus1 + (Smp::TWO * omega_p_r);
            let low_feedback = (self.low_z.1 * quad_term) + (self.low_z.0 * linear_term);
            let band_feedback = (self.band_z.1 * quad_term) + (self.band_z.0 * linear_term);
            let high_feedback = (self.high_z.1 * quad_term) + (self.high_z.0 * linear_term);
            let low_control = (self.input_z.1 + (self.input_z.0 * Smp::TWO) + input[i])*omega_p_2;
            let band_control = (input[i] - self.input_z.1)*omega_p;
            let high_control = self.input_z.1 - (self.input_z.0 * Smp::TWO) + input[i];
            self.low[i] = (low_control - low_feedback) / denom;
            self.band[i] = (band_control - band_feedback) / denom;
            self.high[i] = (high_control - high_feedback) / denom;
            //rotate all the new values into the output arrays and rotate the state arrays
            self.low_z.1 = self.low_z.0;
            self.band_z.1 = self.band_z.0;
            self.high_z.1 = self.high_z.0;
            self.low_z.0 = self.low[i];
            self.band_z.0 = self.band[i];
            self.high_z.0 = self.high[i];
        }
        FiltOutput {
            low: &self.low[0..numsamples],
            band: &self.band[0..numsamples],
            high: &self.high[0..numsamples]
        }
    }
}


pub struct FiltOutputFxP<'a> {
    low: &'a [SampleFxP],
    band: &'a [SampleFxP],
    high: &'a [SampleFxP]
}

pub struct FiltFxP {
    low : BufferT<SampleFxP>,
    band : BufferT<SampleFxP>,
    high : BufferT<SampleFxP>,
    input_z: (SampleFxP, SampleFxP),
    low_z: (SampleFxP, SampleFxP),
    band_z: (SampleFxP, SampleFxP),
    high_z: (SampleFxP, SampleFxP),
}

impl FiltFxP {
    //TODO: Pull this and the one in EnvFxP into fixedmath with generic fractional bits...
    fn one_over(x: fixedmath::U3F29) -> (fixedmath::U1F15, u32) {
        let mut shift = x.leading_zeros();
        let mut x_shifted = fixedmath::U1F31::from_bits(x.to_bits()).unwrapped_shl(shift);
        if x_shifted >= fixedmath::U1F31::SQRT_2 && shift > 0 { //FIXME
            shift -= 1;
            x_shifted = x_shifted.unwrapped_shr(1);
        }
        let x_shifted_trunc = fixedmath::U1F15::from_num(x_shifted);
        let x2 = fixedmath::I3F29::from_num(x_shifted_trunc.wide_mul(x_shifted_trunc));
        let one_minus_x = fixedmath::I3F29::ONE - fixedmath::I3F29::from_num(x_shifted);
        let result = x2 + one_minus_x + one_minus_x.unwrapped_shl(1);
        (fixedmath::U1F15::from_num(result), 2 - shift)
    }
    pub fn create() -> Self {
        Self {
            low: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            band: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            high: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            input_z: (SampleFxP::ZERO, SampleFxP::ZERO),
            low_z: (SampleFxP::ZERO, SampleFxP::ZERO),
            band_z: (SampleFxP::ZERO, SampleFxP::ZERO),
            high_z: (SampleFxP::ZERO, SampleFxP::ZERO)
        }
    }
    pub fn process(&mut self,
        input: &[SampleFxP],
        cutoff: &[NoteFxP],
        resonance: &[ScalarFxP]
    ) -> FiltOutputFxP {
        let numsamples = std::cmp::min(
            std::cmp::min(input.len(), cutoff.len()),
            std::cmp::min(resonance.len(), STATIC_BUFFER_SIZE));
        for i in 0..numsamples {
            // omega_d = omega_c*Ts/2 = 2pi*f_c / (2*SR) = f_c*2pi*2^12 / (SR*2^13)
            let f_c = fixedmath::U14F2::from_num(
                    fixedmath::midi_note_to_frequency(cutoff[i]));
            let omega_d = ScalarFxP::from_num(f_c.wide_mul(ScalarFxP::from_num(FRAC_4096_2PI_SR))
                .unwrapped_shr(13));
            let omega_p = fixedmath::tan_fixed(omega_d);
            let omega_p_2 = omega_p.wide_mul(omega_p);
            let omega_p_r = omega_p.wide_mul(resonance[i]);
            let omega_p_2_plus1 = omega_p_2 + fixedmath::U2F30::ONE;
            let x = omega_p_2_plus1 - fixedmath::U2F30::from_num(omega_p_r);
            let quad_term = if x < omega_p_r { // may be possible with some variety of truncation
                fixedmath::U2F30::ZERO
            }
            else {
                x - fixedmath::U2F30::from_num(omega_p_r)
            };
            // linear term is 2*omega_p^2 - 2 = 2 * (-1 + omega_p^2)
            let linear_term = fixedmath::I2F30::NEG_ONE.add_unsigned(omega_p_2).unwrapped_shl(1);
            let denom = fixedmath::U3F29::from_num(omega_p_2_plus1) +
                fixedmath::U3F29::from_num(omega_p_r).unwrapped_shl(1);
            let (scalar, shift) = Self::one_over(denom);
            //TODO: Are all these demotions strictly necessary, or can we prove that some are not required?
            let low_feedback = 
                fixedmath::I7F25::from_num(self.low_z.1.wide_mul_unsigned(fixedmath::U2F14::from_num(quad_term))) +
                fixedmath::I7F25::from_num(self.low_z.0.wide_mul(fixedmath::I2F14::from_num(linear_term)));
            let band_feedback = 
                fixedmath::I7F25::from_num(self.band_z.1.wide_mul_unsigned(fixedmath::U2F14::from_num(quad_term))) +
                fixedmath::I7F25::from_num(self.band_z.0.wide_mul(fixedmath::I2F14::from_num(linear_term)));
            let high_feedback = 
                fixedmath::I7F25::from_num(self.high_z.1.wide_mul_unsigned(fixedmath::U2F14::from_num(quad_term))) +
                fixedmath::I7F25::from_num(self.high_z.0.wide_mul(fixedmath::I2F14::from_num(linear_term)));
            let low_control_sum = 
                fixedmath::I6F26::from_num(self.input_z.1) + 
                fixedmath::I6F26::from_num(self.input_z.0).unwrapped_shl(1) +
                fixedmath::I6F26::from_num(input[i]);
            let low_control = fixedmath::I6F10::from_num(low_control_sum)
                .wide_mul_unsigned(fixedmath::U2F14::from_num(omega_p_2));
            let band_control_sum =
                fixedmath::I6F26::from_num(input[i]) -
                fixedmath::I6F26::from_num(self.input_z.1);
            let band_control = fixedmath::I6F10::from_num(band_control_sum)
                .wide_mul_unsigned(fixedmath::U2F14::from_num(omega_p));

            let high_control =
                    fixedmath::I6F26::from_num(self.input_z.1) -
                    fixedmath::I6F26::from_num(self.input_z.0).unwrapped_shl(1) +
                    fixedmath::I6F26::from_num(input[i]);
            let low = fixedmath::I7F9::from_num(fixedmath::I7F25::from_num(low_control) - low_feedback)
                .wide_mul_unsigned(scalar).unwrapped_shr(shift);
            let band = fixedmath::I7F9::from_num(fixedmath::I7F25::from_num(band_control) - band_feedback)
                .wide_mul_unsigned(scalar).unwrapped_shr(shift);
            let high = fixedmath::I7F9::from_num(fixedmath::I7F25::from_num(high_control) - high_feedback)
                .wide_mul_unsigned(scalar).unwrapped_shr(shift);
            //rotate all the new values into the output arrays and rotate the state arrays
            self.low[i] = SampleFxP::from_num(low);
            self.band[i] = SampleFxP::from_num(band);
            self.high[i] = SampleFxP::ZERO; //SampleFxP::from_num(high);
            self.low_z.1 = self.low_z.0;
            self.band_z.1 = self.band_z.0;
            self.high_z.1 = self.high_z.0;
            self.low_z.0 = self.low[i];
            self.band_z.0 = self.band[i];
            self.high_z.0 = self.high[i];
        }
        FiltOutputFxP {
            low: &self.low[0..numsamples],
            band: &self.band[0..numsamples],
            high: &self.high[0..numsamples]
        }
    }
}

mod bindings {
    use super::*;

    #[no_mangle]
    pub extern "C" fn janus_filt_u16_new() -> *mut FiltFxP {
        Box::into_raw(Box::new(FiltFxP::create()))
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_u16_free(p: *mut FiltFxP) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_u16_process(
        p: *mut FiltFxP,
        samples: u32,
        input: *const i16,
        cutoff: *const u16,
        resonance: *const u16,
        low: *mut *const i16,
        band: *mut *const i16,
        high: *mut *const i16,
        offset: u32
    ) -> i32 {
        if p.is_null()
            || input.is_null()
            || cutoff.is_null()
            || resonance.is_null()
            || low.is_null()
            || band.is_null()
            || high.is_null()
        {
            return -1;
        }
        unsafe {
            let i = std::slice::from_raw_parts(
                input.offset(offset as isize).cast::<SampleFxP>(), samples as usize);
            let c = std::slice::from_raw_parts(
                cutoff.offset(offset as isize).cast::<NoteFxP>(), samples as usize);
            let r = std::slice::from_raw_parts(
                resonance.offset(offset as isize).cast::<ScalarFxP>(), samples as usize);
            let out = p.as_mut().unwrap().process(i, c, r);
            *low = out.low.as_ptr().cast();
            *band = out.band.as_ptr().cast();
            *high = out.high.as_ptr().cast();
            out.low.len() as i32
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_f32_new() -> *mut Filt<f32> {
        Box::into_raw(Box::new(Filt::create()))
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_f32_free(p: *mut Filt<f32>) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_f32_process(
        p: *mut Filt<f32>,
        samples: u32,
        input: *const f32,
        cutoff: *const f32,
        resonance: *const f32,
        low: *mut *const f32,
        band: *mut *const f32,
        high: *mut *const f32,
        offset: u32
    ) -> i32 {
        if p.is_null()
            || input.is_null()
            || cutoff.is_null()
            || resonance.is_null()
            || low.is_null()
            || band.is_null()
            || high.is_null()
        {
            return -1;
        }
        unsafe {
            let i = std::slice::from_raw_parts(
                input.offset(offset as isize), samples as usize);
            let c = std::slice::from_raw_parts(
                cutoff.offset(offset as isize), samples as usize);
            let r = std::slice::from_raw_parts(
                resonance.offset(offset as isize), samples as usize);
            let out = p.as_mut().unwrap().process(i, c, r);
            *low = out.low.as_ptr().cast();
            *band = out.band.as_ptr().cast();
            *high = out.high.as_ptr().cast();
            out.low.len() as i32
        }
    }
}