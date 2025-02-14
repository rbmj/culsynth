use crate::voicealloc::{PolySynth, VoiceAllocator};
use crate::{MidiHandler, VoiceMode};
use culsynth::context::{Context, GenericContext};
use culsynth::voice::VoiceParams;
use midihandler::MidiSender;
use nih_plug::prelude::*;
use std::sync::atomic::{AtomicI32, AtomicU32, AtomicUsize, Ordering::Relaxed};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;

pub mod editor;
pub mod fixedparam;
pub mod midihandler;
pub mod pluginparams;

pub use pluginparams::CulSynthParams;

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

pub struct PluginContextReader {
    context: Arc<PluginContext>,
}

impl crate::ContextReader for PluginContextReader {
    /// Get the current context.  Returns a tuple of (sample_rate, fixed_point).
    fn get(&self) -> (u32, bool) {
        let mut fixed = false;
        let mut sr = self.context.sample_rate.load(Relaxed);
        if sr < 0 {
            fixed = true;
            sr = -sr;
        }
        (sr as u32, fixed)
    }
    fn sample_rate(&self) -> u32 {
        let (sr, _) = self.get();
        sr
    }
    fn is_fixed(&self) -> bool {
        let (_, fixed) = self.get();
        fixed
    }
    fn bufsz(&self) -> usize {
        self.context.bufsz.load(Relaxed)
    }
    fn voice_mode(&self) -> VoiceMode {
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
    midi_tx: SyncSender<wmidi::MidiMessage<'static>>,

    /// Used by the audio thread to receive MIDI events from the GUI thread.
    ///
    /// The MIDI event is packaged as an i8, where a positive integer indicates
    /// a "Note On" for that note number, and a negative integer indicates a
    /// "Note Off" at the absolute value of the integer
    midi_rx: Receiver<wmidi::MidiMessage<'static>>,

    /// Used by the GUI thread to replace the current synth engine
    synth_tx: SyncSender<Box<dyn VoiceAllocator>>,

    /// Used by the audio thread to receive replacement synth engines from the
    /// GUI thread.
    synth_rx: Receiver<Box<dyn VoiceAllocator>>,

    /// The sound engine currently in use to process audio for the synth.
    voices: Option<Box<dyn VoiceAllocator>>,

    /// Used by the audio thread to send control changes to the GUI
    cc_tx: MidiSender,

    /// Used by the GUI thread to receive control changes
    cc_rx: Option<Receiver<wmidi::MidiMessage<'static>>>,

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
    fn get_context_reader(&mut self) -> PluginContextReader {
        PluginContextReader {
            context: self.context.clone(),
        }
    }
}

impl Default for CulSynthPlugin {
    fn default() -> Self {
        let (midi_tx, midi_rx) = sync_channel::<wmidi::MidiMessage<'static>>(32);
        let (cc_tx, cc_rx) = sync_channel::<wmidi::MidiMessage<'static>>(32);
        let (synth_tx, synth_rx) = sync_channel::<Box<dyn VoiceAllocator>>(1);
        Self {
            params: Arc::new(CulSynthParams::default()),
            midi_tx,
            midi_rx,
            synth_tx,
            synth_rx,
            cc_tx: MidiSender {
                tx: cc_tx,
                ch: wmidi::Channel::Ch1,
            },
            cc_rx: Some(cc_rx),
            voices: None,
            context: Arc::new(Default::default()),
        }
    }
}

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
                let ch = self.cc_tx.ch();
                let (tx, rx) = sync_channel::<wmidi::MidiMessage<'static>>(32);
                self.cc_tx = MidiSender { tx, ch };
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
        while let Ok(msg) = self.midi_rx.try_recv() {
            match msg {
                wmidi::MidiMessage::NoteOff(_, note, vel) => {
                    voices.note_off(note.into(), vel.into())
                }
                wmidi::MidiMessage::NoteOn(_, note, vel) => voices.note_on(note.into(), vel.into()),
                _ => {}
            }
        }
        assert!(buffer.samples() <= self.context.bufsz.load(Relaxed));

        let smps = buffer.iter_samples();
        let mut matrix = Some((&self.params.modmatrix).into());
        // Replace ProcessContext with a MidiReceiver
        let mut next_event = context.next_event();
        for (smpid, ch_smps) in smps.enumerate() {
            let params: VoiceParams<i16> = self.params.as_ref().into();
            // Process MIDI events:
            while let Some(event) = next_event {
                if event.timing() > smpid as u32 {
                    break;
                }
                match event {
                    nih_plug::midi::NoteEvent::NoteOn { note, velocity, .. } => {
                        voices.note_on(note, (velocity * 127f32) as u8);
                    }
                    nih_plug::midi::NoteEvent::NoteOff { note, velocity, .. } => {
                        voices.note_off(note, (velocity * 127f32) as u8);
                    }
                    nih_plug::midi::NoteEvent::MidiCC { cc, value, .. } => {
                        // nih-plug guarantees that cc will be < 127, so panic is appropriate
                        let cc = wmidi::ControlFunction(wmidi::U7::new(cc).unwrap());
                        let midi_value = (value * 127f32) as u8;
                        voices.handle_cc(cc, midi_value);
                        self.cc_tx.send_cc(cc, wmidi::U7::from_u8_lossy(midi_value));
                    }
                    nih_plug::midi::NoteEvent::MidiChannelPressure { pressure, .. } => {
                        voices.aftertouch((pressure * 127f32) as u8);
                    }
                    nih_plug::midi::NoteEvent::MidiPitchBend { value, .. } => {
                        voices.pitch_bend((((value - 0.5) * (i16::MAX as f32)) as i16) << 1);
                    }
                    _ => (),
                }
                next_event = context.next_event();
            }
            let out = voices.next(&params, matrix.take().as_ref());
            for smp in ch_smps {
                *smp = out;
            }
        }
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
