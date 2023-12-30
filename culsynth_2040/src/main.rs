#![no_std]
#![no_main]

use panic_halt as _;
use rp_pico::entry;

use core::mem::MaybeUninit;

use arrayvec::ArrayVec;
use culsynth::voice::VoiceFxP;

mod run;

const NUM_VOICES: usize = 2;

static mut VOICES: [MaybeUninit<VoiceFxP>; NUM_VOICES] =
    [MaybeUninit::uninit(), MaybeUninit::uninit()];

#[entry]
fn start() -> ! {
    const SIZE_OF_VOICE: usize = core::mem::size_of::<VoiceFxP>();
    let mut voices = unsafe {
        VOICES
            .iter_mut()
            .map(|x| x.write(VoiceFxP::new()))
            .collect::<ArrayVec<&mut VoiceFxP, NUM_VOICES>>()
            .into_inner_unchecked()
    };
    run::run(&mut voices)
}
