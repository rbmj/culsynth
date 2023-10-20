pub fn calculate_cents(base: f32, freq: f32) -> f32 {
    1200.0*f32::log2(freq/base)
}

pub fn midi_note_pretty(note: i8) -> String {
    const NOTES : [&str; 12] = [
        "A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"];
    let rel_to_a0 = note - 21;
    format!("{}{}", NOTES[(rel_to_a0 % 12) as usize], rel_to_a0 / 12)
}