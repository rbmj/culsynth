use crate::*;
use culsynth::context::Context;
use nih_plug::prelude::*;
use std::sync::atomic::Ordering::Relaxed;

use std::sync::{mpsc::sync_channel, Arc};

impl Plugin for CulSynthPlugin {
    const NAME: &'static str = crate::NAME;
    const VENDOR: &'static str = crate::VENDOR;
    const URL: &'static str = crate::URL;
    const EMAIL: &'static str = crate::EMAIL;

    const VERSION: &'static str = crate::VERSION;

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_output_channels: NonZeroU32::new(2),
            ..nih_plug::audio_setup::AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_output_channels: std::num::NonZeroU32::new(1),
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
        crate::editor::create(
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
    const CLAP_ID: &'static str = crate::ID;
    const CLAP_DESCRIPTION: Option<&'static str> = crate::DESCRIPTION;
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for CulSynthPlugin {
    const VST3_CLASS_ID: [u8; 16] = crate::CLASS_ID;
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Synth];
}

nih_export_clap!(CulSynthPlugin);
nih_export_vst3!(CulSynthPlugin);
