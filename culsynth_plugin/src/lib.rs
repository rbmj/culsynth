//! This contains all the code required to generate the actual plugins using the `nih-plug`
//! framework.  Most of GUI code is in the [editor] module.
use culsynth::{CoarseTuneFxP, FineTuneFxP};
use wmidi::MidiMessage;

#[cfg(not(feature = "audioworklet"))]
pub mod editor;

mod voicealloc;

pub use culsynth as backend;

#[derive(Default, Clone, Copy)]
pub struct Tuning {
    pub fine: FineTuneFxP,
    pub coarse: CoarseTuneFxP,
}

#[cfg(feature = "nih")]
pub mod nih;

#[cfg(feature = "nih")]
pub(crate) use nih_plug_egui::egui;

#[cfg(all(not(feature = "nih"), not(feature = "audioworklet")))]
pub(crate) use egui;

#[cfg(feature = "eframe")]
pub mod eframe;

#[cfg(target_family = "wasm")]
pub mod wasm;

#[cfg(feature = "audioworklet")]
pub mod audioworklet;

pub mod ownedmidihandler;

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

pub const NAME: &'static str = "CulSynth";
pub const VENDOR: &'static str = "rbmj";
pub const URL: &'static str = "https://github.com/rbmj/culsynth";
pub const EMAIL: &'static str = "rbmj@verizon.net";
pub const ID: &'static str = "com.rbmj.culsynth";
pub const DESCRIPTION: Option<&'static str> = Some("Culsynth Softsynth");
pub const CLASS_ID: [u8; 16] = *b"CulSySynthesizer";
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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

pub trait ContextReader {
    /// Get the current context.  Returns a tuple of (sample_rate, fixed_point).
    fn get(&self) -> (u32, bool) {
        (self.sample_rate(), self.is_fixed())
    }
    fn sample_rate(&self) -> u32;
    fn is_fixed(&self) -> bool;
    fn bufsz(&self) -> usize;
    fn voice_mode(&self) -> VoiceMode;
}
