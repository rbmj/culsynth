use crate::devices::*;
use super::{BufferT, STATIC_BUFFER_SIZE};
use super::{SampleFxP, NoteFxP};

pub struct VoiceFxP {
    osc: MixOscFxP,
    filt: ModFiltFxP,
    env_amp: EnvFxP,
    env_filt: EnvFxP,
    vca: AmpFxP,
    
    vcabuf: BufferT<SampleFxP>
}

impl VoiceFxP {
    pub fn new() -> Self {
        Self {
            osc: MixOscFxP::new(),
            filt: ModFiltFxP::new(),
            env_amp: EnvFxP::new(),
            env_filt: EnvFxP::new(),
            vca: AmpFxP::new(),
            vcabuf: [SampleFxP::ZERO; STATIC_BUFFER_SIZE]
        }
    }
    pub fn process(&mut self, note: &[NoteFxP], gate: &[SampleFxP],
        op: MixOscParamsFxP, fp: ModFiltParamsFxP,
        fep: EnvParamsFxP, aep: EnvParamsFxP)
        -> &[SampleFxP]
    {
        let numsamples = *[note.len(), gate.len(), op.len(), fp.len(),
            fep.len(), aep.len()].iter().min().unwrap();
        let osc_out = self.osc.process(&note[0..numsamples], op);
        let filt_env_out = self.env_filt.process(&gate[0..numsamples], fep);
        let filt_out = self.filt.process(osc_out, filt_env_out, note, fp);
        let vca_env_out = self.env_amp.process(&gate[0..numsamples], aep);
        for i in 0..vca_env_out.len() {
            self.vcabuf[i] = SampleFxP::from_num(vca_env_out[i]);
        }
        self.vca.process(filt_out, &self.vcabuf[0..vca_env_out.len()])
    }
}

impl Default for VoiceFxP {
    fn default() -> Self {
        Self::new()
    }
}