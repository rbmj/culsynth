use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};

use crate::editor::Editor;
use crate::nih::midihandler::{MidiHandlerFactory, MidiSender};
use crate::nih::ContextReader;
use crate::pluginparams::CulSynthParams;
use crate::voicealloc::{MonoSynth, PolySynth, VoiceAllocator};
use crate::{MidiHandler, VoiceMode};

use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(1000, 800)
}

struct PluginEditor {
    params: Arc<CulSynthParams>,
    context: super::ContextReader,
    midi_tx: SyncSender<wmidi::MidiMessage<'static>>,
    synth_tx: SyncSender<Box<dyn VoiceAllocator>>,
    cc_rx: Mutex<Receiver<wmidi::MidiMessage<'static>>>,
    editor: Editor,
    handler_factory: MidiHandlerFactory,
    show_settings: bool,
    show_about: bool,
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
        self.draw_status_bar(ctx);
        let handler = self.handler_factory.create(&to_voice, &self.params, setter);
        if let Ok(channel) = self.cc_rx.get_mut() {
            while let Ok(msg) = channel.try_recv() {
                handler.send(msg);
            }
        }
        let tuning = (self.params.osc1.tuning(), self.params.osc2.tuning());
        let params: culsynth::voice::VoiceParams<i16> = (&*self.params).into();
        let matrix: culsynth::voice::modulation::ModMatrix<i16> = (&self.params.modmatrix).into();
        self.editor.update(ctx, &params, tuning, &matrix, &handler);
        egui::Window::new("Settings").open(&mut self.show_settings).show(ctx, |ui| {
            if let Some(synth) = Self::draw_settings(ui, &self.context, ctx) {
                if let Err(e) = self.synth_tx.try_send(synth) {
                    nih_log!("{}", e);
                }
            }
        });
        egui::Window::new("About")
            .open(&mut self.show_about)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(format!("CulSynth v{}", env!("CARGO_PKG_VERSION")));
                    ui.label("Copyright 2023 Robert Blair Mason");
                    ui.label("This program is open-source software");
                    ui.hyperlink_to(
                        "(see https://github.com/rbmj/culsynth for details)",
                        "https://github.com/rbmj/culsynth",
                    );
                });
            });
    }
    fn draw_status_bar(&mut self, egui_ctx: &egui::Context) {
        egui::TopBottomPanel::top("status")
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(32)))
            .max_height(20f32)
            .show(egui_ctx, |ui| {
                let width = ui.available_width();
                let third = width / 3f32;
                ui.columns(3, |columns| {
                    columns[0].horizontal_centered(|ui| {
                        if ui.button("Settings").clicked() {
                            self.show_settings = true;
                        }
                        if ui.button("Mod Matrix").clicked() {
                            self.editor.show_mod_matrix(true);
                        }
                        if ui.button("About").clicked() {
                            self.show_about = true;
                        }
                    });
                    columns[0].expand_to_include_x(third);
                    columns[1].expand_to_include_x(width - third);
                    columns[2].with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let (sr, fixed_point) = self.context.get();
                            let fixed_str = if fixed_point {
                                "16 bit fixed"
                            } else {
                                "32 bit float"
                            };
                            ui.label(format!(
                                "{}.{} kHz / {}",
                                sr / 1000,
                                (sr % 1000) / 100,
                                fixed_str,
                            ));
                        },
                    );
                    columns[1].centered_and_justified(|ui| {
                        ui.label(format!("CulSynth v{}", env!("CARGO_PKG_VERSION")));
                    });
                });
            });
    }
    fn draw_settings(
        ui: &mut egui::Ui,
        context: &ContextReader,
        _egui_ctx: &egui::Context,
    ) -> Option<Box<dyn VoiceAllocator>> {
        let voice_mode = context.voice_mode();
        let (sr, fixed_point) = context.get();
        let context_strs = ["32 bit float", "16 bit fixed"];
        let fixed_point_idx: usize = if fixed_point { 1 } else { 0 };
        let fixed_context = culsynth::context::ContextFxP::maybe_create(sr);
        let mut new_is_fixed = fixed_point;
        let mut new_voice_mode = voice_mode;
        ui.vertical(|ui| {
            /*
            // Doesn't currently work
            ui.horizontal(|ui| {
                if ui.button("Zoom In").clicked() {
                    egui::gui_zoom::zoom_in(egui_ctx);
                }
                if ui.button("Zoom Out").clicked() {
                    egui::gui_zoom::zoom_out(egui_ctx);
                }
            });
            */
            ui.label(format!(
                "Sample Rate: {}.{} kHz",
                sr / 1000,
                (sr % 1000) / 100
            ));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("FloatFixedSelect")
                    .selected_text(context_strs[fixed_point_idx])
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut new_is_fixed, false, context_strs[0]);
                        ui.add_enabled_ui(fixed_context.is_some(), |ui| {
                            ui.selectable_value(&mut new_is_fixed, true, context_strs[1]);
                        });
                    });
                egui::ComboBox::from_id_source("MonoPoly")
                    .selected_text(voice_mode.to_str())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut new_voice_mode,
                            VoiceMode::Mono,
                            VoiceMode::Mono.to_str(),
                        );
                        ui.selectable_value(
                            &mut new_voice_mode,
                            VoiceMode::Poly16,
                            VoiceMode::Poly16.to_str(),
                        );
                    });
            });
        });
        if new_is_fixed != fixed_point || new_voice_mode != voice_mode {
            if new_is_fixed {
                fixed_context.map(|ctx| {
                    let ret: Box<dyn VoiceAllocator> = match new_voice_mode {
                        crate::VoiceMode::Mono => Box::new(MonoSynth::<i16>::new(ctx)),
                        crate::VoiceMode::Poly16 => Box::new(PolySynth::<i16>::new(ctx, 16)),
                    };
                    ret
                })
            } else {
                Some(match new_voice_mode {
                    VoiceMode::Mono => Box::new(MonoSynth::<f32>::new(
                        culsynth::context::Context::new(sr as f32),
                    )),
                    VoiceMode::Poly16 => Box::new(PolySynth::<f32>::new(
                        culsynth::context::Context::new(sr as f32),
                        16,
                    )),
                })
            }
        } else {
            None
        }
    }
}

pub fn create(
    params: Arc<CulSynthParams>,
    midi_tx: SyncSender<wmidi::MidiMessage<'static>>,
    synth_tx: SyncSender<Box<dyn VoiceAllocator>>,
    cc_rx: Receiver<wmidi::MidiMessage<'static>>,
    context: super::ContextReader,
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
            show_about: false,
            show_settings: false,
        },
        |ctx, editor| editor.initialize(ctx),
        |ctx, setter, editor| editor.update(ctx, setter),
    )
}
