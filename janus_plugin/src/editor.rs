use nih_plug_egui::{create_egui_editor, egui, EguiState};
use nih_plug::prelude::*;
use egui::Context;
use egui::widgets;
use std::sync::Arc;
use crate::JanusParams;


// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(1000, 600)
}

struct JanusEditor {
    params: Arc<JanusParams>,
}

impl JanusEditor {
    pub fn new(p: Arc<JanusParams>) -> Self {
        JanusEditor {
            params: p
        }
    }
    fn param_slider<'a>(setter: &'a ParamSetter, param: &'a IntParam) -> egui::widgets::Slider<'a> {
        let range = param.range();
        let range2 = range; //need a copy to move into the other closure...
        let (min, max) = match range {
            IntRange::Linear{min: x, max: y} => (x, y),
            IntRange::Reversed(IntRange::Linear{min: x, max: y}) => (*x, *y),
            _ => std::unreachable!()
        };
        widgets::Slider::from_get_set(min as f64 ..= max as f64,
            |new_value| {
                match new_value {
                    Some(value) => {
                        setter.begin_set_parameter(param);
                        setter.set_parameter(param, value as i32);
                        setter.end_set_parameter(param);
                        value
                    }
                    None => param.value() as f64
                }
            })
            .integer()
            .show_value(false)
            .suffix(param.unit())
            .custom_parser(move |s| {
                param.string_to_normalized_value(s).map(|x| range.unnormalize(x) as f64)
            })
            .custom_formatter(move |f, _| {
                param.normalized_value_to_string(
                    range2.normalize(f as i32),
                    false)
            })
    }
    pub fn update(&mut self, egui_ctx: &Context, setter: &ParamSetter) {
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Oscillator 1");
                        ui.horizontal(|ui| {
                            ui.vertical( |ui| {
                                ui.horizontal( |ui| {
                                    ui.label("Shape");
                                    ui.add(Self::param_slider(setter, &self.params.osc1_shape));
                                });
                                //Add widgets for tune, etc.
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.osc1_sin).vertical());
                                ui.label("Sin");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.osc1_tri).vertical());
                                ui.label("Tri");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.osc1_sq).vertical());
                                ui.label("Sq");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.osc1_saw).vertical());
                                ui.label("Saw");
                            });
                        });
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label("Filter 1 (SVF)");
                        ui.horizontal(|ui| {
                            ui.vertical( |ui| {
                                ui.horizontal( |ui| {
                                    ui.label("Cutoff");
                                    ui.add(Self::param_slider(setter, &self.params.filt_cutoff));
                                });
                                ui.horizontal( |ui| {
                                    ui.label("Resonance");
                                    ui.add(Self::param_slider(setter, &self.params.filt_res));
                                });
                                ui.horizontal( |ui| {
                                    ui.label("Env Mod");
                                    ui.add(Self::param_slider(setter, &self.params.filt_env));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Keyboard");
                                    ui.add(Self::param_slider(setter, &self.params.filt_kbd));
                                });
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.filt_low).vertical());
                                ui.label("Low");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.filt_band).vertical());
                                ui.label("Band");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.filt_high).vertical());
                                ui.label("High");
                            });
                        });
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Envelope 1 (VCF)");
                        ui.horizontal(|ui| {
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vcf_a).vertical());
                                ui.label("Attack");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vcf_d).vertical());
                                ui.label("Decay");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vcf_s).vertical());
                                ui.label("Sustain");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vcf_r).vertical());
                                ui.label("Release");
                            });
                        });
                    });
                    ui.vertical(|ui| {
                        ui.label("Envelope 2 (VCA)");
                        ui.horizontal(|ui| {
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vca_a).vertical());
                                ui.label("Attack");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vca_d).vertical());
                                ui.label("Decay");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vca_s).vertical());
                                ui.label("Sustain");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vca_r).vertical());
                                ui.label("Release");
                            });
                        });
                    });
                });
            });
        });
    }
    pub fn update_helper(egui_ctx: &Context, setter: &ParamSetter, state: &mut Self) {
        state.update(egui_ctx, setter)
    }
}

pub(crate) fn create(params: Arc<JanusParams>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        JanusEditor::new(params.clone()),
        |_, _| {},
        JanusEditor::update_helper
    )
}