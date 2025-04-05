use pipewire as pw;
use pw::properties::properties;

struct PwAppData {}

impl PwAppData {
    fn new() -> Self {
        Self {}
    }
}

pub fn run() {
    pw::init();
    let mainloop = pw::main_loop::MainLoop::new(None).expect("Failed to create main loop");
    let context = pw::context::Context::new(&mainloop).expect("Failed to create context");
    let core = context.connect(None).expect("Failed to connect to context");
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
        .add_local_listener_with_user_data(PwAppData::new())
        .process(|s, data| {
            //process
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
}
