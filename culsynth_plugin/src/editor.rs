use crate::egui;
use crate::MidiHandler;
use crate::Tuning;
use culsynth::voice::modulation::{ModDest, ModMatrix};
use culsynth::voice::VoiceParams;
use egui::widgets;

mod kbd;
mod main_ui;
mod param_widget;

const SLIDER_WIDTH: f32 = 130f32;
const SLIDER_SPACING: f32 = 40f32;

/// Struct to hold the global state information for the plugin editor (GUI).
pub struct Editor {
    main_ui: main_ui::MainUi,
    kbd_panel: kbd::KbdPanel,
    show_mod_matrix: bool,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            main_ui: Default::default(),
            kbd_panel: Default::default(),
            show_mod_matrix: false,
        }
    }
    pub fn show_mod_matrix(&mut self, val: bool) {
        self.show_mod_matrix = val;
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
                        egui::ComboBox::from_id_source(id_str).selected_text(dst.to_str()).show_ui(
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
        params: &VoiceParams<i16>,
        tuning: (Tuning, Tuning),
        matrix: &ModMatrix<i16>,
        midi_handler: &impl MidiHandler,
    ) {
        self.kbd_panel.show(egui_ctx, midi_handler);
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            self.main_ui.draw(ui, params, tuning, midi_handler);
        });
        egui::Window::new("Modulation Matrix").open(&mut self.show_mod_matrix).show(
            egui_ctx,
            |ui| {
                Self::draw_modmatrix(ui, matrix, midi_handler);
            },
        );
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
