use crate::pluginparams::{
    EnvPluginParams, FiltPluginParams, JanusParams, LfoPluginParams, OscPluginParams,
    RingModPluginParams,
};
use crate::voicealloc::{MonoSynth, MonoSynthFxP, VoiceAllocator};
use crate::ContextReader;
use egui::widgets;
use janus::context::{Context, ContextFxP};
use janus::devices::LfoWave;
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

fn draw_osc(
    osc: &OscPluginParams,
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    label: &str,
    draw_sync: bool,
    sync_on: bool
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
            ui.label("Course");
            ui.label("Fine");
            ui.label("Shape");
            ui.label("Sine");
            ui.label("Tri");
            ui.label("Square");
            ui.label("Saw");
        });
    });
    sync_clicked
}

impl PluginWidget for OscPluginParams {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        draw_osc(self, ui, setter, label, false, false);
    }
}

impl<T: Fn() -> ()> PluginWidget for (&OscPluginParams, bool, T) {
    fn draw_on(&self, ui: &mut egui::Ui, setter: &ParamSetter, label: &str) {
        if draw_osc(self.0, ui, setter, label, true, self.1) {
            self.2()
        }
    }
}

impl PluginWidget for LfoPluginParams {
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
                        if ui.selectable_label(
                            cur_wave == *wave as i32,
                            wave.to_str_short()
                        ).clicked() {
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
    EguiState::from_size(1200, 800)
}

/// Struct to hold the global state information for the plugin editor (GUI).
struct JanusEditor {
    params: Arc<JanusParams>,
    midi_channel: SyncSender<i8>,
    synth_channel: SyncSender<Box<dyn VoiceAllocator>>,
    context: ContextReader,
    last_note: Option<i8>,
}

impl JanusEditor {
    pub fn new(
        p: Arc<JanusParams>,
        midi_tx: SyncSender<i8>,
        synth_tx: SyncSender<Box<dyn VoiceAllocator>>,
        ctx: ContextReader,
    ) -> Self {
        JanusEditor {
            params: p,
            midi_channel: midi_tx,
            synth_channel: synth_tx,
            context: ctx,
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
                        let _ = self.midi_channel.try_send(k);
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
        let cursor = response.rect.left_top();
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
    fn draw_kbd_panel(&mut self, egui_ctx: &egui::Context) {
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
                                let _ = self.midi_channel.try_send(k + (-128));
                            }
                        }
                        match new_note {
                            None => {}
                            Some(k) => {
                                // note on the new note
                                let _ = self.midi_channel.try_send(k);
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
    fn draw_status_bar(&mut self, egui_ctx: &egui::Context) {
        egui::TopBottomPanel::top("status")
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(32)))
            .max_height(20f32)
            .show(egui_ctx, |ui| {
                let width = ui.available_width();
                let third = width / 3f32;
                ui.columns(3, |columns| {
                    columns[0].horizontal_centered(|ui| {
                        //ui.label("CPU PERCENT");
                        ui.label("");
                    });
                    columns[0].expand_to_include_x(third);
                    columns[1].expand_to_include_x(width - third);
                    columns[2].with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let (sr, fixed_point) = self.context.get();
                            let fixed_str = if fixed_point {
                                "16 bit fixed"
                            } else {
                                "32 bit float"
                            };
                            ui.label(format!(
                                "{}.{} kHz / {}",
                                sr / 1000,
                                (sr % 1000) / 100,
                                fixed_str,
                            ))
                            .context_menu(|ui| {
                                let fixed_context = ContextFxP::maybe_create(sr);
                                let mut new_synth: Option<Box<dyn VoiceAllocator>> = None;
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        if fixed_context.is_none() {
                                            ui.set_enabled(false);
                                        }
                                        if ui.selectable_label(fixed_point, "Fixed").clicked() {
                                            ui.close_menu();
                                            if let Some(context) = fixed_context {
                                                if !fixed_point {
                                                    new_synth =
                                                        Some(Box::new(MonoSynthFxP::new(context)));
                                                }
                                            }
                                        }
                                    });
                                    if ui.selectable_label(!fixed_point, "Float").clicked() {
                                        ui.close_menu();
                                        if fixed_point {
                                            let ctx = Context::new(sr as f32);
                                            new_synth = Some(Box::new(MonoSynth::new(ctx)));
                                        }
                                    }
                                    if let Some(mut synth) = new_synth {
                                        synth.initialize(self.context.bufsz());
                                        if let Err(e) = self.synth_channel.try_send(synth) {
                                            nih_log!("{}", e);
                                        }
                                    }
                                });
                            });
                        },
                    );
                    columns[1].centered_and_justified(|ui| {
                        ui.label(format!("Janus v{}", env!("CARGO_PKG_VERSION")));
                    });
                });
            });
    }
    /// Draw the editor panel
    pub fn update(&mut self, egui_ctx: &egui::Context, setter: &ParamSetter) {
        self.draw_status_bar(egui_ctx);
        self.draw_kbd_panel(egui_ctx);
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            ui.spacing_mut().slider_width = 130f32;
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    self.params.osc1.draw_on(ui, setter, "Oscillator 1");
                    ui.separator();
                    let sync_on = self.params.osc_sync.value();
                    (&self.params.osc2, sync_on, || {
                        setter.begin_set_parameter(&self.params.osc_sync);
                        setter.set_parameter(&self.params.osc_sync, !sync_on);
                        setter.end_set_parameter(&self.params.osc_sync);
                    }).draw_on(ui, setter, "Oscillator 2");
                    ui.separator();
                    self.params.ringmod.draw_on(ui, setter, "Mixer/Ring Modulator");
                });
                ui.horizontal(|ui| {
                    self.params.filt.draw_on(ui, setter, "Filter");
                    ui.separator();
                    self.params.lfo1.draw_on(ui, setter, "LFO 1");
                    ui.separator();
                    self.params.lfo2.draw_on(ui, setter, "LFO 2");
                });
                ui.horizontal(|ui| {
                    self.params.env_vcf.draw_on(ui, setter, "Filter Envelope");
                    ui.separator();
                    self.params.env_vca.draw_on(ui, setter, "Amplifier Envelope");
                    ui.separator();
                    self.params.env1.draw_on(ui, setter, "Mod Envelope 1");
                    ui.separator();
                    self.params.env2.draw_on(ui, setter, "Mod Envelope 2");
                });
            });
        });
    }
    pub fn initialize(&mut self, egui_ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "janus_noto_sans_math".to_owned(),
            egui::FontData::from_static(
                include_bytes!("../../resources/fonts/NotoSansMath-Regular.ttf"),
            ),
        );
        fonts.font_data.insert(
            "janus_noto_sans_sym".to_owned(),
            egui::FontData::from_static(
                include_bytes!("../../resources/fonts/NotoSansSymbols-Regular.ttf"),
            ),
        );
        fonts.font_data.insert(
            "janus_noto_sans_math".to_owned(),
            egui::FontData::from_static(
                include_bytes!("../../resources/fonts/NotoSansMath-Regular.ttf"),
            ),
        );
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .extend_from_slice(&[
                "janus_noto_sans_math".to_owned(),
                "janus_noto_sans_sym".to_owned(),
            ]);
        
        egui_ctx.set_fonts(fonts);
    }
}

pub fn create(
    params: Arc<JanusParams>,
    midi_tx: SyncSender<i8>,
    synth_tx: SyncSender<Box<dyn VoiceAllocator>>,
    context: ContextReader,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        JanusEditor::new(params, midi_tx, synth_tx, context),
        |ctx, editor| editor.initialize(ctx),
        |ctx, setter, editor| editor.update(ctx, setter),
    )
}
