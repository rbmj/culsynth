use janus::{EnvParamFxP, ScalarFxP, NoteFxP};
use janus::devices::{EnvParams, EnvParamsFxP, OscParams, OscParamsFxP, FiltParams, FiltParamsFxP};

#[derive(Default)]
pub struct EnvParamBuffer {
    attack: Vec<f32>,
    decay: Vec<f32>,
    sustain: Vec<f32>,
    release: Vec<f32>,
    attack_fxp: Vec<EnvParamFxP>,
    decay_fxp: Vec<EnvParamFxP>,
    sustain_fxp: Vec<ScalarFxP>,
    release_fxp: Vec<EnvParamFxP>
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
        for buf in [&mut self.attack, &mut self.decay, &mut self.sustain, &mut self.release] {
            buf.resize(sz as usize, 0f32);
        }
        for buf in [&mut self.attack_fxp, &mut self.decay_fxp, &mut self.release_fxp] {
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
            release: &self.release[base..end]
        }
    }
    pub fn params(&self, base: usize, end: usize) -> EnvParamsFxP {
        EnvParamsFxP {
            attack: &self.attack_fxp[base..end],
            decay: &self.decay_fxp[base..end],
            sustain: &self.sustain_fxp[base..end],
            release: &self.release_fxp[base..end]
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
}

#[derive(Default)]
pub struct OscParamBuffer {
    shape: Vec<f32>,
    shape_fxp: Vec<ScalarFxP>
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
        self.shape_fxp.resize(sz as usize, ScalarFxP::ZERO);
    }
    pub fn conv_float(&mut self) {
        for i in 0..self.len() {
            self.shape[i] = self.shape_fxp[i].to_num();
        }
    }
    pub fn params_float(&self, base: usize, end: usize) -> OscParams<f32> {
        OscParams {
            shape: &self.shape[base..end],
        }
    }
    pub fn params(&self, base: usize, end: usize) -> OscParamsFxP {
        OscParamsFxP {
            shape: &self.shape_fxp[base..end],
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
}

#[derive(Default)]
pub struct FiltParamBuffer {
    cutoff: Vec<f32>,
    resonance: Vec<f32>,
    cutoff_fxp: Vec<NoteFxP>,
    resonance_fxp: Vec<ScalarFxP>
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
        for buf in [&mut self.cutoff, &mut self.resonance] {
            buf.resize(sz as usize, 0f32);
        }
        self.cutoff_fxp.resize(sz as usize, NoteFxP::ZERO);
        self.resonance_fxp.resize(sz as usize, ScalarFxP::ZERO);
    }
    pub fn conv_float(&mut self) {
        for i in 0..self.len() {
            self.cutoff[i] = self.cutoff_fxp[i].to_num();
            self.resonance[i] = self.resonance_fxp[i].to_num();
        }
    }
    pub fn params_float(&self, base: usize, end: usize) -> FiltParams<f32> {
        FiltParams {
            cutoff: &self.cutoff[base..end],
            resonance: &self.resonance[base..end],
        }
    }
    pub fn params(&self, base: usize, end: usize) -> FiltParamsFxP {
        FiltParamsFxP {
            cutoff: &self.cutoff_fxp[base..end],
            resonance: &self.resonance_fxp[base..end],
        }
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
}