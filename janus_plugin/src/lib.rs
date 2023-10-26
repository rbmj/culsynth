use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;
use fixed::traits::Fixed;

use janus::{ScalarFxP, EnvParamFxP, SampleFxP, NoteFxP, devices::EnvParamsFxP};

mod editor;

mod parambuf;
use parambuf::{EnvParamBuffer, OscParamBuffer, FiltParamBuffer};

mod voicealloc;
use voicealloc::{VoiceAllocator, MonoSynthFxP};

pub struct JanusPlugin {
    params: Arc<JanusParams>,

    /// Needed to normalize the peak meter's response based on the sample rate.
    peak_meter_decay_weight: f32,
    /// The current data for the peak meter. This is stored as an [`Arc`] so we can share it between
    /// the GUI and the audio processing parts. If you have more state to share, then it's a good
    /// idea to put all of that in a struct behind a single `Arc`.
    ///
    /// This is stored as voltage gain.
    peak_meter: Arc<AtomicF32>,

    osc_params: OscParamBuffer,
    filt_params: FiltParamBuffer,
    env_amp_params: EnvParamBuffer,

    gainbuf: Vec<f32>,
    voices: Option<Box<dyn VoiceAllocator>>
}

#[derive(Params)]
pub struct JanusParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "osc1_shape"]
    pub osc1_shape: IntParam,

    #[id = "filt_cutoff"]
    pub filt_cutoff: IntParam,

    #[id = "filt_res"]
    pub filt_res: IntParam,

    #[id = "env_vca_a"]
    pub env_vca_a: IntParam,

    #[id = "env_vca_d"]
    pub env_vca_d: IntParam,

    #[id = "env_vca_s"]
    pub env_vca_s: IntParam,

    #[id = "env_vca_r"]
    pub env_vca_r: IntParam,
}

impl Default for JanusPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(JanusParams::default()),

            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),

            osc_params: Default::default(),
            filt_params: Default::default(),
            env_amp_params: Default::default(),

            gainbuf: Default::default(),
            voices: None
        }
    }
}


fn fixed_v2s<F: Fixed>(x: i32) -> String
    where i32: TryFrom<F::Bits>
{
    F::from_bits(F::Bits::try_from(x).unwrap_or_default()).to_string()
}

fn fixed_s2v<F: Fixed>(s: &str) -> Option<i32> 
    where F::Bits: Into<i32>
{
    F::from_str(s).map(|x| x.to_bits().into()).ok()
}

fn fixed_v2s_percent(x: i32) -> String 
{
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
        .map(|x| NoteFxP::saturating_from_num(
            ((x / 440f32).log2() * 12f32) + 69f32).to_bits() as i32)
        .ok()
}

fn new_fixed_param<F: Fixed>(name: impl Into<String>, default: F) -> IntParam
    where F::Bits: Into<i32>
{
    IntParam::new(name, default.to_bits().into(),
            IntRange::Linear { min: F::MIN.to_bits().into(), max: F::MAX.to_bits().into() })
        .with_smoother(SmoothingStyle::Linear(50.0))
        .with_value_to_string(Arc::new(fixed_v2s::<F>))
        .with_string_to_value(Arc::new(fixed_s2v::<F>))
}

fn new_fixed_param_percent(name: impl Into<String>, default: ScalarFxP) -> IntParam
{
    IntParam::new(name, default.to_bits().into(),
            IntRange::Linear { min: ScalarFxP::MIN.to_bits().into(), max: ScalarFxP::MAX.to_bits().into() })
        .with_smoother(SmoothingStyle::Linear(50.0))
        .with_value_to_string(Arc::new(fixed_v2s_percent))
        .with_string_to_value(Arc::new(fixed_s2v_percent))
        .with_unit(" %")
}

fn new_fixed_param_freq(name: impl Into<String>, default: NoteFxP) -> IntParam
{
    IntParam::new(name, default.to_bits().into(),
            IntRange::Linear { min: NoteFxP::MIN.to_bits().into(), max: NoteFxP::MAX.to_bits().into() })
        .with_smoother(SmoothingStyle::Linear(50.0))
        .with_value_to_string(Arc::new(fixed_v2s_freq))
        .with_string_to_value(Arc::new(fixed_s2v_freq))
        .with_unit(" Hz")
}

impl Default for JanusParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            // See the main gain example for more details
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            osc1_shape: new_fixed_param("Oscillator 1 Shape", ScalarFxP::ZERO),
            filt_cutoff: new_fixed_param_freq("Filter Cutoff", NoteFxP::lit("127")),
            filt_res: new_fixed_param_percent("Filter Resonance", ScalarFxP::ZERO),
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
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
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
        self.gainbuf.resize(bufsz as usize, 1f32);
        self.voices = Some(Box::new(MonoSynthFxP::new()));
        self.voices.as_mut().unwrap().initialize(bufsz as usize);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let voices = &mut self.voices.as_mut().unwrap();
        let mut amplitude = 0f32;
        let mut index = 0;
        let mut next_event = context.next_event();
        for (sample_id, _channel_samples) in buffer.iter_samples().enumerate() {
            // Smoothing is optionally built into the parameters themselves
            self.gainbuf[index] = self.params.gain.smoothed.next();
            // TODO: Actually make these parameters
            self.osc_params.shape_mut()[index] =
                ScalarFxP::from_bits(self.params.osc1_shape.smoothed.next() as u16);
            self.filt_params.cutoff_mut()[index] =
                NoteFxP::from_bits(self.params.filt_cutoff.smoothed.next() as u16);
            self.filt_params.res_mut()[index] =
                ScalarFxP::from_bits(self.params.filt_res.smoothed.next() as u16);
            self.env_amp_params.a_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vca_a.smoothed.next() as u16);
            self.env_amp_params.d_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vca_d.smoothed.next() as u16);
            self.env_amp_params.s_mut()[index] =
                ScalarFxP::from_bits(self.params.env_vca_s.smoothed.next() as u16);
            self.env_amp_params.r_mut()[index] =
                EnvParamFxP::from_bits(self.params.env_vca_r.smoothed.next() as u16);

            // Process MIDI events:
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        voices.note_on(note, (velocity*127f32) as u8);
                    }
                    NoteEvent::NoteOff { note, velocity, .. } => {
                        voices.note_off(note, (velocity*127f32) as u8);
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
        }
        let output = voices.process(&self.osc_params, &self.filt_params, &self.env_amp_params);
        index = 0;
        for channel_samples in buffer.iter_samples() {
            for smp in channel_samples {
                *smp = output[index] * self.gainbuf[index];
                amplitude += *smp;
            }
            index += 1;
        }
        // To save resources, a plugin can (and probably should!) only perform expensive
        // calculations that are only displayed on the GUI while the GUI is open
        if self.params.editor_state.is_open() {
            amplitude = (amplitude / buffer.as_slice().len() as f32).abs();
            let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
            let new_peak_meter = if amplitude > current_peak_meter {
                amplitude
            } else {
                current_peak_meter * self.peak_meter_decay_weight
                    + amplitude * (1.0 - self.peak_meter_decay_weight)
            };

            self.peak_meter
                .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed)
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
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Synth];
}

nih_export_clap!(JanusPlugin);
nih_export_vst3!(JanusPlugin);