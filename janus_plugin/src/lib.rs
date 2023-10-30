//use atomic_float::AtomicF32;
use fixed::traits::Fixed;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;

use janus::{EnvParamFxP, NoteFxP, ScalarFxP};

mod editor;

pub mod parambuf;
use parambuf::{EnvParamBuffer, FiltParamBuffer, OscParamBuffer};

mod voicealloc;
use voicealloc::{MonoSynthFxP, VoiceAllocator};

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

#[derive(Params)]
pub struct JanusParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "osc1_shape"]
    pub osc1_shape: IntParam,

    #[id = "osc1_sin"]
    pub osc1_sin: IntParam,

    #[id = "osc1_sq"]
    pub osc1_sq: IntParam,

    #[id = "osc1_tri"]
    pub osc1_tri: IntParam,

    #[id = "osc1_saw"]
    pub osc1_saw: IntParam,

    #[id = "filt_kbd"]
    pub filt_kbd: IntParam,

    #[id = "filt_env"]
    pub filt_env: IntParam,

    #[id = "filt_cutoff"]
    pub filt_cutoff: IntParam,

    #[id = "filt_res"]
    pub filt_res: IntParam,

    #[id = "filt_low"]
    pub filt_low: IntParam,

    #[id = "filt_band"]
    pub filt_band: IntParam,

    #[id = "filt_high"]
    pub filt_high: IntParam,

    #[id = "env_vca_a"]
    pub env_vca_a: IntParam,

    #[id = "env_vca_d"]
    pub env_vca_d: IntParam,

    #[id = "env_vca_s"]
    pub env_vca_s: IntParam,

    #[id = "env_vca_r"]
    pub env_vca_r: IntParam,

    #[id = "env_vcf_a"]
    pub env_vcf_a: IntParam,

    #[id = "env_vcf_d"]
    pub env_vcf_d: IntParam,

    #[id = "env_vcf_s"]
    pub env_vcf_s: IntParam,

    #[id = "env_vcf_r"]
    pub env_vcf_r: IntParam,
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

fn fixed_v2s<F: Fixed>(x: i32) -> String
where
    i32: TryFrom<F::Bits>,
{
    F::from_bits(F::Bits::try_from(x).unwrap_or_default()).to_string()
}

fn fixed_s2v<F: Fixed>(s: &str) -> Option<i32>
where
    F::Bits: Into<i32>,
{
    F::from_str(s).map(|x| x.to_bits().into()).ok()
}

fn fixed_v2s_percent(x: i32) -> String {
    let percent = ScalarFxP::from_bits(x as u16).to_num::<f32>() * 100f32;
    format!("{}", percent)
}

fn fixed_s2v_percent(s: &str) -> Option<i32> {
    s.trim_end_matches(&[' ', '%'])
        .parse::<f32>()
        .map(|x| ScalarFxP::saturating_from_num(x / 100.0).to_bits() as i32)
        .ok()
}

fn fixed_v2s_freq(x: i32) -> String {
    janus::fixedmath::midi_note_to_frequency(NoteFxP::from_bits(x as u16)).to_string()
}

fn fixed_s2v_freq(s: &str) -> Option<i32> {
    s.trim_end_matches(&[' ', 'H', 'h', 'Z', 'z'])
        .parse::<f32>()
        .map(|x| {
            NoteFxP::saturating_from_num(((x / 440f32).log2() * 12f32) + 69f32).to_bits() as i32
        })
        .ok()
}

fn new_fixed_param<F: Fixed>(name: impl Into<String>, default: F) -> IntParam
where
    F::Bits: Into<i32>,
{
    IntParam::new(
        name,
        default.to_bits().into(),
        IntRange::Linear {
            min: F::MIN.to_bits().into(),
            max: F::MAX.to_bits().into(),
        },
    )
    .with_smoother(SmoothingStyle::Linear(50.0))
    .with_value_to_string(Arc::new(fixed_v2s::<F>))
    .with_string_to_value(Arc::new(fixed_s2v::<F>))
}

fn new_fixed_param_percent(name: impl Into<String>, default: ScalarFxP) -> IntParam {
    IntParam::new(
        name,
        default.to_bits().into(),
        IntRange::Linear {
            min: ScalarFxP::MIN.to_bits().into(),
            max: ScalarFxP::MAX.to_bits().into(),
        },
    )
    .with_smoother(SmoothingStyle::Linear(50.0))
    .with_value_to_string(Arc::new(fixed_v2s_percent))
    .with_string_to_value(Arc::new(fixed_s2v_percent))
    .with_unit(" %")
}

fn new_fixed_param_freq(name: impl Into<String>, default: NoteFxP) -> IntParam {
    IntParam::new(
        name,
        default.to_bits().into(),
        IntRange::Linear {
            min: NoteFxP::MIN.to_bits().into(),
            max: NoteFxP::MAX.to_bits().into(),
        },
    )
    .with_smoother(SmoothingStyle::Linear(50.0))
    .with_value_to_string(Arc::new(fixed_v2s_freq))
    .with_string_to_value(Arc::new(fixed_s2v_freq))
    .with_unit(" Hz")
}

impl Default for JanusParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            osc1_shape: new_fixed_param("Oscillator 1 Shape", ScalarFxP::ZERO),
            osc1_sin: new_fixed_param_percent("Oscillator 1 Sin", ScalarFxP::ZERO),
            osc1_saw: new_fixed_param_percent("Oscillator 1 Saw", ScalarFxP::MAX),
            osc1_sq: new_fixed_param_percent("Oscillator 1 Square", ScalarFxP::ZERO),
            osc1_tri: new_fixed_param_percent("Oscillator 1 Triangle", ScalarFxP::ZERO),

            filt_env: new_fixed_param_percent("Filter Envelope Modulation", ScalarFxP::ZERO),
            filt_kbd: new_fixed_param_percent("Filter Keyboard Tracking", ScalarFxP::ZERO),
            filt_cutoff: new_fixed_param_freq("Filter Cutoff", NoteFxP::lit("127")),
            filt_res: new_fixed_param_percent("Filter Resonance", ScalarFxP::ZERO),
            filt_low: new_fixed_param_percent("Filter Low Pass", ScalarFxP::MAX),
            filt_band: new_fixed_param_percent("Filter Band Pass", ScalarFxP::ZERO),
            filt_high: new_fixed_param_percent("Filter High Pass", ScalarFxP::ZERO),

            env_vcf_a: new_fixed_param("VCF Envelope Attack", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
            env_vcf_d: new_fixed_param("VCF Envelope Decay", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
            env_vcf_s: new_fixed_param_percent("VCF Envelope Sustain", ScalarFxP::MAX),
            env_vcf_r: new_fixed_param("VCF Envelope Release", EnvParamFxP::lit("0.1")),

            env_vca_a: new_fixed_param("VCA Envelope Attack", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
            env_vca_d: new_fixed_param("VCA Envelope Decay", EnvParamFxP::lit("0.1"))
                .with_unit(" sec"),
            env_vca_s: new_fixed_param_percent("VCA Envelope Sustain", ScalarFxP::MAX),
            env_vca_r: new_fixed_param("VCA Envelope Release", EnvParamFxP::lit("0.1")),
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
                ScalarFxP::from_bits(self.params.osc1_shape.smoothed.next() as u16);
            self.osc_params.sin_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1_sin.smoothed.next() as u16);
            self.osc_params.sq_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1_sq.smoothed.next() as u16);
            self.osc_params.saw_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1_saw.smoothed.next() as u16);
            self.osc_params.tri_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1_tri.smoothed.next() as u16);

            self.filt_params.env_mod_mut()[index] =
                ScalarFxP::from_bits(self.params.filt_env.smoothed.next() as u16);
            self.filt_params.kbd_mut()[index] =
                ScalarFxP::from_bits(self.params.filt_kbd.smoothed.next() as u16);
            self.filt_params.cutoff_mut()[index] =
                NoteFxP::from_bits(self.params.filt_cutoff.smoothed.next() as u16);
            self.filt_params.res_mut()[index] =
                ScalarFxP::from_bits(self.params.filt_res.smoothed.next() as u16);
            self.filt_params.low_mix_mut()[index] =
                ScalarFxP::from_bits(self.params.filt_low.smoothed.next() as u16);
            self.filt_params.band_mix_mut()[index] =
                ScalarFxP::from_bits(self.params.filt_band.smoothed.next() as u16);
            self.filt_params.high_mix_mut()[index] =
                ScalarFxP::from_bits(self.params.filt_high.smoothed.next() as u16);

            self.env_amp_params.a_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vca_a.smoothed.next() as u16);
            self.env_amp_params.d_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vca_d.smoothed.next() as u16);
            self.env_amp_params.s_mut()[index] =
                ScalarFxP::from_bits(self.params.env_vca_s.smoothed.next() as u16);
            self.env_amp_params.r_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vca_r.smoothed.next() as u16);

            self.env_filt_params.a_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vcf_a.smoothed.next() as u16);
            self.env_filt_params.d_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vcf_d.smoothed.next() as u16);
            self.env_filt_params.s_mut()[index] =
                ScalarFxP::from_bits(self.params.env_vcf_s.smoothed.next() as u16);
            self.env_filt_params.r_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vcf_r.smoothed.next() as u16);

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
