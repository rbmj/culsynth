//! This module contains data to allow modulation of a `Voice`
use tinyvec::ArrayVec;

use crate::{devices::*, EnvParamFxP, LfoFreqFxP};
use crate::{DspFloat, DspFormat, DspFormatBase, DspType};
use crate::{ScalarFxP, SignedNoteFxP};

mod types;
pub use types::*;

/// The number of modulation slots per [ModSrc] in a [ModMatrix]
pub const MOD_SLOTS: usize = 4;

type ModMatrixRowEntries<T> = [(ModDest, <T as DspFormatBase>::IScalar); MOD_SLOTS];
type ModMatrixEntry<T> = (ModSrc, ModMatrixRowEntries<T>);

/// A Modulation Matrix
///
/// It contains a series of rows, one for each [ModSrc].  Each row is a tuple
/// of `(ModSrc, [(ModDest, IScalar); MOD_SLOTS])` - that is, the first item
/// is the modulation source, and the second is an array of [MOD_SLOTS] tuples,
/// each containing the modulation destination ([ModDest]) and modulation depth
/// as an `IScalar` (see [DspFormat]).
///
/// The implementation of `Default` creates a ModMatrix with rows initialized
/// for each [ModSrc] and each slot mapped to [ModDest::Null] with a depth of 0.
#[derive(Clone)]
pub struct ModMatrix<T: DspFormatBase> {
    /// The rows making up the modmatrix
    pub rows: [ModMatrixEntry<T>; ModSrc::numel()],
}

impl<T: DspFormatBase> Default for ModMatrix<T> {
    fn default() -> Self {
        Self {
            rows: ModSrc::ELEM.map(|src| (src, [(ModDest::Null, Default::default()); MOD_SLOTS])),
        }
    }
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
            rows: value
                .rows
                .map(|(src, dests)| (src, dests.map(|(dest, depth)| (dest, depth.to_num())))),
        }
    }
}

impl From<&ModMatrix<i16>> for ModMatrix<i16> {
    fn from(value: &ModMatrix<i16>) -> Self {
        value.clone()
    }
}

#[derive(Clone)]
/// A parameter pack representing the different parameters to the [ModSection]
pub struct ModSectionParams<T: DspFormatBase> {
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

#[derive(Clone)]
struct ModMatrixExpanded<T: DspFormatBase> {
    rows: [ArrayVec<[(ModSrc, T::IScalar); ModSrc::numel()]>; ModDest::numel()],
}

impl<T: DspFormatBase> From<&ModMatrix<T>> for ModMatrixExpanded<T> {
    fn from(value: &ModMatrix<T>) -> Self {
        let mut ret = Self::default();
        for (src, entries) in value.rows {
            for (dest, depth) in entries {
                if dest == ModDest::Null {
                    continue;
                }
                // FIXME: is it worth making these bigger for pathological
                // code instead of silently failing?  Or should there be some
                // form of validation at construction time?
                let _ = ret.rows[dest as usize].try_push((src, depth));
            }
        }
        ret
    }
}

impl<T: DspFormatBase> Default for ModMatrixExpanded<T> {
    fn default() -> Self {
        Self {
            rows: [ArrayVec::default(); ModDest::numel()],
        }
    }
}

/// A struct containing all of the necessary information to modulate parameters
pub struct Modulator<'a, T: DspFormatBase> {
    velocity: T::Scalar,
    aftertouch: T::Scalar,
    modwheel: T::Scalar,
    env1: T::Scalar,
    env2: T::Scalar,
    lfo1: T::Sample,
    lfo2: T::Sample,
    matrix: &'a ModMatrixExpanded<T>,
}

impl<'a, T: DspFormatBase + ModulatorOps> Modulator<'a, T> {
    /// Apply modulation to [EnvParams] for the provided [EnvModDest]
    /// (e.g. [ENV_FILT_MOD_DEST])
    pub fn modulate_env(&self, params: &mut EnvParams<T>, dest: &EnvModDest) {
        T::modulate_env(self, params, dest)
    }
    /// Apply modulation to [MixOscParams] for the provided [OscModDest]
    /// (e.g. [OSC1_MOD_DEST])
    pub fn modulate_mix_osc(&self, params: &mut MixOscParams<T>, dest: &OscModDest) {
        T::modulate_osc(self, params, dest)
    }
    /// Apply modulation to the [RingModParams]
    pub fn modulate_ring(&self, params: &mut RingModParams<T>) {
        T::modulate_ring(self, params)
    }
    /// Apply modulation to the [ModFiltParams]
    pub fn modulate_mod_filt(&self, params: &mut ModFiltParams<T>) {
        T::modulate_filt(self, params)
    }
    /// Apply modulation to a singular `EnvParam` for a given [ModDest]
    pub fn modulate_env_param(&self, param: &mut T::EnvParam, dest: ModDest) {
        T::modulate_env_param(self, param, dest)
    }
    /// Apply modulation to a singular `Scalar` for a given [ModDest]
    pub fn modulate_scalar(&self, param: &mut T::Scalar, dest: ModDest) {
        T::modulate_scalar(self, param, dest)
    }
    /// Apply modulation to a singular `LfoFreq` for a given [ModDest]
    pub fn modulate_lfo_freq(&self, param: &mut T::LfoFreq, dest: ModDest) {
        T::modulate_lfo_freq(self, param, dest)
    }
}

/// The actual modulation section, containing the modulation LFOs and Envelopes and
/// logic to build the [ModulatorFxP].
#[derive(Clone, Default)]
pub struct ModSection<
    T: DspFormatBase
        + ModulatorOps
        + crate::devices::lfo::detail::LfoOps
        + crate::devices::env::detail::EnvOps,
> {
    lfo1: Lfo<T>,
    lfo2: Lfo<T>,
    env1: Env<T>,
    env2: Env<T>,
    expanded_matrix: ModMatrixExpanded<T>,
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
            expanded_matrix: Default::default(),
        }
    }
    /// Build a [Modulator] from all the required data, to include the
    /// processing context, the gate signal, the [ModSectionParams], and
    /// the actual [ModMatrix].
    pub fn next<'a>(
        &'a mut self,
        context: &T::Context,
        gate: bool,
        mut params: ModSectionParams<T>,
        entries: Option<&ModMatrix<T>>,
    ) -> Modulator<'a, T> {
        let lfo1_out = self.lfo1.next(context, gate, params.lfo1_params);
        let env1_out = self.env1.next(context, gate, params.env1_params);
        if let Some(matrix) = entries {
            self.expanded_matrix = matrix.into();
        }
        // LFO2/ENV2 are default here, so empty slices.
        let modulator = Modulator {
            velocity: params.velocity,
            aftertouch: params.aftertouch,
            modwheel: params.modwheel,
            lfo1: lfo1_out,
            lfo2: T::Sample::zero(),
            env1: env1_out,
            env2: T::Scalar::zero(),
            matrix: &self.expanded_matrix,
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
        let lfo2_out = self.lfo2.next(context, gate, params.lfo2_params);
        let env2_out = self.env2.next(context, gate, params.env2_params);
        Modulator {
            lfo2: lfo2_out,
            env2: env2_out,
            ..modulator
        }
    }
}

pub(crate) mod detail {
    use super::*;
    pub trait ModulatorOps: DspFormatBase {
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
    pub fn modulate<T: crate::Fixed16>(modulator: &Modulator<i16>, dest: ModDest, value: T) -> T {
        use crate::fixedmath::{I16F16, I17F15, I1F31};
        let mut acc = value.widen();
        for (src, depth) in modulator.matrix.rows[dest as usize].iter().copied() {
            let mod_amt = match src {
                ModSrc::Velocity => modulator.velocity.wide_mul_signed(depth),
                ModSrc::Aftertouch => modulator.aftertouch.wide_mul_signed(depth),
                ModSrc::ModWheel => modulator.modwheel.wide_mul_signed(depth),
                ModSrc::Env1 => modulator.env1.wide_mul_signed(depth),
                ModSrc::Env2 => modulator.env2.wide_mul_signed(depth),
                ModSrc::Lfo1 => I1F31::saturating_from_num(modulator.lfo1.wide_mul(depth)),
                ModSrc::Lfo2 => I1F31::saturating_from_num(modulator.lfo2.wide_mul(depth)),
            };
            acc += T::widened_from_bits(if T::IS_SIGNED {
                I17F15::from_num(mod_amt).to_bits()
            } else {
                I16F16::from_num(mod_amt).to_bits()
            });
        }
        T::saturating_from_num(acc)
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
        value: T,
        coeff: T,
    ) -> T {
        let mut acc = T::ZERO;
        for (src, depth) in modulator.matrix.rows[dest as usize].iter().copied() {
            acc = acc
                + (depth
                    * match src {
                        ModSrc::Velocity => modulator.velocity,
                        ModSrc::Aftertouch => modulator.aftertouch,
                        ModSrc::ModWheel => modulator.modwheel,
                        ModSrc::Env1 => modulator.env1,
                        ModSrc::Env2 => modulator.env2,
                        ModSrc::Lfo1 => modulator.lfo1,
                        ModSrc::Lfo2 => modulator.lfo2,
                    });
        }
        acc = value + (acc * coeff);
        if acc > coeff {
            acc = coeff;
        } else if value < -coeff {
            acc = -coeff;
        }
        acc
    }
}

impl detail::ModulatorOps for i16 {
    /// Modulate all of the parameters in `params` for the envelope specified by
    /// `dest`, which should be either [ENV_AMP_MOD_DEST] or [ENV_FILT_MOD_DEST]
    fn modulate_env(m: &Modulator<i16>, params: &mut EnvParams<i16>, dest: &EnvModDest) {
        params.attack = detail::modulate(m, dest.attack, params.attack);
        params.decay = detail::modulate(m, dest.decay, params.decay);
        params.sustain = detail::modulate(m, dest.sustain, params.sustain);
        params.release = detail::modulate(m, dest.release, params.release);
    }
    /// Modulate all of the parameters in `params` for the oscillator specified by
    /// `dest`, which should be either [OSC1_MOD_DEST] or [OSC2_MOD_DEST]
    fn modulate_osc(m: &Modulator<i16>, params: &mut MixOscParams<i16>, dest: &OscModDest) {
        // We have 6 bits of total range (7 - 1 sign bit) in SignedNoteFxP
        // The range of course tune is -32 to +32, or 5 bits + sign, so will need >>= 1
        // The range of fine tune is -2 to +2, or 1 bit + sign, so will need >>= 5
        // If we do fine first and >>= 4, then apply course and >>= 1, that will be equiv.
        let mut tune_mod = SignedNoteFxP::ZERO;
        tune_mod = detail::modulate(m, dest.fine, tune_mod);
        tune_mod >>= 4;
        tune_mod = detail::modulate(m, dest.course, tune_mod);
        params.tune = params.tune.saturating_add(tune_mod >> 1);
        params.shape = detail::modulate(m, dest.shape, params.shape);
        params.sin = detail::modulate(m, dest.sin, params.sin);
        params.sq = detail::modulate(m, dest.sq, params.sq);
        params.tri = detail::modulate(m, dest.tri, params.tri);
        params.saw = detail::modulate(m, dest.saw, params.saw);
    }
    /// Modulate the ring modulator parameters
    fn modulate_ring(m: &Modulator<i16>, params: &mut RingModParams<i16>) {
        params.mix_a = detail::modulate(m, ModDest::RingOsc1, params.mix_a);
        params.mix_b = detail::modulate(m, ModDest::RingOsc2, params.mix_b);
        params.mix_mod = detail::modulate(m, ModDest::RingMod, params.mix_mod);
    }
    /// Modulate the filter parameters
    fn modulate_filt(m: &Modulator<i16>, params: &mut ModFiltParams<i16>) {
        params.env_mod = detail::modulate(m, ModDest::FiltEnv, params.env_mod);
        params.vel_mod = detail::modulate(m, ModDest::FiltVel, params.vel_mod);
        params.kbd_tracking = detail::modulate(m, ModDest::FiltKbd, params.kbd_tracking);
        params.cutoff = detail::modulate(m, ModDest::FiltCutoff, params.cutoff);
        params.resonance = detail::modulate(m, ModDest::FiltRes, params.resonance);
        params.low_mix = detail::modulate(m, ModDest::FiltLow, params.low_mix);
        params.band_mix = detail::modulate(m, ModDest::FiltBand, params.band_mix);
        params.high_mix = detail::modulate(m, ModDest::FiltHigh, params.high_mix);
    }
    fn modulate_env_param(m: &Modulator<i16>, param: &mut EnvParamFxP, dest: ModDest) {
        *param = detail::modulate(m, dest, *param);
    }
    fn modulate_lfo_freq(m: &Modulator<i16>, freq: &mut LfoFreqFxP, dest: ModDest) {
        *freq = detail::modulate(m, dest, *freq);
    }
    fn modulate_scalar(m: &Modulator<i16>, scalar: &mut ScalarFxP, dest: ModDest) {
        *scalar = detail::modulate(m, dest, *scalar);
    }
}

impl<T: DspFloat> detail::ModulatorOps for T {
    /// Modulate all of the parameters in `params` for the envelope specified by
    /// `dest`, which should be either [ENV_AMP_MOD_DEST] or [ENV_FILT_MOD_DEST]
    fn modulate_env(m: &Modulator<T>, params: &mut EnvParams<T>, dest: &EnvModDest) {
        let coeff = detail::coeff_from_fixed::<EnvParamFxP, T>();
        params.attack = detail::modulate_float(m, dest.attack, params.attack, coeff);
        params.decay = detail::modulate_float(m, dest.decay, params.decay, coeff);
        params.sustain = detail::modulate_float(m, dest.sustain, params.sustain, coeff);
        params.release = detail::modulate_float(m, dest.release, params.release, coeff);
    }
    /// Modulate all of the parameters in `params` for the oscillator specified by
    /// `dest`, which should be either [OSC1_MOD_DEST] or [OSC2_MOD_DEST]
    fn modulate_osc(m: &Modulator<T>, params: &mut MixOscParams<T>, dest: &OscModDest) {
        let coeff = detail::coeff_from_fixed::<ScalarFxP, T>();
        params.tune = detail::modulate_float(m, dest.fine, params.tune, T::TWO);
        params.tune = detail::modulate_float(m, dest.course, params.tune, T::from_u16(32));
        params.shape = detail::modulate_float(m, dest.shape, params.shape, coeff);
        params.sin = detail::modulate_float(m, dest.sin, params.sin, coeff);
        params.sq = detail::modulate_float(m, dest.sq, params.sq, coeff);
        params.tri = detail::modulate_float(m, dest.tri, params.tri, coeff);
        params.saw = detail::modulate_float(m, dest.saw, params.saw, coeff);
    }
    /// Modulate the ring modulator parameters
    fn modulate_ring(m: &Modulator<T>, params: &mut RingModParams<T>) {
        let coeff = detail::coeff_from_fixed::<ScalarFxP, T>();
        params.mix_a = detail::modulate_float(m, ModDest::RingOsc1, params.mix_a, coeff);
        params.mix_b = detail::modulate_float(m, ModDest::RingOsc2, params.mix_b, coeff);
        params.mix_mod = detail::modulate_float(m, ModDest::RingMod, params.mix_mod, coeff);
    }
    /// Modulate the filter parameters
    fn modulate_filt(m: &Modulator<T>, params: &mut ModFiltParams<T>) {
        let coeff = detail::coeff_from_fixed::<ScalarFxP, T>();
        let filt_coeff = detail::coeff_from_fixed::<crate::NoteFxP, T>();
        params.env_mod = detail::modulate_float(m, ModDest::FiltEnv, params.env_mod, coeff);
        params.vel_mod = detail::modulate_float(m, ModDest::FiltVel, params.vel_mod, coeff);
        params.kbd_tracking =
            detail::modulate_float(m, ModDest::FiltKbd, params.kbd_tracking, coeff);
        params.cutoff = detail::modulate_float(m, ModDest::FiltCutoff, params.cutoff, filt_coeff);
        params.resonance = detail::modulate_float(m, ModDest::FiltRes, params.resonance, coeff);
        params.low_mix = detail::modulate_float(m, ModDest::FiltLow, params.low_mix, coeff);
        params.band_mix = detail::modulate_float(m, ModDest::FiltBand, params.band_mix, coeff);
        params.high_mix = detail::modulate_float(m, ModDest::FiltHigh, params.high_mix, coeff);
    }
    fn modulate_env_param(m: &Modulator<T>, param: &mut T, dest: ModDest) {
        let coeff = detail::coeff_from_fixed::<EnvParamFxP, T>();
        *param = detail::modulate_float(m, dest, *param, coeff);
    }
    fn modulate_lfo_freq(m: &Modulator<T>, freq: &mut T, dest: ModDest) {
        let coeff = detail::coeff_from_fixed::<LfoFreqFxP, T>();
        *freq = detail::modulate_float(m, dest, *freq, coeff);
    }
    fn modulate_scalar(m: &Modulator<T>, scalar: &mut T, dest: ModDest) {
        let coeff = detail::coeff_from_fixed::<ScalarFxP, T>();
        *scalar = detail::modulate_float(m, dest, *scalar, coeff);
    }
}

use detail::ModulatorOps;
