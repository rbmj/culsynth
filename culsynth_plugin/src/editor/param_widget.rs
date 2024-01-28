//! This module contains a trait and convenience methods to draw the UI for
//! setting synthesizer parameters.

use super::*;

const SLIDER_WIDTH: f32 = 40f32;

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
        param.string_to_normalized_value(s).map(|x| range.unnormalize(x) as f64)
    })
    .custom_formatter(move |f, _| {
        param.normalized_value_to_string(range2.normalize(f as i32), false)
    })
}

struct ParamSlider<'a> {
    param: &'a IntParam,
    setter: &'a ParamSetter<'a>,
    slider: egui::widgets::Slider<'a>,
    label: egui::widgets::Label,
}

impl<'a> ParamSlider<'a> {
    pub fn new(setter: &'a ParamSetter, param: &'a IntParam, label: &'a str) -> Self {
        let (min, max) = match param.range() {
            IntRange::Linear { min: x, max: y } => (x, y),
            IntRange::Reversed(IntRange::Linear { min: x, max: y }) => (*x, *y),
            _ => std::unreachable!(),
        };
        let slider = widgets::Slider::from_get_set(min as f64..=max as f64, |new| match new {
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
        .custom_parser(|s| {
            param.string_to_normalized_value(s).map(|x| param.range().unnormalize(x) as f64)
        })
        .custom_formatter(|f, _| {
            param.normalized_value_to_string(param.range().normalize(f as i32), false)
        });
        Self {
            param,
            setter,
            slider,
            label: egui::widgets::Label::new(label),
        }
    }
}

impl<'a> egui::Widget for ParamSlider<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let param = self.param;
        let setter = self.setter;
        let resp = ui.vertical(move |ui| {
            ui.set_min_width(SLIDER_WIDTH);
            let resp = ui.add(self.slider.vertical());
            if ui.add(self.label.sense(egui::Sense::click())).double_clicked() {
                setter.begin_set_parameter(param);
                setter.set_parameter(param, param.default_plain_value());
                setter.end_set_parameter(param);
            }
            resp
        });
        if resp.inner.dragged() {
            egui::containers::popup::show_tooltip_text(
                ui.ctx(),
                "drag_tooltip".into(),
                param.to_string(),
            );
        }
        resp.response
    }
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
        ui.horizontal(|ui| {
            use culsynth::util::*;
            ui.add(ParamSlider::new(setter, &osc.course, "CRS"));
            ui.add(ParamSlider::new(setter, &osc.fine, "FIN"));
            ui.add(ParamSlider::new(setter, &osc.shape, "SHP"));
            ui.add(ParamSlider::new(setter, &osc.sin, SIN_CHARSTR));
            ui.add(ParamSlider::new(setter, &osc.tri, TRI_CHARSTR));
            ui.add(ParamSlider::new(setter, &osc.sq, SQ_CHARSTR));
            ui.add(ParamSlider::new(setter, &osc.saw, SAW_CHARSTR));
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
                ui.horizontal(|ui| {
                    ui.add(ParamSlider::new(setter, &self.rate, "Rate"));
                    ui.add(ParamSlider::new(setter, &self.depth, "Depth"));
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
                    if ui.selectable_label(self.retrigger.value(), "Retrigger").clicked() {
                        setter.begin_set_parameter(&self.retrigger);
                        setter.set_parameter(&self.retrigger, !self.retrigger.value());
                        setter.end_set_parameter(&self.retrigger);
                    }
                    if ui.selectable_label(self.bipolar.value(), "Bipolar").clicked() {
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
            ui.horizontal(|ui| {
                ui.add(ParamSlider::new(setter, &self.mix_a, "Osc 1"));
                ui.add(ParamSlider::new(setter, &self.mix_b, "Osc 2"));
                ui.add(ParamSlider::new(setter, &self.mix_mod, "Ring"));
            });
        });
    }
}

impl ParamWidget for FiltPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        ui.vertical(|ui| {
            ui.label(label);
            ui.horizontal(|ui| {
                ui.add(ParamSlider::new(setter, &self.cutoff, "Cut"));
                ui.add(ParamSlider::new(setter, &self.res, "Res"));
                ui.add(ParamSlider::new(setter, &self.kbd, "Kbd"));
                ui.add(ParamSlider::new(setter, &self.vel, "Vel"));
                ui.add(ParamSlider::new(setter, &self.env, "Env"));
                ui.add(ParamSlider::new(setter, &self.low, "Low"));
                ui.add(ParamSlider::new(setter, &self.band, "Band"));
                ui.add(ParamSlider::new(setter, &self.high, "High"));
            });
        });
    }
}

impl ParamWidget for EnvPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        ui.vertical(|ui| {
            ui.label(label);
            ui.horizontal(|ui| {
                ui.add(ParamSlider::new(setter, &self.a, "A"));
                ui.add(ParamSlider::new(setter, &self.d, "D"));
                ui.add(ParamSlider::new(setter, &self.s, "S"));
                ui.add(ParamSlider::new(setter, &self.r, "R"));
            });
        });
    }
}
