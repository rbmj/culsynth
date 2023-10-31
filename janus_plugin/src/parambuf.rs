use super::{EnvPluginParams, FiltPluginParams, OscPluginParams};
use janus::devices::{
    EnvParams, EnvParamsFxP, MixOscParams, MixOscParamsFxP, ModFiltParams, ModFiltParamsFxP,
};
use janus::{EnvParamFxP, NoteFxP, ScalarFxP};

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
    pub fn params(&self, base: usize, end: usize) -> EnvParamsFxP {
        EnvParamsFxP {
            attack: &self.attack_fxp[base..end],
            decay: &self.decay_fxp[base..end],
            sustain: &self.sustain_fxp[base..end],
            release: &self.release_fxp[base..end],
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
pub struct OscParamBuffer {
    shape: Vec<f32>,
    sin: Vec<f32>,
    sq: Vec<f32>,
    tri: Vec<f32>,
    saw: Vec<f32>,
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
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        self.shape.resize(sz as usize, 0f32);
        self.sin.resize(sz as usize, 0f32);
        self.sq.resize(sz as usize, 0f32);
        self.tri.resize(sz as usize, 0f32);
        self.saw.resize(sz as usize, 0f32);
        self.shape_fxp.resize(sz as usize, ScalarFxP::ZERO);
        self.sin_fxp.resize(sz as usize, ScalarFxP::ZERO);
        self.sq_fxp.resize(sz as usize, ScalarFxP::ZERO);
        self.tri_fxp.resize(sz as usize, ScalarFxP::ZERO);
        self.saw_fxp.resize(sz as usize, ScalarFxP::ZERO);
    }
    pub fn conv_float(&mut self) {
        for i in 0..self.len() {
            self.shape[i] = self.shape_fxp[i].to_num();
            self.sin[i] = self.sin_fxp[i].to_num();
            self.sq[i] = self.sq_fxp[i].to_num();
            self.tri[i] = self.tri_fxp[i].to_num();
            self.saw[i] = self.saw_fxp[i].to_num();
        }
    }
    pub fn params_float(&self, base: usize, end: usize) -> MixOscParams<f32> {
        MixOscParams {
            shape: &self.shape[base..end],
            sin: &self.sin[base..end],
            sq: &self.sq[base..end],
            tri: &self.tri[base..end],
            saw: &self.saw[base..end],
        }
    }
    pub fn params(&self, base: usize, end: usize) -> MixOscParamsFxP {
        MixOscParamsFxP {
            shape: &self.shape_fxp[base..end],
            sin: &self.sin_fxp[base..end],
            sq: &self.sq_fxp[base..end],
            tri: &self.tri_fxp[base..end],
            saw: &self.saw_fxp[base..end],
        }
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
    }
}

#[derive(Default)]
pub struct FiltParamBuffer {
    env_mod: Vec<f32>,
    kbd: Vec<f32>,
    cutoff: Vec<f32>,
    resonance: Vec<f32>,
    low_mix: Vec<f32>,
    band_mix: Vec<f32>,
    high_mix: Vec<f32>,
    env_mod_fxp: Vec<ScalarFxP>,
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
    pub fn allocate(&mut self, sz: u32) {
        if self.len() >= sz as usize {
            return;
        }
        for buf in [
            &mut self.env_mod,
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
            kbd: &self.kbd[base..end],
            cutoff: &self.cutoff[base..end],
            resonance: &self.resonance[base..end],
            low_mix: &self.low_mix[base..end],
            band_mix: &self.band_mix[base..end],
            high_mix: &self.high_mix[base..end],
        }
    }
    pub fn params(&self, base: usize, end: usize) -> ModFiltParamsFxP {
        ModFiltParamsFxP {
            env_mod: &self.env_mod_fxp[base..end],
            kbd: &self.kbd_fxp[base..end],
            cutoff: &self.cutoff_fxp[base..end],
            resonance: &self.resonance_fxp[base..end],
            low_mix: &self.low_mix_fxp[base..end],
            band_mix: &self.band_mix_fxp[base..end],
            high_mix: &self.high_mix_fxp[base..end],
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
        self.kbd_fxp[idx] = ScalarFxP::from_bits(p.kbd.smoothed.next() as u16);
        self.cutoff_fxp[idx] = NoteFxP::from_bits(p.cutoff.smoothed.next() as u16);
        self.resonance_fxp[idx] = ScalarFxP::from_bits(p.res.smoothed.next() as u16);
        self.low_mix_fxp[idx] = ScalarFxP::from_bits(p.low.smoothed.next() as u16);
        self.band_mix_fxp[idx] = ScalarFxP::from_bits(p.band.smoothed.next() as u16);
        self.high_mix_fxp[idx] = ScalarFxP::from_bits(p.high.smoothed.next() as u16);
    }
}
