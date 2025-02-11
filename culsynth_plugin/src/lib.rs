//! This contains all the code required to generate the actual plugins using the `nih-plug`
//! framework.  Most of GUI code is in the [editor] module.
use culsynth::context::GenericContext;
use culsynth::{CoarseTuneFxP, FineTuneFxP};
use std::sync::atomic::{AtomicI32, AtomicU32, AtomicUsize};
use std::sync::mpsc::{Receiver, SyncSender};

use wmidi::MidiMessage;

mod editor;

mod fixedparam;

pub mod pluginparams;
use pluginparams::CulSynthParams;

mod voicealloc;
use voicealloc::{PolySynth, VoiceAllocator};

struct Tuning {
    fine: FineTuneFxP,
    coarse: CoarseTuneFxP,
}

#[cfg(not(target_family = "wasm"))]
pub mod nih;

#[cfg(not(target_family = "wasm"))]
pub(crate) use nih_plug_egui::egui;

#[cfg(target_family = "wasm")]
pub(crate) use egui;

#[cfg(target_family = "wasm")]
pub mod wasm;

pub trait MidiHandler {
    fn send(&self, msg: MidiMessage<'static>);
    fn ch(&self) -> wmidi::Channel;
    fn send_cc(&self, cc: wmidi::ControlFunction, value: wmidi::U7) {
        self.send(MidiMessage::ControlChange(self.ch(), cc, value));
    }
    fn send_nrpn(&self, nrpn: wmidi::U14, value: wmidi::U14) {
        let nrpn: u16 = nrpn.into();
        let value: u16 = value.into();
        self.send_cc(
            wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_MSB,
            wmidi::U7::from_u8_lossy((nrpn >> 7) as u8),
        );
        self.send_cc(
            wmidi::ControlFunction::NON_REGISTERED_PARAMETER_NUMBER_LSB,
            wmidi::U7::from_u8_lossy(nrpn as u8),
        );
        self.send_cc(
            wmidi::ControlFunction::DATA_ENTRY_MSB,
            wmidi::U7::from_u8_lossy((value >> 7) as u8),
        );
        self.send_cc(
            wmidi::ControlFunction::DATA_ENTRY_LSB,
            wmidi::U7::from_u8_lossy(value as u8),
        );
    }
}

const NAME: &'static str = "CulSynth";
const VENDOR: &'static str = "rbmj";
const URL: &'static str = "https://github.com/rbmj/culsynth";
const EMAIL: &'static str = "rbmj@verizon.net";
const ID: &'static str = "com.rbmj.culsynth";
const DESCRIPTION: Option<&'static str> = Some("Culsynth Softsynth");
const CLASS_ID: [u8; 16] = *b"CulSySynthesizer";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Default, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum VoiceMode {
    #[default]
    Mono,
    Poly16,
}

impl VoiceMode {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Mono => "Mono",
            Self::Poly16 => "Poly16",
        }
    }
}
