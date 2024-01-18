#![no_std]
#![no_main]

use panic_halt as _;
use rp_pico::entry;

use core::mem::MaybeUninit;

use arrayvec::ArrayVec;

type Voice = culsynth::voice::Voice<i16>;
type VoiceInput = culsynth::voice::VoiceInput<i16>;
type VoiceChannelInput = culsynth::voice::VoiceChannelInput<i16>;
type VoiceParams = culsynth::voice::VoiceParams<i16>;
type ModMatrix = culsynth::voice::modulation::ModMatrix<i16>;

mod run;

const NUM_VOICES: usize = 2;

static mut VOICES: [MaybeUninit<Voice>; NUM_VOICES] =
    [MaybeUninit::uninit(), MaybeUninit::uninit()];

#[entry]
fn start() -> ! {
    const SIZE_OF_VOICE: usize = core::mem::size_of::<Voice>();
    let mut voices = unsafe {
        VOICES
            .iter_mut()
            .map(|x| x.write(Voice::new()))
            .collect::<ArrayVec<&mut Voice, NUM_VOICES>>()
            .into_inner_unchecked()
    };
    run::run(&mut voices)
}
