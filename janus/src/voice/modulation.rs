//! This module contains data to allow modulation of a `Voice`
use tinyvec::ArrayVec;

use crate::context::{Context, ContextFxP};
use crate::devices::*;
use crate::{min_size, STATIC_BUFFER_SIZE};
use crate::{IScalarFxP, SampleFxP, ScalarFxP, SignedNoteFxP};

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum ModSrc {
    Velocity,
    Aftertouch,
    ModWheel,
    Env1,
    Env2,
    Lfo1,
    Lfo2,
}

impl ModSrc {
    pub const ELEM: [ModSrc; Self::numel()] = [
        ModSrc::Velocity,
        ModSrc::Aftertouch,
        ModSrc::ModWheel,
        ModSrc::Env1,
        ModSrc::Env2,
        ModSrc::Lfo1,
        ModSrc::Lfo2,
    ];
    pub const fn elements() -> &'static [ModSrc] {
        &Self::ELEM
    }
    pub const fn min() -> Self {
        Self::Velocity
    }
    pub const fn max() -> Self {
        Self::Lfo2
    }
    pub const fn numel() -> usize {
        1 + Self::max() as usize - Self::min() as usize
    }
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Velocity => "Velocity",
            Self::Aftertouch => "Aftertouch",
            Self::ModWheel => "Mod Wheel",
            Self::Env1 => "Envelope 1",
            Self::Env2 => "Envelope 2",
            Self::Lfo1 => "LFO 1",
            Self::Lfo2 => "LFO 2",
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ModDest {
    #[default]
    Null,
    Osc1Course,
    Osc1Fine,
    Osc1Shape,
    Osc1Sin,
    Osc1Sq,
    Osc1Tri,
    Osc1Saw,
    Osc2Course,
    Osc2Fine,
    Osc2Shape,
    Osc2Sin,
    Osc2Sq,
    Osc2Tri,
    Osc2Saw,
    RingOsc1,
    RingOsc2,
    RingMod,
    FiltCutoff,
    FiltRes,
    FiltEnv,
    FiltKbd,
    FiltVel,
    FiltLow,
    FiltBand,
    FiltHigh,
    EnvFiltA,
    EnvFiltD,
    EnvFiltS,
    EnvFiltR,
    EnvAmpA,
    EnvAmpD,
    EnvAmpS,
    EnvAmpR,

    Lfo2Rate,
    Lfo2Depth,
    Env2A,
    Env2D,
    Env2S,
    Env2R,
}

impl ModDest {
    /// Env2/Lfo2 may not modulate themselves/each other, so call this function
    /// when evaluating their modulation matrices to remap these invalid routes
    /// to `Self::Null`
    pub const fn remove_secondary_invalid_dest(self) -> Self {
        match self {
            Self::Lfo2Rate => Self::Null,
            Self::Lfo2Depth => Self::Null,
            Self::Env2A => Self::Null,
            Self::Env2D => Self::Null,
            Self::Env2S => Self::Null,
            Self::Env2R => Self::Null,
            val => val,
        }
    }
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Null => "NONE",
            Self::Osc1Course => "Osc1Course",
            Self::Osc1Fine => "Osc1Fine",
            Self::Osc1Shape => "Osc1Shape",
            Self::Osc1Sin => "Osc1Sin",
            Self::Osc1Sq => "Osc1Sq",
            Self::Osc1Tri => "Osc1Tri",
            Self::Osc1Saw => "Osc1Saw",
            Self::Osc2Course => "Osc2Course",
            Self::Osc2Fine => "Osc2Fine",
            Self::Osc2Shape => "Osc2Shape",
            Self::Osc2Sin => "Osc2Sin",
            Self::Osc2Sq => "Osc2Sq",
            Self::Osc2Tri => "Osc2Tri",
            Self::Osc2Saw => "Osc2Saw",
            Self::RingOsc1 => "RingOsc1",
            Self::RingOsc2 => "RingOsc2",
            Self::RingMod => "RingMod",
            Self::FiltCutoff => "FiltCutoff",
            Self::FiltRes => "FiltRes",
            Self::FiltEnv => "FiltEnv",
            Self::FiltKbd => "FiltKbd",
            Self::FiltVel => "FiltVel",
            Self::FiltLow => "FiltLow",
            Self::FiltBand => "FiltBand",
            Self::FiltHigh => "FiltHigh",
            Self::EnvFiltA => "EnvFiltA",
            Self::EnvFiltD => "EnvFiltD",
            Self::EnvFiltS => "EnvFiltS",
            Self::EnvFiltR => "EnvFiltR",
            Self::EnvAmpA => "EnvAmpA",
            Self::EnvAmpD => "EnvAmpD",
            Self::EnvAmpS => "EnvAmpS",
            Self::EnvAmpR => "EnvAmpR",
            Self::Lfo2Rate => "Lfo2Rate",
            Self::Lfo2Depth => "Lfo2Depth",
            Self::Env2A => "Env2A",
            Self::Env2D => "Env2D",
            Self::Env2S => "Env2S",
            Self::Env2R => "Env2R",
        }
    }
    pub const fn min() -> Self {
        Self::Null
    }
    pub const fn max() -> Self {
        Self::Env2R
    }
    pub const fn max_secondary() -> Self {
        Self::EnvAmpR
    }
    pub fn elements() -> impl core::iter::Iterator<Item = ModDest> {
        Self::elements_secondary_if(false)
    }
    pub fn elements_secondary() -> impl core::iter::Iterator<Item = ModDest> {
        Self::elements_secondary_if(true)
    }
    pub fn elements_secondary_if(sec: bool) -> impl core::iter::Iterator<Item = ModDest> {
        let max = if sec {
            Self::max_secondary()
        } else {
            Self::max()
        };
        ((Self::min() as u16)..=(max as u16)).map(|x| unsafe { core::mem::transmute(x) })
    }
}

impl TryFrom<u16> for ModDest {
    type Error = &'static str;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value >= Self::min() as u16 && value <= Self::max() as u16 {
            unsafe { Ok(core::mem::transmute(value)) }
        } else {
            Err("ModDest out of bounds")
        }
    }
}

impl TryFrom<&str> for ModDest {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::elements()
            .find(|elem| value == elem.to_str())
            .ok_or("ModDest::try_from::<&str> parse failure")
    }
}

pub struct OscModDest {
    course: ModDest,
    fine: ModDest,
    shape: ModDest,
    sin: ModDest,
    sq: ModDest,
    tri: ModDest,
    saw: ModDest,
}

pub const OSC1_MOD_DEST: OscModDest = OscModDest {
    course: ModDest::Osc1Course,
    fine: ModDest::Osc1Fine,
    shape: ModDest::Osc1Shape,
    sin: ModDest::Osc1Sin,
    sq: ModDest::Osc1Sq,
    tri: ModDest::Osc1Tri,
    saw: ModDest::Osc1Saw,
};

pub const OSC2_MOD_DEST: OscModDest = OscModDest {
    course: ModDest::Osc2Course,
    fine: ModDest::Osc2Fine,
    shape: ModDest::Osc2Shape,
    sin: ModDest::Osc2Sin,
    sq: ModDest::Osc2Sq,
    tri: ModDest::Osc2Tri,
    saw: ModDest::Osc2Saw,
};

pub struct EnvModDest {
    attack: ModDest,
    decay: ModDest,
    sustain: ModDest,
    release: ModDest,
}

pub const ENV_AMP_MOD_DEST: EnvModDest = EnvModDest {
    attack: ModDest::EnvAmpA,
    decay: ModDest::EnvAmpD,
    sustain: ModDest::EnvAmpS,
    release: ModDest::EnvAmpR,
};

pub const ENV_FILT_MOD_DEST: EnvModDest = EnvModDest {
    attack: ModDest::EnvFiltA,
    decay: ModDest::EnvFiltD,
    sustain: ModDest::EnvFiltS,
    release: ModDest::EnvFiltR,
};

const MOD_SLOTS: usize = 4;
type ModMatrixRowEntriesFxP = [(ModDest, IScalarFxP); MOD_SLOTS];
type ModMatrixRowEntries<Smp> = [(ModDest, Smp); MOD_SLOTS];
type ModMatrixEntryFxP = (ModSrc, ModMatrixRowEntriesFxP);
type ModMatrixRow<Smp> = (ModSrc, ModMatrixRowEntries<Smp>);

pub struct ModulatorFxP<'a> {
    velocity: &'a [ScalarFxP],
    aftertouch: &'a [ScalarFxP],
    modwheel: &'a [ScalarFxP],
    lfo1: &'a [SampleFxP],
    lfo2: &'a [SampleFxP],
    env1: &'a [ScalarFxP],
    env2: &'a [ScalarFxP],
    matrix: &'a ModMatrixFxP,
}

impl<'a> ModulatorFxP<'a> {
    pub fn len(&self) -> usize {
        min_size(&[
            self.velocity.len(),
            self.aftertouch.len(),
            self.modwheel.len(),
            self.lfo1.len(),
            self.lfo2.len(),
            self.env1.len(),
            self.env2.len(),
        ])
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Apply all modulation to the parameter passed in `dest` in place using `mut buf`
    ///
    /// Returns true if any modulation was performed, or false otherwise
    pub fn modulate<T: crate::Fixed16>(&self, dest: ModDest, buf: &mut [T]) -> bool
    where
        T::Frac: fixed::types::extra::LeEqU32,
    {
        use crate::fixedmath::I1F31;
        use fixed::FixedI32;
        let modulation = ModSrc::ELEM.map(|src| self.matrix.get_modulation(src, dest));
        // All the modulation sources that are not LFOs are ScalarFxPs
        let non_lfos = [
            (self.velocity, modulation[ModSrc::Velocity as usize]),
            (self.aftertouch, modulation[ModSrc::Aftertouch as usize]),
            (self.modwheel, modulation[ModSrc::ModWheel as usize]),
            (self.env1, modulation[ModSrc::Env1 as usize]),
            (self.env2, modulation[ModSrc::Env2 as usize]),
        ];
        // Filter the above and collect them into an array-backed vec
        let non_lfos_filt = non_lfos
            .iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(&[ScalarFxP], IScalarFxP); 5]>>();
        // The LFOs, however, are SampleFxPs, so these need to be separate
        let lfos = [
            (self.lfo1, modulation[ModSrc::Lfo1 as usize]),
            (self.lfo2, modulation[ModSrc::Lfo2 as usize]),
        ];
        let lfos_filt = lfos
            .iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(&[SampleFxP], IScalarFxP); 2]>>();
        // In the common case, where there is no modulation, early exit
        if non_lfos_filt.is_empty() && lfos_filt.is_empty() {
            return false;
        }
        for i in 0..core::cmp::min(self.len(), buf.len()) {
            // All of the modulations for this sample, chain()ed together
            let modulations = non_lfos_filt
                .into_iter()
                .map(|(slc, val)| slc[i].wide_mul_signed(val))
                .chain(
                    lfos_filt
                        .into_iter()
                        .map(|(slc, val)| I1F31::saturating_from_num(slc[i].wide_mul(val))),
                );
            // Add all the modulations.  We'll do some bit twiddling so 100% modulation will
            // correspond to the maximum value of the type, and do all our math in 32 bit signed
            // arithmetic so we can model multiple modulations canceling each other out then
            // check for saturation at the end
            buf[i] = T::saturating_from_num(
                modulations
                    .map(|x| {
                        FixedI32::<T::Frac>::from_bits(IScalarFxP::from_num(x).to_bits() as i32)
                    })
                    .fold(FixedI32::<T::Frac>::from_num(buf[i]), |acc, val| acc + val),
            );
        }
        true
    }
    pub fn modulate_env(&self, params: &mut MutEnvParamsFxP, dest: &EnvModDest) {
        self.modulate(dest.attack, params.attack);
        self.modulate(dest.decay, params.decay);
        self.modulate(dest.sustain, params.sustain);
        self.modulate(dest.release, params.release);
    }
    pub fn modulate_osc(&self, params: &mut MutMixOscParamsFxP, dest: &OscModDest) {
        // Use a temporary buffer here to avoid _massive_ duplication of code
        let mut buf = [SignedNoteFxP::ZERO; STATIC_BUFFER_SIZE];
        // We have 6 bits of total range (7 - 1 sign bit) in SignedNoteFxP
        // The range of course tune is -32 to +32, or 5 bits + sign, so will need >>= 1
        // The range of fine tune is -2 to +2, or 1 bit + sign, so will need >>= 5
        // If we do fine first and >>= 4, then apply course and >>= 1, that will be equiv.
        let mut osc_mod_applied = false;
        if self.modulate(dest.fine, &mut buf) {
            osc_mod_applied = true;
            for i in buf.iter_mut() {
                *i >>= 4;
            }
        }
        osc_mod_applied |= self.modulate(dest.course, &mut buf);
        // Apply the modulation ourselves now
        if osc_mod_applied {
            for (smp, amt) in core::iter::zip(params.tune.iter_mut(), buf.iter()) {
                *smp = smp.saturating_add(amt >> 1);
            }
        }
        self.modulate(dest.shape, params.sin);
        self.modulate(dest.sin, params.sin);
        self.modulate(dest.sq, params.sq);
        self.modulate(dest.tri, params.tri);
        self.modulate(dest.saw, params.saw);
    }
    pub fn modulate_ring(&self, params: &mut MutRingModParamsFxP) {
        self.modulate(ModDest::RingOsc1, params.mix_a);
        self.modulate(ModDest::RingOsc2, params.mix_b);
        self.modulate(ModDest::RingMod, params.mix_out);
    }
    pub fn modulate_filt(&self, params: &mut MutModFiltParamsFxP) {
        self.modulate(ModDest::FiltEnv, params.env_mod);
        self.modulate(ModDest::FiltVel, params.vel_mod);
        self.modulate(ModDest::FiltKbd, params.kbd);
        self.modulate(ModDest::FiltCutoff, params.cutoff);
        self.modulate(ModDest::FiltRes, params.resonance);
        self.modulate(ModDest::FiltLow, params.low_mix);
        self.modulate(ModDest::FiltBand, params.band_mix);
        self.modulate(ModDest::FiltHigh, params.high_mix);
    }
}

pub struct ModSectionParamsFxP<'a> {
    pub velocity: &'a [ScalarFxP],
    pub aftertouch: &'a [ScalarFxP],
    pub modwheel: &'a [ScalarFxP],
    pub lfo1_params: LfoParamsFxP<'a>,
    pub lfo2_params: MutLfoParamsFxP<'a>,
    pub env1_params: EnvParamsFxP<'a>,
    pub env2_params: MutEnvParamsFxP<'a>,
}

impl<'a> ModSectionParamsFxP<'a> {
    pub fn len(&self) -> usize {
        min_size(&[
            self.velocity.len(),
            self.aftertouch.len(),
            self.modwheel.len(),
            self.lfo1_params.len(),
            self.lfo2_params.len(),
            self.env1_params.len(),
            self.env2_params.len(),
        ])
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct ModMatrixFxP {
    pub rows: [ModMatrixEntryFxP; ModSrc::numel()],
}

impl Default for ModMatrixFxP {
    fn default() -> Self {
        Self {
            rows: ModSrc::ELEM.map(|e| (e, [(ModDest::Null, IScalarFxP::ZERO); MOD_SLOTS])),
        }
    }
}

impl ModMatrixFxP {
    pub fn get_modulation(&self, src: ModSrc, dest: ModDest) -> Option<IScalarFxP> {
        self.rows[src as usize]
            .1
            .iter()
            .find_map(|x| if x.0 == dest { Some(x.1) } else { None })
    }
}

pub struct ModSectionFxP {
    lfo1: LfoFxP,
    lfo2: LfoFxP,
    env1: EnvFxP,
    env2: EnvFxP,
}

impl ModSectionFxP {
    pub fn process<'a>(
        &'a mut self,
        ctx: &ContextFxP,
        gate: &[SampleFxP],
        params: ModSectionParamsFxP<'a>,
        entries: &'a ModMatrixFxP,
    ) -> ModulatorFxP<'a> {
        let numsamples = min_size(&[gate.len(), params.len(), STATIC_BUFFER_SIZE]);
        let lfo1_out = self
            .lfo1
            .process(ctx, &gate[0..numsamples], params.lfo1_params);
        let env1_out = self
            .env1
            .process(ctx, &gate[0..numsamples], params.env1_params);
        // LFO2/ENV2 are default here, so empty slices.
        let modulator_initial = ModulatorFxP {
            velocity: params.velocity,
            aftertouch: params.aftertouch,
            modwheel: params.modwheel,
            lfo1: lfo1_out,
            lfo2: fixed_zerobuf_signed::<SampleFxP>(),
            env1: env1_out,
            env2: fixed_zerobuf_unsigned::<ScalarFxP>(),
            matrix: entries,
        };
        modulator_initial.modulate(
            ModDest::Lfo2Rate,
            &mut params.lfo2_params.freq[0..numsamples],
        );
        modulator_initial.modulate(
            ModDest::Lfo2Depth,
            &mut params.lfo2_params.depth[0..numsamples],
        );
        modulator_initial.modulate(
            ModDest::Env2A,
            &mut params.env2_params.attack[0..numsamples],
        );
        modulator_initial.modulate(ModDest::Env2D, &mut params.env2_params.decay[0..numsamples]);
        modulator_initial.modulate(
            ModDest::Env2S,
            &mut params.env2_params.sustain[0..numsamples],
        );
        modulator_initial.modulate(
            ModDest::Env2R,
            &mut params.env2_params.release[0..numsamples],
        );
        let lfo2_out = self
            .lfo2
            .process(ctx, &gate[0..numsamples], params.lfo2_params.into());
        let env2_out = self
            .env2
            .process(ctx, &gate[0..numsamples], params.env2_params.into());
        ModulatorFxP::<'a> {
            lfo2: lfo2_out,
            env2: env2_out,
            ..modulator_initial
        }
    }
}

impl Default for ModSectionFxP {
    fn default() -> Self {
        Self {
            lfo1: LfoFxP::default(),
            lfo2: LfoFxP::default(),
            env1: EnvFxP::new(),
            env2: EnvFxP::new(),
        }
    }
}

pub struct Modulator<'a, Smp: Float> {
    velocity: &'a [Smp],
    aftertouch: &'a [Smp],
    modwheel: &'a [Smp],
    lfo1: &'a [Smp],
    lfo2: &'a [Smp],
    env1: &'a [Smp],
    env2: &'a [Smp],
    matrix: &'a ModMatrix<Smp>,
}

impl<'a, Smp: Float> Modulator<'a, Smp> {
    pub fn len(&self) -> usize {
        min_size(&[
            self.velocity.len(),
            self.aftertouch.len(),
            self.modwheel.len(),
            self.lfo1.len(),
            self.lfo2.len(),
            self.env1.len(),
            self.env2.len(),
        ])
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Apply all modulation to the parameter passed in `dest` in place using `mut buf`
    ///
    /// Returns true if any modulation was performed, or false otherwise
    pub fn modulate(&self, dest: ModDest, buf: &mut [Smp]) -> bool {
        let modulation = ModSrc::ELEM.map(|src| self.matrix.get_modulation(src, dest));
        let mod_params = [
            (self.velocity, modulation[ModSrc::Velocity as usize]),
            (self.aftertouch, modulation[ModSrc::Aftertouch as usize]),
            (self.modwheel, modulation[ModSrc::ModWheel as usize]),
            (self.env1, modulation[ModSrc::Env1 as usize]),
            (self.env2, modulation[ModSrc::Env2 as usize]),
            (self.lfo1, modulation[ModSrc::Lfo1 as usize]),
            (self.lfo2, modulation[ModSrc::Lfo2 as usize]),
        ];
        // Filter the above and collect them into an array-backed vec
        let mod_params_filt = mod_params
            .iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(&[Smp], Smp); 7]>>();
        // In the common case, where there is no modulation, early exit
        if mod_params_filt.is_empty() {
            return false;
        }
        for i in 0..core::cmp::min(self.len(), buf.len()) {
            // All of the modulations for this sample
            let modulations = mod_params_filt.into_iter().map(|(slc, val)| slc[i] * val);
            // FIXME: Add scaling!!
            buf[i] = modulations.fold(buf[i], |acc, val| acc + val);
        }
        true
    }
    pub fn modulate_env(&self, params: &mut MutEnvParams<Smp>, dest: &EnvModDest) {
        self.modulate(dest.attack, params.attack);
        self.modulate(dest.decay, params.decay);
        self.modulate(dest.sustain, params.sustain);
        self.modulate(dest.release, params.release);
    }
    pub fn modulate_osc(&self, params: &mut MutMixOscParams<Smp>, dest: &OscModDest) {
        // Use a temporary buffer here to avoid _massive_ duplication of code
        let mut buf = [Smp::ZERO; STATIC_BUFFER_SIZE];
        // We have 6 bits of total range (7 - 1 sign bit) in SignedNoteFxP
        // The range of course tune is -32 to +32, or 5 bits + sign, so will need >>= 1
        // The range of fine tune is -2 to +2, or 1 bit + sign, so will need >>= 5
        // If we do fine first and >>= 4, then apply course and >>= 1, that will be equiv.
        let mut osc_mod_applied = false;
        if self.modulate(dest.fine, &mut buf) {
            osc_mod_applied = true;
            for i in &mut buf {
                *i = *i / <Smp as From<u16>>::from(16);
            }
        }
        osc_mod_applied |= self.modulate(dest.course, &mut buf);
        // Apply the modulation ourselves now
        if osc_mod_applied {
            for (smp, amt) in core::iter::zip(params.tune.iter_mut(), buf.iter()) {
                *smp = *smp + (*amt / Smp::TWO);
            }
        }
        self.modulate(dest.shape, params.shape);
        self.modulate(dest.sin, params.sin);
        self.modulate(dest.sq, params.sq);
        self.modulate(dest.tri, params.tri);
        self.modulate(dest.saw, params.saw);
    }
}

pub struct ModSectionParams<'a, Smp: Float> {
    pub velocity: &'a [Smp],
    pub aftertouch: &'a [Smp],
    pub modwheel: &'a [Smp],
    pub lfo1_params: LfoParams<'a, Smp>,
    pub lfo2_params: MutLfoParams<'a, Smp>,
    pub env1_params: EnvParams<'a, Smp>,
    pub env2_params: MutEnvParams<'a, Smp>,
}

pub struct ModMatrix<Smp: Float> {
    entries: [ModMatrixRow<Smp>; ModSrc::numel()],
}

impl<Smp: Float> Default for ModMatrix<Smp> {
    fn default() -> Self {
        Self {
            entries: ModSrc::ELEM.map(|e| (e, [(ModDest::Null, Smp::ZERO); MOD_SLOTS])),
        }
    }
}

impl<Smp: Float> ModMatrix<Smp> {
    pub fn get_modulation(&self, src: ModSrc, dest: ModDest) -> Option<Smp> {
        self.entries[src as usize]
            .1
            .iter()
            .find_map(|x| if x.0 == dest { Some(x.1) } else { None })
    }
}

pub struct ModSection<Smp: Float> {
    lfo1: Lfo<Smp>,
    lfo2: Lfo<Smp>,
    env1: Env<Smp>,
    env2: Env<Smp>,
}

impl<Smp: Float> ModSection<Smp> {
    pub fn process<'a>(
        &'a mut self,
        ctx: &Context<Smp>,
        gate: &[Smp],
        params: ModSectionParams<'a, Smp>,
        entries: &'a ModMatrix<Smp>,
    ) -> Modulator<'a, Smp> {
        let lfo1_out = self.lfo1.process(ctx, gate, params.lfo1_params);
        let env1_out = self.env1.process(ctx, gate, params.env1_params);
        let modulator_initial = Modulator::<'a, Smp> {
            velocity: params.velocity,
            aftertouch: params.aftertouch,
            modwheel: params.modwheel,
            lfo1: lfo1_out,
            lfo2: Smp::zerobuf(),
            env1: env1_out,
            env2: Smp::zerobuf(),
            matrix: entries,
        };
        modulator_initial.modulate(ModDest::Lfo2Rate, params.lfo2_params.freq);
        modulator_initial.modulate(ModDest::Lfo2Depth, params.lfo2_params.depth);
        modulator_initial.modulate(ModDest::Env2A, params.env2_params.attack);
        modulator_initial.modulate(ModDest::Env2D, params.env2_params.decay);
        modulator_initial.modulate(ModDest::Env2S, params.env2_params.sustain);
        modulator_initial.modulate(ModDest::Env2R, params.env2_params.release);
        let lfo2_out = self.lfo2.process(ctx, gate, params.lfo2_params.into());
        let env2_out = self.env2.process(ctx, gate, params.env2_params.into());
        Modulator::<'a, Smp> {
            lfo2: lfo2_out,
            env2: env2_out,
            ..modulator_initial
        }
    }
}

impl<Smp: Float> Default for ModSection<Smp> {
    fn default() -> Self {
        Self {
            lfo1: Lfo::default(),
            lfo2: Lfo::default(),
            env1: Env::new(),
            env2: Env::new(),
        }
    }
}
