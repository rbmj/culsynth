use super::*;
use culsynth::context::ContextFxP as Context;

const BUFSZ: usize = 32;

pub fn run(voices: &mut [&mut Voice; NUM_VOICES]) -> ! {
    const CONTEXT: Context = Context::new_480();
    let matrix = ModMatrix::default();
    let input = VoiceInput::default();
    let ch_input = VoiceChannelInput::default();

    let params = VoiceParams::default();
    loop {
        for voice in voices.iter_mut() {
            let smp = voice.next(&CONTEXT, &matrix, &input, &ch_input, params.clone());
        }
    }
}
