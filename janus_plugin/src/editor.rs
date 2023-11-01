use crate::{JanusParams, EnvPluginParams, FiltPluginParams, OscPluginParams, RingModPluginParams};
use egui::widgets;
use egui::Context;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use piano_keyboard::Rectangle as PianoRectangle;
use piano_keyboard::{Element, Keyboard2d, KeyboardBuilder};
use std::sync::{mpsc::SyncSender, Arc};

fn param_slider<'a>(setter: &'a ParamSetter, param: &'a IntParam) -> egui::widgets::Slider<'a> {
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

trait PluginWidget {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str);
}

impl PluginWidget for OscPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        ui.vertical(|ui| {
            ui.label(label);
            egui::Grid::new(label).show(ui, |ui| {
                ui.add(param_slider(setter, &self.shape).vertical());
                //ui.add(param_slider(setter, &self.tune))
                ui.add(param_slider(setter, &self.sin).vertical());
                ui.add(param_slider(setter, &self.tri).vertical());
                ui.add(param_slider(setter, &self.sq).vertical());
                ui.add(param_slider(setter, &self.saw).vertical());
                ui.end_row();
                ui.label("Shape");
                //ui.label("Tune")
                ui.label("Sine");
                ui.label("Tri");
                ui.label("Square");
                ui.label("Saw");
            });
        });
    }
}

impl PluginWidget for RingModPluginParams {
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

impl PluginWidget for FiltPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        ui.vertical(|ui| {
            ui.label(label);
            egui::Grid::new(label).show(ui, |ui| {
                ui.add(param_slider(setter, &self.cutoff).vertical());
                ui.add(param_slider(setter, &self.res).vertical());
                ui.add(param_slider(setter, &self.kbd).vertical());
                ui.add(param_slider(setter, &self.env).vertical());
                ui.add(param_slider(setter, &self.low).vertical());
                ui.add(param_slider(setter, &self.band).vertical());
                ui.add(param_slider(setter, &self.high).vertical());
                ui.end_row();
                ui.label("Cut");
                ui.label("Res");
                ui.label("Kbd");
                ui.label("Env");
                ui.label("Low");
                ui.label("Band");
                ui.label("High");
            });
        });
    }
}

impl PluginWidget for EnvPluginParams {
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

/// Map a keyboard key to a MIDI note number, or `None` if unmapped.
fn key_to_notenum(k: egui::Key) -> Option<i8> {
    match k {
        egui::Key::A => Some(janus::midi_const::C4 as i8),
        egui::Key::S => Some(janus::midi_const::D4 as i8),
        egui::Key::D => Some(janus::midi_const::E4 as i8),
        egui::Key::F => Some(janus::midi_const::F4 as i8),
        egui::Key::G => Some(janus::midi_const::G4 as i8),
        egui::Key::H => Some(janus::midi_const::A4 as i8),
        egui::Key::J => Some(janus::midi_const::B4 as i8),
        egui::Key::K => Some(janus::midi_const::C5 as i8),
        egui::Key::L => Some(janus::midi_const::D5 as i8),

        egui::Key::W => Some(janus::midi_const::Db4 as i8),
        egui::Key::E => Some(janus::midi_const::Eb4 as i8),
        egui::Key::T => Some(janus::midi_const::Gb4 as i8),
        egui::Key::Y => Some(janus::midi_const::Ab4 as i8),
        egui::Key::U => Some(janus::midi_const::Bb4 as i8),
        egui::Key::O => Some(janus::midi_const::Db5 as i8),
        egui::Key::P => Some(janus::midi_const::Eb5 as i8),
        _ => None,
    }
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(1000, 600)
}

/// Struct to hold the global state information for the plugin editor (GUI).
struct JanusEditor {
    params: Arc<JanusParams>,
    channel: SyncSender<i8>,
    last_note: Option<i8>,
}

impl JanusEditor {
    pub fn new(p: Arc<JanusParams>, c: SyncSender<i8>) -> Self {
        JanusEditor {
            params: p,
            channel: c,
            last_note: None,
        }
    }
    /// Helper function to handle keyboard input
    fn handle_kbd_input(&mut self, ui: &egui::Ui) {
        ui.input(|i| {
            for evt in i.events.iter() {
                if let egui::Event::Key{key, pressed, repeat, ..} = evt {
                    if *repeat { continue; }
                    if let Some(mut k) = key_to_notenum(*key) {
                        if !(*pressed) {
                            k += -128; //Note off
                        }
                        let _ = self.channel.try_send(k);
                    }
                }
            }
        });
    }
    /// Internal helper function to generate the egui shape for the border of the key
    /// (for stroke) and two rects for the body of the key (for fill and mouse pointer)
    fn draw_white_key(
        cursor: &egui::Pos2,
        small: &PianoRectangle,
        wide: &PianoRectangle,
        blind: &Option<PianoRectangle>,
    ) -> (egui::epaint::Shape, [egui::epaint::RectShape; 2]) {
        //handle if this is a blind key
        //we can cheat a little since we know that the small (top) portion and the blind
        //portion rectangles will share a vertical edge, so we can treat them as a single
        //rectangle.  Don't know why the library doesn't do this for you...
        let r = match blind {
            Some(blind_key) => PianoRectangle {
                x: std::cmp::min(small.x, blind_key.x),
                y: small.y,
                width: small.width + blind_key.width,
                height: small.height,
            },
            None => small.clone(),
        };
        //calculate the border path:
        let mut points: Vec<egui::Pos2> = vec![
            egui::pos2(cursor.x + r.x as f32, cursor.y + r.y as f32),
            egui::pos2(cursor.x + (r.x + r.width) as f32, cursor.y + r.y as f32),
            egui::pos2(
                cursor.x + (r.x + r.width) as f32,
                cursor.y + (r.y + r.height) as f32,
            ),
            egui::pos2(
                cursor.x + (wide.x + wide.width) as f32,
                cursor.y + wide.y as f32,
            ),
            egui::pos2(
                cursor.x + (wide.x + wide.width) as f32,
                cursor.y + (wide.y + wide.height) as f32,
            ),
            egui::pos2(
                cursor.x + wide.x as f32,
                cursor.y + (wide.y + wide.height) as f32,
            ),
            egui::pos2(cursor.x + wide.x as f32, cursor.y + wide.y as f32),
            egui::pos2(cursor.x + r.x as f32, cursor.y + (r.y + r.height) as f32),
        ];
        points.dedup(); //is this necessary?
        //since the key will not be convex, we must draw the border
        //separately from the rectangles making up the key
        let border = egui::Shape::closed_line(
            points,
            egui::epaint::Stroke {
                width: 1.0,
                color: egui::epaint::Color32::GRAY,
            },
        );
        //rectangles for the top and bottom portion of the key:
        let top_key = egui::epaint::RectShape {
            rect: egui::Rect {
                min: egui::pos2(cursor.x + r.x as f32, cursor.y + r.y as f32),
                max: egui::pos2(
                    cursor.x + (r.x + r.width) as f32,
                    cursor.y + (r.y + r.height) as f32,
                ),
            },
            rounding: egui::epaint::Rounding::none(),
            fill: egui::epaint::Color32::WHITE,
            stroke: egui::epaint::Stroke::NONE,
        };
        let bottom_key = egui::epaint::RectShape {
            rect: egui::Rect {
                min: egui::pos2(cursor.x + wide.x as f32, cursor.y + wide.y as f32),
                max: egui::pos2(
                    cursor.x + (wide.x + wide.width) as f32,
                    cursor.y + (wide.y + wide.height) as f32,
                ),
            },
            rounding: egui::epaint::Rounding::none(),
            fill: egui::epaint::Color32::WHITE,
            stroke: egui::epaint::Stroke::NONE,
        };
        (border, [top_key, bottom_key])
    }
    /// Returns true if `point`` is within (exclusive of borders) `rect`
    fn point_in_rect(point: &egui::Pos2, rect: &egui::epaint::Rect) -> bool {
        point.x > rect.min.x && point.x < rect.max.x && point.y > rect.min.y && point.y < rect.max.y
    }
    /// Draw the keyboard built into `kbd` on `ui`, returning either:
    ///  - the MIDI note number for the key the mouse is on if clicked/dragged
    ///  - `None` if the user has not clicked or the mouse is not on a key
    fn draw_kbd(kbd: Keyboard2d, ui: &mut egui::Ui) -> Option<i8> {
        let response = ui.allocate_response(
            egui::vec2(kbd.width as f32, kbd.height as f32),
            egui::Sense::click_and_drag(),
        );
        let pointer = response.interact_pointer_pos();
        let mut new_note: Option<i8> = None;
        let cursor = response.rect.min;
        for (i, k) in kbd.iter().enumerate() {
            match k {
                Element::WhiteKey { wide, small, blind } => {
                    let (border, mut rects) = Self::draw_white_key(&cursor, small, wide, blind);
                    match pointer {
                        None => {}
                        Some(pos) => {
                            if Self::point_in_rect(&pos, &rects[0].rect)
                                || Self::point_in_rect(&pos, &rects[1].rect)
                            {
                                new_note = Some((kbd.left_white_key + i as u8) as i8);
                                rects[0].fill = egui::epaint::Color32::GOLD;
                                rects[1].fill = egui::epaint::Color32::GOLD;
                            }
                        }
                    }
                    ui.painter().add(border);
                    ui.painter().add(rects[0]);
                    ui.painter().add(rects[1]);
                }
                Element::BlackKey(r) => {
                    let mut key = egui::epaint::RectShape {
                        rect: egui::Rect {
                            min: egui::pos2(cursor.x + r.x as f32, cursor.y + r.y as f32),
                            max: egui::pos2(
                                cursor.x + (r.x + r.width) as f32,
                                cursor.y + (r.y + r.height) as f32,
                            ),
                        },
                        rounding: egui::epaint::Rounding::none(),
                        fill: egui::epaint::Color32::BLACK,
                        stroke: egui::epaint::Stroke {
                            width: 1.0,
                            color: egui::epaint::Color32::GRAY,
                        },
                    };
                    match pointer {
                        None => {}
                        Some(pos) => {
                            if Self::point_in_rect(&pos, &key.rect) {
                                new_note = Some((kbd.left_white_key + i as u8) as i8);
                                key.fill = egui::epaint::Color32::GOLD;
                            }
                        }
                    }
                    ui.painter().add(key);
                }
            }
        }
        new_note
    }
    /// Draw the bottom keyboard panel and handle keyboard/mouse input so the user
    /// can interact with the plugin without sending it MIDI
    fn draw_kbd_panel(&mut self, egui_ctx: &Context) {
        egui::TopBottomPanel::bottom("keyboard").show(egui_ctx, |ui| {
            self.handle_kbd_input(ui);
            let keyboard = KeyboardBuilder::new()
                .white_black_gap_present(false)
                .set_width(ui.available_width() as u16)
                .and_then(|x| x.standard_piano(49))
                .map(|x| x.build2d());
            match keyboard {
                Ok(kbd) => {
                    let new_note = Self::draw_kbd(kbd, ui);
                    // now send the MIDI events if required:
                    if new_note != self.last_note {
                        match self.last_note {
                            None => {}
                            Some(k) => {
                                // note off the last note
                                let _ = self.channel.try_send(k + (-128));
                            }
                        }
                        match new_note {
                            None => {}
                            Some(k) => {
                                // note on the new note
                                let _ = self.channel.try_send(k);
                            }
                        }
                    }
                    self.last_note = new_note;
                }
                Err(s) => {
                    nih_log!("{}", s);
                }
            }
        });
    }
    /// Draw the editor panel
    pub fn update(&mut self, egui_ctx: &Context, setter: &ParamSetter) {
        self.draw_kbd_panel(egui_ctx);
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    self.params.osc1.draw_on(ui, setter, "Oscillator 1");
                    ui.separator();
                    self.params.osc2.draw_on(ui, setter, "Oscillator 2");
                    ui.separator();
                    self.params.ringmod.draw_on(ui, setter, "Mixer/Ring Modulator");
                    ui.separator();
                    self.params.filt.draw_on(ui, setter, "Filter");
                });
                ui.horizontal(|ui| {
                    self.params.env_vca.draw_on(ui, setter, "VCA Envelope");
                    ui.separator();
                    self.params.env_vcf.draw_on(ui, setter, "Filter Envelope");
                });
            });
        });
    }
    /// Helper function to be passed as a callback to `create_egui_editor`
    pub fn update_helper(egui_ctx: &Context, setter: &ParamSetter, state: &mut Self) {
        state.update(egui_ctx, setter)
    }
}

pub fn create(params: Arc<JanusParams>, tx: SyncSender<i8>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        JanusEditor::new(params, tx),
        |_, _| {},
        JanusEditor::update_helper,
    )
}
