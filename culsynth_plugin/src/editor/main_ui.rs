use culsynth::devices::{
    EnvParams, LfoParams, LfoWave, MixOscParams, ModFiltParams, RingModParams,
};
use culsynth::voice::cc::*;
use culsynth::voice::VoiceParams;
use culsynth::{CoarseTuneFxP, FineTuneFxP};

use super::egui;
use super::param_widget::MidiCcSliderBuilder;
use super::EditorModData;
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
        matrix: &culsynth::voice::modulation::ModMatrix<i16>,
        mod_data: &mut EditorModData,
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
                    matrix,
                    mod_data,
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
                    matrix,
                    mod_data,
                );
                ui.separator();
                draw_ringmod(
                    ui,
                    "Mixer/Ring Modulator",
                    &params.ring_p,
                    &RING_CCS_ALL,
                    dispatcher,
                    matrix,
                    mod_data,
                );
            });
            ui.separator();
            ui.horizontal(|ui| {
                draw_filter(
                    ui,
                    "Filter",
                    &params.filt_p,
                    &FILT_CCS_ALL,
                    dispatcher,
                    matrix,
                    mod_data,
                );
                ui.separator();
                draw_lfo(
                    ui,
                    "LFO 1",
                    &params.lfo1_p,
                    &LFO1_CCS_ALL,
                    dispatcher,
                    matrix,
                    mod_data,
                );
                ui.separator();
                draw_lfo(
                    ui,
                    "LFO 2",
                    &params.lfo2_p,
                    &LFO2_CCS_ALL,
                    dispatcher,
                    matrix,
                    mod_data,
                );
            });
            ui.separator();
            ui.horizontal(|ui| {
                draw_env(
                    ui,
                    "Filter Envelope",
                    &params.filt_env_p,
                    &ENV_FILT_CCS_ALL,
                    dispatcher,
                    matrix,
                    mod_data,
                );
                ui.separator();
                draw_env(
                    ui,
                    "Amplifier Envelope",
                    &params.amp_env_p,
                    &ENV_AMP_CCS_ALL,
                    dispatcher,
                    matrix,
                    mod_data,
                );
                ui.separator();
                draw_env(
                    ui,
                    "Mod Envelope 1",
                    &params.env1_p,
                    &ENV_M1_CCS_ALL,
                    dispatcher,
                    matrix,
                    mod_data,
                );
                ui.separator();
                draw_env(
                    ui,
                    "Mod Envelope 2",
                    &params.env2_p,
                    &ENV_M2_CCS_ALL,
                    dispatcher,
                    matrix,
                    mod_data,
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
    matrix: &culsynth::voice::modulation::ModMatrix<i16>,
    mod_data: &mut EditorModData,
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
            ui.add(
                MidiCcSliderBuilder::new("CRS", dispatcher, tuning.coarse)
                    .with_control(osc_ccs.coarse)
                    .with_units("semi")
                    .with_default(CoarseTuneFxP::ZERO)
                    .with_mod_data(mod_data, matrix, osc_ccs.mod_coarse)
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("FIN", dispatcher, tuning.fine)
                    .with_control(osc_ccs.fine)
                    .with_units("semi")
                    .with_default(FineTuneFxP::ZERO)
                    .with_mod_data(mod_data, matrix, osc_ccs.mod_fine)
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("SHP", dispatcher, params.shape)
                    .with_control(osc_ccs.shape)
                    .with_default(default_params.shape)
                    .with_mod_data(mod_data, matrix, osc_ccs.mod_shape)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new(SIN_CHARSTR, dispatcher, params.sin)
                    .with_control(osc_ccs.sin)
                    .with_default(default_params.sin)
                    .with_mod_data(mod_data, matrix, osc_ccs.mod_sin)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new(TRI_CHARSTR, dispatcher, params.tri)
                    .with_control(osc_ccs.tri)
                    .with_default(default_params.tri)
                    .with_mod_data(mod_data, matrix, osc_ccs.mod_tri)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new(SQ_CHARSTR, dispatcher, params.sq)
                    .with_control(osc_ccs.sq)
                    .with_default(default_params.sq)
                    .with_mod_data(mod_data, matrix, osc_ccs.mod_sq)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new(SAW_CHARSTR, dispatcher, params.saw)
                    .with_control(osc_ccs.saw)
                    .with_default(default_params.saw)
                    .with_mod_data(mod_data, matrix, osc_ccs.mod_saw)
                    .as_percent()
                    .build(),
            );
        });
    });
}

fn draw_lfo(
    ui: &mut egui::Ui,
    label: &'static str,
    params: &LfoParams<i16>,
    lfo_ccs: &LfoCCs,
    dispatcher: &dyn MidiHandler,
    matrix: &culsynth::voice::modulation::ModMatrix<i16>,
    mod_data: &mut EditorModData,
) {
    const DEFAULT_PARAMS: LfoParams<i16> = LfoParams::new();
    const DEFAULT_WAVE: LfoWave = DEFAULT_PARAMS.opts.wave().unwrap();
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.add(
                    MidiCcSliderBuilder::new("RATE", dispatcher, params.freq)
                        .with_control(lfo_ccs.rate)
                        .with_default(DEFAULT_PARAMS.freq)
                        .with_mod_data(mod_data, matrix, lfo_ccs.mod_rate)
                        .with_units("Hz")
                        .build(),
                );
                ui.add(
                    MidiCcSliderBuilder::new("DEPTH", dispatcher, params.depth)
                        .with_control(lfo_ccs.depth)
                        .with_default(DEFAULT_PARAMS.depth)
                        .with_mod_data(mod_data, matrix, lfo_ccs.mod_depth)
                        .as_percent()
                        .build(),
                );
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
    matrix: &culsynth::voice::modulation::ModMatrix<i16>,
    mod_data: &mut EditorModData,
) {
    const DEFAULT_PARAMS: RingModParams<i16> = RingModParams::new();
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.add(
                MidiCcSliderBuilder::new("Osc 1", dispatcher, params.mix_a)
                    .with_control(ccs.mix_a)
                    .with_default(DEFAULT_PARAMS.mix_a)
                    .with_mod_data(mod_data, matrix, ccs.mod_mix_a)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("Osc 2", dispatcher, params.mix_b)
                    .with_control(ccs.mix_b)
                    .with_default(DEFAULT_PARAMS.mix_b)
                    .with_mod_data(mod_data, matrix, ccs.mod_mix_b)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("Ring", dispatcher, params.mix_mod)
                    .with_control(ccs.mix_mod)
                    .with_default(DEFAULT_PARAMS.mix_mod)
                    .with_mod_data(mod_data, matrix, ccs.mod_mix_mod)
                    .as_percent()
                    .build(),
            );
        });
    });
}

fn draw_filter(
    ui: &mut egui::Ui,
    label: &'static str,
    params: &ModFiltParams<i16>,
    ccs: &FiltCCs,
    dispatcher: &dyn MidiHandler,
    matrix: &culsynth::voice::modulation::ModMatrix<i16>,
    mod_data: &mut EditorModData,
) {
    let default_params: ModFiltParams<i16> = Default::default();
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.add(
                MidiCcSliderBuilder::new("Cut", dispatcher, params.cutoff)
                    .with_control(ccs.cutoff)
                    .with_default(default_params.cutoff)
                    .with_mod_data(mod_data, matrix, ccs.mod_cutoff)
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("Res", dispatcher, params.resonance)
                    .with_control(ccs.resonance)
                    .with_default(default_params.resonance)
                    .with_mod_data(mod_data, matrix, ccs.mod_resonance)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("Kbd", dispatcher, params.kbd_tracking)
                    .with_control(ccs.kbd)
                    .with_default(default_params.kbd_tracking)
                    .with_mod_data(mod_data, matrix, ccs.mod_kbd)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("Vel", dispatcher, params.vel_mod)
                    .with_control(ccs.vel)
                    .with_default(default_params.vel_mod)
                    .with_mod_data(mod_data, matrix, ccs.mod_vel)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("Env", dispatcher, params.env_mod)
                    .with_control(ccs.env)
                    .with_default(default_params.env_mod)
                    .with_mod_data(mod_data, matrix, ccs.mod_env)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("Low", dispatcher, params.low_mix)
                    .with_control(ccs.low)
                    .with_default(default_params.low_mix)
                    .with_mod_data(mod_data, matrix, ccs.mod_low)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("Band", dispatcher, params.band_mix)
                    .with_control(ccs.band)
                    .with_default(default_params.band_mix)
                    .with_mod_data(mod_data, matrix, ccs.mod_band)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("High", dispatcher, params.high_mix)
                    .with_control(ccs.high)
                    .with_default(default_params.high_mix)
                    .with_mod_data(mod_data, matrix, ccs.mod_high)
                    .as_percent()
                    .build(),
            );
        });
    });
}

fn draw_env(
    ui: &mut egui::Ui,
    label: &'static str,
    params: &EnvParams<i16>,
    ccs: &EnvCCs,
    dispatcher: &dyn MidiHandler,
    matrix: &culsynth::voice::modulation::ModMatrix<i16>,
    mod_data: &mut EditorModData,
) {
    let default_params: EnvParams<i16> = Default::default();
    ui.vertical(|ui| {
        ui.label(label);
        ui.horizontal(|ui| {
            ui.add(
                MidiCcSliderBuilder::new("A", dispatcher, params.attack)
                    .with_control(ccs.attack)
                    .with_default(default_params.attack)
                    .with_mod_data(mod_data, matrix, ccs.mod_attack)
                    .with_units("s")
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("D", dispatcher, params.decay)
                    .with_control(ccs.decay)
                    .with_default(default_params.decay)
                    .with_mod_data(mod_data, matrix, ccs.mod_decay)
                    .with_units("s")
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("S", dispatcher, params.sustain)
                    .with_control(ccs.sustain)
                    .with_default(default_params.sustain)
                    .with_mod_data(mod_data, matrix, ccs.mod_sustain)
                    .as_percent()
                    .build(),
            );
            ui.add(
                MidiCcSliderBuilder::new("R", dispatcher, params.release)
                    .with_control(ccs.release)
                    .with_default(default_params.release)
                    .with_mod_data(mod_data, matrix, ccs.mod_release)
                    .with_units("s")
                    .build(),
            );
        });
    });
}
