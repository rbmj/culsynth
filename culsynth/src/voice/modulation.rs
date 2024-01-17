//! This module contains data to allow modulation of a `Voice`
use tinyvec::ArrayVec;

use crate::{devices::*, EnvParamFxP, LfoFreqFxP};
use crate::{DspFloat, DspFormat, DspFormatBase, DspType};
use crate::{IScalarFxP, SampleFxP, ScalarFxP, SignedNoteFxP};

mod types;
pub use types::*;

const MOD_SLOTS: usize = 4;

type ModMatrixRowEntries<T> = [(ModDest, <T as DspFormatBase>::IScalar); MOD_SLOTS];
type ModMatrixEntry<T> = (ModSrc, ModMatrixRowEntries<T>);

#[derive(Clone)]
pub struct ModMatrix<T: DspFormatBase> {
    pub rows: [ModMatrixEntry<T>; ModSrc::numel()],
}

impl<T: DspFormatBase> ModMatrix<T> {
    /// If there is an entry in this matrix from `src` to `dest`, return the
    /// modulation depth, else return None
    pub fn get_modulation(&self, src: ModSrc, dest: ModDest) -> Option<T::IScalar> {
        self.rows[src as usize]
            .1
            .iter()
            .find_map(|x| if x.0 == dest { Some(x.1) } else { None })
    }
}

impl<T: DspFloat> From<&ModMatrix<i16>> for ModMatrix<T> {
    fn from(value: &ModMatrix<i16>) -> Self {
        Self {
            rows: value.rows.map(|(src, dests)| {
                (src, dests.map(|(dest, depth)| {
                    (dest, depth.to_num())
                }))
            }),
        }
    }
}

impl From<&ModMatrix<i16>> for ModMatrix<i16> {
    fn from(value: &ModMatrix<i16>) -> Self {
        value.clone()
    }
}

#[derive(Clone, Default)]
pub struct ModMatrixInput<T: DspFormatBase> {
    pub velocity: T::Scalar,
    pub aftertouch: T::Scalar,
    pub modwheel: T::Scalar,
}

/// A parameter pack representing the different parameters to the [ModSectionFxP]
pub struct ModSectionParams<T: DspFormatBase + crate::devices::env::detail::EnvOps> {
    /// MIDI Velocity
    pub velocity: T::Scalar,
    /// MIDI Channel aftertouch
    pub aftertouch: T::Scalar,
    /// Modulation wheel (MIDI CC #1)
    pub modwheel: T::Scalar,
    /// Parameters for LFO 1
    pub lfo1_params: LfoParams<T>,
    /// Parameters for LFO 2
    pub lfo2_params: LfoParams<T>,
    /// Parameters for Envelope 1
    pub env1_params: EnvParams<T>,
    /// Parameters for Envelope 2
    pub env2_params: EnvParams<T>,
}

pub struct Modulator<'a, T: DspFormatBase> {
    input: ModMatrixInput<T>,
    lfo1: T::Sample,
    lfo2: T::Sample,
    env1: T::Scalar,
    env2: T::Scalar,
    matrix: &'a ModMatrix<T>,
}

/// The actual modulation section, containing the modulation LFOs and Envelopes and
/// logic to build the [ModulatorFxP].
#[derive(Clone, Default)]
pub struct ModSection<T: DspFormatBase + ModulatorOps + crate::devices::lfo::detail::LfoOps + crate::devices::env::detail::EnvOps> {
    lfo1: Lfo<T>,
    lfo2: Lfo<T>,
    env1: Env<T>,
    env2: Env<T>,
}

impl<T: DspFormat> ModSection<T> {
    /// Build a new modulation section, seeding the LFO RNGs (for S+H/S+G) from
    /// the seeds seed1 and seed2
    pub fn new_with_seeds(seed1: u64, seed2: u64) -> Self {
        Self {
            lfo1: Lfo::new(seed1),
            lfo2: Lfo::new(seed2),
            env1: Default::default(),
            env2: Default::default(),
        }
    }
    /// Build a [Modulator] from all the required data, to include the
    /// processing context, the gate signal, the [ModSectionParams], and
    /// the actual [ModMatrix].
    pub fn next<'a>(
        &mut self,
        context: &T::Context,
        gate: T::Sample,
        mut params: ModSectionParams<T>,
        entries: &'a ModMatrix<T>,
    ) -> Modulator<'a, T> {
        let lfo1_out = self.lfo1.next(context, gate, params.lfo1_params);
        let env1_out = self.env1.next(context, gate, params.env1_params);
        // LFO2/ENV2 are default here, so empty slices.
        let modulator = Modulator {
            input: ModMatrixInput {
                velocity: params.velocity,
                aftertouch: params.aftertouch,
                modwheel: params.modwheel,
            },
            lfo1: lfo1_out,
            lfo2: T::Sample::zero(),
            env1: env1_out,
            env2: T::Scalar::zero(),
            matrix: entries,
        };
        T::modulate_lfo_freq(&modulator, &mut params.lfo2_params.freq, ModDest::Lfo2Rate);
        T::modulate_scalar(
            &modulator,
            &mut params.lfo2_params.depth,
            ModDest::Lfo2Depth,
        );
        T::modulate_env_param(&modulator, &mut params.env2_params.attack, ModDest::Env2A);
        T::modulate_env_param(&modulator, &mut params.env2_params.decay, ModDest::Env2D);
        T::modulate_scalar(&modulator, &mut params.env2_params.sustain, ModDest::Env2S);
        T::modulate_env_param(&modulator, &mut params.env2_params.release, ModDest::Env2R);
        let lfo2_out = self.lfo2.next(context, gate, params.lfo2_params.into());
        let env2_out = self.env2.next(context, gate, params.env2_params.into());
        Modulator {
            lfo2: lfo2_out,
            env2: env2_out,
            ..modulator
        }
    }
}

pub(crate) mod detail {
    use super::*;
    pub trait ModulatorOps: DspFormatBase + crate::devices::env::detail::EnvOps
    {
        fn modulate_env(
            modulator: &Modulator<Self>,
            params: &mut EnvParams<Self>,
            dest: &EnvModDest,
        );
        fn modulate_osc(
            modulator: &Modulator<Self>,
            params: &mut MixOscParams<Self>,
            dest: &OscModDest,
        );
        fn modulate_ring(modulator: &Modulator<Self>, params: &mut RingModParams<Self>);
        fn modulate_filt(modulator: &Modulator<Self>, params: &mut ModFiltParams<Self>);
        fn modulate_env_param(
            modulator: &Modulator<Self>,
            param: &mut Self::EnvParam,
            dest: ModDest,
        );
        fn modulate_scalar(modulator: &Modulator<Self>, scalar: &mut Self::Scalar, dest: ModDest);
        fn modulate_lfo_freq(modulator: &Modulator<Self>, freq: &mut Self::LfoFreq, dest: ModDest);
    }
    /// Apply all modulation to the parameter passed in `dest`
    ///
    /// Returns true if any modulation was performed, or false otherwise
    pub fn modulate<T: crate::Fixed16>(
        modulator: &Modulator<i16>,
        dest: ModDest,
        value: &mut T,
    ) -> bool
    where
        T::Frac: fixed::types::extra::LeEqU32,
    {
        use crate::fixedmath::{I16F16, I17F15, I1F31};
        use fixed::FixedI32;
        let modulation = ModSrc::ELEM.map(|src| modulator.matrix.get_modulation(src, dest));
        // All the modulation sources that are not LFOs are ScalarFxPs
        let non_lfos = [
            (
                modulator.input.velocity,
                modulation[ModSrc::Velocity as usize],
            ),
            (
                modulator.input.aftertouch,
                modulation[ModSrc::Aftertouch as usize],
            ),
            (
                modulator.input.modwheel,
                modulation[ModSrc::ModWheel as usize],
            ),
            (modulator.env1, modulation[ModSrc::Env1 as usize]),
            (modulator.env2, modulation[ModSrc::Env2 as usize]),
        ];
        // Filter the above and collect them into an array-backed vec
        let non_lfos = non_lfos
            .iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(ScalarFxP, IScalarFxP); 5]>>();
        // The LFOs, however, are SampleFxPs, so these need to be separate
        let lfos = [
            (modulator.lfo1, modulation[ModSrc::Lfo1 as usize]),
            (modulator.lfo2, modulation[ModSrc::Lfo2 as usize]),
        ];
        let lfos = lfos
            .iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(SampleFxP, IScalarFxP); 2]>>();
        // In the common case, where there is no modulation, early exit
        if non_lfos.is_empty() && lfos.is_empty() {
            return false;
        }
        let non_lfos = non_lfos.into_iter();
        let lfos = lfos.into_iter();
        // All of the modulations for this sample, chain()ed together
        let non_lfos = non_lfos.map(|(src, amt)| src.wide_mul_signed(amt));
        let lfos = lfos.map(|(src, amt)| I1F31::saturating_from_num(src.wide_mul(amt)));
        let modulations = non_lfos.chain(lfos);
        // Add all the modulations.  We'll do some bit twiddling so 100% modulation will
        // correspond to the maximum value of the type, and do all our math in 32 bit signed
        // arithmetic so we can model multiple modulations canceling each other out then
        // check for saturation at the end
        *value = T::saturating_from_num(
            modulations
                .map(|x| {
                    FixedI32::<T::Frac>::from_bits(if T::IS_SIGNED {
                        I17F15::from_num(x).to_bits()
                    } else {
                        I16F16::from_num(x).to_bits()
                    })
                })
                .fold(FixedI32::<T::Frac>::from_num(*value), |acc, val| acc + val),
        );
        true
    }
    pub fn coeff_from_fixed<T: crate::Fixed16, U: DspFloat>() -> U {
        let num_bits = if T::IS_SIGNED { 15 } else { 16 } - T::FRAC_NBITS as i32;
        if num_bits == -1 {
            U::ONE_HALF
        } else {
            U::from_u16(1u16 << num_bits)
        }
    }
    /// Apply all modulation to the parameter passed in `dest`
    ///
    /// Returns true if any modulation was performed, or false otherwise
    pub fn modulate_float<T: DspFloat>(
        modulator: &Modulator<T>,
        dest: ModDest,
        value: &mut T,
        coeff: T,
    ) -> bool {
        let modulation = ModSrc::ELEM.map(|src| modulator.matrix.get_modulation(src, dest));
        let mod_params = [
            (
                modulator.input.velocity,
                modulation[ModSrc::Velocity as usize],
            ),
            (
                modulator.input.aftertouch,
                modulation[ModSrc::Aftertouch as usize],
            ),
            (
                modulator.input.modwheel,
                modulation[ModSrc::ModWheel as usize],
            ),
            (modulator.env1, modulation[ModSrc::Env1 as usize]),
            (modulator.env2, modulation[ModSrc::Env2 as usize]),
            (modulator.lfo1, modulation[ModSrc::Lfo1 as usize]),
            (modulator.lfo2, modulation[ModSrc::Lfo2 as usize]),
        ];
        // Filter the above and collect them into an array-backed vec
        let mod_params = mod_params
            .iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(T, T); 7]>>();
        // In the common case, where there is no modulation, early exit
        if mod_params.is_empty() {
            return false;
        }
        // All of the modulations for this sample
        let mod_params = mod_params.into_iter().map(|(val, modamt)| val * modamt);
        *value = *value + (coeff * mod_params.fold(T::zero(), |acc, val| acc + val));
        true
    }
}

impl detail::ModulatorOps for i16 {
    /// Modulate all of the parameters in `params` for the envelope specified by
    /// `dest`, which should be either [ENV_AMP_MOD_DEST] or [ENV_FILT_MOD_DEST]
    fn modulate_env(m: &Modulator<i16>, params: &mut EnvParams<i16>, dest: &EnvModDest) {
        detail::modulate(m, dest.attack, &mut params.attack);
        detail::modulate(m, dest.decay, &mut params.decay);
        detail::modulate(m, dest.sustain, &mut params.sustain);
        detail::modulate(m, dest.release, &mut params.release);
    }
    /// Modulate all of the parameters in `params` for the oscillator specified by
    /// `dest`, which should be either [OSC1_MOD_DEST] or [OSC2_MOD_DEST]
    fn modulate_osc(m: &Modulator<i16>, params: &mut MixOscParams<i16>, dest: &OscModDest) {
        // We have 6 bits of total range (7 - 1 sign bit) in SignedNoteFxP
        // The range of course tune is -32 to +32, or 5 bits + sign, so will need >>= 1
        // The range of fine tune is -2 to +2, or 1 bit + sign, so will need >>= 5
        // If we do fine first and >>= 4, then apply course and >>= 1, that will be equiv.
        let mut osc_mod_applied = false;
        let mut tune_mod = SignedNoteFxP::ZERO;
        if detail::modulate(m, dest.fine, &mut tune_mod) {
            osc_mod_applied = true;
            tune_mod >>= 4;
        }
        osc_mod_applied |= detail::modulate(m, dest.course, &mut tune_mod);
        // Apply the modulation ourselves now
        if osc_mod_applied {
            params.tune = params.tune.saturating_add(tune_mod.unwrapped_shr(1));
        }
        detail::modulate(m, dest.shape, &mut params.sin);
        detail::modulate(m, dest.sin, &mut params.sin);
        detail::modulate(m, dest.sq, &mut params.sq);
        detail::modulate(m, dest.tri, &mut params.tri);
        detail::modulate(m, dest.saw, &mut params.saw);
    }
    /// Modulate the ring modulator parameters
    fn modulate_ring(m: &Modulator<i16>, params: &mut RingModParams<i16>) {
        detail::modulate(m, ModDest::RingOsc1, &mut params.mix_a);
        detail::modulate(m, ModDest::RingOsc2, &mut params.mix_b);
        detail::modulate(m, ModDest::RingMod, &mut params.mix_mod);
    }
    /// Modulate the filter parameters
    fn modulate_filt(m: &Modulator<i16>, params: &mut ModFiltParams<i16>) {
        detail::modulate(m, ModDest::FiltEnv, &mut params.env_mod);
        detail::modulate(m, ModDest::FiltVel, &mut params.vel_mod);
        detail::modulate(m, ModDest::FiltKbd, &mut params.kbd_tracking);
        detail::modulate(m, ModDest::FiltCutoff, &mut params.cutoff);
        detail::modulate(m, ModDest::FiltRes, &mut params.resonance);
        detail::modulate(m, ModDest::FiltLow, &mut params.low_mix);
        detail::modulate(m, ModDest::FiltBand, &mut params.band_mix);
        detail::modulate(m, ModDest::FiltHigh, &mut params.high_mix);
    }
    fn modulate_env_param(m: &Modulator<i16>, param: &mut EnvParamFxP, dest: ModDest) {
        detail::modulate(m, dest, param);
    }
    fn modulate_lfo_freq(m: &Modulator<i16>, freq: &mut LfoFreqFxP, dest: ModDest) {
        detail::modulate(m, dest, freq);
    }
    fn modulate_scalar(m: &Modulator<i16>, scalar: &mut ScalarFxP, dest: ModDest) {
        detail::modulate(m, dest, scalar);
    }
}

impl<T: DspFloat> detail::ModulatorOps for T {
    /// Modulate all of the parameters in `params` for the envelope specified by
    /// `dest`, which should be either [ENV_AMP_MOD_DEST] or [ENV_FILT_MOD_DEST]
    fn modulate_env(m: &Modulator<T>, params: &mut EnvParams<T>, dest: &EnvModDest) {
        let coeff = detail::coeff_from_fixed::<EnvParamFxP, T>();
        detail::modulate_float(m, dest.attack, &mut params.attack, coeff);
        detail::modulate_float(m, dest.decay, &mut params.decay, coeff);
        detail::modulate_float(m, dest.sustain, &mut params.sustain, coeff);
        detail::modulate_float(m, dest.release, &mut params.release, coeff);
    }
    /// Modulate all of the parameters in `params` for the oscillator specified by
    /// `dest`, which should be either [OSC1_MOD_DEST] or [OSC2_MOD_DEST]
    fn modulate_osc(m: &Modulator<T>, params: &mut MixOscParams<T>, dest: &OscModDest) {
        let coeff = detail::coeff_from_fixed::<ScalarFxP, T>();
        detail::modulate_float(m, dest.fine, &mut params.tune, T::TWO);
        detail::modulate_float(m, dest.course, &mut params.tune, T::from_u16(32));
        detail::modulate_float(m, dest.shape, &mut params.sin, coeff);
        detail::modulate_float(m, dest.sin, &mut params.sin, coeff);
        detail::modulate_float(m, dest.sq, &mut params.sq, coeff);
        detail::modulate_float(m, dest.tri, &mut params.tri, coeff);
        detail::modulate_float(m, dest.saw, &mut params.saw, coeff);
    }
    /// Modulate the ring modulator parameters
    fn modulate_ring(m: &Modulator<T>, params: &mut RingModParams<T>) {
        let coeff = detail::coeff_from_fixed::<ScalarFxP, T>();
        detail::modulate_float(m, ModDest::RingOsc1, &mut params.mix_a, coeff);
        detail::modulate_float(m, ModDest::RingOsc2, &mut params.mix_b, coeff);
        detail::modulate_float(m, ModDest::RingMod, &mut params.mix_mod, coeff);
    }
    /// Modulate the filter parameters
    fn modulate_filt(m: &Modulator<T>, params: &mut ModFiltParams<T>) {
        let coeff = detail::coeff_from_fixed::<ScalarFxP, T>();
        let filt_coeff = detail::coeff_from_fixed::<crate::NoteFxP, T>();
        detail::modulate_float(m, ModDest::FiltEnv, &mut params.env_mod, coeff);
        detail::modulate_float(m, ModDest::FiltVel, &mut params.vel_mod, coeff);
        detail::modulate_float(m, ModDest::FiltKbd, &mut params.kbd_tracking, coeff);
        detail::modulate_float(m, ModDest::FiltCutoff, &mut params.cutoff, filt_coeff);
        detail::modulate_float(m, ModDest::FiltRes, &mut params.resonance, coeff);
        detail::modulate_float(m, ModDest::FiltLow, &mut params.low_mix, coeff);
        detail::modulate_float(m, ModDest::FiltBand, &mut params.band_mix, coeff);
        detail::modulate_float(m, ModDest::FiltHigh, &mut params.high_mix, coeff);
    }
    fn modulate_env_param(m: &Modulator<T>, param: &mut T, dest: ModDest) {
        let coeff = detail::coeff_from_fixed::<EnvParamFxP, T>();
        detail::modulate_float(m, dest, param, coeff);
    }
    fn modulate_lfo_freq(m: &Modulator<T>, freq: &mut T, dest: ModDest) {
        let coeff = detail::coeff_from_fixed::<LfoFreqFxP, T>();
        detail::modulate_float(m, dest, freq, coeff);
    }
    fn modulate_scalar(m: &Modulator<T>, scalar: &mut T, dest: ModDest) {
        let coeff = detail::coeff_from_fixed::<ScalarFxP, T>();
        detail::modulate_float(m, dest, scalar, coeff);
    }
}

use detail::ModulatorOps;
