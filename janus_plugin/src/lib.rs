use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;

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
    voices: Option<Box<dyn VoiceAllocator + Send>>
}

#[derive(Params)]
pub struct JanusParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "gain"]
    pub gain: FloatParam,

    // TODO: Remove this parameter when we're done implementing the widgets
    #[id = "foobar"]
    pub some_int: IntParam,
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

impl Default for JanusParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(300, 180),

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
            some_int: IntParam::new("Something", 3, IntRange::Linear { min: 0, max: 3 }),
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
        let params = self.params.clone();
        let peak_meter = self.peak_meter.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    // NOTE: See `plugins/diopser/src/editor.rs` for an example using the generic UI widget

                    // This is a fancy widget that can get all the information it needs to properly
                    // display and modify the parameter from the parametr itself
                    // It's not yet fully implemented, as the text is missing.
                    ui.label("Some random integer");
                    ui.add(widgets::ParamSlider::for_param(&params.some_int, setter));

                    ui.label("Gain");
                    ui.add(widgets::ParamSlider::for_param(&params.gain, setter));

                    ui.label(
                        "Also gain, but with a lame widget. Can't even render the value correctly!",
                    );

                    // TODO: Add a proper custom widget instead of reusing a progress bar
                    let peak_meter =
                        util::gain_to_db(peak_meter.load(std::sync::atomic::Ordering::Relaxed));
                    let peak_meter_text = if peak_meter > util::MINUS_INFINITY_DB {
                        format!("{peak_meter:.1} dBFS")
                    } else {
                        String::from("-inf dBFS")
                    };

                    let peak_meter_normalized = (peak_meter + 60.0) / 60.0;
                    ui.allocate_space(egui::Vec2::splat(2.0));
                    ui.add(
                        egui::widgets::ProgressBar::new(peak_meter_normalized)
                            .text(peak_meter_text),
                    );
                });
            },
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
            self.osc_params.shape_mut()[index] = 0f32;
            self.filt_params.cutoff_mut()[index] = 127f32;
            self.filt_params.res_mut()[index] = 0f32;
            self.env_amp_params.a_mut()[index] = 0.1f32;
            self.env_amp_params.d_mut()[index] = 0.1f32;
            self.env_amp_params.s_mut()[index] = 1f32;
            self.env_amp_params.r_mut()[index] = 0.1f32;

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
        if voices.is_fixed_point() {
            self.osc_params.conv_fxp();
            self.filt_params.conv_fxp();
            self.env_amp_params.conv_fxp();
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