use super::*;

pub struct Amp<Smp> {
    outbuf : BufferT<Smp>,
}

impl<Smp: Float> Amp<Smp> {
    pub fn new() -> Self {
        Self {
            outbuf: [Smp::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    pub fn process(&mut self,
        signal: &[Smp],
        gain: &[Smp]
    ) -> &[Smp] {
        let numsamples = std::cmp::min(
            std::cmp::min(signal.len(), gain.len()),
            STATIC_BUFFER_SIZE);
        for i in 0..numsamples {
            self.outbuf[i] = signal[i]*gain[i];
        }
        &self.outbuf[0..numsamples]
    }
}

pub struct AmpFxP {
    outbuf : BufferT<SampleFxP>,
}

impl AmpFxP {
    pub fn new() -> Self {
        Self {
            outbuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
        }
    }
    pub fn process(&mut self,
        signal: &[SampleFxP],
        gain: &[SampleFxP],
    ) -> &[SampleFxP] {
        let numsamples = std::cmp::min(
            std::cmp::min(signal.len(), gain.len()),
            STATIC_BUFFER_SIZE);
        for i in 0..numsamples {
            self.outbuf[i] = signal[i].saturating_mul(gain[i]);
        }
        &self.outbuf[0..numsamples]
    }
}

mod bindings {
    use super::*;

    #[no_mangle]
    pub extern "C" fn janus_amp_u16_new() -> *mut AmpFxP {
        Box::into_raw(Box::new(AmpFxP::new()))
    }

    #[no_mangle]
    pub extern "C" fn janus_amp_u16_free(p: *mut AmpFxP) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_amp_u16_process(
        p: *mut AmpFxP,
        samples: u32,
        signal: *const i16,
        gain: *const i16,
        out: *mut *const u16,
        offset: u32
    ) -> i32 {
        if p.is_null()
            || signal.is_null()
            || gain.is_null()
            || out.is_null()
        {
            return -1;
        }
        unsafe {
            let s = std::slice::from_raw_parts(
                signal.offset(offset as isize).cast::<SampleFxP>(), samples as usize);
            let g = std::slice::from_raw_parts(
                gain.offset(offset as isize).cast::<SampleFxP>(), samples as usize);
            let out_slice = p.as_mut().unwrap().process(s, g);
            *out = out_slice.as_ptr().cast();
            out_slice.len() as i32
        }
    }

    
    #[no_mangle]
    pub extern "C" fn janus_amp_f32_new() -> *mut Amp<f32> {
        Box::into_raw(Box::new(Amp::new()))
    }

    #[no_mangle]
    pub extern "C" fn janus_amp_f32_free(p: *mut Amp<f32>) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_amp_f32_process(
        p: *mut Amp<f32>,
        samples: u32,
        signal: *const f32,
        gain: *const f32,
        out: *mut *const f32,
        offset: u32
    ) -> i32 {
        if p.is_null()
            || signal.is_null()
            || gain.is_null()
            || out.is_null()
        {
            return -1;
        }
        unsafe {
            let s = std::slice::from_raw_parts(
                signal.offset(offset as isize), samples as usize);
            let g = std::slice::from_raw_parts(
                gain.offset(offset as isize), samples as usize);
            let out_slice = p.as_mut().unwrap().process(s, g);
            *out = out_slice.as_ptr().cast();
            out_slice.len() as i32
        }
    }
}