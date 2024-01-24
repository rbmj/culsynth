#![no_std]

extern crate alloc;
use alloc::boxed::Box;

use culsynth::context::{Context, ContextFxP};
use culsynth::devices::*;
use culsynth::{EnvParamFxP, NoteFxP, SampleFxP, ScalarFxP, SignedNoteFxP};

use core::iter::zip;

struct PtrIterator<T> {
    data: *mut T,
}

impl<T> PtrIterator<T> {
    unsafe fn new(data: *mut T) -> Self {
        Self { data }
    }
}

// This blows a gigantic hole through the borrow checker.  Only use this local
// to a specific function otherwise memory leaks are *bound* to happen
// TODO:  Make this less of a footgun
impl<T: 'static> Iterator for PtrIterator<T> {
    type Item = &'static mut T;
    fn next(&mut self) -> Option<&'static mut T> {
        unsafe {
            let ret = &mut *(self.data);
            self.data = self.data.add(1);
            Some(ret)
        }
    }
}

const SR_480_VAL: u32 = 0;
const SR_441_VAL: u32 = 1;

#[no_mangle]
pub static CULSYNTH_SR_480: u32 = SR_480_VAL;
#[no_mangle]
pub static CULSYNTH_SR_441: u32 = SR_441_VAL;

fn contextfxp_from_u32(sr: u32) -> Option<ContextFxP> {
    match sr {
        SR_480_VAL => Some(ContextFxP::new_480()),
        SR_441_VAL => Some(ContextFxP::new_441()),
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn culsynth_amp_i16_new() -> *mut Amp<i16> {
    Box::into_raw(Box::new(Amp::<i16>::default()))
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_amp_i16_free(p: *mut Amp<i16>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_amp_i16_process(
    p: *mut Amp<i16>,
    sr: u32,
    samples: u32,
    signal: *const i16,
    gain: *const u16,
    out: *mut i16,
) -> i32 {
    if p.is_null() || signal.is_null() || gain.is_null() || out.is_null() {
        return -1;
    }
    let context = match contextfxp_from_u32(sr) {
        Some(x) => x,
        None => return -1,
    };
    let s = core::slice::from_raw_parts(signal.cast::<SampleFxP>(), samples as usize);
    let g = core::slice::from_raw_parts(gain.cast::<ScalarFxP>(), samples as usize);
    let mut processed = 0i32;
    for (o, smp) in zip(
        PtrIterator::new(out),
        (*p).process(&context, s.iter().copied(), g.into_iter().copied()),
    ) {
        *o = smp.to_bits();
        processed += 1;
    }
    processed
}

#[no_mangle]
pub extern "C" fn culsynth_amp_f32_new() -> *mut Amp<f32> {
    Box::into_raw(Box::new(Amp::default()))
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_amp_f32_free(p: *mut Amp<f32>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_amp_f32_process(
    p: *mut Amp<f32>,
    sr: f32,
    samples: u32,
    signal: *const f32,
    gain: *const f32,
    out: *mut f32,
) -> i32 {
    if p.is_null() || signal.is_null() || gain.is_null() || out.is_null() {
        return -1;
    }
    let s = core::slice::from_raw_parts(signal, samples as usize);
    let g = core::slice::from_raw_parts(gain, samples as usize);
    let mut processed = 0i32;
    for (o, smp) in zip(
        PtrIterator::new(out),
        (*p).process(&Context::new(sr), s.iter().copied(), g.iter().copied()),
    ) {
        *o = smp;
        processed += 1;
    }
    processed
}

#[no_mangle]
pub extern "C" fn culsynth_env_i16_new() -> *mut Env<i16> {
    Box::into_raw(Box::new(Env::<i16>::default()))
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_env_i16_free(p: *mut Env<i16>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_env_i16_process(
    p: *mut Env<i16>,
    sr: u32,
    samples: u32,
    gate: *const bool,
    attack: *const u16,
    decay: *const u16,
    sustain: *const u16,
    release: *const u16,
    signal: *mut u16,
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
    let context = match contextfxp_from_u32(sr) {
        Some(x) => x,
        None => return -1,
    };
    let g = core::slice::from_raw_parts(gate, samples as usize);
    let a = core::slice::from_raw_parts(attack.cast::<EnvParamFxP>(), samples as usize);
    let d = core::slice::from_raw_parts(decay.cast::<EnvParamFxP>(), samples as usize);
    let s = core::slice::from_raw_parts(sustain.cast::<ScalarFxP>(), samples as usize);
    let r = core::slice::from_raw_parts(release.cast::<EnvParamFxP>(), samples as usize);
    let paramiter = new_env_param_iter()
        .with_attack(a.iter().copied())
        .with_decay(d.iter().copied())
        .with_sustain(s.iter().copied())
        .with_release(r.iter().copied());
    let out = (*p).process(&context, g.iter().copied(), paramiter);
    let mut processed = 0i32;
    for (o, smp) in zip(PtrIterator::new(signal), out) {
        *o = smp.to_bits();
        processed += 1;
    }
    processed
}

#[no_mangle]
pub extern "C" fn culsynth_env_f32_new() -> *mut Env<f32> {
    Box::into_raw(Box::new(Env::default()))
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_env_f32_free(p: *mut Env<f32>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_env_f32_process(
    p: *mut Env<f32>,
    sr: f32,
    samples: u32,
    gate: *const bool,
    attack: *const f32,
    decay: *const f32,
    sustain: *const f32,
    release: *const f32,
    signal: *mut f32,
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
    let g = core::slice::from_raw_parts(gate, samples as usize);
    let a = core::slice::from_raw_parts(attack, samples as usize);
    let d = core::slice::from_raw_parts(decay, samples as usize);
    let s = core::slice::from_raw_parts(sustain, samples as usize);
    let r = core::slice::from_raw_parts(release, samples as usize);
    let paramiter = new_env_param_iter()
        .with_attack(a.iter().copied())
        .with_decay(d.iter().copied())
        .with_sustain(s.iter().copied())
        .with_release(r.iter().copied());
    let ctx = Context::<f32> { sample_rate: sr };
    let out = (*p).process(&ctx, g.iter().copied(), paramiter);
    let mut processed = 0i32;
    for (o, smp) in zip(PtrIterator::new(signal), out) {
        *o = smp;
        processed += 1;
    }
    processed
}

#[no_mangle]
pub extern "C" fn culsynth_filt_i16_new() -> *mut Filt<i16> {
    Box::into_raw(Box::new(Filt::new()))
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_filt_i16_free(p: *mut Filt<i16>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_filt_i16_process(
    p: *mut Filt<i16>,
    sr: u32,
    samples: u32,
    input: *const i16,
    cutoff: *const u16,
    resonance: *const u16,
    low: *mut i16,
    band: *mut i16,
    high: *mut i16,
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
    let low = PtrIterator::new(low);
    let band = PtrIterator::new(band);
    let high = PtrIterator::new(high);
    let ctx = match contextfxp_from_u32(sr) {
        Some(x) => x,
        None => return -1,
    };
    let i = core::slice::from_raw_parts(input.cast::<SampleFxP>(), samples as usize);
    let c = core::slice::from_raw_parts(cutoff.cast::<NoteFxP>(), samples as usize);
    let r = core::slice::from_raw_parts(resonance.cast::<ScalarFxP>(), samples as usize);
    let params = new_filt_param_iter()
        .with_cutoff(c.iter().copied())
        .with_resonance(r.iter().copied());
    let out = (*p).process(&ctx, i.iter().copied(), params);
    let mut processed = 0i32;
    for (l, (b, (h, o))) in zip(low, zip(band, zip(high, out))) {
        *l = o.low.to_bits();
        *b = o.band.to_bits();
        *h = o.high.to_bits();
        processed += 1;
    }
    processed
}

#[no_mangle]
pub extern "C" fn culsynth_filt_f32_new() -> *mut Filt<f32> {
    Box::into_raw(Box::new(Filt::new()))
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_filt_f32_free(p: *mut Filt<f32>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_filt_f32_process(
    p: *mut Filt<f32>,
    sr: f32,
    samples: u32,
    input: *const f32,
    cutoff: *const f32,
    resonance: *const f32,
    low: *mut f32,
    band: *mut f32,
    high: *mut f32,
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
    let low = PtrIterator::new(low);
    let band = PtrIterator::new(band);
    let high = PtrIterator::new(high);
    let i = core::slice::from_raw_parts(input, samples as usize);
    let c = core::slice::from_raw_parts(cutoff, samples as usize);
    let r = core::slice::from_raw_parts(resonance, samples as usize);
    let params = new_filt_param_iter()
        .with_cutoff(c.iter().copied())
        .with_resonance(r.iter().copied());
    let ctx = Context::<f32> { sample_rate: sr };
    let out = (*p).process(&ctx, i.iter().copied(), params);
    let mut processed = 0i32;
    for (l, (b, (h, o))) in zip(low, zip(band, zip(high, out))) {
        *l = o.low;
        *b = o.band;
        *h = o.high;
        processed += 1;
    }
    processed
}

#[no_mangle]
pub extern "C" fn culsynth_osc_i16_new() -> *mut Osc<i16> {
    Box::into_raw(Box::new(Osc::new()))
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_osc_i16_free(p: *mut Osc<i16>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_osc_i16_process(
    p: *mut Osc<i16>,
    sr: u32,
    samples: u32,
    note: *const u16,
    tune: *const i16,
    shape: *const u16,
    sin: *mut i16,
    tri: *mut i16,
    sq: *mut i16,
    saw: *mut i16,
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
    let sin = PtrIterator::new(sin);
    let tri = PtrIterator::new(tri);
    let sq = PtrIterator::new(sq);
    let saw = PtrIterator::new(saw);
    let ctx = match contextfxp_from_u32(sr) {
        Some(x) => x,
        None => return -1,
    };
    let note_s = core::slice::from_raw_parts(note.cast::<NoteFxP>(), samples as usize);
    let shape_s = core::slice::from_raw_parts(shape.cast::<ScalarFxP>(), samples as usize);
    let tune_s = core::slice::from_raw_parts(tune.cast::<SignedNoteFxP>(), samples as usize);
    let params = new_osc_param_iter()
        .with_tune(tune_s.iter().copied())
        .with_shape(shape_s.iter().copied());
    let out = (*p).process(&ctx, note_s.iter().copied(), params);
    let mut processed = 0i32;
    for (n, (t, (q, (s, o)))) in zip(sin, zip(tri, zip(sq, zip(saw, out)))) {
        *n = o.sin.to_bits();
        *t = o.tri.to_bits();
        *q = o.sq.to_bits();
        *s = o.saw.to_bits();
        processed += 1;
    }
    processed
}

#[no_mangle]
pub extern "C" fn culsynth_osc_f32_new() -> *mut Osc<f32> {
    Box::into_raw(Box::new(Osc::<f32>::new()))
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_osc_f32_free(p: *mut Osc<f32>) {
    if !p.is_null() {
        let _ = Box::from_raw(p);
    }
}

#[no_mangle]
pub unsafe extern "C" fn culsynth_osc_f32_process(
    p: *mut Osc<f32>,
    sr: f32,
    samples: u32,
    note: *const f32,
    tune: *const f32,
    shape: *const f32,
    sin: *mut f32,
    tri: *mut f32,
    sq: *mut f32,
    saw: *mut f32,
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
    let sin = PtrIterator::new(sin);
    let tri = PtrIterator::new(tri);
    let sq = PtrIterator::new(sq);
    let saw = PtrIterator::new(saw);
    let note_s = core::slice::from_raw_parts(note, samples as usize);
    let shape_s = core::slice::from_raw_parts(shape, samples as usize);
    let tune_s = core::slice::from_raw_parts(tune, samples as usize);
    let params = new_osc_param_iter()
        .with_tune(tune_s.iter().copied())
        .with_shape(shape_s.iter().copied());
    let ctx = Context::<f32> { sample_rate: sr };
    let out = (*p).process(&ctx, note_s.iter().copied(), params);
    let mut processed = 0i32;
    for (n, (t, (q, (s, o)))) in zip(sin, zip(tri, zip(sq, zip(saw, out)))) {
        *n = o.sin;
        *t = o.tri;
        *q = o.sq;
        *s = o.saw;
        processed += 1;
    }
    processed
}
