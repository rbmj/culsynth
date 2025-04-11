use std::sync::mpsc;
use std::thread;

use culsynth_plugin::editor::Editor;
use culsynth_plugin::ownedmidihandler::OwnedMidiHandler;
use culsynth_plugin::voicealloc::VoiceAllocator;

use culsynth_plugin::ContextReader;
use pipewire as pw;
use pw::properties::properties;

use wmidi::MidiMessage;

mod pwapp;

pub use pwapp::audioprocessor::PwAudioProcessor;
use pwapp::supportedformats::PwSupportedFormats;

pub struct PwContext {
    sample_rate: u32,
    bufsize: usize,
    fixed: bool,
    voice_mode: culsynth_plugin::VoiceMode,
}

struct PwMidiHandler {
    owned: OwnedMidiHandler,
    sender: mpsc::SyncSender<MidiMessage<'static>>,
}

impl PwMidiHandler {
    fn new(sender: mpsc::SyncSender<MidiMessage<'static>>, ch: wmidi::Channel) -> Self {
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
        if let Err(e) = self.sender.send(msg.clone()) {
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
    synth_tx: mpsc::SyncSender<Box<dyn VoiceAllocator>>,
    context_rx: mpsc::Receiver<PwContext>,
    param_handler: PwMidiHandler,
    editor: Editor,
    context: PwContext,
}

impl PwApp {
    fn new(
        midi_tx: mpsc::SyncSender<MidiMessage<'static>>,
        synth_tx: mpsc::SyncSender<Box<dyn VoiceAllocator>>,
        ctx_rx: mpsc::Receiver<PwContext>,
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
            context_rx: ctx_rx,
        }
    }
}

impl eframe::App for PwApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(context) = self.context_rx.try_recv() {
            self.context = context;
        }
        self.editor.update(
            ctx,
            &self.context,
            &self.param_handler.get_params(),
            self.param_handler.get_tuning(),
            &self.param_handler.get_matrix(),
            &self.param_handler,
            Some(&self.synth_tx),
        );
    }
}

fn pw_audio(
    midi_rx: mpsc::Receiver<MidiMessage<'static>>,
    synth_rx: mpsc::Receiver<Box<dyn VoiceAllocator>>,
    ctx_tx: mpsc::SyncSender<PwContext>,
) {
    pw::init();
    let mainloop = pw::main_loop::MainLoop::new(None).expect("Failed to create main loop");
    let context = pw::context::Context::new(&mainloop).expect("Failed to create context");
    let core = context.connect(None).expect("Failed to connect to context");

    let props = properties! {
        *pw::keys::APP_NAME => culsynth_plugin::NAME,
        *pw::keys::APP_VERSION => culsynth_plugin::VERSION,
        *pw::keys::MEDIA_CATEGORY => "Playback",
        *pw::keys::MEDIA_ROLE => "DSP",
        *pw::keys::MEDIA_TYPE => "Audio",
    };
    let stream = pw::stream::Stream::new(&core, culsynth_plugin::NAME, props)
        .expect("Failed to create stream");
    log::info!("Stream Properties: {:?}", stream.properties());
    log::info!("Context Properties: {:?}", context.properties());
    let _listener = stream
        .add_local_listener_with_user_data(PwAudioProcessor::new(midi_rx, synth_rx, ctx_tx))
        .param_changed(|_s, processor, id, param| {
            if let Some(val) = param {
                processor.spa_param_changed(id, val);
            }
        })
        .process(|s, processor| {
            if let Some(buf) = s.dequeue_buffer() {
                if let Err(errstr) = processor.process(buf) {
                    log::error!("Processing Error: {}", errstr);
                }
            }
        })
        .register()
        .expect("Failed to register stream with pipewire");

    let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &PwSupportedFormats {},
    )
    .unwrap()
    .0
    .into_inner();
    let mut params = [pw::spa::pod::Pod::from_bytes(&values).unwrap()];
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

fn main() {
    colog::init();
    let (midi_tx, midi_rx) = mpsc::sync_channel::<MidiMessage<'static>>(32);
    let (synth_tx, synth_rx) = mpsc::sync_channel::<Box<dyn VoiceAllocator>>(1);
    let (ctx_tx, ctx_rx) = mpsc::sync_channel::<PwContext>(1);

    let _audio_thread = thread::spawn(move || {
        pw_audio(midi_rx, synth_rx, ctx_tx);
    });
    let pw_context = ctx_rx.recv().expect("Failed to receive context from audio thread");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        culsynth_plugin::NAME,
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(PwApp::new(
                midi_tx, synth_tx, ctx_rx, cc, pw_context,
            )))
        }),
    )
    .expect("Failed to run eframe app");
}
