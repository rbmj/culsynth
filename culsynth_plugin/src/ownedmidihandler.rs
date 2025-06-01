use culsynth::{
    voice::{
        cc,
        modulation::{ModDest, ModMatrix},
        nrpn::NrpnMsb,
        VoiceParams,
    },
    SignedNoteFxP,
};
use culsynth::{Fixed16, IScalarFxP};
use std::cell::RefCell;
use wmidi::MidiMessage;

use crate::{MidiHandler, Tuning};

struct MidiData {
    params: VoiceParams<i16>,
    tuning: (Tuning, Tuning),
    matrix: ModMatrix<i16>,
    nrpn_lsb: wmidi::U7,
    nrpn_msb: wmidi::U7,
    data_msb: wmidi::U7,
}

impl MidiData {
    fn handle_nrpn(&mut self, data_lsb: wmidi::U7) {
        match NrpnMsb::from_msb(self.nrpn_msb) {
            Some(NrpnMsb::Cc) => {
                // This will be treated as a high fidelity CC, but leaving unimplemented for now
            }
            Some(NrpnMsb::Modulation(src)) => {
                // Modulation
                if let Some(dest) = ModDest::from_u7(data_lsb) {
                    let msb: u8 = self.data_msb.into();
                    let lsb: u8 = data_lsb.into();
                    let mut value = msb as i16;
                    value <<= 7;
                    value |= lsb as i16;
                    value -= 1 << 13;
                    value <<= 2;
                    *self.matrix.slot_mut(src, dest) = IScalarFxP::from_bits(value);
                }
            }
            _ => {}
        }
    }
}

pub struct OwnedMidiHandler {
    data: RefCell<MidiData>,
    channel: wmidi::Channel,
}

impl OwnedMidiHandler {
    pub fn new(channel: wmidi::Channel) -> Self {
        OwnedMidiHandler {
            data: RefCell::new(MidiData {
                params: VoiceParams::default(),
                tuning: (Tuning::default(), Tuning::default()),
                matrix: ModMatrix::default(),
                nrpn_lsb: wmidi::U7::default(),
                nrpn_msb: wmidi::U7::default(),
                data_msb: wmidi::U7::default(),
            }),
            channel,
        }
    }
    pub fn get_params(&self) -> VoiceParams<i16> {
        let mut data = self.data.borrow_mut();
        let osc1_tune = SignedNoteFxP::from_num(data.tuning.0.coarse)
            + SignedNoteFxP::from_num(data.tuning.0.fine);
        let osc2_tune = SignedNoteFxP::from_num(data.tuning.1.coarse)
            + SignedNoteFxP::from_num(data.tuning.1.fine);
        data.params.oscs_p.primary.tune = osc1_tune;
        data.params.oscs_p.secondary.tune = osc2_tune;
        data.params.clone()
    }
    pub fn get_matrix(&self) -> ModMatrix<i16> {
        self.data.borrow().matrix.clone()
    }
    pub fn get_tuning(&self) -> (Tuning, Tuning) {
        self.data.borrow().tuning.clone()
    }
}

impl MidiHandler for OwnedMidiHandler {
    fn send(&self, msg: MidiMessage<'static>) {
        if let MidiMessage::ControlChange(_, cc, value) = msg {
            let mut d = self.data.borrow_mut();
            match cc {
                cc::OSC1_COARSE => d.tuning.0.coarse.set_from_u7(value),
                cc::OSC1_FINE => d.tuning.0.fine.set_from_u7(value),
                cc::OSC2_COARSE => d.tuning.1.coarse.set_from_u7(value),
                cc::OSC2_FINE => d.tuning.1.fine.set_from_u7(value),
                wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_LSB => {
                    d.nrpn_lsb = value.into();
                }
                wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_MSB => {
                    d.nrpn_msb = value;
                }
                wmidi::ControlFunction::DATA_ENTRY_MSB => {
                    d.data_msb = value;
                }
                wmidi::ControlFunction::DATA_ENTRY_LSB => {
                    d.handle_nrpn(value);
                }
                _ => {
                    d.params.apply_cc(cc, value);
                }
            }
        }
    }
    fn ch(&self) -> wmidi::Channel {
        self.channel
    }
}
