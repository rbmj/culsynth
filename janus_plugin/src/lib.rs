//! This contains all the code required to generate the actual plugins using the `nih-plug`
//! framework.  Most of GUI code is in the [editor] module.
use janus::context::{Context, ContextFxP, GenericContext};
use nih_plug::prelude::*;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{atomic::AtomicI32, Arc};

mod editor;

mod fixedparam;

pub mod parambuf;
use parambuf::{
    EnvParamBuffer, FiltParamBuffer, GlobalParamBuffer, OscParamBuffer, RingModParamBuffer,
};

pub mod pluginparams;
use pluginparams::JanusParams;

mod voicealloc;
use voicealloc::{MonoSynth, MonoSynthFxP, VoiceAllocator};

struct PluginContext {
    sample_rate: AtomicI32,
    bufsz: AtomicUsize,
}

impl Default for PluginContext {
    fn default() -> Self {
        Self {
            sample_rate: AtomicI32::new(-44100),
            bufsz: AtomicUsize::new(2048),
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
}

/// Contains all of the global state for the plugin
pub struct JanusPlugin {
    params: Arc<JanusParams>,

    glob_params: GlobalParamBuffer,
    osc1_params: OscParamBuffer,
    osc2_params: OscParamBuffer,
    ringmod_params: RingModParamBuffer,
    filt_params: FiltParamBuffer,
    env_amp_params: EnvParamBuffer,
    env_filt_params: EnvParamBuffer,

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

    context: Arc<PluginContext>,
}

impl JanusPlugin {
    fn update_sample_rate(&mut self, sr: u32, fixed: bool) {
        let mut value = sr as i32;
        if fixed {
            value = -value;
        }
        self.context.sample_rate.store(value, Relaxed);
    }
    fn update_context(&mut self, ctx: &dyn GenericContext) {
        let sr = ctx.sample_rate();
        let fixed = ctx.is_fixed_point();
        self.update_sample_rate(sr, fixed)
    }
    fn get_context_reader(&mut self) -> ContextReader {
        ContextReader {
            context: self.context.clone(),
        }
    }
}

impl Default for JanusPlugin {
    fn default() -> Self {
        let (midi_tx, midi_rx) = sync_channel::<i8>(32);
        let (synth_tx, synth_rx) = sync_channel::<Box<dyn VoiceAllocator>>(1);
        Self {
            params: Arc::new(JanusParams::default()),
            glob_params: Default::default(),
            osc1_params: Default::default(),
            osc2_params: Default::default(),
            ringmod_params: Default::default(),
            filt_params: Default::default(),
            env_amp_params: Default::default(),
            env_filt_params: Default::default(),
            midi_tx,
            midi_rx,
            synth_tx,
            synth_rx,
            voices: None,
            context: Arc::new(Default::default()),
        }
    }
}

impl Plugin for JanusPlugin {
    const NAME: &'static str = "Janus";
    const VENDOR: &'static str = "rbmj";
    const URL: &'static str = "https://github.com/rbmj/janus";
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

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.midi_tx.clone(),
            self.synth_tx.clone(),
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
        nih_trace!(
            "Initializing Plugin with {}Hz SR and {} sample buffers",
            buffer_config.sample_rate,
            buffer_config.max_buffer_size,
        );
        let bufsz = buffer_config.max_buffer_size;
        self.glob_params.allocate(bufsz);
        self.osc1_params.allocate(bufsz);
        self.osc2_params.allocate(bufsz);
        self.ringmod_params.allocate(bufsz);
        self.filt_params.allocate(bufsz);
        self.env_amp_params.allocate(bufsz);
        self.env_filt_params.allocate(bufsz);
        let mut voice_alloc: Box<dyn VoiceAllocator> = if buffer_config.sample_rate == 44100f32 {
            Box::new(MonoSynthFxP::new(ContextFxP::new_441()))
        } else if buffer_config.sample_rate == 48000f32 {
            Box::new(MonoSynthFxP::new(ContextFxP::new_480()))
        } else {
            Box::new(MonoSynth::new(Context::new(buffer_config.sample_rate)))
        };
        voice_alloc.initialize(bufsz as usize);
        let ctx = voice_alloc.get_context();
        self.update_sample_rate(ctx.sample_rate(), ctx.is_fixed_point());
        self.voices = Some(voice_alloc);
        self.context.bufsz.store(bufsz as usize, Relaxed);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if let Ok(synth) = self.synth_rx.try_recv() {
            self.update_context(synth.get_context());
            self.voices = Some(synth);
        }
        let voices = match self.voices {
            Some(ref mut x) => x,
            None => return ProcessStatus::Error("Uninitialized"),
        };
        let mut index = 0;
        while let Ok(note) = self.midi_rx.try_recv() {
            if note < 0 {
                voices.note_off((note - (-128)) as u8, 0);
            } else {
                voices.note_on(note as u8, 100);
            }
        }
        assert!(buffer.samples() <= self.context.bufsz.load(Relaxed));
        let mut next_event = context.next_event();
        for (sample_id, _channel_samples) in buffer.iter_samples().enumerate() {
            self.glob_params.update_index(index, &self.params.osc_sync);
            self.osc1_params.update_index(index, &self.params.osc1);
            self.osc2_params.update_index(index, &self.params.osc2);
            self.ringmod_params.update_index(index, &self.params.ringmod);
            self.filt_params.update_index(index, &self.params.filt);
            self.env_amp_params.update_index(index, &self.params.env_vca);
            self.env_filt_params.update_index(index, &self.params.env_vcf);

            // Process MIDI events:
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        voices.note_on(note, (velocity * 127f32) as u8);
                    }
                    NoteEvent::NoteOff { note, velocity, .. } => {
                        voices.note_off(note, (velocity * 127f32) as u8);
                    }
                    _ => (),
                }
                next_event = context.next_event();
            }
            voices.sample_tick();
            index += 1;
        }
        if !(voices.get_context().is_fixed_point()) {
            self.glob_params.conv_float();
            self.osc1_params.conv_float();
            self.osc2_params.conv_float();
            self.ringmod_params.conv_float();
            self.filt_params.conv_float();
            self.env_amp_params.conv_float();
            self.env_filt_params.conv_float();
        }
        let output = voices.process(
            &mut self.glob_params,
            &self.osc1_params,
            &self.osc2_params,
            &self.ringmod_params,
            &self.filt_params,
            &self.env_filt_params,
            &self.env_amp_params,
        );
        index = 0;
        for channel_samples in buffer.iter_samples() {
            for smp in channel_samples {
                *smp = output[index];
            }
            index += 1;
        }
        // To save resources, a plugin can (and probably should!) only perform expensive
        // calculations that are only displayed on the GUI while the GUI is open
        if self.params.editor_state.is_open() {
            //Do editor update logic
        }
        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for JanusPlugin {
    const CLAP_ID: &'static str = "com.rbmj.janus";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Janus Softsynth");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for JanusPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"JanusSynthesizer";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Synth];
}

nih_export_clap!(JanusPlugin);
nih_export_vst3!(JanusPlugin);
