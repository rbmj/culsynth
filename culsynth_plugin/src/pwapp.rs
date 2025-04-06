use std::sync::mpsc;
use std::thread;

use culsynth_plugin::editor::Editor;
use culsynth_plugin::ownedmidihandler::OwnedMidiHandler;
use culsynth_plugin::voicealloc::PolySynth;
use culsynth_plugin::voicealloc::VoiceAllocator;
use culsynth_plugin::ContextReader;
use pipewire as pw;
use pw::properties::properties;
use wmidi::MidiMessage;

struct PwContext {
    sample_rate: u32,
    bufsize: usize,
    fixed: bool,
    voice_mode: culsynth_plugin::VoiceMode,
}

struct PwMidiHandler {
    owned: OwnedMidiHandler,
    sender: mpsc::Sender<MidiMessage<'static>>,
}

impl PwMidiHandler {
    fn new(sender: mpsc::Sender<MidiMessage<'static>>, ch: wmidi::Channel) -> Self {
        Self {
            owned: OwnedMidiHandler::new(ch),
            sender,
        }
    }
    fn get_params(&self) -> culsynth::voice::VoiceParams<i16> {
        self.owned.get_params()
    }
    fn get_tuning(&self) -> (culsynth_plugin::Tuning, culsynth_plugin::Tuning) {
        self.owned.get_tuning()
    }
    fn get_matrix(&self) -> culsynth::voice::modulation::ModMatrix<i16> {
        self.owned.get_matrix()
    }
}

impl culsynth_plugin::MidiHandler for PwMidiHandler {
    fn send(&self, msg: MidiMessage<'static>) {
        if let Err(e) = self.sender.send(msg) {
            log::error!("Failed to send midi message: {}", e);
        } else {
            self.owned.send(msg);
        }
    }
    fn ch(&self) -> wmidi::Channel {
        self.owned.ch()
    }
}

impl ContextReader for PwContext {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn is_fixed(&self) -> bool {
        self.fixed
    }
    fn bufsz(&self) -> usize {
        self.bufsize
    }
    fn voice_mode(&self) -> culsynth_plugin::VoiceMode {
        self.voice_mode
    }
}

struct PwApp {
    synth_tx: mpsc::Sender<Box<dyn VoiceAllocator>>,
    param_handler: PwMidiHandler,
    editor: Editor,
    context: PwContext,
}

impl PwApp {
    fn new(
        midi_tx: mpsc::Sender<MidiMessage<'static>>,
        synth_tx: mpsc::Sender<Box<dyn VoiceAllocator>>,
        eframe_ctx: &eframe::CreationContext<'_>,
        context: PwContext,
    ) -> Self {
        let mut editor = Editor::new();
        editor.initialize(&eframe_ctx.egui_ctx);
        Self {
            synth_tx,
            param_handler: PwMidiHandler::new(midi_tx, wmidi::Channel::Ch1),
            editor,
            context,
        }
    }
}

impl eframe::App for PwApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.editor.update(
            ctx,
            &self.audio_context,
            &self.param_handler.get_params(),
            self.param_handler.get_tuning(),
            &self.param_handler.get_matrix(),
            &self.param_handler,
            Some(&self.synth_tx),
        );
    }
}

struct PwAudioProcessor {
    midi_rx: mpsc::Receiver<MidiMessage<'static>>,
    synth_rx: mpsc::Receiver<Box<dyn VoiceAllocator>>,
    voicealloc: Box<dyn VoiceAllocator>,
    param_handler: OwnedMidiHandler,
}

impl PwAudioProcessor {
    fn new(
        midi_rx: mpsc::Receiver<MidiMessage<'static>>,
        synth_rx: mpsc::Receiver<Box<dyn VoiceAllocator>>,
        voicealloc: Box<dyn VoiceAllocator>,
    ) -> Self {
        Self {
            midi_rx,
            synth_rx,
            voicealloc,
            param_handler: OwnedMidiHandler::new(wmidi::Channel::Ch1),
        }
    }
    fn process(&mut self, buf: pw::buffer::Buffer) {
        while let Ok(msg) = self.midi_rx.try_recv() {
            self.voicealloc.process_midi(msg);
            self.param_handler.send(msg);
        }
        while let Ok(synth) = self.synth_rx.try_recv() {
            self.voicealloc = synth;
        }
        let data = buf.datas_mut();
        if datas.is_empty() {
            return;
        }
    }
}

fn pw_audio(
    midi_rx: mpsc::Receiver<MidiMessage<'static>>,
    synth_rx: mpsc::Receiver<Box<dyn VoiceAllocator>>,
    mainloop: pw::main_loop::MainLoop,
    context: pw::context::Context,
    core: pw::core::Core,
    voicealloc: Box<dyn VoiceAllocator>,
) {
    let props = properties! {
        *pw::keys::APP_NAME => culsynth_plugin::NAME,
        *pw::keys::APP_VERSION => culsynth_plugin::VERSION,
        *pw::keys::MEDIA_CATEGORY => "Playback",
        *pw::keys::MEDIA_ROLE => "DSP",
        *pw::keys::MEDIA_TYPE => "Midi",
    };
    let stream = pw::stream::Stream::new(&core, culsynth_plugin::NAME, props)
        .expect("Failed to create stream");
    let _listener = stream
        .add_local_listener_with_user_data(PwAudioProcessor::new(midi_rx, synth_rx, voicealloc))
        .process(|s, processor| {
            if let Some(buf) = s.dequeue_buffer() {
                processor.process(buf);
            }
        })
        .register()
        .expect("Failed to register stream with pipewire");
    let mut audio_info = pw::spa::param::audio::AudioInfoRaw::new();
    //TODO: Add multiple supported formats
    audio_info.set_format(pw::spa::param::audio::AudioFormat::F32P);
    let obj = pw::spa::pod::Object {
        type_: pw::spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
        id: pw::spa::param::ParamType::EnumFormat.as_raw(),
        properties: audio_info.into(),
    };
    let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(obj),
    )
    .unwrap()
    .0
    .into_inner();
    let mut params = [Pod::from_bytes(&values).unwrap()];
    stream
        .connect(
            pw::spa::utils::Direction::Output,
            None,
            pw::stream::StreamFlags::AUTOCONNECT
                | pw::stream::StreamFlags::RT_PROCESS
                | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut params,
        )
        .expect("Failed to connect stream");
    mainloop.run();
}

pub fn run() {
    colog::init();
    let (midi_tx, midi_rx) = mpsc::sync_channel::<MidiMessage<'static>>(32);
    let (synth_tx, synth_rx) = mpsc::sync_channel::<Box<dyn VoiceAllocator>>(1);

    pw::init();
    let mainloop = pw::main_loop::MainLoop::new(None).expect("Failed to create main loop");
    let context = pw::context::Context::new(&mainloop).expect("Failed to create context");
    let core = context.connect(None).expect("Failed to connect to context");
    const KEY_AUDIO_RATE: &'static str = "audio.rate";
    let sample_rate = if let Some(sr_str) = context.properties().get(KEY_AUDIO_RATE) {
        if let Ok(sr) = sr_str.parse::<u32>() {
            sr
        } else {
            log::error!("Failed to parse sample rate");
            48000u32
        }
    } else {
        log::error!("Sample rate not found in context properties");
        48000u32
    };
    let voicealloc = Box::new(PolySynth::<f32>::new(
        culsynth::context::Context::new(sample_rate as f32),
        16,
    ));
    let pw_context = PwContext {
        sample_rate,
        bufsize: 512,
        fixed: false,
        voice_mode: culsynth_plugin::VoiceMode::Poly16,
    };
    let _audio_thread = thread::spawn(move || {
        pw_audio(midi_rx, synth_rx, mainloop, context, core, voicealloc);
    });
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        culsynth_plugin::NAME,
        native_options,
        Box::new(move |cc| Ok(Box::new(PwApp::new(midi_tx, synth_tx, cc, pw_context)))),
    );
}
