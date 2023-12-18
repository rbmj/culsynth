use crate::pluginparams::{
    EnvPluginParams, FiltPluginParams, JanusParams, LfoPluginParams, OscPluginParams,
    RingModPluginParams,
};
use janus::devices::*;
use janus::{EnvParamFxP, LfoFreqFxP, NoteFxP, ScalarFxP, SignedNoteFxP};

#[derive(Default, Clone)]
pub struct EnvParamBufFxP {
    attack: Vec<EnvParamFxP>,
    decay: Vec<EnvParamFxP>,
    sustain: Vec<ScalarFxP>,
    release: Vec<EnvParamFxP>,
}

#[derive(Default, Clone)]
pub struct EnvParamBuf {
    attack: Vec<f32>,
    decay: Vec<f32>,
    sustain: Vec<f32>,
    release: Vec<f32>,
}

impl EnvParamBuf {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.attack.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        for buf in [
            &mut self.attack,
            &mut self.decay,
            &mut self.sustain,
            &mut self.release,
        ] {
            buf.resize(sz as usize, 0f32);
        }
    }
    pub fn params(&self, base: usize, end: usize) -> EnvParams<f32> {
        EnvParams {
            attack: &self.attack[base..end],
            decay: &self.decay[base..end],
            sustain: &self.sustain[base..end],
            release: &self.release[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutEnvParams<f32> {
        MutEnvParams {
            attack: &mut self.attack[base..end],
            decay: &mut self.decay[base..end],
            sustain: &mut self.sustain[base..end],
            release: &mut self.release[base..end],
        }
    }
}

impl EnvParamBufFxP {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.attack.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        for buf in [&mut self.attack, &mut self.decay, &mut self.release] {
            buf.resize(sz as usize, EnvParamFxP::ZERO);
        }
        self.sustain.resize(sz as usize, ScalarFxP::ZERO);
    }

    pub fn params(&self, base: usize, end: usize) -> EnvParamsFxP {
        EnvParamsFxP {
            attack: &self.attack[base..end],
            decay: &self.decay[base..end],
            sustain: &self.sustain[base..end],
            release: &self.release[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutEnvParamsFxP {
        MutEnvParamsFxP {
            attack: &mut self.attack[base..end],
            decay: &mut self.decay[base..end],
            sustain: &mut self.sustain[base..end],
            release: &mut self.release[base..end],
        }
    }
    pub fn update_index(&mut self, idx: usize, p: &EnvPluginParams) {
        self.attack[idx] = EnvParamFxP::from_bits(p.a.smoothed.next() as u16);
        self.decay[idx] = EnvParamFxP::from_bits(p.d.smoothed.next() as u16);
        self.sustain[idx] = ScalarFxP::from_bits(p.s.smoothed.next() as u16);
        self.release[idx] = EnvParamFxP::from_bits(p.r.smoothed.next() as u16);
    }
    pub fn into_float(&self, buf: &mut EnvParamBuf) {
        for idx in 0..std::cmp::min(self.len(), buf.len()) {
            buf.attack[idx] = self.attack[idx].to_num();
            buf.decay[idx] = self.decay[idx].to_num();
            buf.sustain[idx] = self.sustain[idx].to_num();
            buf.release[idx] = self.release[idx].to_num();
        }
    }
    pub fn copy_to(&self, buf: &mut Self) {
        for idx in 0..std::cmp::min(self.len(), buf.len()) {
            buf.attack[idx] = self.attack[idx];
            buf.decay[idx] = self.decay[idx];
            buf.sustain[idx] = self.sustain[idx];
            buf.release[idx] = self.release[idx];
        }
    }
}

#[derive(Default, Clone)]
pub struct RingModParamBufFxP {
    mix_a: Vec<ScalarFxP>,
    mix_b: Vec<ScalarFxP>,
    mix_mod: Vec<ScalarFxP>,
}

impl RingModParamBufFxP {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.mix_a.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        for buf in [&mut self.mix_a, &mut self.mix_b, &mut self.mix_mod] {
            buf.resize(sz as usize, ScalarFxP::ZERO);
        }
    }
    pub fn into_float(&self, buf: &mut RingModParamBuf) {
        for i in 0..std::cmp::min(self.len(), buf.len()) {
            buf.mix_a[i] = self.mix_a[i].to_num();
            buf.mix_b[i] = self.mix_b[i].to_num();
            buf.mix_mod[i] = self.mix_mod[i].to_num();
        }
    }
    pub fn params(&self, base: usize, end: usize) -> RingModParamsFxP {
        RingModParamsFxP {
            mix_a: &self.mix_a[base..end],
            mix_b: &self.mix_b[base..end],
            mix_out: &self.mix_mod[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutRingModParamsFxP {
        MutRingModParamsFxP {
            mix_a: &mut self.mix_a[base..end],
            mix_b: &mut self.mix_b[base..end],
            mix_out: &mut self.mix_mod[base..end],
        }
    }
    pub fn update_index(&mut self, idx: usize, p: &RingModPluginParams) {
        self.mix_a[idx] = ScalarFxP::from_bits(p.mix_a.smoothed.next() as u16);
        self.mix_b[idx] = ScalarFxP::from_bits(p.mix_b.smoothed.next() as u16);
        self.mix_mod[idx] = ScalarFxP::from_bits(p.mix_mod.smoothed.next() as u16);
    }
    pub fn copy_to(&self, buf: &mut Self) {
        for idx in 0..std::cmp::min(self.len(), buf.len()) {
            buf.mix_a[idx] = self.mix_a[idx];
            buf.mix_b[idx] = self.mix_b[idx];
            buf.mix_mod[idx] = self.mix_mod[idx];
        }
    }
}

#[derive(Default, Clone)]
pub struct RingModParamBuf {
    mix_a: Vec<f32>,
    mix_b: Vec<f32>,
    mix_mod: Vec<f32>,
}

impl RingModParamBuf {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.mix_a.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        for buf in [&mut self.mix_a, &mut self.mix_b, &mut self.mix_mod] {
            buf.resize(sz as usize, 0f32);
        }
    }
    pub fn params(&self, base: usize, end: usize) -> RingModParams<f32> {
        RingModParams {
            mix_a: &self.mix_a[base..end],
            mix_b: &self.mix_b[base..end],
            mix_out: &self.mix_mod[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutRingModParams<f32> {
        MutRingModParams {
            mix_a: &mut self.mix_a[base..end],
            mix_b: &mut self.mix_b[base..end],
            mix_out: &mut self.mix_mod[base..end],
        }
    }
}

#[derive(Default, Clone)]
pub struct GlobalParamBufFxP {
    sync: Vec<ScalarFxP>,
}

impl GlobalParamBufFxP {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.sync.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        for buf in [&mut self.sync] {
            buf.resize(sz as usize, ScalarFxP::ZERO);
        }
    }
    pub fn into_float(&self, buf: &mut GlobalParamBuf) {
        for i in 0..self.len() {
            buf.sync[i] = self.sync[i].to_bits().into();
        }
    }
    pub fn sync_mut(&mut self, base: usize, end: usize) -> &mut [ScalarFxP] {
        &mut self.sync[base..end]
    }
    pub fn update_index(&mut self, idx: usize, osc_sync: &nih_plug::params::BoolParam) {
        self.sync[idx] = if osc_sync.value() {
            ScalarFxP::DELTA
        } else {
            ScalarFxP::ZERO
        };
    }
    pub fn copy_to(&self, buf: &mut Self) {
        for idx in 0..std::cmp::min(self.len(), buf.len()) {
            buf.sync[idx] = self.sync[idx];
        }
    }
}

#[derive(Default, Clone)]
pub struct GlobalParamBuf {
    sync: Vec<f32>,
}

impl GlobalParamBuf {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.sync.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        for buf in [&mut self.sync] {
            buf.resize(sz as usize, 0f32);
        }
    }
    pub fn sync_mut(&mut self, base: usize, end: usize) -> &mut [f32] {
        &mut self.sync[base..end]
    }
}

#[derive(Default, Clone)]
pub struct OscParamBufFxP {
    tune: Vec<SignedNoteFxP>,
    shape: Vec<ScalarFxP>,
    sin: Vec<ScalarFxP>,
    sq: Vec<ScalarFxP>,
    tri: Vec<ScalarFxP>,
    saw: Vec<ScalarFxP>,
}

impl OscParamBufFxP {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.shape.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        self.tune.resize(sz as usize, SignedNoteFxP::ZERO);
        self.shape.resize(sz as usize, ScalarFxP::ZERO);
        self.sin.resize(sz as usize, ScalarFxP::ZERO);
        self.sq.resize(sz as usize, ScalarFxP::ZERO);
        self.tri.resize(sz as usize, ScalarFxP::ZERO);
        self.saw.resize(sz as usize, ScalarFxP::ZERO);
    }
    pub fn into_float(&self, buf: &mut OscParamBuf) {
        for i in 0..std::cmp::min(self.len(), buf.len()) {
            buf.tune[i] = self.tune[i].to_num();
            buf.shape[i] = self.shape[i].to_num();
            buf.sin[i] = self.sin[i].to_num();
            buf.sq[i] = self.sq[i].to_num();
            buf.tri[i] = self.tri[i].to_num();
            buf.saw[i] = self.saw[i].to_num();
        }
    }
    pub fn params(&self, base: usize, end: usize) -> MixOscParamsFxP {
        MixOscParamsFxP {
            tune: &self.tune[base..end],
            shape: &self.shape[base..end],
            sync: janus::devices::OscSync::Off,
            sin: &self.sin[base..end],
            sq: &self.sq[base..end],
            tri: &self.tri[base..end],
            saw: &self.saw[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutMixOscParamsFxP {
        MutMixOscParamsFxP {
            tune: &mut self.tune[base..end],
            shape: &mut self.shape[base..end],
            sync: janus::devices::OscSync::Off,
            sin: &mut self.sin[base..end],
            sq: &mut self.sq[base..end],
            tri: &mut self.tri[base..end],
            saw: &mut self.saw[base..end],
        }
    }
    pub fn update_index(&mut self, idx: usize, p: &OscPluginParams) {
        self.shape[idx] = ScalarFxP::from_bits(p.shape.smoothed.next() as u16);
        self.sin[idx] = ScalarFxP::from_bits(p.sin.smoothed.next() as u16);
        self.sq[idx] = ScalarFxP::from_bits(p.sq.smoothed.next() as u16);
        self.tri[idx] = ScalarFxP::from_bits(p.tri.smoothed.next() as u16);
        self.saw[idx] = ScalarFxP::from_bits(p.saw.smoothed.next() as u16);
        self.tune[idx] = SignedNoteFxP::from_bits(
            ((p.course.smoothed.next() << 9) + p.fine.smoothed.next()) as i16,
        )
    }
    pub fn copy_to(&self, buf: &mut Self) {
        for idx in 0..std::cmp::min(self.len(), buf.len()) {
            buf.shape[idx] = self.shape[idx];
            buf.sin[idx] = self.sin[idx];
            buf.sq[idx] = self.sq[idx];
            buf.tri[idx] = self.tri[idx];
            buf.saw[idx] = self.saw[idx];
            buf.tune[idx] = self.tune[idx];
        }
    }
}

#[derive(Default, Clone)]
pub struct OscParamBuf {
    tune: Vec<f32>,
    shape: Vec<f32>,
    sin: Vec<f32>,
    sq: Vec<f32>,
    tri: Vec<f32>,
    saw: Vec<f32>,
}

impl OscParamBuf {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.shape.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        self.tune.resize(sz as usize, 0f32);
        self.shape.resize(sz as usize, 0f32);
        self.sin.resize(sz as usize, 0f32);
        self.sq.resize(sz as usize, 0f32);
        self.tri.resize(sz as usize, 0f32);
        self.saw.resize(sz as usize, 0f32);
    }
    pub fn params(&self, base: usize, end: usize) -> MixOscParams<f32> {
        MixOscParams {
            tune: &self.tune[base..end],
            shape: &self.shape[base..end],
            sync: janus::devices::OscSync::Off,
            sin: &self.sin[base..end],
            sq: &self.sq[base..end],
            tri: &self.tri[base..end],
            saw: &self.saw[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutMixOscParams<f32> {
        MutMixOscParams {
            tune: &mut self.tune[base..end],
            shape: &mut self.shape[base..end],
            sync: janus::devices::OscSync::Off,
            sin: &mut self.sin[base..end],
            sq: &mut self.sq[base..end],
            tri: &mut self.tri[base..end],
            saw: &mut self.saw[base..end],
        }
    }
}

#[derive(Default, Clone)]
pub struct LfoParamBufFxP {
    opts: Vec<LfoOptions>,
    freq: Vec<LfoFreqFxP>,
    depth: Vec<ScalarFxP>,
}

impl LfoParamBufFxP {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.freq.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        self.opts.resize(sz as usize, Default::default());
        self.freq.resize(sz as usize, LfoFreqFxP::ONE);
        self.depth.resize(sz as usize, ScalarFxP::MAX);
    }
    pub fn into_float(&self, buf: &mut LfoParamBuf) {
        for i in 0..std::cmp::min(self.len(), buf.len()) {
            buf.freq[i] = self.freq[i].to_num();
            buf.depth[i] = self.depth[i].to_num();
            buf.opts[i] = self.opts[i];
        }
    }
    pub fn params(&self, base: usize, end: usize) -> LfoParamsFxP {
        LfoParamsFxP {
            freq: &self.freq[base..end],
            depth: &self.depth[base..end],
            opts: &self.opts[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutLfoParamsFxP {
        MutLfoParamsFxP {
            freq: &mut self.freq[base..end],
            depth: &mut self.depth[base..end],
            opts: &mut self.opts[base..end],
        }
    }
    pub fn update_index(&mut self, idx: usize, p: &LfoPluginParams) {
        self.freq[idx] = LfoFreqFxP::from_bits(p.rate.smoothed.next() as u16);
        self.depth[idx] = ScalarFxP::from_bits(p.depth.smoothed.next() as u16);
        self.opts[idx] = LfoOptions::from(p);
    }
    pub fn copy_to(&self, buf: &mut Self) {
        for idx in 0..std::cmp::min(self.len(), buf.len()) {
            buf.freq[idx] = self.freq[idx];
            buf.depth[idx] = self.depth[idx];
            buf.opts[idx] = self.opts[idx];
        }
    }
}

#[derive(Default, Clone)]
pub struct LfoParamBuf {
    freq: Vec<f32>,
    depth: Vec<f32>,
    opts: Vec<LfoOptions>,
}

impl LfoParamBuf {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.freq.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        self.freq.resize(sz as usize, 0f32);
        self.depth.resize(sz as usize, 0f32);
        self.opts.resize(sz as usize, Default::default());
    }
    pub fn params(&self, base: usize, end: usize) -> LfoParams<f32> {
        LfoParams {
            freq: &self.freq[base..end],
            depth: &self.depth[base..end],
            opts: &self.opts[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutLfoParams<f32> {
        MutLfoParams {
            freq: &mut self.freq[base..end],
            depth: &mut self.depth[base..end],
            opts: &mut self.opts[base..end],
        }
    }
}

#[derive(Default, Clone)]
pub struct FiltParamBufFxP {
    env_mod: Vec<ScalarFxP>,
    vel_mod: Vec<ScalarFxP>,
    kbd: Vec<ScalarFxP>,
    cutoff: Vec<NoteFxP>,
    resonance: Vec<ScalarFxP>,
    low_mix: Vec<ScalarFxP>,
    band_mix: Vec<ScalarFxP>,
    high_mix: Vec<ScalarFxP>,
}

impl FiltParamBufFxP {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.cutoff.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        self.cutoff.resize(sz as usize, NoteFxP::ZERO);
        for buf in [
            &mut self.env_mod,
            &mut self.vel_mod,
            &mut self.kbd,
            &mut self.resonance,
            &mut self.low_mix,
            &mut self.band_mix,
            &mut self.high_mix,
        ] {
            buf.resize(sz as usize, ScalarFxP::ZERO);
        }
    }
    pub fn into_float(&self, buf: &mut FiltParamBuf) {
        for i in 0..std::cmp::min(self.len(), buf.len()) {
            buf.env_mod[i] = self.env_mod[i].to_num();
            buf.vel_mod[i] = self.vel_mod[i].to_num();
            buf.kbd[i] = self.kbd[i].to_num();
            buf.cutoff[i] = self.cutoff[i].to_num();
            buf.resonance[i] = self.resonance[i].to_num();
            buf.low_mix[i] = self.low_mix[i].to_num();
            buf.band_mix[i] = self.band_mix[i].to_num();
            buf.high_mix[i] = self.high_mix[i].to_num();
        }
    }
    pub fn params(&self, base: usize, end: usize) -> ModFiltParamsFxP {
        ModFiltParamsFxP {
            env_mod: &self.env_mod[base..end],
            vel_mod: &self.vel_mod[base..end],
            kbd: &self.kbd[base..end],
            cutoff: &self.cutoff[base..end],
            resonance: &self.resonance[base..end],
            low_mix: &self.low_mix[base..end],
            band_mix: &self.band_mix[base..end],
            high_mix: &self.high_mix[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutModFiltParamsFxP {
        MutModFiltParamsFxP {
            env_mod: &mut self.env_mod[base..end],
            vel_mod: &mut self.vel_mod[base..end],
            kbd: &mut self.kbd[base..end],
            cutoff: &mut self.cutoff[base..end],
            resonance: &mut self.resonance[base..end],
            low_mix: &mut self.low_mix[base..end],
            band_mix: &mut self.band_mix[base..end],
            high_mix: &mut self.high_mix[base..end],
        }
    }
    pub fn update_index(&mut self, idx: usize, p: &FiltPluginParams) {
        self.env_mod[idx] = ScalarFxP::from_bits(p.env.smoothed.next() as u16);
        self.vel_mod[idx] = ScalarFxP::from_bits(p.vel.smoothed.next() as u16);
        self.kbd[idx] = ScalarFxP::from_bits(p.kbd.smoothed.next() as u16);
        self.cutoff[idx] = NoteFxP::from_bits(p.cutoff.smoothed.next() as u16);
        self.resonance[idx] = ScalarFxP::from_bits(p.res.smoothed.next() as u16);
        self.low_mix[idx] = ScalarFxP::from_bits(p.low.smoothed.next() as u16);
        self.band_mix[idx] = ScalarFxP::from_bits(p.band.smoothed.next() as u16);
        self.high_mix[idx] = ScalarFxP::from_bits(p.high.smoothed.next() as u16);
    }
    pub fn copy_to(&self, buf: &mut Self) {
        for idx in 0..std::cmp::min(self.len(), buf.len()) {
            buf.env_mod[idx] = self.env_mod[idx];
            buf.vel_mod[idx] = self.vel_mod[idx];
            buf.kbd[idx] = self.kbd[idx];
            buf.cutoff[idx] = self.cutoff[idx];
            buf.resonance[idx] = self.resonance[idx];
            buf.low_mix[idx] = self.low_mix[idx];
            buf.band_mix[idx] = self.band_mix[idx];
            buf.high_mix[idx] = self.high_mix[idx];
        }
    }
}

#[derive(Default, Clone)]
pub struct FiltParamBuf {
    env_mod: Vec<f32>,
    vel_mod: Vec<f32>,
    kbd: Vec<f32>,
    cutoff: Vec<f32>,
    resonance: Vec<f32>,
    low_mix: Vec<f32>,
    band_mix: Vec<f32>,
    high_mix: Vec<f32>,
}

impl FiltParamBuf {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.cutoff.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        for buf in [
            &mut self.env_mod,
            &mut self.vel_mod,
            &mut self.kbd,
            &mut self.cutoff,
            &mut self.resonance,
            &mut self.low_mix,
            &mut self.band_mix,
            &mut self.high_mix,
        ] {
            buf.resize(sz as usize, 0f32);
        }
    }
    pub fn params(&self, base: usize, end: usize) -> ModFiltParams<f32> {
        ModFiltParams {
            env_mod: &self.env_mod[base..end],
            vel_mod: &self.vel_mod[base..end],
            kbd: &self.kbd[base..end],
            cutoff: &self.cutoff[base..end],
            resonance: &self.resonance[base..end],
            low_mix: &self.low_mix[base..end],
            band_mix: &self.band_mix[base..end],
            high_mix: &self.high_mix[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutModFiltParams<f32> {
        MutModFiltParams {
            env_mod: &mut self.env_mod[base..end],
            vel_mod: &mut self.vel_mod[base..end],
            kbd: &mut self.kbd[base..end],
            cutoff: &mut self.cutoff[base..end],
            resonance: &mut self.resonance[base..end],
            low_mix: &mut self.low_mix[base..end],
            band_mix: &mut self.band_mix[base..end],
            high_mix: &mut self.high_mix[base..end],
        }
    }
}

#[derive(Default, Clone)]
pub struct PluginParamBufFxP {
    pub global: GlobalParamBufFxP,
    pub osc1: OscParamBufFxP,
    pub osc2: OscParamBufFxP,
    pub ringmod: RingModParamBufFxP,
    pub filt: FiltParamBufFxP,
    pub env_filt: EnvParamBufFxP,
    pub env_amp: EnvParamBufFxP,
    pub lfo1: LfoParamBufFxP,
    pub lfo2: LfoParamBufFxP,
    pub env1: EnvParamBufFxP,
    pub env2: EnvParamBufFxP,
}

impl PluginParamBufFxP {
    pub fn allocate(&mut self, sz: u32) {
        self.global.allocate(sz);
        self.osc1.allocate(sz);
        self.osc2.allocate(sz);
        self.ringmod.allocate(sz);
        self.filt.allocate(sz);
        self.env_filt.allocate(sz);
        self.env_amp.allocate(sz);
        self.lfo1.allocate(sz);
        self.lfo2.allocate(sz);
        self.env1.allocate(sz);
        self.env2.allocate(sz);
    }
    pub fn update_index(&mut self, index: usize, p: &JanusParams) {
        self.global.update_index(index, &p.osc_sync);
        self.osc1.update_index(index, &p.osc1);
        self.osc2.update_index(index, &p.osc2);
        self.ringmod.update_index(index, &p.ringmod);
        self.filt.update_index(index, &p.filt);
        self.env_amp.update_index(index, &p.env_vca);
        self.env_filt.update_index(index, &p.env_vcf);
        self.env1.update_index(index, &p.env1);
        self.env2.update_index(index, &p.env2);
        self.lfo1.update_index(index, &p.lfo1);
        self.lfo2.update_index(index, &p.lfo2);
    }
    pub fn into_float(&self, buf: &mut PluginParamBuf) {
        self.global.into_float(&mut buf.global);
        self.osc1.into_float(&mut buf.osc1);
        self.osc2.into_float(&mut buf.osc2);
        self.ringmod.into_float(&mut buf.ringmod);
        self.filt.into_float(&mut buf.filt);
        self.env_filt.into_float(&mut buf.env_filt);
        self.env_amp.into_float(&mut buf.env_amp);
        self.lfo1.into_float(&mut buf.lfo1);
        self.lfo2.into_float(&mut buf.lfo2);
        self.env1.into_float(&mut buf.env1);
        self.env2.into_float(&mut buf.env2);
    }
    pub fn copy_to(&self, buf: &mut Self) {
        self.global.copy_to(&mut buf.global);
        self.osc1.copy_to(&mut buf.osc1);
        self.osc2.copy_to(&mut buf.osc2);
        self.ringmod.copy_to(&mut buf.ringmod);
        self.filt.copy_to(&mut buf.filt);
        self.env_filt.copy_to(&mut buf.env_filt);
        self.env_amp.copy_to(&mut buf.env_amp);
        self.lfo1.copy_to(&mut buf.lfo1);
        self.lfo2.copy_to(&mut buf.lfo2);
        self.env1.copy_to(&mut buf.env1);
        self.env2.copy_to(&mut buf.env2);
    }
}

#[derive(Default, Clone)]
pub struct PluginParamBuf {
    pub global: GlobalParamBuf,
    pub osc1: OscParamBuf,
    pub osc2: OscParamBuf,
    pub ringmod: RingModParamBuf,
    pub filt: FiltParamBuf,
    pub env_filt: EnvParamBuf,
    pub env_amp: EnvParamBuf,
    pub lfo1: LfoParamBuf,
    pub lfo2: LfoParamBuf,
    pub env1: EnvParamBuf,
    pub env2: EnvParamBuf,
}

impl PluginParamBuf {
    pub fn allocate(&mut self, sz: u32) {
        self.global.allocate(sz);
        self.osc1.allocate(sz);
        self.osc2.allocate(sz);
        self.ringmod.allocate(sz);
        self.filt.allocate(sz);
        self.env_filt.allocate(sz);
        self.env_amp.allocate(sz);
        self.lfo1.allocate(sz);
        self.lfo2.allocate(sz);
        self.env1.allocate(sz);
        self.env2.allocate(sz);
    }
}
