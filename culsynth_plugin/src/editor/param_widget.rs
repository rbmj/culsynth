//! This module contains a trait and convenience methods to draw the UI for
//! setting synthesizer parameters.

use culsynth::{Fixed16, ScalarFxP};

use crate::voicealloc::MidiCcHandler;

use super::*;

pub struct MidiCcSlider<'a> {
    //dispatch: &'a mut dyn MidiCcHandler,
    slider: egui::widgets::Slider<'a>,
    label: egui::widgets::Label,
    default_value: f64,
}

impl<'a> MidiCcSlider<'a> {
    pub fn new(
        value: f64,
        default_value: f64,
        control: wmidi::ControlFunction,
        label: &'static str,
        dispatcher: &'a mut dyn MidiCcHandler,
        range: core::ops::RangeInclusive<f64>,
    ) -> Self {
        let r_max = *range.end();
        let r_min = *range.start();
        Self {
            //dispatch: dispatcher,
            slider: widgets::Slider::from_get_set(range, move |val| match val {
                Some(val) => {
                    let cc_value = 127f64 * ((val - r_min) / (r_max - r_min));
                    dispatcher.handle_cc(control, cc_value as u8);
                    val
                }
                None => value,
            }),
            label: egui::widgets::Label::new(label),
            default_value,
        }
    }
    pub fn new_fixed<T: Fixed16>(
        value: T,
        default: T,
        units: Option<&'static str>,
        control: wmidi::ControlFunction,
        label: &'static str,
        dispatcher: &'a mut dyn MidiCcHandler,
    ) -> Self
    where
        T::Bits: Into<f64> + Into<i32>,
    {
        let r_max: f64 = T::MAX.to_num();
        let r_min: f64 = T::MIN.to_num();
        let mut slider = widgets::Slider::from_get_set(r_min..=r_max, move |newval| match newval {
            Some(x) => {
                let val_int: i32 = T::saturating_from_num(x).to_bits().into();
                let low_int: i32 = T::MIN.to_bits().into();
                let cc_value = (val_int - low_int) as u32;
                let _lsb = ((cc_value >> 2) & 0x7F) as u8;
                let msb = ((cc_value >> 9) & 0x7F) as u8;
                dispatcher.handle_cc(control, msb);
                x
            }
            None => value.wrapping_to_num(),
        })
        .show_value(false);
        if let Some(units) = units {
            slider = slider.suffix(units);
        }
        Self {
            //dispatch: dispatcher,
            slider: slider,
            label: egui::widgets::Label::new(label),
            default_value: default.to_num(),
        }
    }
    pub fn new_percent(
        value: ScalarFxP,
        default: ScalarFxP,
        units: Option<&'static str>,
        control: wmidi::ControlFunction,
        label: &'static str,
        dispatcher: &'a mut dyn MidiCcHandler,
    ) -> Self {
        let mut ret = Self::new_fixed(
            value,
            default,
            units.or(Some("%")),
            control,
            label,
            dispatcher,
        );
        ret.slider = ret.slider.custom_formatter(|x, _| (x * 100f64).round().to_string());
        ret
    }
}

impl<'a> egui::Widget for MidiCcSlider<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let resp = ui.vertical(move |ui| {
            ui.set_min_width(SLIDER_SPACING);
            let resp = ui.add(self.slider.vertical());
            if ui.add(self.label.sense(egui::Sense::click())).double_clicked() {
                //TODO: Reset to default
            }
            resp
        });
        /*
        if resp.inner.dragged() {
            egui::containers::popup::show_tooltip_text(
                ui.ctx(),
                "drag_tooltip".into(),
                param.to_string(),
            );
        }
        */
        resp.response
    }
}
