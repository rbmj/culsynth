use culsynth::devices::{
    EnvParams, LfoParams, LfoWave, MixOscParams, ModFiltParams, RingModParams,
};
use culsynth::voice::cc::*;
use culsynth::voice::VoiceParams;
use culsynth::{CoarseTuneFxP, FineTuneFxP};

use super::egui;
use super::param_widget::MidiCcSlider;
use crate::MidiHandler;
use crate::Tuning;

#[derive(Default)]
pub struct MainUi {}

impl MainUi {
    pub fn draw(
        &self,
        ui: &mut egui::Ui,
        params: &VoiceParams<i16>,
        tune: (Tuning, Tuning),
        dispatcher: &dyn MidiHandler,
    ) {
        ui.spacing_mut().slider_width = super::SLIDER_WIDTH;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                draw_osc(
                    ui,
                    "Oscillator 1",
                    &params.oscs_p.primary,
                    tune.0,
                    &OSC1_CC_ALL,
                    None,
                    dispatcher,
                );
                ui.separator();
                draw_osc(
                    ui,
                    "Oscillator 2",
                    &params.oscs_p.secondary,
                    tune.1,
                    &OSC2_CC_ALL,
                    Some((OSC_SYNC, params.oscs_p.sync)),
                    dispatcher,
                );
                ui.separator();
                draw_ringmod(
                    ui,
                    "Mixer/Ring Modulator",
                    &params.ring_p,
                    &RING_CCS_ALL,
                    dispatcher,
                );
            });
            ui.separator();
            ui.horizontal(|ui| {
                draw_filter(ui, "Filter", &params.filt_p, &FILT_CCS_ALL, dispatcher);
                ui.separator();
                draw_lfo(ui, "LFO 1", &params.lfo1_p, &LFO1_CCS_ALL, dispatcher);
                ui.separator();
                draw_lfo(ui, "LFO 2", &params.lfo2_p, &LFO2_CCS_ALL, dispatcher);
            });
            ui.separator();
            ui.horizontal(|ui| {
                draw_env(
                    ui,
                    "Filter Envelope",
                    &params.filt_env_p,
                    &ENV_FILT_CCS_ALL,
                    dispatcher,
                );
                ui.separator();
                draw_env(
                    ui,
                    "Amplifier Envelope",
                    &params.amp_env_p,
                    &ENV_AMP_CCS_ALL,
                    dispatcher,
                );
                ui.separator();
                draw_env(
                    ui,
                    "Mod Envelope 1",
                    &params.env1_p,
                    &ENV_M1_CCS_ALL,
                    dispatcher,
                );
                ui.separator();
                draw_env(
                    ui,
                    "Mod Envelope 2",
                    &params.env2_p,
                    &ENV_M2_CCS_ALL,
                    dispatcher,
                );
            });
        });
    }
}

fn draw_osc(
    ui: &mut egui::Ui,
    label: &'static str,
    params: &MixOscParams<i16>,
    tuning: Tuning,
    osc_ccs: &OscCCs,
    sync: Option<(wmidi::ControlFunction, bool)>,
    dispatcher: &dyn MidiHandler,
) {
    let default_params = MixOscParams::<i16>::default();
    ui.vertical(|ui| {
        if let Some((sync_cc, sync_on)) = sync {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.label(" - ");
                let sync_str = if sync_on {
                    "Sync On"
                } else {
                    "Click to Enable Sync"
                };
                if ui.selectable_label(sync_on, sync_str).clicked() {
                    dispatcher.send_cc(
                        sync_cc,
                        if sync_on {
                            wmidi::U7::MIN
                        } else {
                            wmidi::U7::MAX
                        },
                    );
                }
            });
        } else {
            ui.label(label);
        }
        ui.horizontal(|ui| {
            use culsynth::util::*;
            ui.add(MidiCcSlider::new_fixed(
                tuning.coarse,
                CoarseTuneFxP::ZERO,
                Some("semi"),
                osc_ccs.coarse,
                "CRS",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_fixed(
                tuning.fine,
                FineTuneFxP::ZERO,
                Some("semi"),
                osc_ccs.fine,
                "FIN",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.shape,
                default_params.shape,
                None,
                osc_ccs.shape,
                "SHP",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.sin,
                default_params.sin,
                None,
                osc_ccs.sin,
                SIN_CHARSTR,
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.tri,
                default_params.tri,
                None,
                osc_ccs.tri,
                TRI_CHARSTR,
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.sq,
                default_params.sq,
                None,
                osc_ccs.sq,
                SQ_CHARSTR,
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.saw,
                default_params.saw,
                None,
                osc_ccs.saw,
                SAW_CHARSTR,
                dispatcher,
            ));
        });
    });
}

fn draw_lfo(
    ui: &mut egui::Ui,
    label: &'static str,
    params: &LfoParams<i16>,
    lfo_ccs: &LfoCCs,
    dispatcher: &dyn MidiHandler,
) {
    const DEFAULT_PARAMS: LfoParams<i16> = LfoParams::new();
    const DEFAULT_WAVE: LfoWave = DEFAULT_PARAMS.opts.wave().unwrap();
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.add(MidiCcSlider::new_fixed(
                    params.freq,
                    DEFAULT_PARAMS.freq,
                    Some("Hz"),
                    lfo_ccs.rate,
                    "RATE",
                    dispatcher,
                ));
                ui.add(MidiCcSlider::new_percent(
                    params.depth,
                    DEFAULT_PARAMS.depth,
                    None,
                    lfo_ccs.depth,
                    "DEPTH",
                    dispatcher,
                ));
            });
            ui.vertical(|ui| {
                let cur_wave = params.opts.wave().unwrap_or(DEFAULT_WAVE);
                for wave in LfoWave::waves() {
                    if ui.selectable_label(cur_wave == *wave, wave.to_str_short()).clicked() {
                        dispatcher.send_cc(lfo_ccs.wave, wmidi::U7::from_u8_lossy(*wave as u8));
                    }
                }
            });
            ui.vertical(|ui| {
                if ui.selectable_label(params.opts.retrigger(), "Retrigger").clicked() {
                    dispatcher.send_cc(
                        lfo_ccs.retrigger,
                        if params.opts.retrigger() {
                            wmidi::U7::MIN
                        } else {
                            wmidi::U7::MAX
                        },
                    );
                }
                if ui.selectable_label(params.opts.bipolar(), "Bipolar").clicked() {
                    dispatcher.send_cc(
                        lfo_ccs.bipolar,
                        if params.opts.bipolar() {
                            wmidi::U7::MIN
                        } else {
                            wmidi::U7::MAX
                        },
                    );
                }
            });
        });
    });
}

fn draw_ringmod(
    ui: &mut egui::Ui,
    label: &'static str,
    params: &RingModParams<i16>,
    ccs: &RingModCCs,
    dispatcher: &dyn MidiHandler,
) {
    const DEFAULT_PARAMS: RingModParams<i16> = RingModParams::new();
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.add(MidiCcSlider::new_percent(
                params.mix_a,
                DEFAULT_PARAMS.mix_a,
                None,
                ccs.mix_a,
                "Osc 1",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.mix_b,
                DEFAULT_PARAMS.mix_b,
                None,
                ccs.mix_b,
                "Osc 2",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.mix_mod,
                DEFAULT_PARAMS.mix_mod,
                None,
                ccs.mix_mod,
                "Ring",
                dispatcher,
            ));
        });
    });
}

fn draw_filter(
    ui: &mut egui::Ui,
    label: &'static str,
    params: &ModFiltParams<i16>,
    ccs: &FiltCCs,
    dispatcher: &dyn MidiHandler,
) {
    let default_params: ModFiltParams<i16> = Default::default();
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.add(MidiCcSlider::new_fixed(
                params.cutoff,
                default_params.cutoff,
                None,
                ccs.cutoff,
                "Cut",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.resonance,
                default_params.resonance,
                None,
                ccs.resonance,
                "Res",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.kbd_tracking,
                default_params.kbd_tracking,
                None,
                ccs.kbd,
                "Kbd",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.vel_mod,
                default_params.vel_mod,
                None,
                ccs.vel,
                "Vel",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.env_mod,
                default_params.env_mod,
                None,
                ccs.env,
                "Env",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.low_mix,
                default_params.low_mix,
                None,
                ccs.low,
                "Low",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.band_mix,
                default_params.band_mix,
                None,
                ccs.band,
                "Band",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.high_mix,
                default_params.high_mix,
                None,
                ccs.high,
                "High",
                dispatcher,
            ));
        });
    });
}

fn draw_env(
    ui: &mut egui::Ui,
    label: &'static str,
    params: &EnvParams<i16>,
    ccs: &EnvCCs,
    dispatcher: &dyn MidiHandler,
) {
    let default_params: EnvParams<i16> = Default::default();
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.add(MidiCcSlider::new_fixed(
                params.attack,
                default_params.attack,
                Some("s"),
                ccs.attack,
                "A",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_fixed(
                params.decay,
                default_params.decay,
                Some("s"),
                ccs.decay,
                "D",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_percent(
                params.sustain,
                default_params.sustain,
                None,
                ccs.sustain,
                "S",
                dispatcher,
            ));
            ui.add(MidiCcSlider::new_fixed(
                params.release,
                default_params.release,
                Some("s"),
                ccs.release,
                "R",
                dispatcher,
            ));
        });
    });
}
