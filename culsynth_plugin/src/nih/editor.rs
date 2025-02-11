use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};

use super::CulSynthParams;
use crate::editor::Editor;
use crate::nih::midihandler::{MidiHandlerFactory, MidiSender};
use crate::voicealloc::VoiceAllocator;
use crate::MidiHandler;

use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(1000, 800)
}

struct PluginEditor {
    params: Arc<CulSynthParams>,
    context: super::PluginContextReader,
    midi_tx: SyncSender<wmidi::MidiMessage<'static>>,
    synth_tx: SyncSender<Box<dyn VoiceAllocator>>,
    cc_rx: Mutex<Receiver<wmidi::MidiMessage<'static>>>,
    editor: Editor,
    handler_factory: MidiHandlerFactory,
}

impl PluginEditor {
    fn initialize(&mut self, ctx: &egui::Context) {
        self.editor.initialize(ctx);
    }
    fn update(&mut self, ctx: &egui::Context, setter: &ParamSetter) {
        let to_voice = MidiSender {
            tx: self.midi_tx.clone(),
            ch: self.handler_factory.ch(),
        };
        let handler = self.handler_factory.create(&to_voice, &self.params, setter);
        if let Ok(channel) = self.cc_rx.get_mut() {
            while let Ok(msg) = channel.try_recv() {
                handler.send(msg);
            }
        }
        let tuning = (self.params.osc1.tuning(), self.params.osc2.tuning());
        let params: culsynth::voice::VoiceParams<i16> = (&*self.params).into();
        let matrix: culsynth::voice::modulation::ModMatrix<i16> = (&self.params.modmatrix).into();
        self.editor.update(
            ctx,
            &self.context,
            &params,
            tuning,
            &matrix,
            &handler,
            Some(&self.synth_tx),
        );
    }
}

pub fn create(
    params: Arc<CulSynthParams>,
    midi_tx: SyncSender<wmidi::MidiMessage<'static>>,
    synth_tx: SyncSender<Box<dyn VoiceAllocator>>,
    cc_rx: Receiver<wmidi::MidiMessage<'static>>,
    context: super::PluginContextReader,
) -> Option<Box<dyn nih_plug::editor::Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        PluginEditor {
            params,
            context,
            midi_tx,
            synth_tx,
            cc_rx: cc_rx.into(),
            editor: Editor::new(),
            handler_factory: MidiHandlerFactory::new(wmidi::Channel::Ch1),
        },
        |ctx, editor| editor.initialize(ctx),
        |ctx, setter, editor| editor.update(ctx, setter),
    )
}
