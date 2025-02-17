//! This module contains a trait and convenience methods to draw the UI for
//! setting synthesizer parameters.

use crate::MidiHandler;
use culsynth::Fixed16;

use super::*;

enum MidiParam {
    Control(wmidi::ControlFunction),
    Nrpn(wmidi::U14),
}

pub struct MidiCcSlider<'a, T: Fixed16>
where
    T::Bits: Into<f64> + Into<i32>,
{
    dispatcher: &'a dyn MidiHandler,
    slider: egui::widgets::Slider<'a>,
    label: egui::widgets::Label,
    param: MidiParam,
    default_value: T,
    current_value: T,
    units: Option<&'static str>,
    percent: bool,
}

impl<'a, T: Fixed16> MidiCcSlider<'a, T>
where
    T::Bits: Into<f64> + Into<i32>,
{
    fn dispatch_cc(handler: &'a dyn MidiHandler, control: wmidi::ControlFunction, value: T) {
        let val_int: i32 = value.to_bits().into();
        let low_int: i32 = T::MIN.to_bits().into();
        let cc_value = (val_int - low_int) as u32;
        let _lsb = wmidi::U7::from_u8_lossy((cc_value >> 2) as u8);
        let msb = wmidi::U7::from_u8_lossy((cc_value >> 9) as u8);
        handler.send_cc(control, msb);
    }
    fn dispatch_nrpn(handler: &'a dyn MidiHandler, nrpn: wmidi::U14, value: T) {
        let val_int: i32 = value.to_bits().into();
        let low_int: i32 = T::MIN.to_bits().into();
        let cc_value = (val_int - low_int) as u32;
        let nrpn: u16 = nrpn.into();
        let value_lsb = wmidi::U7::from_u8_lossy((cc_value >> 2) as u8);
        let value_msb = wmidi::U7::from_u8_lossy((cc_value >> 9) as u8);
        let nrpn_lsb = wmidi::U7::from_u8_lossy(nrpn as u8);
        let nrpn_msb = wmidi::U7::from_u8_lossy((nrpn >> 7) as u8);
        handler.send_cc(
            wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_MSB,
            nrpn_msb,
        );
        handler.send_cc(
            wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_LSB,
            nrpn_lsb,
        );
        handler.send_cc(wmidi::ControlFunction::DATA_ENTRY_MSB, value_msb);
        handler.send_cc(wmidi::ControlFunction::DATA_ENTRY_LSB, value_lsb);
    }
    pub fn new_fixed(
        value: T,
        default: T,
        units: Option<&'static str>,
        control: wmidi::ControlFunction,
        label: &'static str,
        dispatcher: &'a dyn MidiHandler,
    ) -> Self {
        let r_max: f64 = T::MAX.to_num();
        let r_min: f64 = T::MIN.to_num();
        let mut slider = widgets::Slider::from_get_set(r_min..=r_max, move |newval| match newval {
            Some(x) => {
                let x_fixed = T::saturating_from_num(x);
                if x_fixed != value {
                    Self::dispatch_cc(dispatcher, control, x_fixed);
                }
                x
            }
            None => value.wrapping_to_num(),
        })
        .show_value(false);
        if let Some(units) = units {
            slider = slider.suffix(units);
        }
        Self {
            dispatcher,
            slider: slider,
            label: egui::widgets::Label::new(label),
            default_value: default,
            current_value: value,
            param: MidiParam::Control(control),
            units,
            percent: false,
        }
    }
    pub fn new_percent(
        value: T,
        default: T,
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
        ret.percent = true;
        ret.slider = ret.slider.custom_formatter(|x, _| (x * 100f64).round().to_string());
        ret
    }
    pub fn new_fixed_nrpn(
        value: T,
        default: T,
        units: Option<&'static str>,
        nrpn: wmidi::U14,
        label: &'static str,
        dispatcher: &'a dyn MidiHandler,
    ) -> Self {
        let r_max: f64 = T::MAX.to_num();
        let r_min: f64 = T::MIN.to_num();
        let mut slider = widgets::Slider::from_get_set(r_min..=r_max, move |newval| match newval {
            Some(x) => {
                let x_fixed = T::saturating_from_num(x);
                if x_fixed != value {
                    Self::dispatch_nrpn(dispatcher, nrpn, x_fixed);
                }
                x
            }
            None => value.wrapping_to_num(),
        })
        .show_value(false);
        if let Some(units) = units {
            slider = slider.suffix(units);
        }
        Self {
            dispatcher,
            slider: slider,
            label: egui::widgets::Label::new(label),
            default_value: default.to_num(),
            current_value: value.to_num(),
            param: MidiParam::Nrpn(nrpn),
            units,
            percent: false,
        }
    }
    pub fn vertical(self) -> Self {
        Self {
            slider: self.slider.vertical(),
            ..self
        }
    }
}

impl<'a, T: Fixed16> egui::Widget for MidiCcSlider<'a, T>
where
    T::Bits: Into<f64> + Into<i32>,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let resp = ui.vertical(move |ui| {
            ui.set_min_width(SLIDER_SPACING);
            let resp = ui.add(self.slider);
            if ui.add(self.label.sense(egui::Sense::click())).double_clicked() {
                match self.param {
                    MidiParam::Control(cc) => {
                        Self::dispatch_cc(self.dispatcher, cc, self.default_value)
                    }
                    MidiParam::Nrpn(nrpn) => {
                        Self::dispatch_nrpn(self.dispatcher, nrpn, self.default_value)
                    }
                }
            }
            resp
        });
        if resp.inner.dragged() {
            let mut value: f32 = self.current_value.to_num();
            let mut precision: usize = 2;
            if self.percent {
                value *= 100f32;
                value = value.round();
                precision = 0;
            }
            let value_str = if let Some(units) = self.units {
                std::format!("{:.2$} {}", value, units, precision)
            } else {
                std::format!("{:.1$}", value, precision)
            };
            egui::containers::popup::show_tooltip_text(
                ui.ctx(),
                ui.layer_id(),
                "drag_tooltip".into(),
                value_str,
            );
        }
        resp.response
    }
}
