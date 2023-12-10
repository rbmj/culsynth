//! This module contains data to allow modulation of a `Voice`
use fixed::types::extra::{Unsigned, LeEqU32, LeEqU16};
use fixed::{FixedI16, FixedU16, FixedI32};

use tinyvec::ArrayVec;

use crate::context::{ContextFxP, Context};
use crate::devices::*;
use crate::{IScalarFxP, ScalarFxP, SampleFxP, SignedNoteFxP};
use crate::{min_size, STATIC_BUFFER_SIZE};

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
    const ELEM: [ModSrc; Self::numel()] = [
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
#[derive(Clone, Copy, PartialEq)]
pub enum ModDest {
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
}

impl TryFrom<u16> for ModDest {
    type Error = &'static str;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value >= Self::min() as u16 && value <= Self::max() as u16 {
            unsafe {
                Ok(core::mem::transmute(value))
            }
        }
        else {
            Err("ModDest out of bounds")
        }
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
type ModSetFxP = [(ModDest, IScalarFxP); MOD_SLOTS];
type ModMatrixEntryFxP = (ModSrc, ModSetFxP);
type ModSet<Smp> = [(ModDest, Smp); MOD_SLOTS];
type ModMatrixEntry<Smp> = (ModSrc, ModSet<Smp>);

pub struct ModulatorFxP<'a> {
    velocity: &'a [ScalarFxP],
    aftertouch: &'a [ScalarFxP],
    modwheel: &'a [ScalarFxP],
    lfo1: &'a [SampleFxP],
    lfo2: &'a [SampleFxP],
    env1: &'a [ScalarFxP],
    env2: &'a [ScalarFxP],
    matrix: &'a ModMatrixEntriesFxP,
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
    pub fn apply_unsigned<N: Unsigned>(&self, dest: ModDest, buf: &mut [FixedU16<N>]) -> bool
        where N: LeEqU16 + LeEqU32
    {
        use crate::fixedmath::I1F31;
        let modulation = ModSrc::ELEM.map(|src|
            self.matrix.get_modulation(src, dest)
        );
        // All the modulation sources that are not LFOs are ScalarFxPs
        let non_lfos = [
            (self.velocity, modulation[ModSrc::Velocity as usize]),
            (self.aftertouch, modulation[ModSrc::Aftertouch as usize]),
            (self.modwheel, modulation[ModSrc::ModWheel as usize]),
            (self.env1, modulation[ModSrc::Env1 as usize]),
            (self.env2, modulation[ModSrc::Env2 as usize]),
        ];
        // Filter the above and collect them into an array-backed vec
        let non_lfos_filt = non_lfos.iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(&[ScalarFxP], IScalarFxP); 5]>>();
        // The LFOs, however, are SampleFxPs, so these need to be separate
        let lfos = [
            (self.lfo1, modulation[ModSrc::Lfo1 as usize]),
            (self.lfo2, modulation[ModSrc::Lfo2 as usize]),
        ];
        let lfos_filt = lfos.iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(&[SampleFxP], IScalarFxP); 2]>>();
        // In the common case, where there is no modulation, early exit
        if non_lfos_filt.is_empty() && lfos_filt.is_empty() {
            return false;
        }
        for i in 0..core::cmp::min(self.len(), buf.len()) {
            // All of the modulations for this sample, chain()ed together
            let modulations = non_lfos_filt.into_iter()
                .map(|(slc, val)| slc[i].wide_mul_signed(val))
                .chain(lfos_filt.into_iter().map(|(slc, val)| I1F31::saturating_from_num(
                    slc[i].wide_mul(val),
                )));
            // Add all the modulations.  We'll do some bit twiddling so 100% modulation will
            // correspond to the maximum value of the type, and do all our math in 32 bit signed
            // arithmetic so we can model multiple modulations canceling each other out then
            // check for saturation at the end
            buf[i] = FixedU16::<N>::saturating_from_num(modulations
                .map(|x| FixedI32::<N>::from_bits(IScalarFxP::from_num(x).to_bits() as i32))
                .fold(FixedI32::<N>::from_num(buf[i]), |acc, val| acc + val)
            );
        }
        true
    }
    /// Apply all modulation to the parameter passed in `dest` in place using `mut buf`
    /// 
    /// Returns true if any modulation was performed, or false otherwise
    pub fn apply_signed<N: Unsigned>(&self, dest: ModDest, buf: &mut [FixedI16<N>]) -> bool
        where N: LeEqU16 + LeEqU32
    {
        use crate::fixedmath::I1F31;
        let modulation = ModSrc::ELEM.map(|src|
            self.matrix.get_modulation(src, dest)
        );
        // All the modulation sources that are not LFOs are ScalarFxPs
        let non_lfos = [
            (self.velocity, modulation[ModSrc::Velocity as usize]),
            (self.aftertouch, modulation[ModSrc::Aftertouch as usize]),
            (self.modwheel, modulation[ModSrc::ModWheel as usize]),
            (self.env1, modulation[ModSrc::Env1 as usize]),
            (self.env2, modulation[ModSrc::Env2 as usize]),
        ];
        // Filter the above and collect them into an array-backed vec
        let non_lfos_filt = non_lfos.iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(&[ScalarFxP], IScalarFxP); 5]>>();
        // The LFOs, however, are SampleFxPs, so these need to be separate
        let lfos = [
            (self.lfo1, modulation[ModSrc::Lfo1 as usize]),
            (self.lfo2, modulation[ModSrc::Lfo2 as usize]),
        ];
        let lfos_filt = lfos.iter()
            .filter_map(|x| x.1.map(|y| (x.0, y)))
            .collect::<ArrayVec<[(&[SampleFxP], IScalarFxP); 2]>>();
        // In the common case, where there is no modulation, early exit
        if non_lfos_filt.is_empty() && lfos_filt.is_empty() {
            return false;
        }
        for i in 0..core::cmp::min(self.len(), buf.len()) {
            // All of the modulations for this sample, chain()ed together
            let modulations = non_lfos_filt.into_iter()
                .map(|(slc, val)| slc[i].wide_mul_signed(val))
                .chain(lfos_filt.into_iter().map(|(slc, val)| I1F31::saturating_from_num(
                    slc[i].wide_mul(val),
                )));
            // Add all the modulations.  We'll do some bit twiddling so 100% modulation will
            // correspond to the maximum value of the type, and do all our math in 32 bit signed
            // arithmetic so we can model multiple modulations canceling each other out then
            // check for saturation at the end
            buf[i] = FixedI16::<N>::saturating_from_num(modulations
                .map(|x| FixedI32::<N>::from_bits(IScalarFxP::from_num(x).to_bits() as i32))
                .fold(FixedI32::<N>::from_num(buf[i]), |acc, val| acc + val)
            );
        }
        true
    }
    pub fn modulate_env(&self, params: &mut MutEnvParamsFxP, dest: &EnvModDest) {
        self.apply_unsigned(dest.attack, params.attack);
        self.apply_unsigned(dest.decay, params.decay);
        self.apply_unsigned(dest.sustain, params.sustain);
        self.apply_unsigned(dest.release, params.release);
    }
    pub fn modulate_osc(&self, params: &mut MutMixOscParamsFxP, dest: &OscModDest) {
        // Use a temporary buffer here to avoid _massive_ duplication of code
        let mut buf = [SignedNoteFxP::ZERO; STATIC_BUFFER_SIZE];
        // We have 6 bits of total range (7 - 1 sign bit) in SignedNoteFxP
        // The range of course tune is -32 to +32, or 5 bits + sign, so will need >>= 1
        // The range of fine tune is -2 to +2, or 1 bit + sign, so will need >>= 5
        // If we do fine first and >>= 4, then apply course and >>= 1, that will be equiv.
        let mut osc_mod_applied = false;
        if self.apply_signed(dest.fine, &mut buf) {
            osc_mod_applied = true;
            for mut i in buf {
                i >>= 4;
            }
        }
        osc_mod_applied |= self.apply_signed(dest.course, &mut buf);
        // Apply the modulation ourselves now
        if osc_mod_applied {
            for (smp, amt) in core::iter::zip(params.tune.iter_mut(), buf.iter()) {
                *smp = smp.saturating_add(amt >> 1);
            }
        }
        self.apply_unsigned(dest.shape, params.shape);
        self.apply_unsigned(dest.sin, params.sin);
        self.apply_unsigned(dest.sq, params.sq);
        self.apply_unsigned(dest.tri, params.tri);
        self.apply_unsigned(dest.saw, params.saw);
    }
}

pub struct ModMatrixParamsFxP<'a> {
    pub velocity: &'a [ScalarFxP],
    pub aftertouch: &'a [ScalarFxP],
    pub modwheel: &'a [ScalarFxP],
    pub lfo1_params: LfoParamsFxP<'a>,
    pub lfo2_params: MutLfoParamsFxP<'a>,
    pub env1_params: EnvParamsFxP<'a>,
    pub env2_params: MutEnvParamsFxP<'a>,
}

struct ModMatrixEntriesFxP {
    entries: [ModMatrixEntryFxP; ModSrc::numel()],
}

impl Default for ModMatrixEntriesFxP {
    fn default() -> Self {
        Self {
            entries: ModSrc::ELEM.map(|e| (e, [(ModDest::Null, IScalarFxP::ZERO); MOD_SLOTS])),
        }
    }
}

impl ModMatrixEntriesFxP {
    pub fn get_modulation(&self, src: ModSrc, dest: ModDest) -> Option<IScalarFxP> {
        self.entries[src as usize].1.iter()
            .find_map(|x| if x.0 == dest { Some(x.1) } else { None })
    }
}

pub struct ModMatrixFxP {
    entries: ModMatrixEntriesFxP,
    lfo1: LfoFxP,
    lfo2: LfoFxP,
    env1: EnvFxP,
    env2: EnvFxP,
}

impl ModMatrixFxP {
    pub fn process<'a>(
        &'a mut self,
        ctx: &ContextFxP,
        gate: &[SampleFxP],
        params: ModMatrixParamsFxP<'a>,
    ) -> ModulatorFxP<'a> {
        let lfo1_out = self.lfo1.process(ctx, gate, params.lfo1_params);
        let env1_out = self.env1.process(ctx, gate, params.env1_params);
        // LFO2/ENV2 are default here, so empty slices.
        let modulator_initial = ModulatorFxP {
            velocity: params.velocity,
            aftertouch: params.aftertouch,
            modwheel: params.modwheel,
            lfo1: lfo1_out,
            lfo2: Default::default(),
            env1: env1_out,
            env2: Default::default(),
            matrix: &self.entries,
        };
        modulator_initial.apply_unsigned(ModDest::Lfo2Rate, params.lfo2_params.freq);
        modulator_initial.apply_unsigned(ModDest::Lfo2Depth, params.lfo2_params.depth);
        modulator_initial.apply_unsigned(ModDest::Env2A, params.env2_params.attack);
        modulator_initial.apply_unsigned(ModDest::Env2D, params.env2_params.decay);
        modulator_initial.apply_unsigned(ModDest::Env2S, params.env2_params.sustain);
        modulator_initial.apply_unsigned(ModDest::Env2R, params.env2_params.release);
        let lfo2_out = self.lfo2.process(ctx, gate, params.lfo2_params.into());
        let env2_out = self.env2.process(ctx, gate, params.env2_params.into());
        /*
        ModulatorFxP {
            velocity: params.velocity,
            aftertouch: params.aftertouch,
            modwheel: params.modwheel,
            lfo1: lfo1_out,
            lfo2: lfo2_out,
            env1: env1_out,
            env2: env2_out,
            matrix: &self.entries,
        }
        */
        ModulatorFxP {
            lfo2: lfo2_out,
            env2: env2_out,
            ..modulator_initial
        }
    }
}

impl Default for ModMatrixFxP {
    fn default() -> Self {
        Self {
            entries: ModMatrixEntriesFxP::default(),
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
    matrix: &'a ModMatrixEntries<Smp>,
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
    pub fn apply(&self, dest: ModDest, buf: &mut [Smp]) -> bool
    {
        let modulation = ModSrc::ELEM.map(|src|
            self.matrix.get_modulation(src, dest)
        );
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
        let mod_params_filt = mod_params.iter()
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
        self.apply(dest.attack, params.attack);
        self.apply(dest.decay, params.decay);
        self.apply(dest.sustain, params.sustain);
        self.apply(dest.release, params.release);
    }
    pub fn modulate_osc(&self, params: &mut MutMixOscParams<Smp>, dest: &OscModDest) {
        // Use a temporary buffer here to avoid _massive_ duplication of code
        let mut buf = [Smp::ZERO; STATIC_BUFFER_SIZE];
        // We have 6 bits of total range (7 - 1 sign bit) in SignedNoteFxP
        // The range of course tune is -32 to +32, or 5 bits + sign, so will need >>= 1
        // The range of fine tune is -2 to +2, or 1 bit + sign, so will need >>= 5
        // If we do fine first and >>= 4, then apply course and >>= 1, that will be equiv.
        let mut osc_mod_applied = false;
        if self.apply(dest.fine, &mut buf) {
            osc_mod_applied = true;
            for i in &mut buf {
                *i = *i / <Smp as From<u16>>::from(16);
            }
        }
        osc_mod_applied |= self.apply(dest.course, &mut buf);
        // Apply the modulation ourselves now
        if osc_mod_applied {
            for (smp, amt) in core::iter::zip(params.tune.iter_mut(), buf.iter()) {
                *smp = *smp + (*amt / Smp::TWO);
            }
        }
        self.apply(dest.shape, params.shape);
        self.apply(dest.sin, params.sin);
        self.apply(dest.sq, params.sq);
        self.apply(dest.tri, params.tri);
        self.apply(dest.saw, params.saw);
    }
}

pub struct ModMatrixParams<'a, Smp: Float> {
    pub velocity: &'a [Smp],
    pub aftertouch: &'a [Smp],
    pub modwheel: &'a [Smp],
    pub lfo1_params: LfoParams<'a, Smp>,
    pub lfo2_params: MutLfoParams<'a, Smp>,
    pub env1_params: EnvParams<'a, Smp>,
    pub env2_params: MutEnvParams<'a, Smp>,
}

struct ModMatrixEntries<Smp: Float> {
    entries: [ModMatrixEntry<Smp>; ModSrc::numel()],
}

impl<Smp: Float> Default for ModMatrixEntries<Smp> {
    fn default() -> Self {
        Self {
            entries: ModSrc::ELEM.map(|e| (e, [(ModDest::Null, Smp::ZERO); MOD_SLOTS])),
        }
    }
}

impl<Smp: Float> ModMatrixEntries<Smp> {
    pub fn get_modulation(&self, src: ModSrc, dest: ModDest) -> Option<Smp> {
        self.entries[src as usize].1.iter()
            .find_map(|x| if x.0 == dest { Some(x.1) } else { None })
    }
}

pub struct ModMatrix<Smp: Float> {
    entries: ModMatrixEntries<Smp>,
    lfo1: Lfo<Smp>,
    lfo2: Lfo<Smp>,
    env1: Env<Smp>,
    env2: Env<Smp>,
}

impl<Smp: Float> ModMatrix<Smp> {
    pub fn process<'a>(
        &'a mut self,
        ctx: &Context<Smp>,
        gate: &[Smp],
        params: ModMatrixParams<'a, Smp>,
    ) -> Modulator<'a, Smp> {
        let lfo1_out = self.lfo1.process(ctx, gate, params.lfo1_params);
        let env1_out = self.env1.process(ctx, gate, params.env1_params);
        let modulator_initial = Modulator {
            velocity: params.velocity,
            aftertouch: params.aftertouch,
            modwheel: params.modwheel,
            lfo1: lfo1_out,
            lfo2: Default::default(),
            env1: env1_out,
            env2: Default::default(),
            matrix: &self.entries,
        };
        modulator_initial.apply(ModDest::Lfo2Rate, params.lfo2_params.freq);
        modulator_initial.apply(ModDest::Lfo2Depth, params.lfo2_params.depth);
        modulator_initial.apply(ModDest::Env2A, params.env2_params.attack);
        modulator_initial.apply(ModDest::Env2D, params.env2_params.decay);
        modulator_initial.apply(ModDest::Env2S, params.env2_params.sustain);
        modulator_initial.apply(ModDest::Env2R, params.env2_params.release);
        let lfo2_out = self.lfo2.process(ctx, gate, params.lfo2_params.into());
        let env2_out = self.env2.process(ctx, gate, params.env2_params.into());
        Modulator {
            velocity: params.velocity,
            aftertouch: params.aftertouch,
            modwheel: params.modwheel,
            lfo1: lfo1_out,
            lfo2: lfo2_out,
            env1: env1_out,
            env2: env2_out,
            matrix: &self.entries,
        }
    }
}

impl<Smp: Float> Default for ModMatrix<Smp> {
    fn default() -> Self {
        Self {
            entries: ModMatrixEntries::default(),
            lfo1: Lfo::default(),
            lfo2: Lfo::default(),
            env1: Env::new(),
            env2: Env::new(),
        }
    }
}

