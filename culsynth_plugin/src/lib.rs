//! This contains all the code required to generate the actual plugins using the `nih-plug`
//! framework.  Most of GUI code is in the [editor] module.
use culsynth::context::{Context, GenericContext};
use nih_plug::prelude::*;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU32, AtomicUsize};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{atomic::AtomicI32, Arc};

mod editor;

mod fixedparam;

pub mod pluginparams;
use pluginparams::CulSynthParams;

mod voicealloc;
use voicealloc::{PolySynth, VoiceAllocator};

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

impl Plugin for CulSynthPlugin {
    const NAME: &'static str = "CulSynth";
    const VENDOR: &'static str = "rbmj";
    const URL: &'static str = "https://github.com/rbmj/culsynth";
    const EMAIL: &'static str = "rbmj@verizon.net";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let cc_rx = match self.cc_rx.take() {
            Some(x) => x,
            None => {
                let (tx, rx) = sync_channel::<(u8, u8)>(32);
                self.cc_tx = tx;
                rx
            }
        };
        editor::create(
            self.params.clone(),
            self.midi_tx.clone(),
            self.synth_tx.clone(),
            cc_rx,
            self.get_context_reader(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // TODO
        nih_log!(
            "Initializing Plugin with {}Hz SR and {} sample buffers",
            buffer_config.sample_rate,
            buffer_config.max_buffer_size,
        );
        if culsynth::USE_LIBM {
            nih_log!("Using libm for floating-point math");
        } else {
            nih_log!("Using internal floating-point math");
        }
        // JACK doesn't seem to honor max_buffer_size, so allocate more...
        let bufsz = std::cmp::max(buffer_config.max_buffer_size, 2048) as usize;
        let voice_alloc: Box<dyn VoiceAllocator> = Box::new(PolySynth::<f32>::new(
            Context::new(buffer_config.sample_rate),
            16,
        ));
        let ctx = voice_alloc.get_context();
        self.update_context(
            ctx,
            if voice_alloc.is_poly() {
                VoiceMode::Poly16
            } else {
                VoiceMode::Mono
            },
        );
        self.context.bufsz.store(bufsz, Relaxed);
        self.voices = Some(voice_alloc);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if let Ok(synth) = self.synth_rx.try_recv() {
            self.update_context(
                synth.get_context(),
                if synth.is_poly() {
                    VoiceMode::Poly16
                } else {
                    VoiceMode::Mono
                },
            );
            self.voices = Some(synth);
        }
        let voices = match self.voices {
            Some(ref mut x) => x,
            None => return ProcessStatus::Error("Uninitialized"),
        };
        while let Ok(note) = self.midi_rx.try_recv() {
            if note < 0 {
                voices.note_off((note - (-128)) as u8, 0);
            } else {
                voices.note_on(note as u8, 100);
            }
        }
        assert!(buffer.samples() <= self.context.bufsz.load(Relaxed));
        voices.process(
            buffer.iter_samples(),
            context,
            self.params.as_ref(),
            &mut self.cc_tx,
            Some((&self.params.modmatrix).into()),
        );
        // To save resources, a plugin can (and probably should!) only perform expensive
        // calculations that are only displayed on the GUI while the GUI is open
        if self.params.editor_state.is_open() {
            //Do editor update logic
        }
        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for CulSynthPlugin {
    const CLAP_ID: &'static str = "com.rbmj.culsynth";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Culsynth Softsynth");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for CulSynthPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"CulSySynthesizer";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Synth];
}

nih_export_clap!(CulSynthPlugin);
nih_export_vst3!(CulSynthPlugin);
