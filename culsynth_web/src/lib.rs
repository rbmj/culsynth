use std::cell::RefCell;

use culsynth_plugin::backend::voice::cc::OSC1_COARSE;
use culsynth_plugin::backend::voice::modulation::ModDest;
use culsynth_plugin::backend::voice::{cc, modulation::ModMatrix, VoiceParams};
use culsynth_plugin::backend::{Fixed16, IScalarFxP};
use culsynth_plugin::editor::Editor;
use culsynth_plugin::{ContextReader, MidiHandler, Tuning, VoiceMode};
use wasm_bindgen::prelude::*;
use web_sys::AudioContext;
use wmidi::MidiMessage;

pub struct WebAudioContext {
    ctx: AudioContext,
    fixed: bool,
    voice_mode: VoiceMode,
}

impl WebAudioContext {
    const WEB_AUDIO_SAMPLES_PER_BUFFER: usize = 128;
    fn new() -> Result<Self, JsValue> {
        Ok(Self {
            ctx: AudioContext::new()?,
            fixed: false,
            voice_mode: VoiceMode::Mono,
        })
    }
}

impl ContextReader for WebAudioContext {
    fn sample_rate(&self) -> u32 {
        self.ctx.sample_rate() as u32
    }
    fn is_fixed(&self) -> bool {
        self.fixed
    }
    fn bufsz(&self) -> usize {
        128
    }
    fn voice_mode(&self) -> VoiceMode {
        self.voice_mode
    }
}

pub struct WebMidiHandler {
    messages: RefCell<Vec<MidiMessage<'static>>>,
    nrpn_lsb: u8,
    nrpn_msb: wmidi::U7,
    data_msb: u8,
}

impl Default for WebMidiHandler {
    fn default() -> Self {
        Self {
            messages: RefCell::new(Vec::with_capacity(32)),
            nrpn_lsb: 0,
            nrpn_msb: wmidi::U7::from_u8_lossy(0),
            data_msb: 0,
        }
    }
}

impl MidiHandler for WebMidiHandler {
    fn send(&self, msg: MidiMessage<'static>) {
        self.messages.borrow_mut().push(msg);
    }
    fn ch(&self) -> wmidi::Channel {
        wmidi::Channel::Ch1
    }
}

impl WebMidiHandler {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn handle_messages(
        &mut self,
        params: &mut VoiceParams<i16>,
        matrix: &mut ModMatrix<i16>,
        tuning: &mut (Tuning, Tuning),
    ) {
        for msg in self.messages.get_mut().iter() {
            if let MidiMessage::ControlChange(_, cc, value) = *msg {
                match cc {
                    cc::OSC1_COARSE => tuning.0.coarse.set_from_u7(value),
                    cc::OSC1_FINE => tuning.0.fine.set_from_u7(value),
                    cc::OSC2_COARSE => tuning.1.coarse.set_from_u7(value),
                    cc::OSC2_FINE => tuning.1.fine.set_from_u7(value),
                    wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_LSB => {
                        self.nrpn_lsb = value.into();
                    }
                    wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_MSB => {
                        self.nrpn_msb = value;
                    }
                    wmidi::ControlFunction::DATA_ENTRY_MSB => {
                        self.data_msb = value.into();
                    }
                    wmidi::ControlFunction::DATA_ENTRY_LSB => {
                        let data_lsb: u8 = value.into();
                        match self.nrpn_msb {
                            cc::NRPN_CATEGORY_CC => {
                                // This will be treated as a high fidelity CC, but leaving unimplemented for now
                            }
                            cc::NRPN_CATEGORY_MODDEST => {
                                // Mod matrix destination
                                if let Some((src, slot)) = matrix.nrpn_to_slot(self.nrpn_lsb as u8)
                                {
                                    let mut dest = self.data_msb as u16;
                                    dest <<= 7;
                                    dest |= data_lsb as u16;
                                    if let Ok(mut mod_dest) = ModDest::try_from(dest) {
                                        if src.is_secondary() {
                                            mod_dest = mod_dest.remove_secondary_invalid_dest()
                                        }
                                        matrix.get_mut(src, slot).map(|x| x.0 = mod_dest);
                                    }
                                }
                            }
                            cc::NRPN_CATEGORY_MODMAG => {
                                // Mod matrix magnitude
                                if let Some((src, slot)) = matrix.nrpn_to_slot(self.nrpn_lsb as u8)
                                {
                                    let mut mag = self.data_msb as i16;
                                    mag <<= 7;
                                    mag |= data_lsb as i16;
                                    mag -= 1 << 13;
                                    mag <<= 2;
                                    matrix
                                        .get_mut(src, slot)
                                        .map(|x| x.1 = IScalarFxP::from_bits(mag));
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        params.apply_cc(cc, value);
                    }
                }
            }
        }
        self.messages.get_mut().clear();
    }
}

pub struct SynthApp {
    audio_context: WebAudioContext,
    editor: Editor,
    params: VoiceParams<i16>,
    mod_matrix: ModMatrix<i16>,
    midi_handler: WebMidiHandler,
    tuning: (Tuning, Tuning),
}

impl SynthApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>, audioctx: WebAudioContext) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Self {
            audio_context: audioctx,
            editor: Editor::new(),
            params: VoiceParams::default(),
            mod_matrix: ModMatrix::default(),
            midi_handler: WebMidiHandler::default(),
            tuning: (Tuning::default(), Tuning::default()),
        }
    }
}

impl eframe::App for SynthApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.editor.update(
            ctx,
            &self.audio_context,
            &self.params,
            self.tuning,
            &self.mod_matrix,
            &self.midi_handler,
            Editor::null_sender(),
        );
        self.midi_handler
            .handle_messages(&mut self.params, &mut self.mod_matrix, &mut self.tuning);
    }
}

#[wasm_bindgen(start)]
fn start_app() -> Result<(), JsValue> {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().expect("No window").document().expect("No document");

        let canvas = document
            .get_element_by_id("culsynth_canvas")
            .expect("Failed to find drawing canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Canvas was not a HtmlCanvasElement");

        let context = WebAudioContext::new().expect("Failed to create audio context");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(move |cc| Ok(Box::new(SynthApp::new(cc, context)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
    Ok(())
}
