//! This module contains data to allow modulation of a `Voice`
use tinyvec::ArrayVec;

use crate::context::{Context, ContextFxP};
use crate::devices::*;
use crate::{min_size, STATIC_BUFFER_SIZE};
use crate::{EnvParamFxP, IScalarFxP, LfoFreqFxP, SampleFxP, ScalarFxP, SignedNoteFxP};

/// An enum representing a choice in modulation source
#[repr(u16)]
#[derive(Clone, Copy)]
pub enum ModSrc {
    /// MIDI Note On velocity
    Velocity,
    /// MIDI Channel aftertouch
    Aftertouch,
    /// The modulation wheel (MIDI CC #1)
    ModWheel,
    /// Modulation envelope #1
    Env1,
    /// Modulation envelope #2
    Env2,
    /// LFO #1
    Lfo1,
    /// LFO #2
    Lfo2,
}

impl ModSrc {
    /// An array containing all possible `ModSrc` values, in order
    pub const ELEM: [ModSrc; Self::numel()] = [
        ModSrc::Velocity,
        ModSrc::Aftertouch,
        ModSrc::ModWheel,
        ModSrc::Env1,
        ModSrc::Env2,
        ModSrc::Lfo1,
        ModSrc::Lfo2,
    ];
    /// An iterator over all the different elements in `ModSrc`
    pub const fn elements() -> &'static [ModSrc] {
        &Self::ELEM
    }
    /// The first value in elements
    pub const fn min() -> Self {
        Self::Velocity
    }
    /// The last value in elements
    pub const fn max() -> Self {
        Self::Lfo2
    }
    /// The number of different modualtion sources
    pub const fn numel() -> usize {
        1 + Self::max() as usize - Self::min() as usize
    }
    /// The string representation of the modulation source
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

/// An enum representing a modulation destination
#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ModDest {
    /// The default is `ModDest::Null`, which is equivalent to no modulation
    #[default]
    Null,
    /// Course tune for oscillator 1, ranging from -32 to +32 semitones
    Osc1Course,
    /// Fine tune for oscillator 1, ranging from -2 to +2 semitones
    Osc1Fine,
    /// The wave shape (phase distortion) of oscillator 1
    Osc1Shape,
    /// The mix of the sine wave output for oscillator 1
    Osc1Sin,
    /// The mix of the square wave output for oscillator 1
    Osc1Sq,
    /// The mix of the triangel wave output for oscillator 1
    Osc1Tri,
    /// The mix of the sawtooth wave output for oscillator 1
    Osc1Saw,
    /// Course tune for oscillator 2, ranging from -32 to +32 semitones
    Osc2Course,
    /// Fine tune for oscillator 1, ranging from -2 to +2 semitones
    Osc2Fine,
    /// The wave shape (phase distortion) of oscillator 2
    Osc2Shape,
    /// The mix of the sine wave output for oscillator 2
    Osc2Sin,
    /// The mix of the square wave output for oscillator 2
    Osc2Sq,
    /// The mix of the triangle wave output for oscillator 2
    Osc2Tri,
    /// The mix of the sawtooth wave output for oscillator 2
    Osc2Saw,
    /// The mix of the dry signal from oscillator 1 in the output of the
    /// ring modulation section
    RingOsc1,
    /// The mix of the dry signal from oscillator 2 in the output of the
    /// ring modulation section
    RingOsc2,
    /// The mix of the wet (modulated) signal in the output of the ring
    /// modulation section
    RingMod,
    /// The filter cutoff frequency
    FiltCutoff,
    /// The filter resonance parameter
    FiltRes,
    /// The filter envelope modulation
    FiltEnv,
    /// The filter keyboard tracking
    FiltKbd,
    /// The filter velocity modulation
    FiltVel,
    /// The filter low-pass output mix
    FiltLow,
    /// The filter band-pass output mix
    FiltBand,
    /// The filter high-pass output mix
    FiltHigh,
    /// The filter envelope attack
    EnvFiltA,
    /// The filter envelope decay
    EnvFiltD,
    /// The filter envelope sustain
    EnvFiltS,
    /// The filter envelope release
    EnvFiltR,
    /// The VCA envelope attack
    EnvAmpA,
    /// The VCA envelope decay
    EnvAmpD,
    /// The VCA envelope sustain
    EnvAmpS,
    /// The VCA envelope release
    EnvAmpR,

    /// The rate/frequency of LFO 2, in Hz
    Lfo2Rate,
    /// The modulation depth of LFO 2, from 0 to 1
    Lfo2Depth,
    /// The attack of modulation envelope 2
    Env2A,
    /// The decay of modulation envelope 2
    Env2D,
    /// The sustain of modulation envelope 2
    Env2S,
    /// The release of modulation envelope 2
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
    /// The string representation of this modulation destination.
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
    /// The first modulation destination, in order
    pub const fn min() -> Self {
        Self::Null
    }
    /// The last modulation destination, in order
    pub const fn max() -> Self {
        Self::Env2R
    }
    /// The last modulation destination before the secondary destinations
    ///
    /// The secondary modulation destinations are invalid destinations from
    /// LFO2/ENV2 to avoid self/co-modulation
    pub const fn max_secondary() -> Self {
        Self::EnvAmpR
    }
    /// An iterator over all modulation destinations
    pub fn elements() -> impl core::iter::Iterator<Item = ModDest> {
        Self::elements_secondary_if(false)
    }
    /// An iterator over all non-secondary modulation destinations
    ///
    /// FIXME: Bad name
    pub fn elements_secondary() -> impl core::iter::Iterator<Item = ModDest> {
        Self::elements_secondary_if(true)
    }
    /// An iterator that excludes the secondary modulation destinations if the
    /// argument is true, and includes them if it is false
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

/// A struct to allow expressing the different modulation destinations for a
/// particular oscillator.  See [OSC1_MOD_DEST]/[OSC2_MOD_DEST] and
/// [Modulator]/[ModulatorFxP]
pub struct OscModDest {
    course: ModDest,
    fine: ModDest,
    shape: ModDest,
    sin: ModDest,
    sq: ModDest,
    tri: ModDest,
    saw: ModDest,
}

/// The modulation destinations corresponding to oscillator 1
pub const OSC1_MOD_DEST: OscModDest = OscModDest {
    course: ModDest::Osc1Course,
    fine: ModDest::Osc1Fine,
    shape: ModDest::Osc1Shape,
    sin: ModDest::Osc1Sin,
    sq: ModDest::Osc1Sq,
    tri: ModDest::Osc1Tri,
    saw: ModDest::Osc1Saw,
};

/// The modulation destinations corresponding to oscillator 2
pub const OSC2_MOD_DEST: OscModDest = OscModDest {
    course: ModDest::Osc2Course,
    fine: ModDest::Osc2Fine,
    shape: ModDest::Osc2Shape,
    sin: ModDest::Osc2Sin,
    sq: ModDest::Osc2Sq,
    tri: ModDest::Osc2Tri,
    saw: ModDest::Osc2Saw,
};

/// A struct to allow expressing the different modulation destinations for a
/// particular oscillator.  See [ENV_AMP_MOD_DEST]/[ENV_FILT_MOD_DEST] and
/// [Modulator]/[ModulatorFxP]
pub struct EnvModDest {
    attack: ModDest,
    decay: ModDest,
    sustain: ModDest,
    release: ModDest,
}

/// The modulation destinations corresponding to the VCA envelope
pub const ENV_AMP_MOD_DEST: EnvModDest = EnvModDest {
    attack: ModDest::EnvAmpA,
    decay: ModDest::EnvAmpD,
    sustain: ModDest::EnvAmpS,
    release: ModDest::EnvAmpR,
};

/// The modulation destinations corresponding to the filter envelope
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

/// A struct used to modulate fixed-point parameters.  Obtained from
/// [ModSectionFxP]
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
    /// The "length" of this modulator, i.e. the length of parameter slice it
    /// has sufficient data to modulate
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
    /// True if `self.len() == 0`
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
        use crate::fixedmath::{I16F16, I17F15, I1F31};
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
                        FixedI32::<T::Frac>::from_bits(if T::IS_SIGNED {
                            I17F15::from_num(x).to_bits()
                        } else {
                            I16F16::from_num(x).to_bits()
                        })
                    })
                    .fold(FixedI32::<T::Frac>::from_num(buf[i]), |acc, val| acc + val),
            );
        }
        true
    }
    /// Modulate all of the parameters in `params` for the envelope specified by
    /// `dest`, which should be either [ENV_AMP_MOD_DEST] or [ENV_FILT_MOD_DEST]
    pub fn modulate_env(&self, params: &mut MutEnvParamsFxP, dest: &EnvModDest) {
        self.modulate(dest.attack, params.attack);
        self.modulate(dest.decay, params.decay);
        self.modulate(dest.sustain, params.sustain);
        self.modulate(dest.release, params.release);
    }
    /// Modulate all of the parameters in `params` for the oscillator specified by
    /// `dest`, which should be either [OSC1_MOD_DEST] or [OSC2_MOD_DEST]
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
    /// Modulate the ring modulator parameters
    pub fn modulate_ring(&self, params: &mut MutRingModParamsFxP) {
        self.modulate(ModDest::RingOsc1, params.mix_a);
        self.modulate(ModDest::RingOsc2, params.mix_b);
        self.modulate(ModDest::RingMod, params.mix_out);
    }
    /// Modulate the filter parameters
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

/// A parameter pack representing the different parameters to the [ModSectionFxP]
pub struct ModSectionParamsFxP<'a> {
    /// MIDI Velocity
    pub velocity: &'a [ScalarFxP],
    /// MIDI Channel aftertouch
    pub aftertouch: &'a [ScalarFxP],
    /// Modulation wheel (MIDI CC #1)
    pub modwheel: &'a [ScalarFxP],
    /// Parameters for LFO 1
    pub lfo1_params: LfoParamsFxP<'a>,
    /// Parameters for LFO 2
    pub lfo2_params: MutLfoParamsFxP<'a>,
    /// Parameters for Envelope 1
    pub env1_params: EnvParamsFxP<'a>,
    /// Parameters for Envelope 2
    pub env2_params: MutEnvParamsFxP<'a>,
}

impl<'a> ModSectionParamsFxP<'a> {
    /// The length of this parameter pack, defined as the length of the shortest
    /// subslice
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
    /// True if any subslice is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The actual data that describes the fixed point modulation matrix
pub struct ModMatrixFxP {
    /// The rows of the matrix, one per [ModSrc]
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
    /// If there is an entry in this matrix from `src` to `dest`, return the
    /// modulation depth, else return None
    pub fn get_modulation(&self, src: ModSrc, dest: ModDest) -> Option<IScalarFxP> {
        self.rows[src as usize]
            .1
            .iter()
            .find_map(|x| if x.0 == dest { Some(x.1) } else { None })
    }
}

/// The actual modulation section, containing the modulation LFOs and Envelopes and
/// logic to build the [ModulatorFxP].
#[derive(Clone)]
pub struct ModSectionFxP {
    lfo1: LfoFxP,
    lfo2: LfoFxP,
    env1: EnvFxP,
    env2: EnvFxP,
}

impl ModSectionFxP {
    /// Build a new modulation section, seeding the LFO RNGs (for S+H/S+G) from
    /// the seeds seed1 and seed2
    pub fn new_with_seeds(seed1: u64, seed2: u64) -> Self {
        Self {
            lfo1: LfoFxP::new(seed1),
            lfo2: LfoFxP::new(seed2),
            env1: Default::default(),
            env2: Default::default(),
        }
    }
    /// Build a [ModulatorFxP] from all the required data, to include the
    /// processing context, the gate signal, the [ModSectionParamsFxP], and
    /// the actual [ModMatrixFxP].
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
        let modulator = ModulatorFxP {
            velocity: params.velocity,
            aftertouch: params.aftertouch,
            modwheel: params.modwheel,
            lfo1: lfo1_out,
            lfo2: fixed_zerobuf::<SampleFxP>(),
            env1: env1_out,
            env2: fixed_zerobuf::<ScalarFxP>(),
            matrix: entries,
        };
        modulator.modulate(
            ModDest::Lfo2Rate,
            &mut params.lfo2_params.freq[0..numsamples],
        );
        modulator.modulate(
            ModDest::Lfo2Depth,
            &mut params.lfo2_params.depth[0..numsamples],
        );
        modulator.modulate(
            ModDest::Env2A,
            &mut params.env2_params.attack[0..numsamples],
        );
        modulator.modulate(ModDest::Env2D, &mut params.env2_params.decay[0..numsamples]);
        modulator.modulate(
            ModDest::Env2S,
            &mut params.env2_params.sustain[0..numsamples],
        );
        modulator.modulate(
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
            ..modulator
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

/// A struct used to modulate floating-point parameters.  Obtained from
/// [ModSection]
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
    /// The "length" of this modulator, i.e. the length of parameter slice it
    /// has sufficient data to modulate
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
    /// True if `self.len() == 0`
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn coeff_from_fixed<T: crate::Fixed16>() -> Smp {
        let num_bits = if T::IS_SIGNED { 15 } else { 16 } - T::FRAC_NBITS as i32;
        if num_bits == -1 {
            Smp::ONE_HALF
        } else {
            Smp::from_u16(1u16 << num_bits)
        }
    }
    /// Apply all modulation to the parameter passed in `dest` in place using `mut buf`
    ///
    /// Returns true if any modulation was performed, or false otherwise
    pub fn modulate(&self, dest: ModDest, buf: &mut [Smp], coeff: Smp) -> bool {
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
            .filter_map(|x| x.1.map(|y| (x.0, y * coeff)))
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
    /// Modulate all of the parameters in `params` for the envelope specified by
    /// `dest`, which should be either [ENV_AMP_MOD_DEST] or [ENV_FILT_MOD_DEST]
    pub fn modulate_env(&self, params: &mut MutEnvParams<Smp>, dest: &EnvModDest) {
        let coeff = Self::coeff_from_fixed::<EnvParamFxP>();
        self.modulate(dest.attack, params.attack, coeff);
        self.modulate(dest.decay, params.decay, coeff);
        self.modulate(
            dest.sustain,
            params.sustain,
            Self::coeff_from_fixed::<ScalarFxP>(),
        );
        self.modulate(dest.release, params.release, coeff);
    }
    /// Modulate all of the parameters in `params` for the oscillator specified by
    /// `dest`, which should be either [OSC1_MOD_DEST] or [OSC2_MOD_DEST]
    pub fn modulate_osc(&self, params: &mut MutMixOscParams<Smp>, dest: &OscModDest) {
        let coeff = Self::coeff_from_fixed::<ScalarFxP>();
        self.modulate(dest.fine, params.tune, Smp::TWO);
        self.modulate(dest.course, params.tune, Smp::from_u16(32));
        self.modulate(dest.shape, params.shape, coeff);
        self.modulate(dest.sin, params.sin, coeff);
        self.modulate(dest.sq, params.sq, coeff);
        self.modulate(dest.tri, params.tri, coeff);
        self.modulate(dest.saw, params.saw, coeff);
    }
    /// Modulate the ring modulator parameters
    pub fn modulate_ring(&self, params: &mut MutRingModParams<Smp>) {
        let coeff = Self::coeff_from_fixed::<ScalarFxP>();
        self.modulate(ModDest::RingOsc1, params.mix_a, coeff);
        self.modulate(ModDest::RingOsc2, params.mix_b, coeff);
        self.modulate(ModDest::RingMod, params.mix_out, coeff);
    }
    /// Modulate the filter parameters
    pub fn modulate_filt(&self, params: &mut MutModFiltParams<Smp>) {
        let coeff = Self::coeff_from_fixed::<ScalarFxP>();
        let filt_coeff = Self::coeff_from_fixed::<crate::NoteFxP>();
        self.modulate(ModDest::FiltEnv, params.env_mod, coeff);
        self.modulate(ModDest::FiltVel, params.vel_mod, coeff);
        self.modulate(ModDest::FiltKbd, params.kbd, coeff);
        self.modulate(ModDest::FiltCutoff, params.cutoff, filt_coeff);
        self.modulate(ModDest::FiltRes, params.resonance, coeff);
        self.modulate(ModDest::FiltLow, params.low_mix, coeff);
        self.modulate(ModDest::FiltBand, params.band_mix, coeff);
        self.modulate(ModDest::FiltHigh, params.high_mix, coeff);
    }
}

/// A parameter pack representing the different parameters to the [ModSectionFxP]
pub struct ModSectionParams<'a, Smp: Float> {
    /// MIDI Velocity
    pub velocity: &'a [Smp],
    /// MIDI Channel aftertouch
    pub aftertouch: &'a [Smp],
    /// Modulation wheel (MIDI CC #1)
    pub modwheel: &'a [Smp],
    /// Parameters for LFO 1
    pub lfo1_params: LfoParams<'a, Smp>,
    /// Parameters for LFO 2
    pub lfo2_params: MutLfoParams<'a, Smp>,
    /// Parameters for Envelope 1
    pub env1_params: EnvParams<'a, Smp>,
    /// Parameters for Envelope 2
    pub env2_params: MutEnvParams<'a, Smp>,
}

/// A struct representing the actual modulation matrix
pub struct ModMatrix<Smp: Float> {
    /// The actual rows of the modulation matrix, one per [ModSrc]
    pub rows: [ModMatrixRow<Smp>; ModSrc::numel()],
}

impl<Smp: Float> Default for ModMatrix<Smp> {
    fn default() -> Self {
        Self {
            rows: ModSrc::ELEM.map(|e| (e, [(ModDest::Null, Smp::ZERO); MOD_SLOTS])),
        }
    }
}

impl<Smp: Float> ModMatrix<Smp> {
    /// If there is an entry in this matrix from `src` to `dest`, return the
    /// modulation depth, else return None
    pub fn get_modulation(&self, src: ModSrc, dest: ModDest) -> Option<Smp> {
        self.rows[src as usize]
            .1
            .iter()
            .find_map(|x| if x.0 == dest { Some(x.1) } else { None })
    }
}

/// The actual modulation section, containing the modulation LFOs and Envelopes and
/// logic to build the [Modulator].
#[derive(Clone)]
pub struct ModSection<Smp: Float> {
    lfo1: Lfo<Smp>,
    lfo2: Lfo<Smp>,
    env1: Env<Smp>,
    env2: Env<Smp>,
}

impl<Smp: Float> ModSection<Smp> {
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
    pub fn process<'a>(
        &'a mut self,
        ctx: &Context<Smp>,
        gate: &[Smp],
        params: ModSectionParams<'a, Smp>,
        entries: &'a ModMatrix<Smp>,
    ) -> Modulator<'a, Smp> {
        let lfo1_out = self.lfo1.process(ctx, gate, params.lfo1_params);
        let env1_out = self.env1.process(ctx, gate, params.env1_params);
        let modulator = Modulator::<'a, Smp> {
            velocity: params.velocity,
            aftertouch: params.aftertouch,
            modwheel: params.modwheel,
            lfo1: lfo1_out,
            lfo2: Smp::zerobuf(),
            env1: env1_out,
            env2: Smp::zerobuf(),
            matrix: entries,
        };
        let env_coeff = Modulator::coeff_from_fixed::<EnvParamFxP>();
        let lfo_coeff = Modulator::coeff_from_fixed::<LfoFreqFxP>();
        let scalar_coeff = Modulator::coeff_from_fixed::<ScalarFxP>();
        modulator.modulate(ModDest::Lfo2Rate, params.lfo2_params.freq, lfo_coeff);
        modulator.modulate(ModDest::Lfo2Depth, params.lfo2_params.depth, scalar_coeff);
        modulator.modulate(ModDest::Env2A, params.env2_params.attack, env_coeff);
        modulator.modulate(ModDest::Env2D, params.env2_params.decay, env_coeff);
        modulator.modulate(ModDest::Env2S, params.env2_params.sustain, scalar_coeff);
        modulator.modulate(ModDest::Env2R, params.env2_params.release, env_coeff);
        let lfo2_out = self.lfo2.process(ctx, gate, params.lfo2_params.into());
        let env2_out = self.env2.process(ctx, gate, params.env2_params.into());
        Modulator::<'a, Smp> {
            lfo2: lfo2_out,
            env2: env2_out,
            ..modulator
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
