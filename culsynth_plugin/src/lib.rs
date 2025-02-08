//! This contains all the code required to generate the actual plugins using the `nih-plug`
//! framework.  Most of GUI code is in the [editor] module.
use culsynth::context::GenericContext;
use culsynth::{CoarseTuneFxP, FineTuneFxP};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicI32, AtomicU32, AtomicUsize};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::Arc;

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

#[cfg(target_family = "wasm")]
pub mod wasm;

pub trait MidiSender {
    fn send_midi(&mut self, msg: MidiMessage<'static>);
}

pub trait MidiReceiver {
    fn recv_midi(&mut self) -> Option<MidiMessage<'static>>;
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

struct PluginContext {
    sample_rate: AtomicI32,
    bufsz: AtomicUsize,
    voice_mode: AtomicU32,
}

impl Default for PluginContext {
    fn default() -> Self {
        Self {
            sample_rate: AtomicI32::new(-44100),
            bufsz: AtomicUsize::new(2048),
            voice_mode: AtomicU32::new(0),
        }
    }
}

pub struct ContextReader {
    context: Arc<PluginContext>,
}

impl ContextReader {
    /// Get the current context.  Returns a tuple of (sample_rate, fixed_point).
    pub fn get(&self) -> (u32, bool) {
        let mut fixed = false;
        let mut sr = self.context.sample_rate.load(Relaxed);
        if sr < 0 {
            fixed = true;
            sr = -sr;
        }
        (sr as u32, fixed)
    }
    pub fn sample_rate(&self) -> u32 {
        let (sr, _) = self.get();
        sr
    }
    pub fn is_fixed(&self) -> bool {
        let (_, fixed) = self.get();
        fixed
    }
    pub fn bufsz(&self) -> usize {
        self.context.bufsz.load(Relaxed)
    }
    pub fn voice_mode(&self) -> VoiceMode {
        let mode_u32 = self.context.voice_mode.load(Relaxed);
        unsafe { std::mem::transmute((mode_u32 & 0xFF) as u8) }
    }
}
