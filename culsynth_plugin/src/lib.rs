//! This contains all the code required to generate the actual plugins using the `nih-plug`
//! framework.  Most of GUI code is in the [editor] module.
use culsynth::context::GenericContext;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicI32, AtomicU32, AtomicUsize};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;

mod editor;

mod fixedparam;

pub mod pluginparams;
use pluginparams::CulSynthParams;

mod voicealloc;
use voicealloc::{PolySynth, VoiceAllocator};

#[cfg(not(target_family = "wasm"))]
mod nih;

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

/// Contains all of the global state for the plugin
pub struct CulSynthPlugin {
    params: Arc<CulSynthParams>,

    /// Used by the GUI thread to send MIDI events to the audio thread when,
    /// for example, a user presses a key on a on screen virtual keyboard.
    ///
    /// The MIDI event is packaged as an i8, where a positive integer indicates
    /// a "Note On" for that note number, and a negative integer indicates a
    /// "Note Off" at the absolute value of the integer
    midi_tx: SyncSender<i8>,

    /// Used by the audio thread to receive MIDI events from the GUI thread.
    ///
    /// The MIDI event is packaged as an i8, where a positive integer indicates
    /// a "Note On" for that note number, and a negative integer indicates a
    /// "Note Off" at the absolute value of the integer
    midi_rx: Receiver<i8>,

    /// Used by the GUI thread to replace the current synth engine
    synth_tx: SyncSender<Box<dyn VoiceAllocator>>,

    /// Used by the audio thread to receive replacement synth engines from the
    /// GUI thread.
    synth_rx: Receiver<Box<dyn VoiceAllocator>>,

    /// The sound engine currently in use to process audio for the synth.
    voices: Option<Box<dyn VoiceAllocator>>,

    /// Used by the audio thread to send control changes to the GUI
    cc_tx: SyncSender<(u8, u8)>,

    /// Used by the GUI thread to receive control changes
    cc_rx: Option<Receiver<(u8, u8)>>,

    context: Arc<PluginContext>,
}

impl CulSynthPlugin {
    fn update_sample_rate(&mut self, sr: u32, fixed: bool) {
        let mut value = sr as i32;
        if fixed {
            value = -value;
        }
        self.context.sample_rate.store(value, Relaxed);
    }
    fn update_context(&mut self, ctx: &dyn GenericContext, mode: VoiceMode) {
        let sr = ctx.sample_rate();
        let fixed = ctx.is_fixed_point();
        self.update_sample_rate(sr, fixed);
        self.context.voice_mode.store(mode as u32, Relaxed);
    }
    fn get_context_reader(&mut self) -> ContextReader {
        ContextReader {
            context: self.context.clone(),
        }
    }
}

impl Default for CulSynthPlugin {
    fn default() -> Self {
        let (midi_tx, midi_rx) = sync_channel::<i8>(32);
        let (cc_tx, cc_rx) = sync_channel::<(u8, u8)>(32);
        let (synth_tx, synth_rx) = sync_channel::<Box<dyn VoiceAllocator>>(1);
        Self {
            params: Arc::new(CulSynthParams::default()),
            midi_tx,
            midi_rx,
            synth_tx,
            synth_rx,
            cc_tx,
            cc_rx: Some(cc_rx),
            voices: None,
            context: Arc::new(Default::default()),
        }
    }
}
