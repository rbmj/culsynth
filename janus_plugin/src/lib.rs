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
use parambuf::{EnvParamBuffer, FiltParamBuffer, OscParamBuffer};

mod voicealloc;
use voicealloc::{MonoSynthFxP, VoiceAllocator};

/// Contains all of the global state for the plugin
pub struct JanusPlugin {
    params: Arc<JanusParams>,

    osc_params: OscParamBuffer,
    filt_params: FiltParamBuffer,
    env_amp_params: EnvParamBuffer,
    env_filt_params: EnvParamBuffer,

    tx: SyncSender<i8>,
    rx: Receiver<i8>,

    voices: Option<Box<dyn VoiceAllocator>>,
    max_buffer_size: usize,
}

/// Contains all of the parameters for an oscillator within the plugin
#[derive(Params)]
pub struct OscPluginParams {
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
            shape: new_fixed_param("Shape", ScalarFxP::ZERO),
            sin: new_fixed_param_percent("Sin", ScalarFxP::ZERO),
            saw: new_fixed_param_percent("Saw", ScalarFxP::MAX),
            sq: new_fixed_param_percent("Square", ScalarFxP::ZERO),
            tri: new_fixed_param_percent("Triangle", ScalarFxP::ZERO),
        }
    }
}

/// Contains all of the parameters for a filter within the plugin
#[derive(Params)]
pub struct FiltPluginParams {
    #[id = "kbd"]
    pub kbd: IntParam,

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

    #[nested(id_prefix = "o1", group = "osc1")]
    pub osc1: OscPluginParams,

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
            osc_params: Default::default(),
            filt_params: Default::default(),
            env_amp_params: Default::default(),
            env_filt_params: Default::default(),
            tx: tx,
            rx: rx,
            voices: None,
            max_buffer_size: 0,
        }
    }
}

impl Default for JanusParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            osc1: Default::default(),
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
        self.osc_params.allocate(bufsz);
        self.filt_params.allocate(bufsz);
        self.env_amp_params.allocate(bufsz);
        self.env_filt_params.allocate(bufsz);
        self.voices = Some(Box::new(MonoSynthFxP::new()));
        self.voices.as_mut().unwrap().initialize(bufsz as usize);
        self.max_buffer_size = bufsz as usize;
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let voices = &mut self.voices.as_mut().unwrap();
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
            self.osc_params.shape_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1.shape.smoothed.next() as u16);
            self.osc_params.sin_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1.sin.smoothed.next() as u16);
            self.osc_params.sq_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1.sq.smoothed.next() as u16);
            self.osc_params.saw_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1.saw.smoothed.next() as u16);
            self.osc_params.tri_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1.tri.smoothed.next() as u16);

            self.filt_params.env_mod_mut()[index] =
                ScalarFxP::from_bits(self.params.filt.env.smoothed.next() as u16);
            self.filt_params.kbd_mut()[index] =
                ScalarFxP::from_bits(self.params.filt.kbd.smoothed.next() as u16);
            self.filt_params.cutoff_mut()[index] =
                NoteFxP::from_bits(self.params.filt.cutoff.smoothed.next() as u16);
            self.filt_params.res_mut()[index] =
                ScalarFxP::from_bits(self.params.filt.res.smoothed.next() as u16);
            self.filt_params.low_mix_mut()[index] =
                ScalarFxP::from_bits(self.params.filt.low.smoothed.next() as u16);
            self.filt_params.band_mix_mut()[index] =
                ScalarFxP::from_bits(self.params.filt.band.smoothed.next() as u16);
            self.filt_params.high_mix_mut()[index] =
                ScalarFxP::from_bits(self.params.filt.high.smoothed.next() as u16);

            self.env_amp_params.a_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vca.a.smoothed.next() as u16);
            self.env_amp_params.d_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vca.d.smoothed.next() as u16);
            self.env_amp_params.s_mut()[index] =
                ScalarFxP::from_bits(self.params.env_vca.s.smoothed.next() as u16);
            self.env_amp_params.r_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vca.r.smoothed.next() as u16);

            self.env_filt_params.a_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vcf.a.smoothed.next() as u16);
            self.env_filt_params.d_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vcf.d.smoothed.next() as u16);
            self.env_filt_params.s_mut()[index] =
                ScalarFxP::from_bits(self.params.env_vcf.s.smoothed.next() as u16);
            self.env_filt_params.r_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vcf.r.smoothed.next() as u16);

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
            self.osc_params.conv_float();
            self.filt_params.conv_float();
            self.env_amp_params.conv_float();
            self.env_filt_params.conv_float();
        }
        let output = voices.process(
            &self.osc_params,
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
