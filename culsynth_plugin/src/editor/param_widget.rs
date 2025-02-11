//! This module contains a trait and convenience methods to draw the UI for
//! setting synthesizer parameters.

use culsynth::{Fixed16, ScalarFxP};

use crate::MidiHandler;

use super::*;

pub struct MidiCcSlider<'a> {
    //dispatch: &'a mut dyn MidiHandler,
    slider: egui::widgets::Slider<'a>,
    label: egui::widgets::Label,
    default_value: f64,
}

impl<'a> MidiCcSlider<'a> {
    pub fn new_fixed<T: Fixed16>(
        value: T,
        default: T,
        units: Option<&'static str>,
        control: wmidi::ControlFunction,
        label: &'static str,
        dispatcher: &'a dyn MidiHandler,
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
                let _lsb = wmidi::U7::from_u8_lossy((cc_value >> 2) as u8);
                let msb = wmidi::U7::from_u8_lossy((cc_value >> 9) as u8);
                dispatcher.send_cc(control, msb);
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
        dispatcher: &'a dyn MidiHandler,
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
    pub fn new_fixed_nrpn<T: Fixed16>(
        value: T,
        default: T,
        units: Option<&'static str>,
        nrpn: wmidi::U14,
        label: &'static str,
        dispatcher: &'a dyn MidiHandler,
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
                let nrpn: u16 = nrpn.into();
                let value_lsb = wmidi::U7::from_u8_lossy((cc_value >> 2) as u8);
                let value_msb = wmidi::U7::from_u8_lossy((cc_value >> 9) as u8);
                let nrpn_lsb = wmidi::U7::from_u8_lossy(nrpn as u8);
                let nrpn_msb = wmidi::U7::from_u8_lossy((nrpn >> 7) as u8);
                dispatcher.send_cc(
                    wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_MSB,
                    nrpn_msb,
                );
                dispatcher.send_cc(
                    wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_LSB,
                    nrpn_lsb,
                );
                dispatcher.send_cc(wmidi::ControlFunction::DATA_ENTRY_MSB, value_msb);
                dispatcher.send_cc(wmidi::ControlFunction::DATA_ENTRY_LSB, value_lsb);
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
    pub fn vertical(self) -> Self {
        Self {
            slider: self.slider.vertical(),
            label: self.label,
            default_value: self.default_value,
        }
    }
}

impl<'a> egui::Widget for MidiCcSlider<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let resp = ui.vertical(move |ui| {
            ui.set_min_width(SLIDER_SPACING);
            let resp = ui.add(self.slider);
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
