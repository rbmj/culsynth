use culsynth::{VoiceFxP, voice::VoiceParamsFxP, voice::modulation::ModMatrixFxP};
use culsynth::context::ContextFxP;

fn synth_params<'a>() -> VoiceParamsFxP<'a> {
    todo!()
}

pub fn run(voices: &mut [&mut VoiceFxP; crate::NUM_VOICES]) -> ! {
    const CONTEXT: ContextFxP = ContextFxP::new_480();
    let matrix = ModMatrixFxP::default();
    let notebuf = culsynth::devices::fixed_zerobuf::<culsynth::NoteFxP>();
    let gatebuf = culsynth::devices::fixed_zerobuf::<culsynth::SampleFxP>();
    let velbuf = culsynth::devices::fixed_zerobuf::<culsynth::ScalarFxP>();
    let aftertouchbuf = culsynth::devices::fixed_zerobuf::<culsynth::ScalarFxP>();
    let modwheelbuf = culsynth::devices::fixed_zerobuf::<culsynth::ScalarFxP>();
    loop {
        for voice in voices.iter_mut() {
            voice.process(&CONTEXT, &matrix, notebuf, gatebuf, velbuf, aftertouchbuf, modwheelbuf, synth_params());
        }
    }
}