use crate::voicealloc::{MonoSynth, PolySynth, VoiceAllocator};
use crate::{egui, ContextReader};
use crate::{MidiHandler, Tuning, VoiceMode};
use culsynth::voice::modulation::{ModDest, ModMatrix};
use culsynth::voice::VoiceParams;
use egui::widgets;
#[cfg(feature = "instrumentation")]
use ringbuffer::RingBuffer;

use std::sync::Arc;

mod kbd;
mod main_ui;
mod param_widget;

const SLIDER_WIDTH: f32 = 130f32;
const SLIDER_SPACING: f32 = 40f32;

pub trait SynthSender {
    fn send(&self, synth: Box<dyn VoiceAllocator>);
}

impl SynthSender for std::sync::mpsc::SyncSender<Box<dyn VoiceAllocator>> {
    fn send(&self, synth: Box<dyn VoiceAllocator>) {
        if let Err(_) = self.try_send(synth) {
            //TODO: log this
        }
    }
}

struct NullSender {}
impl SynthSender for NullSender {
    fn send(&self, _synth: Box<dyn VoiceAllocator>) {}
}

#[cfg(feature = "instrumentation")]
struct EditorInstrumentation {
    pub buffer: ringbuffer::ConstGenericRingBuffer<u32, 256>,
    pub should_show: bool,
}

#[cfg(feature = "instrumentation")]
impl EditorInstrumentation {
    pub fn new() -> Self {
        Self {
            buffer: ringbuffer::ConstGenericRingBuffer::from([0; 256]),
            should_show: false,
        }
    }
    pub fn draw(&mut self, egui_ctx: &egui::Context) {
        self.buffer.push(crate::instrumentation::get_last());
        egui::Window::new("Instrumentation")
            .open(&mut self.should_show)
            .show(egui_ctx, |ui| {
                use egui_plot::{Line, Plot, PlotPoints};
                const TIME: f64 = 1e9 * 1024f64 / 48000f64;
                let points = self
                    .buffer
                    .iter()
                    .enumerate()
                    .map(|(n, t)| [n as f64, *t as f64 / TIME])
                    .collect::<Vec<[f64; 2]>>();
                let line = Line::new("Instrumentation", PlotPoints::new(points));
                Plot::new("Data")
                    .view_aspect(2.0)
                    .include_y(0.0)
                    .set_margin_fraction(egui::Vec2::new(0.0, 0.2))
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                    })
            });
    }
}

/// Struct to hold the global state information for the plugin editor (GUI).
pub struct Editor {
    main_ui: main_ui::MainUi,
    kbd_panel: kbd::KbdPanel,
    show_mod_matrix: bool,
    show_settings: bool,
    show_about: bool,
    #[cfg(feature = "instrumentation")]
    instrumentation: EditorInstrumentation,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn null_sender() -> Option<&'static impl SynthSender> {
        let n: Option<&NullSender> = None;
        n
    }
    pub fn new() -> Self {
        #[cfg(feature = "instrumentation")]
        {
            Editor {
                main_ui: Default::default(),
                kbd_panel: Default::default(),
                show_mod_matrix: false,
                show_settings: false,
                show_about: false,
                instrumentation: EditorInstrumentation::new(),
            }
        }
        #[cfg(not(feature = "instrumentation"))]
        {
            Editor {
                main_ui: Default::default(),
                kbd_panel: Default::default(),
                show_mod_matrix: false,
                show_settings: false,
                show_about: false,
            }
        }
    }
    fn draw_modmatrix(ui: &mut egui::Ui, matrix: &ModMatrix<i16>, handler: &impl MidiHandler) {
        egui::Grid::new("MODMATRIX").show(ui, |ui| {
            ui.label("");
            ui.label("Slot A");
            ui.label("Slot B");
            ui.label("Slot C");
            ui.label("Slot D");
            ui.end_row();
            for (src, slots) in matrix.rows {
                ui.label(src.to_str());
                for (idx, (dst, mag)) in slots.iter().enumerate() {
                    let id_str = format!("MMRow{}Slot{}", src as u16, idx);
                    let nrpn_lsb = culsynth::voice::cc::modmatrix_nrpn_lsb(src, idx);
                    ui.vertical(|ui| {
                        ui.separator();
                        let mut new_dst = *dst;
                        egui::ComboBox::from_id_salt(id_str).selected_text(dst.to_str()).show_ui(
                            ui,
                            |ui| {
                                let sec = src.is_secondary();
                                for value in ModDest::elements_secondary_if(sec) {
                                    ui.selectable_value(&mut new_dst, value, value.to_str());
                                }
                            },
                        );
                        if new_dst != *dst {
                            let nrpn = (1u16 << 7) | nrpn_lsb as u16;
                            handler.send_nrpn(
                                wmidi::U14::try_from(nrpn).unwrap(),
                                wmidi::U14::try_from(new_dst as u16).unwrap(),
                            );
                        }
                        let nrpn = (2u16 << 7) | nrpn_lsb as u16;
                        ui.add(param_widget::MidiCcSlider::new_fixed_nrpn(
                            *mag,
                            culsynth::IScalarFxP::ZERO,
                            Some("%"),
                            wmidi::U14::try_from(nrpn).unwrap(),
                            "",
                            handler,
                        ));
                    });
                }
                ui.end_row();
            }
        });
    }
    /// Draw the editor panel
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        proc_ctx: &impl ContextReader,
        params: &VoiceParams<i16>,
        tuning: (Tuning, Tuning),
        matrix: &ModMatrix<i16>,
        midi_handler: &impl MidiHandler,
        synth_sender: Option<&impl SynthSender>,
    ) {
        self.draw_status_bar(egui_ctx, proc_ctx);
        self.kbd_panel.show(egui_ctx, midi_handler);
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            self.main_ui.draw(ui, params, tuning, midi_handler);
        });
        egui::Window::new("Settings")
            .open(&mut self.show_settings)
            .show(egui_ctx, |ui| {
                Self::draw_settings(ui, egui_ctx, proc_ctx, synth_sender)
            });
        egui::Window::new("About").open(&mut self.show_about).collapsible(false).show(
            egui_ctx,
            |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(format!("CulSynth v{}", env!("CARGO_PKG_VERSION")));
                    ui.label("Copyright 2023 Robert Blair Mason");
                    ui.label("This program is open-source software");
                    ui.hyperlink_to(
                        "(see https://github.com/rbmj/culsynth for details)",
                        "https://github.com/rbmj/culsynth",
                    );
                });
            },
        );
        egui::Window::new("Modulation Matrix").open(&mut self.show_mod_matrix).show(
            egui_ctx,
            |ui| {
                Self::draw_modmatrix(ui, matrix, midi_handler);
            },
        );
        #[cfg(feature = "instrumentation")]
        {
            self.instrumentation.draw(egui_ctx);
        }
    }
    fn draw_status_bar(&mut self, egui_ctx: &egui::Context, proc_ctx: &impl ContextReader) {
        egui::TopBottomPanel::top("status")
            .frame(egui::Frame::NONE.fill(egui::Color32::from_gray(32)))
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
                        #[cfg(feature = "instrumentation")]
                        {
                            if ui.button("INST").clicked() {
                                self.instrumentation.should_show = true;
                            }
                        }
                    });
                    columns[0].expand_to_include_x(third);
                    columns[1].expand_to_include_x(width - third);
                    columns[2].with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let (sr, fixed_point) = proc_ctx.get();
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
        _egui_ctx: &egui::Context,
        context: &impl ContextReader,
        synth_sender: Option<&impl SynthSender>,
    ) {
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
            if let Some(sender) = synth_sender {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("FloatFixedSelect")
                        .selected_text(context_strs[fixed_point_idx])
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut new_is_fixed, false, context_strs[0]);
                            ui.add_enabled_ui(fixed_context.is_some(), |ui| {
                                ui.selectable_value(&mut new_is_fixed, true, context_strs[1]);
                            });
                        });
                    egui::ComboBox::from_id_salt("MonoPoly")
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
                if new_is_fixed != fixed_point || new_voice_mode != voice_mode {
                    let new_synth: Option<Box<dyn VoiceAllocator>> = if new_is_fixed {
                        fixed_context.map(|ctx| {
                            let ret: Box<dyn VoiceAllocator> = match new_voice_mode {
                                crate::VoiceMode::Mono => Box::new(MonoSynth::<i16>::new(ctx)),
                                crate::VoiceMode::Poly16 => {
                                    Box::new(PolySynth::<i16>::new(ctx, 16))
                                }
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
                    };
                    new_synth.map(|s| sender.send(s));
                }
            }
        });
    }
    pub fn initialize(&mut self, egui_ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "culsynth_noto_sans_math".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../resources/fonts/NotoSansMath-Regular.ttf"
            ))),
        );
        fonts.font_data.insert(
            "culsynth_noto_sans_sym".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../resources/fonts/NotoSansSymbols-Regular.ttf"
            ))),
        );
        fonts.font_data.insert(
            "culsynth_noto_sans_math".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../resources/fonts/NotoSansMath-Regular.ttf"
            ))),
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
