//! This contains all the code required to generate the actual plugins using the `nih-plug`
//! framework.  Most of GUI code is in the [editor] module.

use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;

use janus::{EnvParamFxP, NoteFxP, ScalarFxP};

mod editor;

mod fixedparam;
use fixedparam::{new_fixed_param, new_fixed_param_freq, new_fixed_param_percent};

pub mod parambuf;
use parambuf::{EnvParamBuffer, FiltParamBuffer, OscParamBuffer, RingModParamBuffer, GlobalParamBuffer};

mod voicealloc;
use voicealloc::{MonoSynthFxP, VoiceAllocator};

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
    tx: SyncSender<i8>,

    /// Used by the audio thread to receive MIDI events from the GUI thread.
    ///
    /// The MIDI event is packaged as an i8, where a positive integer indicates
    /// a "Note On" for that note number, and a negative integer indicates a
    /// "Note Off" at the absolute value of the integer
    rx: Receiver<i8>,

    /// The sound engine currently in use to process audio for the synth.
    voices: Option<Box<dyn VoiceAllocator>>,
    
    /// The maximum number of samples we will be expected to process
    max_buffer_size: usize,
}

/// Contains all of the parameters for an oscillator within the plugin
#[derive(Params)]
pub struct OscPluginParams {
    #[id = "course"]
    pub course: IntParam,

    #[id = "fine"]
    pub fine: IntParam,

    #[id = "shape"]
    pub shape: IntParam,

    #[id = "sin"]
    pub sin: IntParam,

    #[id = "sq"]
    pub sq: IntParam,

    #[id = "tri"]
    pub tri: IntParam,

    #[id = "saw"]
    pub saw: IntParam,
}

impl Default for OscPluginParams {
    fn default() -> Self {
        Self {
            course: IntParam::new("Course", 0, IntRange::Linear{ min: -36, max: 36}),
            fine: IntParam::new("Fine", 0, IntRange::Linear{ min: -512, max: 512}),
            shape: new_fixed_param("Shape", ScalarFxP::ZERO),
            sin: new_fixed_param_percent("Sin", ScalarFxP::ZERO),
            saw: new_fixed_param_percent("Saw", ScalarFxP::MAX),
            sq: new_fixed_param_percent("Square", ScalarFxP::ZERO),
            tri: new_fixed_param_percent("Triangle", ScalarFxP::ZERO),
        }
    }
}

/// Contains all of the parameters for an oscillator within the plugin
#[derive(Params)]
pub struct RingModPluginParams {
    #[id = "vol_o1"]
    pub mix_a: IntParam,

    #[id = "vol_o2"]
    pub mix_b: IntParam,

    #[id = "ringmd"]
    pub mix_mod: IntParam,
}

impl Default for RingModPluginParams {
    fn default() -> Self {
        Self {
            mix_a: new_fixed_param_percent("Osc 1", ScalarFxP::MAX),
            mix_b: new_fixed_param_percent("Osc 2", ScalarFxP::ZERO),
            mix_mod: new_fixed_param_percent("Ring Mod", ScalarFxP::ZERO),
        }
    }
}

/// Contains all of the parameters for a filter within the plugin
#[derive(Params)]
pub struct FiltPluginParams {
    #[id = "kbd"]
    pub kbd: IntParam,

    #[id = "vel"]
    pub vel: IntParam,

    #[id = "env"]
    pub env: IntParam,

    #[id = "cut"]
    pub cutoff: IntParam,

    #[id = "res"]
    pub res: IntParam,

    #[id = "low"]
    pub low: IntParam,

    #[id = "bnd"]
    pub band: IntParam,

    #[id = "hi"]
    pub high: IntParam,
}

impl Default for FiltPluginParams {
    fn default() -> Self {
        Self {
            env: new_fixed_param_percent("Filter Envelope Modulation", ScalarFxP::ZERO),
            kbd: new_fixed_param_percent("Filter Keyboard Tracking", ScalarFxP::ZERO),
            vel: new_fixed_param_percent("Filter Velocity Modulation", ScalarFxP::ZERO),
            cutoff: new_fixed_param_freq("Filter Cutoff", NoteFxP::lit("127")),
            res: new_fixed_param_percent("Filter Resonance", ScalarFxP::ZERO),
            low: new_fixed_param_percent("Filter Low Pass", ScalarFxP::MAX),
            band: new_fixed_param_percent("Filter Band Pass", ScalarFxP::ZERO),
            high: new_fixed_param_percent("Filter High Pass", ScalarFxP::ZERO),
        }
    }
}

/// Contains all of the parameters for an envelope within the plugin
#[derive(Params)]
pub struct EnvPluginParams {
    #[id = "a"]
    pub a: IntParam,

    #[id = "d"]
    pub d: IntParam,

    #[id = "s"]
    pub s: IntParam,

    #[id = "r"]
    pub r: IntParam,
}

impl EnvPluginParams {
    fn new(name: &str) -> Self {
        Self {
            a: new_fixed_param(name.to_owned() + " Attack", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
            d: new_fixed_param(name.to_owned() + " Decay", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
            s: new_fixed_param_percent(name.to_owned() + " Sustain", ScalarFxP::MAX),
            r: new_fixed_param(name.to_owned() + " Release", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
        }
    }
}

/// Holds all of the plugin parameters
#[derive(Params)]
pub struct JanusParams {
    /// The editor state, saved together with the parameter state so the
    /// custom scaling can be restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "osync"]
    pub osc_sync: BoolParam,

    #[nested(id_prefix = "o1", group = "osc1")]
    pub osc1: OscPluginParams,

    #[nested(id_prefix = "o2", group = "osc2")]
    pub osc2: OscPluginParams,

    #[nested(group = "ringmod")]
    pub ringmod: RingModPluginParams,

    #[nested(group = "filt")]
    pub filt: FiltPluginParams,

    #[nested(id_prefix = "env1", group = "envvca")]
    pub env_vca: EnvPluginParams,

    #[nested(id_prefix = "env2", group = "envvcf")]
    pub env_vcf: EnvPluginParams,
}

impl Default for JanusPlugin {
    fn default() -> Self {
        let (tx, rx) = sync_channel::<i8>(128);
        Self {
            params: Arc::new(JanusParams::default()),
            glob_params: Default::default(),
            osc1_params: Default::default(),
            osc2_params: Default::default(),
            ringmod_params: Default::default(),
            filt_params: Default::default(),
            env_amp_params: Default::default(),
            env_filt_params: Default::default(),
            tx,
            rx,
            voices: None,
            max_buffer_size: 0,
        }
    }
}

impl Default for JanusParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            osc_sync: BoolParam::new("Oscillator Sync", false),
            osc1: Default::default(),
            osc2: Default::default(),
            ringmod: Default::default(),
            filt: Default::default(),
            env_vca: EnvPluginParams::new("VCA Envelope"),
            env_vcf: EnvPluginParams::new("VCF Envelope"),
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
        editor::create(self.params.clone(), self.tx.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // TODO
        let bufsz = buffer_config.max_buffer_size;
        self.glob_params.allocate(bufsz);
        self.osc1_params.allocate(bufsz);
        self.osc2_params.allocate(bufsz);
        self.ringmod_params.allocate(bufsz);
        self.filt_params.allocate(bufsz);
        self.env_amp_params.allocate(bufsz);
        self.env_filt_params.allocate(bufsz);
        let mut voice_alloc = Box::new(MonoSynthFxP::new());
        voice_alloc.initialize(bufsz as usize);
        self.voices = Some(voice_alloc);
        self.max_buffer_size = bufsz as usize;
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let voices = match self.voices {
            Some(ref mut x) => x,
            None => return ProcessStatus::Error("Uninitialized"),
        };
        let mut index = 0;
        while let Ok(note) = self.rx.try_recv() {
            if note < 0 {
                voices.note_off((note - (-128)) as u8, 0);
            } else {
                voices.note_on(note as u8, 100);
            }
        }
        assert!(buffer.samples() <= self.max_buffer_size);
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
        if !(voices.is_fixed_point()) {
            self.osc1_params.conv_float();
            self.osc2_params.conv_float();
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
