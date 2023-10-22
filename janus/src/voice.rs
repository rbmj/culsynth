use crate::devices::*;
use super::{BufferT, STATIC_BUFFER_SIZE};
use super::{SampleFxP, NoteFxP};

pub struct VoiceFxP {
    osc: OscFxP,
    filt: FiltFxP,
    env_amp: EnvFxP,
    //env_filt: EnvFxP,
    vca: AmpFxP,
    
    vcabuf: BufferT<SampleFxP>
}

impl VoiceFxP {
    pub fn new() -> Self {
        Self {
            osc: OscFxP::new(),
            filt: FiltFxP::new(),
            env_amp: EnvFxP::new(),
            //env_filt: EnvFxP::create(),
            vca: AmpFxP::new(),
            vcabuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE]
        }
    }
    pub fn process(&mut self, note: &[NoteFxP], gate: &[SampleFxP],
        op: OscParamsFxP, fp: FiltParamsFxP, aep: EnvParamsFxP)
        -> &[SampleFxP]
    {
        let osc_out = self.osc.process(note, op);
        let filt_out = self.filt.process(osc_out.saw, fp);
        let env_out = self.env_amp.process(gate, aep);
        for i in 0..env_out.len() {
            self.vcabuf[i] = SampleFxP::from_num(env_out[i]);
        }
        self.vca.process(filt_out.low, &self.vcabuf[0..env_out.len()])
    }
}

impl Default for VoiceFxP {
    fn default() -> Self {
        Self::new()
    }
}