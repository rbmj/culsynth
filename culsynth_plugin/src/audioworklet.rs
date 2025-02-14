extern crate console_error_panic_hook;

use crate::ownedmidihandler::OwnedMidiHandler;
use crate::voicealloc::{PolySynth, VoiceAllocator};
use crate::MidiHandler;
use culsynth::context::Context;
use log::info;
use wasm_bindgen::prelude::*;
use wmidi::MidiMessage;

#[wasm_bindgen]
pub struct SynthWorklet {
    voicealloc: PolySynth<f32>,
    params: OwnedMidiHandler,
    debug_counter: u16,
}

#[wasm_bindgen]
impl SynthWorklet {
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32, channel: u8) -> Self {
        let context = Context::<f32>::new(sample_rate);
        info!("Worklet Created");
        Self {
            voicealloc: PolySynth::new(context, 4),
            params: OwnedMidiHandler::new(wmidi::Channel::from_index(channel - 1).unwrap()),
            debug_counter: 0,
        }
    }
    #[wasm_bindgen]
    pub fn on_message_js(&mut self, msg: u32) {
        if let Err(e) = MidiMessage::from_bytes(msg.to_be_bytes().as_slice()).map(|m| {
            if let Some(midi) = m.drop_unowned_sysex() {
                self.params.send(midi.clone());
                self.voicealloc.handle_midi(midi);
            }
        }) {
            log::error!("MIDI parsing error: {:?}", e);
        }
    }
    #[wasm_bindgen]
    pub fn process(&mut self, audio: &mut [f32]) {
        self.debug_counter = self.debug_counter.wrapping_add(1);
        let params = self.params.get_params();
        let mut matrix = Some(self.params.get_matrix());
        for smp in audio.iter_mut() {
            *smp = self.voicealloc.next(&params, matrix.take().as_ref())
        }
        let chunk_size = 4;
    }
}

#[wasm_bindgen(start)]
fn init_module() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);
    info!("Init WebAudioWorklet");
}
