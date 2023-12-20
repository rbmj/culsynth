//! Various utility functions and helpful constants

// currently the only users of this function are unit tests... shut up dead code warning
#[cfg(test)]
pub fn calculate_cents(base: f32, freq: f32) -> f32 {
    1200.0 * f32::log2(freq / base)
}

/*
// Is this the right place for this?
pub fn midi_note_pretty(note: i8) -> String {
    const NOTES: [&str; 12] = [
        "A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#",
    ];
    let rel_to_a0 = note - 21;
    format!("{}{}", NOTES[(rel_to_a0 % 12) as usize], rel_to_a0 / 12)
}
*/

/// A character depicting a sine wave (∿)
pub const SIN_CHARSTR: &'static str = "\u{223F}";
/// A character depicting a square wave (⎍).
/// This is actually the Unicode "monostable symbol"
pub const SQ_CHARSTR: &'static str = "\u{238D}";
/// A character depicting a triangle wave (Λ).  This is the greek capital
/// lambda, so use a sans-serif font for this to appear correct
pub const TRI_CHARSTR: &'static str = "\u{039B}";
/// A character depicting a sawtooth wave (⩘).  This is the "sloping large and".
pub const SAW_CHARSTR: &'static str = "\u{2A58}";
