use crate::pluginparams::{
    CulSynthParams, EnvPluginParams, FiltPluginParams, LfoPluginParams, ModMatrixPluginParams,
    OscPluginParams, RingModPluginParams,
};
use crate::voicealloc::{MonoSynth, PolySynth, VoiceAllocator};
use crate::{ContextReader, VoiceMode};
use culsynth::context::{Context, ContextFxP};
use culsynth::devices::LfoWave;
use culsynth::voice::modulation::{ModDest, ModSrc};
use egui::widgets;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::{mpsc::SyncSender, Arc};

mod kbd;
mod param_widget;
use param_widget::{param_slider, ParamWidget};

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(1000, 800)
}

/// Struct to hold the global state information for the plugin editor (GUI).
struct CulSynthEditor {
    params: Arc<CulSynthParams>,
    midi_channel: SyncSender<i8>,
    synth_channel: SyncSender<Box<dyn VoiceAllocator>>,
    context: ContextReader,
    kbd_panel: kbd::KbdPanel,
    show_mod_matrix: bool,
    show_settings: bool,
    show_about: bool,
}

impl CulSynthEditor {
    pub fn new(
        p: Arc<CulSynthParams>,
        midi_tx: SyncSender<i8>,
        synth_tx: SyncSender<Box<dyn VoiceAllocator>>,
        ctx: ContextReader,
    ) -> Self {
        CulSynthEditor {
            params: p,
            midi_channel: midi_tx,
            synth_channel: synth_tx,
            context: ctx,
            kbd_panel: Default::default(),
            show_mod_matrix: false,
            show_settings: false,
            show_about: false,
        }
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
                            self.show_mod_matrix = true;
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
    fn draw_main_controls(&mut self, setter: &ParamSetter, ui: &mut egui::Ui) {
        ui.spacing_mut().slider_width = 130f32;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.params.osc1.draw_on(ui, setter, "Oscillator 1");
                ui.separator();
                param_widget::osc_with_sync(&self.params.osc2, &self.params.osc_sync).draw_on(
                    ui,
                    setter,
                    "Oscillator 2",
                );
                ui.separator();
                self.params
                    .ringmod
                    .draw_on(ui, setter, "Mixer/Ring Modulator");
            });
            ui.horizontal(|ui| {
                self.params.filt.draw_on(ui, setter, "Filter");
                ui.separator();
                self.params.lfo1.draw_on(ui, setter, "LFO 1");
                ui.separator();
                self.params.lfo2.draw_on(ui, setter, "LFO 2");
            });
            ui.horizontal(|ui| {
                self.params.env_vcf.draw_on(ui, setter, "Filter Envelope");
                ui.separator();
                self.params
                    .env_vca
                    .draw_on(ui, setter, "Amplifier Envelope");
                ui.separator();
                self.params.env1.draw_on(ui, setter, "Mod Envelope 1");
                ui.separator();
                self.params.env2.draw_on(ui, setter, "Mod Envelope 2");
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
        let fixed_context = ContextFxP::maybe_create(sr);
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
                        VoiceMode::Mono => Box::new(MonoSynth::<i16>::new(ctx)),
                        VoiceMode::Poly16 => Box::new(PolySynth::<i16>::new(ctx, 16)),
                    };
                    ret
                })
            } else {
                Some(match new_voice_mode {
                    VoiceMode::Mono => Box::new(MonoSynth::<f32>::new(Context::new(sr as f32))),
                    VoiceMode::Poly16 => {
                        Box::new(PolySynth::<f32>::new(Context::new(sr as f32), 16))
                    }
                })
            }
        } else {
            None
        }
    }
    fn draw_modmatrix(matrix: &ModMatrixPluginParams, ui: &mut egui::Ui, setter: &ParamSetter) {
        egui::Grid::new("MODMATRIX").show(ui, |ui| {
            ui.label("");
            ui.label("Slot A");
            ui.label("Slot B");
            ui.label("Slot C");
            ui.label("Slot D");
            ui.end_row();
            for src in ModSrc::elements() {
                ui.label(src.to_str());
                let row = matrix.row(*src);
                for (idx, slot) in row.iter().enumerate() {
                    let mut dest = ModDest::try_from(slot.0.value() as u16).unwrap();
                    let id_str = format!("MMRow{}Slot{}", *src as u16, idx);
                    ui.vertical(|ui| {
                        ui.separator();
                        egui::ComboBox::from_id_source(id_str)
                            .selected_text(dest.to_str())
                            .show_ui(ui, |ui| {
                                let sec = row.is_secondary();
                                for value in ModDest::elements_secondary_if(sec) {
                                    ui.selectable_value(&mut dest, value, value.to_str());
                                }
                            });
                        if dest as i32 != slot.0.value() {
                            setter.begin_set_parameter(slot.0);
                            setter.set_parameter(slot.0, dest as i32);
                            setter.end_set_parameter(slot.0);
                        }
                        ui.add(param_slider(setter, slot.1));
                    });
                }
                ui.end_row();
            }
        });
    }
    /// Draw the editor panel
    pub fn update(&mut self, egui_ctx: &egui::Context, setter: &ParamSetter) {
        self.draw_status_bar(egui_ctx);
        for midi_evt in self.kbd_panel.show(egui_ctx) {
            if let Err(e) = self.midi_channel.try_send(midi_evt) {
                nih_error!("{}", e);
            }
        }
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            self.draw_main_controls(setter, ui);
        });
        egui::Window::new("Modulation Matrix")
            .open(&mut self.show_mod_matrix)
            .show(egui_ctx, |ui| {
                Self::draw_modmatrix(&self.params.modmatrix, ui, setter);
            });
        egui::Window::new("Settings")
            .open(&mut self.show_settings)
            .show(egui_ctx, |ui| {
                if let Some(synth) = Self::draw_settings(ui, &self.context, &egui_ctx) {
                    if let Err(e) = self.synth_channel.try_send(synth) {
                        nih_log!("{}", e);
                    }
                }
            });
        egui::Window::new("About")
            .open(&mut self.show_about)
            .collapsible(false)
            .show(egui_ctx, |ui| {
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
    pub fn initialize(&mut self, egui_ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "culsynth_noto_sans_math".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../resources/fonts/NotoSansMath-Regular.ttf"
            )),
        );
        fonts.font_data.insert(
            "culsynth_noto_sans_sym".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../resources/fonts/NotoSansSymbols-Regular.ttf"
            )),
        );
        fonts.font_data.insert(
            "culsynth_noto_sans_math".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../resources/fonts/NotoSansMath-Regular.ttf"
            )),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .extend_from_slice(&[
                "culsynth_noto_sans_math".to_owned(),
                "culsynth_noto_sans_sym".to_owned(),
            ]);

        egui_ctx.set_fonts(fonts);
    }
}

pub fn create(
    params: Arc<CulSynthParams>,
    midi_tx: SyncSender<i8>,
    synth_tx: SyncSender<Box<dyn VoiceAllocator>>,
    context: ContextReader,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        CulSynthEditor::new(params, midi_tx, synth_tx, context),
        |ctx, editor| editor.initialize(ctx),
        |ctx, setter, editor| editor.update(ctx, setter),
    )
}
