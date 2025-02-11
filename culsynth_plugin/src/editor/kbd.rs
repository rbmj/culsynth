use nih_plug::nih_log;
use nih_plug_egui::egui;
use piano_keyboard::Rectangle as PianoRectangle;
use piano_keyboard::{Element, Keyboard2d, KeyboardBuilder};

use crate::MidiHandler;

/// Map a keyboard key to a MIDI note number, or `None` if unmapped.
fn key_to_notenum(k: egui::Key) -> Option<u8> {
    match k {
        egui::Key::A => Some(culsynth::midi_const::C4 as u8),
        egui::Key::S => Some(culsynth::midi_const::D4 as u8),
        egui::Key::D => Some(culsynth::midi_const::E4 as u8),
        egui::Key::F => Some(culsynth::midi_const::F4 as u8),
        egui::Key::G => Some(culsynth::midi_const::G4 as u8),
        egui::Key::H => Some(culsynth::midi_const::A4 as u8),
        egui::Key::J => Some(culsynth::midi_const::B4 as u8),
        egui::Key::K => Some(culsynth::midi_const::C5 as u8),
        egui::Key::L => Some(culsynth::midi_const::D5 as u8),

        egui::Key::W => Some(culsynth::midi_const::Db4 as u8),
        egui::Key::E => Some(culsynth::midi_const::Eb4 as u8),
        egui::Key::T => Some(culsynth::midi_const::Gb4 as u8),
        egui::Key::Y => Some(culsynth::midi_const::Ab4 as u8),
        egui::Key::U => Some(culsynth::midi_const::Bb4 as u8),
        egui::Key::O => Some(culsynth::midi_const::Db5 as u8),
        egui::Key::P => Some(culsynth::midi_const::Eb5 as u8),
        _ => None,
    }
}

/// A keyboard panel that provides a UI to provide note events to the synth
/// without having a MIDI controller.  It will currently draw itself as an
/// [egui::TopBottomPanel]
#[derive(Default)]
pub struct KbdPanel {
    last_note: Option<u8>,
}

impl KbdPanel {
    /// Helper function to handle keyboard input
    #[rustfmt::skip]
    fn handle_kbd_input(&mut self, ui: &egui::Ui, midi_handler: &impl MidiHandler) {
        ui.input(|i| {
            for evt in i.events.iter() {
                if let egui::Event::Key{key, pressed, repeat, ..} = evt {
                    if *repeat { continue; }
                    if let Some(k) = key_to_notenum(*key) {
                        let note = wmidi::Note::from_u8_lossy(k);
                        if !(*pressed) {
                            midi_handler.send(wmidi::MidiMessage::NoteOff(wmidi::Channel::Ch1, note, wmidi::Velocity::MIN))
                        } else {
                            midi_handler.send(wmidi::MidiMessage::NoteOn(wmidi::Channel::Ch1, note, wmidi::Velocity::MAX))
                        }
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
        let top_key = egui::epaint::RectShape::new(
            egui::Rect {
                min: egui::pos2(cursor.x + r.x as f32, cursor.y + r.y as f32),
                max: egui::pos2(
                    cursor.x + (r.x + r.width) as f32,
                    cursor.y + (r.y + r.height) as f32,
                ),
            },
            egui::epaint::Rounding::ZERO,
            egui::epaint::Color32::WHITE,
            egui::epaint::Stroke::NONE,
        );
        let bottom_key = egui::epaint::RectShape::new(
            egui::Rect {
                min: egui::pos2(cursor.x + wide.x as f32, cursor.y + wide.y as f32),
                max: egui::pos2(
                    cursor.x + (wide.x + wide.width) as f32,
                    cursor.y + (wide.y + wide.height) as f32,
                ),
            },
            egui::epaint::Rounding::ZERO,
            egui::epaint::Color32::WHITE,
            egui::epaint::Stroke::NONE,
        );
        (border, [top_key, bottom_key])
    }
    /// Returns true if `point`` is within (exclusive of borders) `rect`
    fn point_in_rect(point: &egui::Pos2, rect: &egui::epaint::Rect) -> bool {
        point.x > rect.min.x && point.x < rect.max.x && point.y > rect.min.y && point.y < rect.max.y
    }
    /// Draw the keyboard built into `kbd` on `ui`, returning either:
    ///  - the MIDI note number for the key the mouse is on if clicked/dragged
    ///  - `None` if the user has not clicked or the mouse is not on a key
    fn draw_kbd(kbd: Keyboard2d, ui: &mut egui::Ui) -> Option<u8> {
        let response = ui.allocate_response(
            egui::vec2(kbd.width as f32, kbd.height as f32),
            egui::Sense::click_and_drag(),
        );
        let pointer = response.interact_pointer_pos();
        let mut new_note: Option<u8> = None;
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
                                new_note = Some(kbd.left_white_key + i as u8);
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
                    let mut key = egui::epaint::RectShape::new(
                        egui::Rect {
                            min: egui::pos2(cursor.x + r.x as f32, cursor.y + r.y as f32),
                            max: egui::pos2(
                                cursor.x + (r.x + r.width) as f32,
                                cursor.y + (r.y + r.height) as f32,
                            ),
                        },
                        egui::epaint::Rounding::ZERO,
                        egui::epaint::Color32::BLACK,
                        egui::epaint::Stroke {
                            width: 1.0,
                            color: egui::epaint::Color32::GRAY,
                        },
                    );
                    match pointer {
                        None => {}
                        Some(pos) => {
                            if Self::point_in_rect(&pos, &key.rect) {
                                new_note = Some(kbd.left_white_key + i as u8);
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
    /// Draw the bottom keyboard panel and handle keyboard/mouse input so the
    /// user can interact with the plugin without a MIDI controller
    pub fn show(&mut self, egui_ctx: &egui::Context, midi_handler: &impl MidiHandler) {
        egui::TopBottomPanel::bottom("keyboard").show(egui_ctx, |ui| {
            self.handle_kbd_input(ui, midi_handler);
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
                        if let Some(k) = self.last_note {
                            midi_handler.send(wmidi::MidiMessage::NoteOff(
                                wmidi::Channel::Ch1,
                                wmidi::Note::from_u8_lossy(k),
                                wmidi::Velocity::MIN,
                            ));
                        }
                        if let Some(k) = new_note {
                            midi_handler.send(wmidi::MidiMessage::NoteOn(
                                wmidi::Channel::Ch1,
                                wmidi::Note::from_u8_lossy(k),
                                wmidi::Velocity::MAX,
                            ));
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
}
