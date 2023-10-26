use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use nih_plug::prelude::*;
use egui::Context;
use std::sync::Arc;
use atomic_float::AtomicF32;
use crate::JanusParams;


// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(300, 200)
}

struct Monitored<T> {
    data: T,
    last_value: T
}

impl<T: Copy> Monitored<T> {
    fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }
    fn changed() {

    }
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
        let range2 = range;
        let (min, max) = match range {
            IntRange::Linear{min: x, max: y} => (x, y),
            IntRange::Reversed(IntRange::Linear{min: x, max: y}) => (*x, *y),
            _ => std::unreachable!()
        };
        egui::widgets::Slider::from_get_set(min as f64 ..= max as f64,
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
            ui.label("Gain");
            ui.add(widgets::ParamSlider::for_param(&self.params.gain, setter));

            ui.label(
                "Also gain, but with a lame widget. Can't even render the value correctly!",
            );
            // This is a simple naieve version of a parameter slider that's not aware of how
            // the parameters work
            ui.add(
                egui::widgets::Slider::from_get_set(-30.0..=30.0, |new_value| {
                    match new_value {
                        Some(new_value_db) => {
                            let new_value = nih_plug::util::gain_to_db(new_value_db as f32);

                            setter.begin_set_parameter(&self.params.gain);
                            setter.set_parameter(&self.params.gain, new_value);
                            setter.end_set_parameter(&self.params.gain);

                            new_value_db
                        }
                        None => nih_plug::util::gain_to_db(self.params.gain.value()) as f64,
                    }
                })
                .suffix(" dB")
            );
            ui.label("Cutoff");
            ui.add(Self::param_slider(setter, &self.params.filt_cutoff));

        });
    }
    pub fn update_helper(egui_ctx: &Context, setter: &ParamSetter, state: &mut Self) {
        state.update(egui_ctx, setter)
    }
}

pub(crate) fn create(
    params: Arc<JanusParams>,
    editor_state: Arc<EguiState>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        JanusEditor::new(params.clone()),
        |_, _| {},
        JanusEditor::update_helper
    )
}