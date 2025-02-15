use culsynth::{
    voice::{
        cc,
        modulation::{ModDest, ModMatrix},
        VoiceParams,
    },
    CoarseTuneFxP, SignedNoteFxP,
};
use culsynth::{Fixed16, IScalarFxP};
use std::cell::RefCell;
use wmidi::MidiMessage;

use crate::{MidiHandler, Tuning};

struct MidiData {
    params: VoiceParams<i16>,
    tuning: (Tuning, Tuning),
    matrix: ModMatrix<i16>,
    nrpn_lsb: u8,
    nrpn_msb: wmidi::U7,
    data_msb: u8,
}

impl MidiData {
    fn handle_nrpn(&mut self, data_lsb: u8) {
        match self.nrpn_msb {
            cc::NRPN_CATEGORY_CC => {
                // This will be treated as a high fidelity CC, but leaving unimplemented for now
            }
            cc::NRPN_CATEGORY_MODDEST => {
                // Mod matrix destination
                if let Some((src, slot)) = self.matrix.nrpn_to_slot(self.nrpn_lsb as u8) {
                    let mut dest = self.data_msb as u16;
                    dest <<= 7;
                    dest |= data_lsb as u16;
                    if let Ok(mut mod_dest) = ModDest::try_from(dest) {
                        if src.is_secondary() {
                            mod_dest = mod_dest.remove_secondary_invalid_dest()
                        }
                        self.matrix.get_mut(src, slot).map(|x| x.0 = mod_dest);
                    }
                }
            }
            cc::NRPN_CATEGORY_MODMAG => {
                // Mod matrix magnitude
                if let Some((src, slot)) = self.matrix.nrpn_to_slot(self.nrpn_lsb as u8) {
                    let mut mag = self.data_msb as i16;
                    mag <<= 7;
                    mag |= data_lsb as i16;
                    mag -= 1 << 13;
                    mag <<= 2;
                    self.matrix.get_mut(src, slot).map(|x| x.1 = IScalarFxP::from_bits(mag));
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
                nrpn_lsb: 0,
                nrpn_msb: wmidi::U7::default(),
                data_msb: 0,
            }),
            channel,
        }
    }
    pub fn get_params(&self) -> VoiceParams<i16> {
        let mut data = self.data.borrow_mut();
        let osc1_tune = SignedNoteFxP::from_num(
            data.tuning.0.coarse + CoarseTuneFxP::from_num(data.tuning.0.fine),
        );
        let osc2_tune = SignedNoteFxP::from_num(
            data.tuning.1.coarse + CoarseTuneFxP::from_num(data.tuning.1.fine),
        );
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
                    d.data_msb = value.into();
                }
                wmidi::ControlFunction::DATA_ENTRY_LSB => {
                    d.handle_nrpn(value.into());
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
