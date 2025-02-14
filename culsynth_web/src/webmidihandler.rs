use culsynth_plugin::backend::voice::{modulation::ModMatrix, VoiceParams};
use culsynth_plugin::ownedmidihandler::OwnedMidiHandler;
use culsynth_plugin::{MidiHandler, Tuning};
use log::error;
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use web_sys::AudioWorkletNode;
use wmidi::MidiMessage;

pub struct WebMidiHandler {
    owned: OwnedMidiHandler,
    node: AudioWorkletNode,
    sent_msg: RefCell<bool>,
}

impl WebMidiHandler {
    pub fn get_params(&self) -> VoiceParams<i16> {
        self.owned.get_params()
    }
    pub fn get_matrix(&self) -> ModMatrix<i16> {
        self.owned.get_matrix()
    }
    pub fn get_tuning(&self) -> (Tuning, Tuning) {
        self.owned.get_tuning()
    }
    pub fn new(node: AudioWorkletNode) -> Self {
        Self {
            owned: OwnedMidiHandler::new(wmidi::Channel::Ch1),
            node,
            sent_msg: RefCell::new(false),
        }
    }
    pub fn has_interacted(&self) -> bool {
        *self.sent_msg.borrow()
    }
}

impl MidiHandler for WebMidiHandler {
    fn send(&self, msg: MidiMessage<'static>) {
        if let wmidi::MidiMessage::NoteOn(_ch, _, _) = &msg {
            *self.sent_msg.borrow_mut() = true;
        }
        self.owned.send(msg.clone());
        let mut buf = [0u8; 4];
        if msg.copy_to_slice(buf.as_mut_slice()).is_ok() {
            let value = JsValue::from(u32::from_be_bytes(buf));
            if let Err(e) = self.node.port().map(|p| p.post_message(&value)) {
                error!("Unable to send message: {:?}", e);
            }
        }
    }
    fn ch(&self) -> wmidi::Channel {
        wmidi::Channel::Ch1
    }
}
