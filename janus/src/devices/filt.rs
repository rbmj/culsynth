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
    low_z: Smp,
    band_z: Smp,    
}

impl<Smp: Float> Filt<Smp> {
    pub fn create() -> Self {
        Self {
            low: [Smp::ZERO; STATIC_BUFFER_SIZE],
            band: [Smp::ZERO; STATIC_BUFFER_SIZE],
            high: [Smp::ZERO; STATIC_BUFFER_SIZE],
            
            low_z: Smp::ZERO,
            band_z: Smp::ZERO,
        }
    }
    
    fn prewarped_gain(f: Smp) -> Smp {
        let f_c = midi_note_to_frequency(f);
        Smp::tan(Smp::PI()*f_c / Smp::from(SAMPLE_RATE).unwrap())
    }
    pub fn process(&mut self, input: &[Smp], cutoff: &[Smp], resonance: &[Smp]) -> FiltOutput<Smp> {
        let numsamples = std::cmp::min(
            std::cmp::min(input.len(), cutoff.len()),
            std::cmp::min(resonance.len(), STATIC_BUFFER_SIZE));
        for i in 0..numsamples {
            let res = Smp::ONE - if resonance[i] < Smp::RES_MAX { resonance[i] } else { Smp::RES_MAX };
            let gain = Self::prewarped_gain(cutoff[i]);
            let denom = gain*gain + Smp::TWO*res*gain + Smp::ONE;
            self.high[i] = (input[i] - (Smp::TWO*res + gain)*self.band_z - self.low_z) /
                    denom;
            let band_gain = gain*self.high[i];
            self.band[i] = band_gain + self.band_z;
            self.band_z = self.band[i] + band_gain;

            let low_gain = gain*self.band[i];
            self.low[i] = low_gain + self.low_z;
            self.low_z = self.low[i] + low_gain;
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
    low_z: fixedmath::I12F20,
    band_z: fixedmath::I12F20
}

impl FiltFxP {
    const RES_MAX : ScalarFxP = ScalarFxP::lit("0x0.F000");
    //TODO: Pull this and the one in EnvFxP into fixedmath with generic fractional bits...
    fn one_over(x: fixedmath::U3F29) -> (fixedmath::U1F15, u32) {
        //x must be at least 1, or this will panic at the return statement (2 - shift < 0)
        let mut shift = x.leading_zeros() as i32;
        let mut x_shifted = fixedmath::U1F31::from_bits(x.to_bits())
            .unwrapped_shl(shift as u32);
        if x_shifted >= fixedmath::U1F31::SQRT_2 {
            shift -= 1;
            x_shifted = x_shifted.unwrapped_shr(1);
        }
        let x_shifted_trunc = fixedmath::U1F15::from_num(x_shifted);
        let x2 = fixedmath::I3F29::from_num(x_shifted_trunc.wide_mul(x_shifted_trunc));
        let one_minus_x = fixedmath::I3F29::ONE - fixedmath::I3F29::from_num(x_shifted);
        let result = x2 + one_minus_x + one_minus_x.unwrapped_shl(1);
        (fixedmath::U1F15::from_num(result), (2 - shift) as u32)
    }
    pub fn create() -> Self {
        Self {
            low: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            band: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            high: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            low_z: fixedmath::I12F20::ZERO,
            band_z: fixedmath::I12F20::ZERO
        }
    }
    fn prewarped_gain(n: NoteFxP) -> fixedmath::U1F15 {
        let f_c = fixedmath::U14F2::from_num(
            fixedmath::midi_note_to_frequency(n));
        let omega_d =
            ScalarFxP::from_num(
                f_c.wide_mul(ScalarFxP::from_num(FRAC_4096_2PI_SR))
            .unwrapped_shr(13));
        fixedmath::tan_fixed(omega_d)
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
            let res = ScalarFxP::MAX - std::cmp::min(resonance[i], Self::RES_MAX);
            // include type annotations to make the fixed point logic more explicit
            let gain : fixedmath::U1F15 = Self::prewarped_gain(cutoff[i]);
            let gain2 = fixedmath::U3F29::from_num(gain.wide_mul(gain));
            // resonance * gain is a U1F31, so this will only lose the least significant bit
            // and provides space for the shift left below (should be optimized out)
            let gain_r = fixedmath::U3F29::from_num(res.wide_mul(gain));
            let denom = gain2 + gain_r.unwrapped_shl(1) + fixedmath::U3F29::ONE;
            let (denom_inv, shift) = Self::one_over(denom);

            let gain_plus_2r = fixedmath::U3F29::from_num(res).unwrapped_shl(1) +
                fixedmath::U3F29::from_num(gain);
            let band_high_feedback: fixedmath::I7F25 = fixedmath::U3F13::from_num(gain_plus_2r)
                .wide_mul_signed(SampleFxP::saturating_from_num(self.band_z));
            let high_num = SampleFxP::saturating_from_num(
                fixedmath::I12F20::from_num(input[i])
                - fixedmath::I12F20::from_num(band_high_feedback)
                - self.low_z);
            let high_unshifted : fixedmath::I5F27 = high_num.wide_mul_unsigned(denom_inv);
            self.high[i] = SampleFxP::saturating_from_num(high_unshifted.unwrapped_shr(shift));

            let band_gain = fixedmath::I12F20::from_num(gain.wide_mul_signed(self.high[i]));
            let band = band_gain + self.band_z;
            self.band[i] = SampleFxP::saturating_from_num(band_gain + self.band_z);
            self.band_z = band + band_gain;

            let low_gain = fixedmath::I12F20::from_num(gain.wide_mul_signed(self.band[i]));
            let low = low_gain + self.low_z;
            self.low[i] = SampleFxP::saturating_from_num(low);
            self.low_z = low + low_gain;
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