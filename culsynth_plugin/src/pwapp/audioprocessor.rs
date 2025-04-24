use std::sync::mpsc;

use pipewire::spa::param::audio::AudioInfoRaw;
use pipewire::spa::pod::{Pod, PodObject};
use wmidi::MidiMessage;

use crate::voicealloc::{MonoSynth, PolySynth, VoiceAllocator};
use crate::{ownedmidihandler::OwnedMidiHandler, MidiHandler};

use super::supportedformats::PwSupportedFormats;
use super::PwContext;

unsafe fn transmute_buf<'a, T>(buf: &'a mut [u8]) -> &'a mut [T] {
    unsafe {
        std::slice::from_raw_parts_mut(
            buf.as_mut_ptr() as *mut T,
            buf.len() / std::mem::size_of::<T>(),
        )
    }
}

pub struct PwAudioProcessor {
    midi_rx: mpsc::Receiver<MidiMessage<'static>>,
    synth_rx: mpsc::Receiver<Box<dyn VoiceAllocator>>,
    ctx_tx: mpsc::SyncSender<PwContext>,
    voicealloc: Option<Box<dyn VoiceAllocator>>,
    param_handler: OwnedMidiHandler,
    audio_fmt: Option<AudioInfoRaw>,
    context: Option<PwContext>,
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
            voice_mode: crate::VoiceMode::Poly16,
        };
        Self {
            midi_rx,
            synth_rx,
            ctx_tx,
            voicealloc: None,
            param_handler: OwnedMidiHandler::new(wmidi::Channel::Ch1),
            audio_fmt: None,
            context: Some(context),
        }
    }
    pub fn process(&mut self, mut pwbuf: pipewire::buffer::Buffer) -> Result<(), &'static str> {
        let instrumentation = crate::instrumentation::begin();
        while let Ok(synth) = self.synth_rx.try_recv() {
            if let Some(fmt) = self.audio_fmt {
                let _ = self.ctx_tx.try_send(PwContext {
                    sample_rate: fmt.rate(),
                    bufsize: 0,
                    fixed: synth.get_context().is_fixed_point(),
                    voice_mode: if synth.is_poly() {
                        crate::VoiceMode::Poly16
                    } else {
                        crate::VoiceMode::Mono
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
        let fmt = self.audio_fmt.ok_or("Audio format not set")?;
        let num_ch = fmt.channels() as usize;
        let mut matrix = Some(self.param_handler.get_matrix());
        let params = self.param_handler.get_params();
        match fmt.format() {
            PwSupportedFormats::FMT_F32 => {
                let buf = unsafe { transmute_buf::<f32>(buf) };
                let frame_stride = num_ch * std::mem::size_of::<f32>();
                let mut num_frames = 0usize;
                for frame in buf.chunks_exact_mut(num_ch) {
                    let smp_value = voicealloc.next(&params, matrix.take().as_ref());
                    for smp in frame {
                        *smp = smp_value;
                    }
                    num_frames += 1;
                }
                let chunk = first_block.chunk_mut();
                *chunk.offset_mut() = 0;
                *chunk.stride_mut() = frame_stride as i32;
                *chunk.size_mut() = (num_frames * frame_stride) as u32;
            }
            PwSupportedFormats::FMT_I16 => {
                let buf = unsafe { transmute_buf::<i16>(buf) };
                let frame_stride = num_ch * std::mem::size_of::<i16>();
                let mut num_frames = 0usize;
                for frame in buf.chunks_exact_mut(num_ch) {
                    let smp_float = voicealloc.next(&params, matrix.take().as_ref());
                    let smp_value = culsynth::SampleFxP::from_num(smp_float).to_bits();
                    for smp in frame {
                        *smp = smp_value;
                    }
                    num_frames += 1;
                }
                let chunk = first_block.chunk_mut();
                *chunk.offset_mut() = 0;
                *chunk.stride_mut() = frame_stride as i32;
                *chunk.size_mut() = (num_frames * frame_stride) as u32;
            }
            _ => return Err("Unsupported audio format"),
        }
        crate::instrumentation::end(instrumentation);
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
                        crate::VoiceMode::Mono => Box::new(MonoSynth::<f32>::new(
                            culsynth::context::Context::new(ctx.sample_rate as f32),
                        )),
                        crate::VoiceMode::Poly16 => Box::new(PolySynth::<f32>::new(
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
