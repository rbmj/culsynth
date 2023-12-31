//! This module contains a trait and convenience methods to draw the UI for
//! setting synthesizer parameters.

use super::*;

/// Returns a [egui::widgets::Slider] that can manipulate an [IntParam], getting
/// the range and to/from string information from the parameter itself.
pub fn param_slider<'a>(setter: &'a ParamSetter, param: &'a IntParam) -> egui::widgets::Slider<'a> {
    let range = param.range();
    let range2 = range; //need a copy to move into the other closure...
    let (min, max) = match range {
        IntRange::Linear { min: x, max: y } => (x, y),
        IntRange::Reversed(IntRange::Linear { min: x, max: y }) => (*x, *y),
        _ => std::unreachable!(),
    };
    widgets::Slider::from_get_set(min as f64..=max as f64, |new_value| match new_value {
        Some(value) => {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, value as i32);
            setter.end_set_parameter(param);
            value
        }
        None => param.value() as f64,
    })
    .integer()
    .show_value(false)
    .suffix(param.unit())
    .custom_parser(move |s| {
        param
            .string_to_normalized_value(s)
            .map(|x| range.unnormalize(x) as f64)
    })
    .custom_formatter(move |f, _| {
        param.normalized_value_to_string(range2.normalize(f as i32), false)
    })
}

/// A trait that provides a user interface for setting parameters on a given
/// type to avoid code duplication.
pub trait ParamWidget {
    /// Draw the interface on `ui`
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str);
}

/// Internal function to draw an oscillator UI, used for both the sync and
/// non-sync drawing functions
fn draw_osc(
    osc: &OscPluginParams,
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    label: &str,
    draw_sync: bool,
    sync_on: bool,
) -> bool {
    let mut sync_clicked = false;
    ui.vertical(|ui| {
        if draw_sync {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.label(" - ");
                let sync_str = if sync_on {
                    "Sync On"
                } else {
                    "Click to Enable Sync"
                };
                sync_clicked = ui.selectable_label(sync_on, sync_str).clicked();
            });
        } else {
            ui.label(label);
        }
        egui::Grid::new(label).show(ui, |ui| {
            ui.add(param_slider(setter, &osc.course).vertical());
            ui.add(param_slider(setter, &osc.fine).vertical());
            ui.add(param_slider(setter, &osc.shape).vertical());
            ui.add(param_slider(setter, &osc.sin).vertical());
            ui.add(param_slider(setter, &osc.tri).vertical());
            ui.add(param_slider(setter, &osc.sq).vertical());
            ui.add(param_slider(setter, &osc.saw).vertical());
            ui.end_row();
            ui.label("CRS");
            ui.label("FIN");
            ui.label("SHP");
            ui.label(culsynth::util::SIN_CHARSTR);
            ui.label(culsynth::util::TRI_CHARSTR);
            ui.label(culsynth::util::SQ_CHARSTR);
            ui.label(culsynth::util::SAW_CHARSTR);
        });
    });
    sync_clicked
}

impl ParamWidget for OscPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        draw_osc(self, ui, setter, label, false, false);
    }
}

pub struct OscPluginParamsWithSync<'a> {
    osc: &'a OscPluginParams,
    param: &'a BoolParam,
}

/// Draw an oscillator as well as a button to enable/disable oscillator sync
pub fn osc_with_sync<'a>(
    osc: &'a OscPluginParams,
    sync: &'a BoolParam,
) -> OscPluginParamsWithSync<'a> {
    OscPluginParamsWithSync { osc, param: sync }
}

impl<'a> ParamWidget for OscPluginParamsWithSync<'a> {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        let sync_on = self.param.value();
        if draw_osc(self.osc, ui, setter, label, true, sync_on) {
            setter.begin_set_parameter(self.param);
            setter.set_parameter(self.param, !sync_on);
            setter.end_set_parameter(self.param);
        }
    }
}

impl ParamWidget for LfoPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        ui.vertical(|ui| {
            ui.label(label);
            ui.horizontal(|ui| {
                egui::Grid::new(label).show(ui, |ui| {
                    ui.add(param_slider(setter, &self.rate).vertical());
                    ui.add(param_slider(setter, &self.depth).vertical());
                    ui.end_row();
                    ui.label("Rate");
                    ui.label("Depth");
                });
                ui.vertical(|ui| {
                    let cur_wave = self.wave.value();
                    for wave in LfoWave::waves() {
                        if ui
                            .selectable_label(cur_wave == *wave as i32, wave.to_str_short())
                            .clicked()
                        {
                            setter.begin_set_parameter(&self.wave);
                            setter.set_parameter(&self.wave, *wave as i32);
                            setter.end_set_parameter(&self.wave);
                        }
                    }
                });
                ui.vertical(|ui| {
                    if ui
                        .selectable_label(self.retrigger.value(), "Retrigger")
                        .clicked()
                    {
                        setter.begin_set_parameter(&self.retrigger);
                        setter.set_parameter(&self.retrigger, !self.retrigger.value());
                        setter.end_set_parameter(&self.retrigger);
                    }
                    if ui
                        .selectable_label(self.bipolar.value(), "Bipolar")
                        .clicked()
                    {
                        setter.begin_set_parameter(&self.bipolar);
                        setter.set_parameter(&self.bipolar, !self.bipolar.value());
                        setter.end_set_parameter(&self.bipolar);
                    }
                });
            });
        });
    }
}

impl ParamWidget for RingModPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        ui.vertical(|ui| {
            ui.label(label);
            egui::Grid::new(label).show(ui, |ui| {
                ui.add(param_slider(setter, &self.mix_a).vertical());
                ui.add(param_slider(setter, &self.mix_b).vertical());
                ui.add(param_slider(setter, &self.mix_mod).vertical());
                ui.end_row();
                ui.label("Osc 1");
                ui.label("Osc 2");
                ui.label("Ring");
            });
        });
    }
}

impl ParamWidget for FiltPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        ui.vertical(|ui| {
            ui.label(label);
            egui::Grid::new(label).show(ui, |ui| {
                ui.add(param_slider(setter, &self.cutoff).vertical());
                ui.add(param_slider(setter, &self.res).vertical());
                ui.add(param_slider(setter, &self.kbd).vertical());
                ui.add(param_slider(setter, &self.vel).vertical());
                ui.add(param_slider(setter, &self.env).vertical());
                ui.add(param_slider(setter, &self.low).vertical());
                ui.add(param_slider(setter, &self.band).vertical());
                ui.add(param_slider(setter, &self.high).vertical());
                ui.end_row();
                ui.label("Cut");
                ui.label("Res");
                ui.label("Kbd");
                ui.label("Vel");
                ui.label("Env");
                ui.label("Low");
                ui.label("Band");
                ui.label("High");
            });
        });
    }
}

impl ParamWidget for EnvPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        ui.vertical(|ui| {
            ui.label(label);
            egui::Grid::new(label).show(ui, |ui| {
                ui.add(param_slider(setter, &self.a).vertical());
                ui.add(param_slider(setter, &self.d).vertical());
                ui.add(param_slider(setter, &self.s).vertical());
                ui.add(param_slider(setter, &self.r).vertical());
                ui.end_row();
                ui.label("A");
                ui.label("D");
                ui.label("S");
                ui.label("R");
            });
        });
    }
}
