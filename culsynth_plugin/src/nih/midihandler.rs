use super::CulSynthParams;
use crate::MidiHandler;
use culsynth::voice::{modulation::ModDest, nrpn::NrpnMsb};
use nih_plug::context::gui::ParamSetter;
use nih_plug::nih_log;
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
use wmidi::{Channel, ControlFunction, MidiMessage, U7};

pub struct MidiSender {
    pub tx: std::sync::mpsc::SyncSender<MidiMessage<'static>>,
    pub ch: wmidi::Channel,
}

impl MidiHandler for MidiSender {
    fn send(&self, msg: MidiMessage<'static>) {
        if let Err(e) = self.tx.try_send(msg) {
            nih_log!("Error sending MIDI message: {}", e);
        }
    }
    fn ch(&self) -> wmidi::Channel {
        self.ch
    }
}

pub struct MidiHandlerFactory {
    channel: wmidi::Channel,
    nrpn_cc_msb: AtomicU8,
    nrpn_cc_lsb: AtomicU8,
    nrpn_value_msb: AtomicU8,
}

impl MidiHandlerFactory {
    pub fn new(channel: wmidi::Channel) -> Self {
        Self {
            channel,
            nrpn_cc_msb: AtomicU8::new(0),
            nrpn_cc_lsb: AtomicU8::new(0),
            nrpn_value_msb: AtomicU8::new(0),
        }
    }
    pub fn create<'a>(
        &'a self,
        to_voice: &'a dyn MidiHandler,
        params: &'a CulSynthParams,
        setter: &'a ParamSetter<'a>,
    ) -> PluginMidiHandler<'a> {
        PluginMidiHandler {
            to_voice,
            params,
            setter,
            parent: self,
        }
    }
    pub fn ch(&self) -> wmidi::Channel {
        self.channel
    }
}

pub struct PluginMidiHandler<'a> {
    to_voice: &'a dyn MidiHandler,
    params: &'a CulSynthParams,
    setter: &'a ParamSetter<'a>,
    parent: &'a MidiHandlerFactory,
}

impl<'a> PluginMidiHandler<'a> {
    fn handle_nrpn(&self, val: U7) {
        let val_lsb: u8 = val.into();
        let val_msb = self.parent.nrpn_value_msb.load(Relaxed);
        let value = ((val_msb as u16) << 7) | val_lsb as u16;
        let msb = wmidi::U7::from_u8_lossy(self.parent.nrpn_cc_msb.load(Relaxed));
        let lsb = self.parent.nrpn_cc_lsb.load(Relaxed);
        const MAX_VAL: f32 = ((1 << 14) - 1) as f32;
        let norm_value = (value as f32) / MAX_VAL;
        match NrpnMsb::from_msb(msb) {
            Some(NrpnMsb::Cc) => {
                // If MSB of NRPN is 0, treat as high-fidelity CC
                let new_cc = wmidi::ControlFunction(wmidi::U7::from_u8_lossy(lsb));
                // This will screw up LFOs, so ignore:
                if new_cc == culsynth::voice::cc::LFO1_WAVE
                    || new_cc == culsynth::voice::cc::LFO2_WAVE
                {
                    return;
                }
                if let Some(param) = self.params.int_param_from_cc(new_cc) {
                    self.setter.begin_set_parameter(param);
                    self.setter.set_parameter_normalized(param, norm_value);
                    self.setter.end_set_parameter(param);
                }
            }
            Some(NrpnMsb::Modulation(src)) => {
                //Treat as ModMatrix Destination Assignment
                if let Some(dst) = ModDest::from_u7(U7::from_u8_lossy(lsb)) {
                    let param = self.params.modmatrix.entry(src, dst);
                    self.setter.begin_set_parameter(param);
                    self.setter.set_parameter_normalized(param, norm_value);
                    self.setter.end_set_parameter(param);
                }
            }
            _ => {
                let msb: u8 = msb.into();
                nih_log!("Unhandled NRPN: {} {} {}", msb, lsb, value);
            }
        }
    }
    fn handle_cc(&self, ch: Channel, cc: ControlFunction, val: U7) {
        let val_raw: u8 = val.into();
        match cc {
            wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_LSB => {
                self.parent.nrpn_cc_lsb.store(val_raw, Relaxed);
            }
            wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_MSB => {
                self.parent.nrpn_cc_msb.store(val_raw, Relaxed);
            }
            wmidi::ControlFunction::DATA_ENTRY_MSB => {
                self.parent.nrpn_value_msb.store(val_raw, Relaxed);
            }
            wmidi::ControlFunction::DATA_ENTRY_LSB => {
                self.handle_nrpn(val);
            }
            culsynth::voice::cc::LFO1_WAVE => {
                self.setter.begin_set_parameter(&self.params.lfo1.wave);
                self.setter.set_parameter(&self.params.lfo1.wave, val_raw as i32);
                self.setter.end_set_parameter(&self.params.lfo1.wave);
            }
            culsynth::voice::cc::LFO2_WAVE => {
                self.setter.begin_set_parameter(&self.params.lfo2.wave);
                self.setter.set_parameter(&self.params.lfo2.wave, val_raw as i32);
                self.setter.end_set_parameter(&self.params.lfo2.wave);
            }
            cc => {
                if let Some(param) = self.params.int_param_from_cc(cc) {
                    self.setter.begin_set_parameter(param);
                    self.setter.set_parameter_normalized(param, val_raw as f32 / 127.);
                    self.setter.end_set_parameter(param);
                } else if let Some(param) = self.params.bool_param_from_cc(cc) {
                    self.setter.begin_set_parameter(param);
                    self.setter.set_parameter(param, val_raw > 64);
                    self.setter.end_set_parameter(param);
                } else {
                    self.to_voice.send(MidiMessage::ControlChange(ch, cc, val));
                }
            }
        }
    }
}

impl<'a> MidiHandler for PluginMidiHandler<'a> {
    fn send(&self, msg: MidiMessage<'static>) {
        match msg {
            wmidi::MidiMessage::ControlChange(ch, cc, val) => {
                if self.parent.channel == ch {
                    self.handle_cc(ch, cc, val);
                }
            }
            unhandled => {
                self.to_voice.send(unhandled);
            }
        }
    }
    fn ch(&self) -> wmidi::Channel {
        self.parent.channel
    }
}
