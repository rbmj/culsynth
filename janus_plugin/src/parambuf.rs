use crate::pluginparams::{
    EnvPluginParams, FiltPluginParams, LfoPluginParams, OscPluginParams, RingModPluginParams,
};
use janus::devices::*;
use janus::{EnvParamFxP, LfoFreqFxP, NoteFxP, ScalarFxP, SignedNoteFxP};

#[derive(Default)]
pub struct EnvParamBuffer {
    attack: Vec<f32>,
    decay: Vec<f32>,
    sustain: Vec<f32>,
    release: Vec<f32>,
    attack_fxp: Vec<EnvParamFxP>,
    decay_fxp: Vec<EnvParamFxP>,
    sustain_fxp: Vec<ScalarFxP>,
    release_fxp: Vec<EnvParamFxP>,
}

impl EnvParamBuffer {
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
        for buf in [
            &mut self.attack,
            &mut self.decay,
            &mut self.sustain,
            &mut self.release,
        ] {
            buf.resize(sz as usize, 0f32);
        }
        for buf in [
            &mut self.attack_fxp,
            &mut self.decay_fxp,
            &mut self.release_fxp,
        ] {
            buf.resize(sz as usize, EnvParamFxP::ZERO);
        }
        self.sustain_fxp.resize(sz as usize, ScalarFxP::ZERO);
    }
    pub fn conv_float(&mut self) {
        for i in 0..self.len() {
            self.attack[i] = self.attack_fxp[i].to_num();
            self.decay[i] = self.decay_fxp[i].to_num();
            self.sustain[i] = self.sustain_fxp[i].to_num();
            self.release[i] = self.release_fxp[i].to_num();
        }
    }
    pub fn params_float(&self, base: usize, end: usize) -> EnvParams<f32> {
        EnvParams {
            attack: &self.attack[base..end],
            decay: &self.decay[base..end],
            sustain: &self.sustain[base..end],
            release: &self.release[base..end],
        }
    }
    pub fn params_float_mut(&mut self, base: usize, end: usize) -> MutEnvParams<f32> {
        MutEnvParams {
            attack: &mut self.attack[base..end],
            decay: &mut self.decay[base..end],
            sustain: &mut self.sustain[base..end],
            release: &mut self.release[base..end],
        }
    }
    pub fn params(&self, base: usize, end: usize) -> EnvParamsFxP {
        EnvParamsFxP {
            attack: &self.attack_fxp[base..end],
            decay: &self.decay_fxp[base..end],
            sustain: &self.sustain_fxp[base..end],
            release: &self.release_fxp[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutEnvParamsFxP {
        MutEnvParamsFxP {
            attack: &mut self.attack_fxp[base..end],
            decay: &mut self.decay_fxp[base..end],
            sustain: &mut self.sustain_fxp[base..end],
            release: &mut self.release_fxp[base..end],
        }
    }
    pub fn a(&self) -> &[EnvParamFxP] {
        self.attack_fxp.as_slice()
    }
    pub fn a_mut(&mut self) -> &mut [EnvParamFxP] {
        self.attack_fxp.as_mut_slice()
    }
    pub fn a_float(&self) -> &[f32] {
        self.attack.as_slice()
    }
    pub fn d(&self) -> &[EnvParamFxP] {
        self.decay_fxp.as_slice()
    }
    pub fn d_mut(&mut self) -> &mut [EnvParamFxP] {
        self.decay_fxp.as_mut_slice()
    }
    pub fn d_float(&self) -> &[f32] {
        self.decay.as_slice()
    }
    pub fn s(&self) -> &[ScalarFxP] {
        self.sustain_fxp.as_slice()
    }
    pub fn s_mut(&mut self) -> &mut [ScalarFxP] {
        self.sustain_fxp.as_mut_slice()
    }
    pub fn s_float(&self) -> &[f32] {
        self.sustain.as_slice()
    }
    pub fn r(&self) -> &[EnvParamFxP] {
        self.release_fxp.as_slice()
    }
    pub fn r_mut(&mut self) -> &mut [EnvParamFxP] {
        self.release_fxp.as_mut_slice()
    }
    pub fn r_float(&self) -> &[f32] {
        self.release.as_slice()
    }
    pub fn update_index(&mut self, idx: usize, p: &EnvPluginParams) {
        self.attack_fxp[idx] = EnvParamFxP::from_bits(p.a.smoothed.next() as u16);
        self.decay_fxp[idx] = EnvParamFxP::from_bits(p.d.smoothed.next() as u16);
        self.sustain_fxp[idx] = ScalarFxP::from_bits(p.s.smoothed.next() as u16);
        self.release_fxp[idx] = EnvParamFxP::from_bits(p.r.smoothed.next() as u16);
    }
}

#[derive(Default)]
pub struct RingModParamBuffer {
    mix_a: Vec<f32>,
    mix_b: Vec<f32>,
    mix_mod: Vec<f32>,
    mix_a_fxp: Vec<ScalarFxP>,
    mix_b_fxp: Vec<ScalarFxP>,
    mix_mod_fxp: Vec<ScalarFxP>,
}

impl RingModParamBuffer {
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
        for buf in [
            &mut self.mix_a_fxp,
            &mut self.mix_b_fxp,
            &mut self.mix_mod_fxp,
        ] {
            buf.resize(sz as usize, ScalarFxP::ZERO);
        }
    }
    pub fn conv_float(&mut self) {
        for i in 0..self.len() {
            self.mix_a[i] = self.mix_a_fxp[i].to_num();
            self.mix_b[i] = self.mix_b_fxp[i].to_num();
            self.mix_mod[i] = self.mix_mod_fxp[i].to_num();
        }
    }
    pub fn params_float(&self, base: usize, end: usize) -> RingModParams<f32> {
        RingModParams {
            mix_a: &self.mix_a[base..end],
            mix_b: &self.mix_b[base..end],
            mix_out: &self.mix_mod[base..end],
        }
    }
    pub fn params_float_mut(&mut self, base: usize, end: usize) -> MutRingModParams<f32> {
        MutRingModParams {
            mix_a: &mut self.mix_a[base..end],
            mix_b: &mut self.mix_b[base..end],
            mix_out: &mut self.mix_mod[base..end],
        }
    }
    pub fn params(&self, base: usize, end: usize) -> RingModParamsFxP {
        RingModParamsFxP {
            mix_a: &self.mix_a_fxp[base..end],
            mix_b: &self.mix_b_fxp[base..end],
            mix_out: &self.mix_mod_fxp[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutRingModParamsFxP {
        MutRingModParamsFxP {
            mix_a: &mut self.mix_a_fxp[base..end],
            mix_b: &mut self.mix_b_fxp[base..end],
            mix_out: &mut self.mix_mod_fxp[base..end],
        }
    }
    pub fn mix_a(&self) -> &[ScalarFxP] {
        self.mix_a_fxp.as_slice()
    }
    pub fn mix_a_mut(&mut self) -> &mut [ScalarFxP] {
        self.mix_a_fxp.as_mut_slice()
    }
    pub fn mix_a_float(&self) -> &[f32] {
        self.mix_a.as_slice()
    }
    pub fn mix_b(&self) -> &[ScalarFxP] {
        self.mix_b_fxp.as_slice()
    }
    pub fn mix_b_mut(&mut self) -> &mut [ScalarFxP] {
        self.mix_b_fxp.as_mut_slice()
    }
    pub fn mix_b_float(&self) -> &[f32] {
        self.mix_b.as_slice()
    }
    pub fn mix_mod(&self) -> &[ScalarFxP] {
        self.mix_mod_fxp.as_slice()
    }
    pub fn mix_mod_mut(&mut self) -> &mut [ScalarFxP] {
        self.mix_mod_fxp.as_mut_slice()
    }
    pub fn mix_mod_float(&self) -> &[f32] {
        self.mix_mod.as_slice()
    }
    pub fn update_index(&mut self, idx: usize, p: &RingModPluginParams) {
        self.mix_a_fxp[idx] = ScalarFxP::from_bits(p.mix_a.smoothed.next() as u16);
        self.mix_b_fxp[idx] = ScalarFxP::from_bits(p.mix_b.smoothed.next() as u16);
        self.mix_mod_fxp[idx] = ScalarFxP::from_bits(p.mix_mod.smoothed.next() as u16);
    }
}

#[derive(Default)]
pub struct GlobalParamBuffer {
    sync: Vec<f32>,
    sync_fxp: Vec<ScalarFxP>,
}

impl GlobalParamBuffer {
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
        for buf in [&mut self.sync_fxp] {
            buf.resize(sz as usize, ScalarFxP::ZERO);
        }
    }
    pub fn conv_float(&mut self) {
        for i in 0..self.len() {
            self.sync[i] = self.sync_fxp[i].to_num();
        }
    }
    pub fn sync(&mut self, base: usize, end: usize) -> &mut [ScalarFxP] {
        &mut self.sync_fxp[base..end]
    }
    pub fn sync_float(&mut self, base: usize, end: usize) -> &mut [f32] {
        &mut self.sync[base..end]
    }
    pub fn update_index(&mut self, idx: usize, osc_sync: &nih_plug::params::BoolParam) {
        self.sync_fxp[idx] = if osc_sync.value() {
            ScalarFxP::DELTA
        } else {
            ScalarFxP::ZERO
        };
    }
}

#[derive(Default)]
pub struct OscParamBuffer {
    tune: Vec<f32>,
    shape: Vec<f32>,
    sin: Vec<f32>,
    sq: Vec<f32>,
    tri: Vec<f32>,
    saw: Vec<f32>,
    tune_fxp: Vec<SignedNoteFxP>,
    shape_fxp: Vec<ScalarFxP>,
    sin_fxp: Vec<ScalarFxP>,
    sq_fxp: Vec<ScalarFxP>,
    tri_fxp: Vec<ScalarFxP>,
    saw_fxp: Vec<ScalarFxP>,
}

impl OscParamBuffer {
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
        self.tune_fxp.resize(sz as usize, SignedNoteFxP::ZERO);
        self.shape_fxp.resize(sz as usize, ScalarFxP::ZERO);
        self.sin_fxp.resize(sz as usize, ScalarFxP::ZERO);
        self.sq_fxp.resize(sz as usize, ScalarFxP::ZERO);
        self.tri_fxp.resize(sz as usize, ScalarFxP::ZERO);
        self.saw_fxp.resize(sz as usize, ScalarFxP::ZERO);
    }
    pub fn conv_float(&mut self) {
        for i in 0..self.len() {
            self.tune[i] = self.tune_fxp[i].to_num();
            self.shape[i] = self.shape_fxp[i].to_num();
            self.sin[i] = self.sin_fxp[i].to_num();
            self.sq[i] = self.sq_fxp[i].to_num();
            self.tri[i] = self.tri_fxp[i].to_num();
            self.saw[i] = self.saw_fxp[i].to_num();
        }
    }
    pub fn params_float(&self, base: usize, end: usize) -> MixOscParams<f32> {
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
    pub fn params_float_mut(&mut self, base: usize, end: usize) -> MutMixOscParams<f32> {
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
    pub fn params(&self, base: usize, end: usize) -> MixOscParamsFxP {
        MixOscParamsFxP {
            tune: &self.tune_fxp[base..end],
            shape: &self.shape_fxp[base..end],
            sync: janus::devices::OscSync::Off,
            sin: &self.sin_fxp[base..end],
            sq: &self.sq_fxp[base..end],
            tri: &self.tri_fxp[base..end],
            saw: &self.saw_fxp[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutMixOscParamsFxP {
        MutMixOscParamsFxP {
            tune: &mut self.tune_fxp[base..end],
            shape: &mut self.shape_fxp[base..end],
            sync: janus::devices::OscSync::Off,
            sin: &mut self.sin_fxp[base..end],
            sq: &mut self.sq_fxp[base..end],
            tri: &mut self.tri_fxp[base..end],
            saw: &mut self.saw_fxp[base..end],
        }
    }
    pub fn tune(&self) -> &[SignedNoteFxP] {
        self.tune_fxp.as_slice()
    }
    pub fn tune_mut(&mut self) -> &mut [SignedNoteFxP] {
        self.tune_fxp.as_mut_slice()
    }
    pub fn tune_float(&self) -> &[f32] {
        self.tune.as_slice()
    }
    pub fn shape(&self) -> &[ScalarFxP] {
        self.shape_fxp.as_slice()
    }
    pub fn shape_mut(&mut self) -> &mut [ScalarFxP] {
        self.shape_fxp.as_mut_slice()
    }
    pub fn shape_float(&self) -> &[f32] {
        self.shape.as_slice()
    }
    pub fn sin(&self) -> &[ScalarFxP] {
        self.sin_fxp.as_slice()
    }
    pub fn sin_mut(&mut self) -> &mut [ScalarFxP] {
        self.sin_fxp.as_mut_slice()
    }
    pub fn sin_float(&self) -> &[f32] {
        self.sin.as_slice()
    }
    pub fn sq(&self) -> &[ScalarFxP] {
        self.sq_fxp.as_slice()
    }
    pub fn sq_mut(&mut self) -> &mut [ScalarFxP] {
        self.sq_fxp.as_mut_slice()
    }
    pub fn sq_float(&self) -> &[f32] {
        self.sq.as_slice()
    }
    pub fn tri(&self) -> &[ScalarFxP] {
        self.tri_fxp.as_slice()
    }
    pub fn tri_mut(&mut self) -> &mut [ScalarFxP] {
        self.tri_fxp.as_mut_slice()
    }
    pub fn tri_float(&self) -> &[f32] {
        self.tri.as_slice()
    }
    pub fn saw(&self) -> &[ScalarFxP] {
        self.saw_fxp.as_slice()
    }
    pub fn saw_mut(&mut self) -> &mut [ScalarFxP] {
        self.saw_fxp.as_mut_slice()
    }
    pub fn saw_float(&self) -> &[f32] {
        self.saw.as_slice()
    }
    pub fn update_index(&mut self, idx: usize, p: &OscPluginParams) {
        self.shape_fxp[idx] = ScalarFxP::from_bits(p.shape.smoothed.next() as u16);
        self.sin_fxp[idx] = ScalarFxP::from_bits(p.sin.smoothed.next() as u16);
        self.sq_fxp[idx] = ScalarFxP::from_bits(p.sq.smoothed.next() as u16);
        self.tri_fxp[idx] = ScalarFxP::from_bits(p.tri.smoothed.next() as u16);
        self.saw_fxp[idx] = ScalarFxP::from_bits(p.saw.smoothed.next() as u16);
        self.tune_fxp[idx] = SignedNoteFxP::from_bits(
            ((p.course.smoothed.next() << 9) + p.fine.smoothed.next()) as i16,
        )
    }
}

#[derive(Default)]
pub struct LfoParamBuffer {
    freq: Vec<f32>,
    depth: Vec<f32>,
    opts: Vec<LfoOptions>,
    freq_fxp: Vec<LfoFreqFxP>,
    depth_fxp: Vec<ScalarFxP>,
}

impl LfoParamBuffer {
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
        self.freq_fxp.resize(sz as usize, LfoFreqFxP::ONE);
        self.depth_fxp.resize(sz as usize, ScalarFxP::MAX);
    }
    pub fn conv_float(&mut self) {
        for i in 0..self.len() {
            self.freq[i] = self.freq_fxp[i].to_num();
            self.depth[i] = self.depth_fxp[i].to_num();
        }
    }
    pub fn params_float(&self, base: usize, end: usize) -> LfoParams<f32> {
        LfoParams {
            freq: &self.freq[base..end],
            depth: &self.depth[base..end],
            opts: &self.opts[base..end],
        }
    }
    pub fn params_float_mut(&mut self, base: usize, end: usize) -> MutLfoParams<f32> {
        MutLfoParams {
            freq: &mut self.freq[base..end],
            depth: &mut self.depth[base..end],
            opts: &mut self.opts[base..end],
        }
    }
    pub fn params(&self, base: usize, end: usize) -> LfoParamsFxP {
        LfoParamsFxP {
            freq: &self.freq_fxp[base..end],
            depth: &self.depth_fxp[base..end],
            opts: &self.opts[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutLfoParamsFxP {
        MutLfoParamsFxP {
            freq: &mut self.freq_fxp[base..end],
            depth: &mut self.depth_fxp[base..end],
            opts: &mut self.opts[base..end],
        }
    }
    pub fn freq(&self) -> &[LfoFreqFxP] {
        self.freq_fxp.as_slice()
    }
    pub fn freq_mut(&mut self) -> &mut [LfoFreqFxP] {
        self.freq_fxp.as_mut_slice()
    }
    pub fn freq_float(&self) -> &[f32] {
        self.freq.as_slice()
    }
    pub fn depth(&self) -> &[ScalarFxP] {
        self.depth_fxp.as_slice()
    }
    pub fn depth_mut(&mut self) -> &mut [ScalarFxP] {
        self.depth_fxp.as_mut_slice()
    }
    pub fn depth_float(&self) -> &[f32] {
        self.depth.as_slice()
    }
    pub fn opts(&self) -> &[LfoOptions] {
        self.opts.as_slice()
    }
    pub fn opts_mut(&mut self) -> &mut [LfoOptions] {
        self.opts.as_mut_slice()
    }
    pub fn update_index(&mut self, idx: usize, p: &LfoPluginParams) {
        self.freq_fxp[idx] = LfoFreqFxP::from_bits(p.rate.smoothed.next() as u16);
        self.depth_fxp[idx] = ScalarFxP::from_bits(p.depth.smoothed.next() as u16);
        self.opts[idx] = LfoOptions::from(p);
    }
}

#[derive(Default)]
pub struct FiltParamBuffer {
    env_mod: Vec<f32>,
    vel_mod: Vec<f32>,
    kbd: Vec<f32>,
    cutoff: Vec<f32>,
    resonance: Vec<f32>,
    low_mix: Vec<f32>,
    band_mix: Vec<f32>,
    high_mix: Vec<f32>,
    env_mod_fxp: Vec<ScalarFxP>,
    vel_mod_fxp: Vec<ScalarFxP>,
    kbd_fxp: Vec<ScalarFxP>,
    cutoff_fxp: Vec<NoteFxP>,
    resonance_fxp: Vec<ScalarFxP>,
    low_mix_fxp: Vec<ScalarFxP>,
    band_mix_fxp: Vec<ScalarFxP>,
    high_mix_fxp: Vec<ScalarFxP>,
}

impl FiltParamBuffer {
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
        self.cutoff_fxp.resize(sz as usize, NoteFxP::ZERO);
        for buf in [
            &mut self.env_mod_fxp,
            &mut self.vel_mod_fxp,
            &mut self.kbd_fxp,
            &mut self.resonance_fxp,
            &mut self.low_mix_fxp,
            &mut self.band_mix_fxp,
            &mut self.high_mix_fxp,
        ] {
            buf.resize(sz as usize, ScalarFxP::ZERO);
        }
    }
    pub fn conv_float(&mut self) {
        for i in 0..self.len() {
            self.env_mod[i] = self.env_mod_fxp[i].to_num();
            self.vel_mod[i] = self.vel_mod_fxp[i].to_num();
            self.kbd[i] = self.kbd_fxp[i].to_num();
            self.cutoff[i] = self.cutoff_fxp[i].to_num();
            self.resonance[i] = self.resonance_fxp[i].to_num();
            self.low_mix[i] = self.low_mix_fxp[i].to_num();
            self.band_mix[i] = self.band_mix_fxp[i].to_num();
            self.high_mix[i] = self.high_mix_fxp[i].to_num();
        }
    }
    pub fn params_float(&self, base: usize, end: usize) -> ModFiltParams<f32> {
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
    pub fn params_float_mut(&mut self, base: usize, end: usize) -> MutModFiltParams<f32> {
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
    pub fn params(&self, base: usize, end: usize) -> ModFiltParamsFxP {
        ModFiltParamsFxP {
            env_mod: &self.env_mod_fxp[base..end],
            vel_mod: &self.vel_mod_fxp[base..end],
            kbd: &self.kbd_fxp[base..end],
            cutoff: &self.cutoff_fxp[base..end],
            resonance: &self.resonance_fxp[base..end],
            low_mix: &self.low_mix_fxp[base..end],
            band_mix: &self.band_mix_fxp[base..end],
            high_mix: &self.high_mix_fxp[base..end],
        }
    }
    pub fn params_mut(&mut self, base: usize, end: usize) -> MutModFiltParamsFxP {
        MutModFiltParamsFxP {
            env_mod: &mut self.env_mod_fxp[base..end],
            vel_mod: &mut self.vel_mod_fxp[base..end],
            kbd: &mut self.kbd_fxp[base..end],
            cutoff: &mut self.cutoff_fxp[base..end],
            resonance: &mut self.resonance_fxp[base..end],
            low_mix: &mut self.low_mix_fxp[base..end],
            band_mix: &mut self.band_mix_fxp[base..end],
            high_mix: &mut self.high_mix_fxp[base..end],
        }
    }
    pub fn env_mod(&self) -> &[ScalarFxP] {
        self.env_mod_fxp.as_slice()
    }
    pub fn env_mod_mut(&mut self) -> &mut [ScalarFxP] {
        self.env_mod_fxp.as_mut_slice()
    }
    pub fn env_mod_float(&self) -> &[f32] {
        self.env_mod.as_slice()
    }
    pub fn vel_mod(&self) -> &[ScalarFxP] {
        self.vel_mod_fxp.as_slice()
    }
    pub fn vel_mod_mut(&mut self) -> &mut [ScalarFxP] {
        self.vel_mod_fxp.as_mut_slice()
    }
    pub fn vel_mod_float(&self) -> &[f32] {
        self.vel_mod.as_slice()
    }
    pub fn kbd(&self) -> &[ScalarFxP] {
        self.kbd_fxp.as_slice()
    }
    pub fn kbd_mut(&mut self) -> &mut [ScalarFxP] {
        self.kbd_fxp.as_mut_slice()
    }
    pub fn kbd_float(&self) -> &[f32] {
        self.kbd.as_slice()
    }
    pub fn cutoff(&self) -> &[NoteFxP] {
        self.cutoff_fxp.as_slice()
    }
    pub fn cutoff_mut(&mut self) -> &mut [NoteFxP] {
        self.cutoff_fxp.as_mut_slice()
    }
    pub fn cutoff_float(&self) -> &[f32] {
        self.cutoff.as_slice()
    }
    pub fn res(&self) -> &[ScalarFxP] {
        self.resonance_fxp.as_slice()
    }
    pub fn res_mut(&mut self) -> &mut [ScalarFxP] {
        self.resonance_fxp.as_mut_slice()
    }
    pub fn res_fxp(&self) -> &[f32] {
        self.resonance.as_slice()
    }
    pub fn low_mix(&self) -> &[ScalarFxP] {
        self.low_mix_fxp.as_slice()
    }
    pub fn low_mix_mut(&mut self) -> &mut [ScalarFxP] {
        self.low_mix_fxp.as_mut_slice()
    }
    pub fn low_mix_float(&self) -> &[f32] {
        self.low_mix.as_slice()
    }
    pub fn band_mix(&self) -> &[ScalarFxP] {
        self.band_mix_fxp.as_slice()
    }
    pub fn band_mix_mut(&mut self) -> &mut [ScalarFxP] {
        self.band_mix_fxp.as_mut_slice()
    }
    pub fn band_mix_float(&self) -> &[f32] {
        self.band_mix.as_slice()
    }
    pub fn high_mix(&self) -> &[ScalarFxP] {
        self.high_mix_fxp.as_slice()
    }
    pub fn high_mix_mut(&mut self) -> &mut [ScalarFxP] {
        self.high_mix_fxp.as_mut_slice()
    }
    pub fn high_mix_float(&self) -> &[f32] {
        self.high_mix.as_slice()
    }
    pub fn update_index(&mut self, idx: usize, p: &FiltPluginParams) {
        self.env_mod_fxp[idx] = ScalarFxP::from_bits(p.env.smoothed.next() as u16);
        self.vel_mod_fxp[idx] = ScalarFxP::from_bits(p.vel.smoothed.next() as u16);
        self.kbd_fxp[idx] = ScalarFxP::from_bits(p.kbd.smoothed.next() as u16);
        self.cutoff_fxp[idx] = NoteFxP::from_bits(p.cutoff.smoothed.next() as u16);
        self.resonance_fxp[idx] = ScalarFxP::from_bits(p.res.smoothed.next() as u16);
        self.low_mix_fxp[idx] = ScalarFxP::from_bits(p.low.smoothed.next() as u16);
        self.band_mix_fxp[idx] = ScalarFxP::from_bits(p.band.smoothed.next() as u16);
        self.high_mix_fxp[idx] = ScalarFxP::from_bits(p.high.smoothed.next() as u16);
    }
}
