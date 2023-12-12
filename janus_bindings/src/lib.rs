use janus::context::{Context, ContextFxP};
use janus::devices::*;
use janus::{EnvParamFxP, NoteFxP, SampleFxP, ScalarFxP, SignedNoteFxP};

#[no_mangle]
pub extern "C" fn janus_amp_u16_new() -> *mut AmpFxP {
    Box::into_raw(Box::new(AmpFxP::new()))
}

#[no_mangle]
pub unsafe extern "C" fn janus_amp_u16_free(p: *mut AmpFxP) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn janus_amp_u16_process(
    p: *mut AmpFxP,
    samples: u32,
    signal: *const i16,
    gain: *const i16,
    out: *mut *const u16,
    offset: u32,
) -> i32 {
    if p.is_null() || signal.is_null() || gain.is_null() || out.is_null() {
        return -1;
    }
    let s = std::slice::from_raw_parts(
        signal.offset(offset as isize).cast::<SampleFxP>(),
        samples as usize,
    );
    let g = std::slice::from_raw_parts(
        gain.offset(offset as isize).cast::<SampleFxP>(),
        samples as usize,
    );
    let out_slice = (*p).process(s, g);
    *out = out_slice.as_ptr().cast();
    out_slice.len() as i32
}

#[no_mangle]
pub extern "C" fn janus_amp_f32_new() -> *mut Amp<f32> {
    Box::into_raw(Box::new(Amp::new()))
}

#[no_mangle]
pub unsafe extern "C" fn janus_amp_f32_free(p: *mut Amp<f32>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn janus_amp_f32_process(
    p: *mut Amp<f32>,
    samples: u32,
    signal: *const f32,
    gain: *const f32,
    out: *mut *const f32,
    offset: u32,
) -> i32 {
    if p.is_null() || signal.is_null() || gain.is_null() || out.is_null() {
        return -1;
    }
    let s = std::slice::from_raw_parts(signal.offset(offset as isize), samples as usize);
    let g = std::slice::from_raw_parts(gain.offset(offset as isize), samples as usize);
    let out_slice = (*p).process(s, g);
    *out = out_slice.as_ptr().cast();
    out_slice.len() as i32
}

#[no_mangle]
pub extern "C" fn janus_env_u16_new() -> *mut EnvFxP {
    Box::into_raw(Box::new(EnvFxP::new()))
}

#[no_mangle]
pub unsafe extern "C" fn janus_env_u16_free(p: *mut EnvFxP) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn janus_env_u16_process(
    p: *mut EnvFxP,
    samples: u32,
    gate: *const i16,
    attack: *const u16,
    decay: *const u16,
    sustain: *const u16,
    release: *const u16,
    signal: *mut *const u16,
    offset: u32,
) -> i32 {
    if p.is_null()
        || gate.is_null()
        || attack.is_null()
        || decay.is_null()
        || sustain.is_null()
        || release.is_null()
        || signal.is_null()
    {
        return -1;
    }
    let g = std::slice::from_raw_parts(
        gate.offset(offset as isize).cast::<SampleFxP>(),
        samples as usize,
    );
    let a = std::slice::from_raw_parts(
        attack.offset(offset as isize).cast::<EnvParamFxP>(),
        samples as usize,
    );
    let d = std::slice::from_raw_parts(
        decay.offset(offset as isize).cast::<EnvParamFxP>(),
        samples as usize,
    );
    let s = std::slice::from_raw_parts(
        sustain.offset(offset as isize).cast::<ScalarFxP>(),
        samples as usize,
    );
    let r = std::slice::from_raw_parts(
        release.offset(offset as isize).cast::<EnvParamFxP>(),
        samples as usize,
    );
    let params = EnvParamsFxP {
        attack: a,
        decay: d,
        sustain: s,
        release: r,
    };
    let ctx = ContextFxP::default();
    let out = (*p).process(&ctx, g, params);
    *signal = out.as_ptr().cast();
    out.len() as i32
}

#[no_mangle]
pub extern "C" fn janus_env_f32_new() -> *mut Env<f32> {
    Box::into_raw(Box::new(Env::new()))
}

#[no_mangle]
pub unsafe extern "C" fn janus_env_f32_free(p: *mut Env<f32>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn janus_env_f32_process(
    p: *mut Env<f32>,
    samples: u32,
    gate: *const f32,
    attack: *const f32,
    decay: *const f32,
    sustain: *const f32,
    release: *const f32,
    signal: *mut *const f32,
    offset: u32,
) -> i32 {
    if p.is_null()
        || gate.is_null()
        || attack.is_null()
        || decay.is_null()
        || sustain.is_null()
        || release.is_null()
        || signal.is_null()
    {
        return -1;
    }
    let g = std::slice::from_raw_parts(gate.offset(offset as isize), samples as usize);
    let a = std::slice::from_raw_parts(attack.offset(offset as isize), samples as usize);
    let d = std::slice::from_raw_parts(decay.offset(offset as isize), samples as usize);
    let s = std::slice::from_raw_parts(sustain.offset(offset as isize), samples as usize);
    let r = std::slice::from_raw_parts(release.offset(offset as isize), samples as usize);
    let params = EnvParams::<f32> {
        attack: a,
        decay: d,
        sustain: s,
        release: r,
    };
    //FIXME
    let ctx = Context::<f32> {
        sample_rate: 44100f32,
    };
    let out = (*p).process(&ctx, g, params);
    *signal = out.as_ptr().cast();
    out.len() as i32
}

#[no_mangle]
pub extern "C" fn janus_filt_u16_new() -> *mut FiltFxP {
    Box::into_raw(Box::new(FiltFxP::new()))
}

#[no_mangle]
pub unsafe extern "C" fn janus_filt_u16_free(p: *mut FiltFxP) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn janus_filt_u16_process(
    p: *mut FiltFxP,
    samples: u32,
    input: *const i16,
    cutoff: *const u16,
    resonance: *const u16,
    low: *mut *const i16,
    band: *mut *const i16,
    high: *mut *const i16,
    offset: u32,
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
    let i = std::slice::from_raw_parts(
        input.offset(offset as isize).cast::<SampleFxP>(),
        samples as usize,
    );
    let c = std::slice::from_raw_parts(
        cutoff.offset(offset as isize).cast::<NoteFxP>(),
        samples as usize,
    );
    let r = std::slice::from_raw_parts(
        resonance.offset(offset as isize).cast::<ScalarFxP>(),
        samples as usize,
    );
    let params = FiltParamsFxP {
        cutoff: c,
        resonance: r,
    };
    //FIXME
    let ctx = ContextFxP::default();
    let out = (*p).process(&ctx, i, params);
    *low = out.low.as_ptr().cast();
    *band = out.band.as_ptr().cast();
    *high = out.high.as_ptr().cast();
    out.low.len() as i32
}

#[no_mangle]
pub extern "C" fn janus_filt_f32_new() -> *mut Filt<f32> {
    Box::into_raw(Box::new(Filt::new()))
}

#[no_mangle]
pub unsafe extern "C" fn janus_filt_f32_free(p: *mut Filt<f32>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn janus_filt_f32_process(
    p: *mut Filt<f32>,
    samples: u32,
    input: *const f32,
    cutoff: *const f32,
    resonance: *const f32,
    low: *mut *const f32,
    band: *mut *const f32,
    high: *mut *const f32,
    offset: u32,
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
    let i = std::slice::from_raw_parts(input.offset(offset as isize), samples as usize);
    let c = std::slice::from_raw_parts(cutoff.offset(offset as isize), samples as usize);
    let r = std::slice::from_raw_parts(resonance.offset(offset as isize), samples as usize);
    let params = FiltParams::<f32> {
        cutoff: c,
        resonance: r,
    };
    //FIXME
    let ctx = Context::<f32> {
        sample_rate: 44100f32,
    };
    let out = (*p).process(&ctx, i, params);
    *low = out.low.as_ptr().cast();
    *band = out.band.as_ptr().cast();
    *high = out.high.as_ptr().cast();
    out.low.len() as i32
}

#[no_mangle]
pub extern "C" fn janus_osc_u16_new() -> *mut OscFxP {
    Box::into_raw(Box::new(OscFxP::new()))
}

#[no_mangle]
pub unsafe extern "C" fn janus_osc_u16_free(p: *mut OscFxP) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn janus_osc_u16_process(
    p: *mut OscFxP,
    samples: u32,
    note: *const u16,
    tune: *const i16,
    shape: *const u16,
    sin: *mut *const i16,
    tri: *mut *const i16,
    sq: *mut *const i16,
    saw: *mut *const i16,
    offset: u32,
) -> i32 {
    if p.is_null()
        || tune.is_null()
        || note.is_null()
        || shape.is_null()
        || sin.is_null()
        || tri.is_null()
        || sq.is_null()
    {
        return -1;
    }
    let note_s = std::slice::from_raw_parts(
        note.offset(offset as isize).cast::<NoteFxP>(),
        samples as usize,
    );
    let shape_s = std::slice::from_raw_parts(
        shape.offset(offset as isize).cast::<ScalarFxP>(),
        samples as usize,
    );
    let tune_s = std::slice::from_raw_parts(
        tune.offset(offset as isize).cast::<SignedNoteFxP>(),
        samples as usize,
    );
    let params = OscParamsFxP {
        tune: tune_s,
        shape: shape_s,
        sync: OscSync::Off,
    };
    //FIXME
    let ctx = ContextFxP::default();
    let out = (*p).process(&ctx, note_s, params);
    *sin = out.sin.as_ptr().cast();
    *tri = out.tri.as_ptr().cast();
    *sq = out.sq.as_ptr().cast();
    *saw = out.saw.as_ptr().cast();
    out.sin.len() as i32
}

#[no_mangle]
pub extern "C" fn janus_osc_f32_new() -> *mut Osc<f32> {
    Box::into_raw(Box::new(Osc::<f32>::new()))
}

#[no_mangle]
pub unsafe extern "C" fn janus_osc_f32_free(p: *mut Osc<f32>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn janus_osc_f32_process(
    p: *mut Osc<f32>,
    samples: u32,
    note: *const f32,
    tune: *const f32,
    shape: *const f32,
    sin: *mut *const f32,
    tri: *mut *const f32,
    sq: *mut *const f32,
    saw: *mut *const f32,
    offset: u32,
) -> i32 {
    if p.is_null()
        || tune.is_null()
        || note.is_null()
        || shape.is_null()
        || sin.is_null()
        || tri.is_null()
        || sq.is_null()
        || saw.is_null()
    {
        return -1;
    }
    let note_s = std::slice::from_raw_parts(note.offset(offset as isize), samples as usize);
    let shape_s = std::slice::from_raw_parts(shape.offset(offset as isize), samples as usize);
    let tune_s = std::slice::from_raw_parts(tune.offset(offset as isize), samples as usize);
    let params = OscParams::<f32> {
        tune: tune_s,
        shape: shape_s,
        sync: OscSync::Off,
    };
    //FIXME
    let ctx = Context::<f32> {
        sample_rate: 44100f32,
    };
    let out = (*p).process(&ctx, note_s, params);
    *sin = out.sin.as_ptr().cast();
    *tri = out.tri.as_ptr().cast();
    *sq = out.sq.as_ptr().cast();
    *saw = out.saw.as_ptr().cast();
    out.sin.len() as i32
}
