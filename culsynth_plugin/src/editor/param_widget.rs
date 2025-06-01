//! This module contains a trait and convenience methods to draw the UI for
//! setting synthesizer parameters.

use crate::MidiHandler;
use culsynth::Fixed16;

use super::*;

#[derive(Clone, Copy)]
enum MidiParam {
    Control(wmidi::ControlFunction),
    Nrpn(wmidi::U14),
}

pub struct MidiCcSliderBuilder<'a, T: Fixed16>
where
    T::Bits: Into<f64> + Into<i32>,
{
    dispatcher: &'a dyn MidiHandler,
    label: &'static str,
    param: Option<MidiParam>,
    default_value: Option<T>,
    current_value: T,
    units: Option<&'static str>,
    color_intensity: Option<f32>,
    mod_data: Option<&'a mut EditorModData>,
    mod_dest: Option<ModDest>,
    percent: bool,
    vertical: bool,
}

pub struct MidiCcSlider<'a, T: Fixed16>
where
    T::Bits: Into<f64> + Into<i32>,
{
    dispatcher: &'a dyn MidiHandler,
    mod_data: Option<&'a mut EditorModData>,
    mod_dest: Option<ModDest>,
    slider: egui::widgets::Slider<'a>,
    label: egui::widgets::Label,
    units: Option<&'static str>,
    param: Option<MidiParam>,
    current_value: T,
    default_value: Option<T>,
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
    fn new(builder: MidiCcSliderBuilder<'a, T>) -> Self {
        let r_max: f64 = T::MAX.to_num();
        let r_min: f64 = T::MIN.to_num();
        let mut slider = widgets::Slider::from_get_set(r_min..=r_max, move |newval| match newval {
            Some(x) => {
                let x_fixed = T::saturating_from_num(x);
                if x_fixed != builder.current_value {
                    match builder.param {
                        Some(MidiParam::Control(control)) => {
                            Self::dispatch_cc(builder.dispatcher, control, x_fixed);
                        }
                        Some(MidiParam::Nrpn(nrpn)) => {
                            Self::dispatch_nrpn(builder.dispatcher, nrpn, x_fixed);
                        }
                        None => {}
                    }
                }
                x
            }
            None => builder.current_value.wrapping_to_num(),
        })
        .show_value(false);
        if builder.percent {
            slider = slider.custom_formatter(|x, _| (x * 100f64).round().to_string());
        }
        if builder.vertical {
            slider = slider.vertical();
        }
        if let Some(units) = builder.units {
            slider = slider.suffix(units);
        }
        let label = if let Some(intensity) = builder.color_intensity {
            let clr = egui::Color32::WHITE.lerp_to_gamma(egui::Color32::CYAN, intensity);
            egui::Label::new(egui::RichText::new(builder.label).color(clr))
        } else {
            egui::Label::new(builder.label)
        };
        Self {
            slider,
            dispatcher: builder.dispatcher,
            label,
            param: builder.param,
            current_value: builder.current_value,
            default_value: builder.default_value,
            percent: builder.percent,
            units: builder.units,
            mod_data: builder.mod_data,
            mod_dest: builder.mod_dest,
        }
    }
}

impl<'a, T: Fixed16> MidiCcSliderBuilder<'a, T>
where
    T::Bits: Into<f64> + Into<i32>,
{
    pub fn new(label: &'static str, dispatcher: &'a dyn MidiHandler, value: T) -> Self {
        Self {
            dispatcher,
            label,
            default_value: None,
            current_value: value,
            param: None,
            units: None,
            color_intensity: None,
            mod_data: None,
            percent: false,
            vertical: true,
            mod_dest: None,
        }
    }
    pub fn with_default(self, default: T) -> Self {
        Self {
            default_value: Some(default),
            ..self
        }
    }
    pub fn with_units(self, units: &'static str) -> Self {
        Self {
            units: Some(units),
            ..self
        }
    }
    pub fn as_percent(self) -> Self {
        Self {
            percent: true,
            ..self
        }
    }
    pub fn with_control(self, control: wmidi::ControlFunction) -> Self {
        Self {
            param: Some(MidiParam::Control(control)),
            ..self
        }
    }
    pub fn with_nrpn(self, nrpn: wmidi::U14) -> Self {
        Self {
            param: Some(MidiParam::Nrpn(nrpn)),
            ..self
        }
    }
    pub fn with_mod_data(
        self,
        mod_data: &'a mut EditorModData,
        matrix: &'_ ModMatrix<i16>,
        dst: ModDest,
    ) -> Self {
        let has_mod = mod_data.src().map_or(false, |s| !matrix.slot(s, dst).is_zero());
        let is_active = mod_data.dst().map_or(false, |d| d == dst);
        let intensity = if is_active {
            Some(mod_data.intensity())
        } else if has_mod {
            Some(0.75)
        } else {
            None
        };
        Self {
            color_intensity: intensity,
            mod_data: Some(mod_data),
            mod_dest: Some(dst),
            ..self
        }
    }
    pub fn horizontal(self) -> Self {
        Self {
            vertical: false,
            ..self
        }
    }
    pub fn build(self) -> MidiCcSlider<'a, T> {
        MidiCcSlider::new(self)
    }
}

impl<'a, T: Fixed16> egui::Widget for MidiCcSlider<'a, T>
where
    T::Bits: Into<f64> + Into<i32>,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let resp = ui.vertical(move |ui| {
            ui.set_min_width(SLIDER_SPACING);
            let slider_resp = ui.add(self.slider);
            let label_resp = ui.add(self.label.sense(egui::Sense::click()));
            if label_resp.double_clicked() {
                match self.param {
                    Some(MidiParam::Control(cc)) => {
                        if let Some(def) = self.default_value {
                            Self::dispatch_cc(self.dispatcher, cc, def);
                        }
                    }
                    Some(MidiParam::Nrpn(nrpn)) => {
                        if let Some(def) = self.default_value {
                            Self::dispatch_nrpn(self.dispatcher, nrpn, def);
                        }
                    }
                    None => {}
                }
            } else if label_resp.secondary_clicked() {
                if let (Some(data), Some(dst)) = (self.mod_data, self.mod_dest) {
                    data.set_toggle_dst(dst);
                }
            }
            slider_resp
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
