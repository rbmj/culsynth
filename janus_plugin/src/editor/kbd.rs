use nih_plug::nih_log;
use nih_plug_egui::egui;
use piano_keyboard::Rectangle as PianoRectangle;
use piano_keyboard::{Element, Keyboard2d, KeyboardBuilder};

pub struct KbdPanel {
    last_note: Option<i8>,
}

impl Default for KbdPanel {
    fn default() -> Self {
        Self { last_note: None }
    }
}

impl KbdPanel {
    /// Helper function to handle keyboard input
    #[rustfmt::skip]
    fn handle_kbd_input(&mut self, ui: &egui::Ui, events: &mut Vec<i8>) {
        ui.input(|i| {
            for evt in i.events.iter() {
                if let egui::Event::Key{key, pressed, repeat, ..} = evt {
                    if *repeat { continue; }
                    if let Some(mut k) = super::key_to_notenum(*key) {
                        if !(*pressed) {
                            k += -128; //Note off
                        }
                        events.push(k);
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
    /// Draw the bottom keyboard panel and handle keyboard/mouse input so the
    /// user can interact with the plugin without a MIDI controller
    ///
    /// The return value is a Vec of MIDI note on/off events, with off denoted
    /// by (notenum - 128)
    pub fn show(&mut self, egui_ctx: &egui::Context) -> Vec<i8> {
        let mut ret: Vec<i8> = Default::default();
        egui::TopBottomPanel::bottom("keyboard").show(egui_ctx, |ui| {
            self.handle_kbd_input(ui, &mut ret);
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
                                ret.push(k + (-128));
                            }
                        }
                        match new_note {
                            None => {}
                            Some(k) => {
                                // note on the new note
                                ret.push(k);
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
        ret
    }
}
