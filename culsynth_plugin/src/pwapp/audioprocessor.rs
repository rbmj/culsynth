use std::io::Write;
use std::sync::mpsc;

use pipewire::spa::param::audio::{AudioFormat, AudioInfoRaw};
use pipewire::spa::pod::{Pod, PodObject};
use wmidi::MidiMessage;

use culsynth_plugin::voicealloc::{MonoSynth, PolySynth, VoiceAllocator};
use culsynth_plugin::{ownedmidihandler::OwnedMidiHandler, MidiHandler};

use crate::PwContext;
pub struct PwAudioProcessor {
    midi_rx: mpsc::Receiver<MidiMessage<'static>>,
    synth_rx: mpsc::Receiver<Box<dyn VoiceAllocator>>,
    ctx_tx: mpsc::SyncSender<PwContext>,
    voicealloc: Option<Box<dyn VoiceAllocator>>,
    param_handler: OwnedMidiHandler,
    audio_fmt: Option<AudioInfoRaw>,
    context: Option<PwContext>,
    acc: f32,
}

impl PwAudioProcessor {
    pub fn new(
        midi_rx: mpsc::Receiver<MidiMessage<'static>>,
        synth_rx: mpsc::Receiver<Box<dyn VoiceAllocator>>,
        ctx_tx: mpsc::SyncSender<PwContext>,
    ) -> Self {
        let context = PwContext {
            sample_rate: 0,
            bufsize: 0,
            fixed: false,
            voice_mode: culsynth_plugin::VoiceMode::Poly16,
        };
        Self {
            midi_rx,
            synth_rx,
            ctx_tx,
            voicealloc: None,
            param_handler: OwnedMidiHandler::new(wmidi::Channel::Ch1),
            audio_fmt: None,
            context: Some(context),
            acc: 0f32,
        }
    }
    pub fn process(&mut self, mut pwbuf: pipewire::buffer::Buffer) -> Result<(), &'static str> {
        while let Ok(synth) = self.synth_rx.try_recv() {
            if let Some(fmt) = self.audio_fmt {
                let _ = self.ctx_tx.try_send(PwContext {
                    sample_rate: fmt.rate(),
                    bufsize: 0,
                    fixed: synth.get_context().is_fixed_point(),
                    voice_mode: if synth.is_poly() {
                        culsynth_plugin::VoiceMode::Poly16
                    } else {
                        culsynth_plugin::VoiceMode::Mono
                    },
                });
            }
            self.voicealloc = Some(synth);
        }
        if let Some(ctx) = self.context.take() {
            self.ctx_tx.send(ctx).expect("Failed to send context to GUI thread");
        }
        let voicealloc = self.voicealloc.as_mut().expect("Synth Engine not Initialized");
        while let Ok(msg) = self.midi_rx.try_recv() {
            voicealloc.handle_midi(msg.clone());
            self.param_handler.send(msg);
        }

        let first_block = pwbuf.datas_mut().first_mut().ok_or("No data blocks")?;
        let buf = first_block.data().ok_or("Buffer not memory mapped")?;
        let bufsz = buf.len();
        let mut buf = std::io::Cursor::new(buf);
        let fmt = self.audio_fmt.ok_or("Audio format not set")?;
        let mut matrix = Some(self.param_handler.get_matrix());
        let params = self.param_handler.get_params();
        const MAX_FRAMES: usize = 1024;
        match fmt.format() {
            AudioFormat::F32LE => {
                let frame_stride = fmt.channels() as usize * std::mem::size_of::<f32>();
                let mut num_frames = bufsz / frame_stride;
                //num_frames = std::cmp::max(num_frames, MAX_FRAMES);
                for _frameidx in 0..num_frames {
                    let smp_value = voicealloc.next(&params, matrix.take().as_ref());

                    let smp_bytes = smp_value.to_le_bytes();
                    for _smpidx in 0..fmt.channels() {
                        buf.write(&smp_bytes).unwrap();
                    }
                }
                let chunk = first_block.chunk_mut();
                *chunk.offset_mut() = 0;
                *chunk.stride_mut() = frame_stride as i32;
                *chunk.size_mut() = num_frames as u32 * frame_stride as u32;
            }
            AudioFormat::S16LE => {
                let frame_stride = fmt.channels() as usize * std::mem::size_of::<i16>();
                let mut num_frames = bufsz / frame_stride;
                //num_frames = std::cmp::max(num_frames, MAX_FRAMES);
                for _frameidx in 0..num_frames {
                    let smp_value = voicealloc.next(&params, matrix.take().as_ref());

                    let smp_bytes = culsynth::SampleFxP::from_num(smp_value).to_le_bytes();
                    for _smpidx in 0..fmt.channels() {
                        buf.write(&smp_bytes).unwrap();
                    }
                }
                let chunk = first_block.chunk_mut();
                *chunk.offset_mut() = 0;
                *chunk.stride_mut() = frame_stride as i32;
                *chunk.size_mut() = num_frames as u32 * frame_stride as u32;
            }
            _ => return Err("Unsupported audio format"),
        }

        Ok(())
    }
    pub fn spa_param_changed(&mut self, id: u32, value: &Pod) {
        use pipewire::spa::param::ParamType;
        match ParamType(id) {
            ParamType::Format => self.spa_format(value),
            ParamType::Props => {
                if let Ok(obj) = value.as_object() {
                    self.spa_props(obj)
                }
            }
            ParamType::Latency => {
                if let Ok(obj) = value.as_object() {
                    self.spa_latency(obj);
                }
            }
            _ => {
                log::info!("Unprocessed param type: {}", id);
            }
        }
    }
    fn spa_props(&mut self, obj: &PodObject) {
        for prop in obj.props() {
            match prop.key() {
                unmatched => {
                    log::info!(
                        "Unmatched property {} with type {}",
                        unmatched.0,
                        prop.value().type_().as_raw()
                    );
                }
            }
        }
    }
    fn spa_latency(&mut self, latency: &PodObject) {
        use pipewire::spa::sys;
        use pipewire::spa::utils::Id;
        for prop in latency.props() {
            match prop.key() {
                Id(sys::SPA_PARAM_LATENCY_minRate) => {
                    if let Ok(l) = prop.value().get_int() {
                        log::info!("Min latency rate: {}", l);
                    }
                }
                Id(sys::SPA_PARAM_LATENCY_maxRate) => {
                    if let Ok(l) = prop.value().get_int() {
                        log::info!("Max latency rate: {}", l);
                    }
                }
                unmatched => {
                    log::info!(
                        "Unmatched latency {} type {}",
                        unmatched.0,
                        prop.value().type_().as_raw()
                    );
                }
            }
        }
    }
    fn spa_format(&mut self, format: &Pod) {
        use pipewire::spa::param::format::{MediaSubtype, MediaType};
        let Ok((media_type, media_subtype)) =
            pipewire::spa::param::format_utils::parse_format(format)
        else {
            log::error!("Failed to parse format");
            return;
        };
        if media_type != MediaType::Audio {
            log::error!("Unsupported media type: {:?}", media_type);
            return;
        }
        if media_subtype != MediaSubtype::Raw {
            log::error!("Unsupported media subtype: {:?}", media_subtype);
            return;
        }
        let mut info_raw = pipewire::spa::param::audio::AudioInfoRaw::default();
        if let Ok(_) = info_raw.parse(format) {
            log::info!(
                "Audio Sample Rate: {} Hz / {} channels",
                info_raw.rate(),
                info_raw.channels()
            );
            match info_raw.format() {
                pipewire::spa::param::audio::AudioFormat::F32LE => {
                    log::info!("Audio format: F32LE");
                }
                pipewire::spa::param::audio::AudioFormat::S16LE => {
                    log::info!("Audio format: S16LE");
                }
                _ => {
                    log::error!("Unsupported audio format");
                    return;
                }
            }
            if let Some(ctx) = self.context.as_mut() {
                ctx.sample_rate = info_raw.rate();
                if self.voicealloc.is_none() {
                    self.voicealloc = Some(match ctx.voice_mode {
                        culsynth_plugin::VoiceMode::Mono => Box::new(MonoSynth::<f32>::new(
                            culsynth::context::Context::new(ctx.sample_rate as f32),
                        )),
                        culsynth_plugin::VoiceMode::Poly16 => Box::new(PolySynth::<f32>::new(
                            culsynth::context::Context::new(ctx.sample_rate as f32),
                            16,
                        )),
                    });
                }
            }
            self.audio_fmt = Some(info_raw);
        } else {
            log::error!("Failed to parse audio format");
        }
    }
}
