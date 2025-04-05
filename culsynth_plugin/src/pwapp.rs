use std::sync::mpsc;
use std::thread;

use culsynth_plugin::voicealloc::PolySynth;
use culsynth_plugin::voicealloc::VoiceAllocator;
use pipewire as pw;
use pw::properties::properties;
use wmidi::MidiMessage;

struct PwAudioProcessor {
    midi_rx: mpsc::Receiver<MidiMessage<'static>>,
    synth_rx: mpsc::Receiver<Box<dyn VoiceAllocator>>,
    voicealloc: Box<dyn VoiceAllocator>,
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
        }
    }
    fn process(&mut self, buf: pw::buffer::Buffer) {
        while let Ok(msg) = self.midi_rx.try_recv() {
            match msg {
                MidiMessage::NoteOn(_, _, _) => {
                    // Handle Note On message
                }
                MidiMessage::NoteOff(_, _, _) => {
                    // Handle Note Off message
                }
                _ => {}
            }
        }
        while let Ok(synth) = self.synth_rx.try_recv() {
            // Handle synth messages
        }
    }
}

fn pw_audio(
    midi_rx: mpsc::Receiver<MidiMessage<'static>>,
    synth_rx: mpsc::Receiver<Box<dyn VoiceAllocator>>,
) {
    pw::init();
    let mainloop = pw::main_loop::MainLoop::new(None).expect("Failed to create main loop");
    let context = pw::context::Context::new(&mainloop).expect("Failed to create context");
    let core = context.connect(None).expect("Failed to connect to context");
    let sample_rate = 48000f32;
    let voicealloc = Box::new(PolySynth::<f32>::new(
        culsynth::context::Context::new(sample_rate),
        16,
    ));
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
    stream
        .connect(
            pw::spa::utils::Direction::Output,
            None,
            pw::stream::StreamFlags::AUTOCONNECT
                | pw::stream::StreamFlags::RT_PROCESS
                | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut [],
        )
        .expect("Failed to connect stream");
    mainloop.run();
}

pub fn run() {
    let (midi_tx, midi_rx) = mpsc::sync_channel::<MidiMessage<'static>>(16);
    let (synth_tx, synth_rx) = mpsc::sync_channel::<Box<dyn VoiceAllocator>>(1);
    let _audio_thread = thread::spawn(move || {
        pw_audio(midi_rx, synth_rx);
    });
}
