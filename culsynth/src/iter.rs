//! This module contains a more 'rusty' iterator-based approach to DSP.

use core::iter::{repeat, Iterator, Repeat};
use core::ops::{Add, Sub};
use fixed::traits::Fixed;

use crate::context::{Context, ContextFxP, GenericContext};
use crate::fixedmath::{Frequency, Note, Sample, Scalar};
use crate::{devices::PhaseFxP, EnvParamFxP, IScalarFxP, LfoFreqFxP, SignedNoteFxP};

use crate::fixedmath::I3F29 as EnvSignalFxP;

pub mod amp;
pub mod env;
pub mod filt;
pub mod lfo;
pub mod mixosc;
pub mod modfilt;
pub mod osc;
pub mod ringmod;

pub(crate) mod detail {
    use super::*;

    #[derive(PartialEq, Clone, Copy)]
    pub enum OscSync {
        /// No sync behavior - do not calculate
        Off,
        /// This is the master oscillator
        Master,
        /// This is the slave oscillator
        Slave,
    }

    pub trait OscOps: DspTypeAliases {
        const FRAC_2_PI: Self::Scalar;
        fn calc_osc(
            context: &Self::Context,
            freq: Self::Frequency,
            phase: &mut Self::Phase,
            shape: Self::Scalar,
            sync: &mut Self::Scalar,
            sync_mode: OscSync,
        ) -> osc::OscOutput<Self>;
        fn apply_note_offset(note: Self::Note, offset: Self::NoteOffset) -> Self::Note;
    }

    #[derive(Eq, PartialEq, Clone, Copy, Default)]
    pub enum EnvMode {
        #[default]
        Release,
        Attack,
        Decay,
    }

    pub trait EnvType<T: DspTypeAliases>: Copy + Default + From<T::Scalar> + PartialOrd {
        fn to_scalar(self) -> T::Scalar;
    }

    impl EnvType<i16> for EnvSignalFxP {
        fn to_scalar(self) -> Scalar {
            Scalar::saturating_from_num(self)
        }
    }

    impl<T: crate::devices::Float> EnvType<T> for T {
        fn to_scalar(self) -> Self {
            self
        }
    }

    pub trait EnvOps: DspTypeAliases {
        const SIGNAL_MIN: Self::EnvSignal;
        const SIGNAL_MAX: Self::EnvSignal;
        const ATTACK_THRESHOLD: Self::EnvSignal;
        const GATE_THRESHOLD: Self::Sample;
        const ADR_DEFAULT: Self::EnvParam;
        fn calc_env(
            context: &Self::Context,
            setpoint: Self::EnvSignal,
            setpoint_old: Self::EnvSignal,
            last: Self::EnvSignal,
            rise_time: Self::EnvParam,
        ) -> Self::EnvSignal;
    }

    pub trait FiltOps: DspTypeAliases {
        const RES_MAX: Self::Scalar;
        type FiltGain;
        type FiltFeedback: Default;
        fn prewarped_gain(context: &Self::Context, cutoff: Self::Note) -> Self::FiltGain;
        fn calc_filt(
            context: &Self::Context,
            signal: Self::Sample,
            cutoff: Self::Note,
            resonance: Self::Scalar,
            low_z: &mut Self::FiltFeedback,
            band_z: &mut Self::FiltFeedback,
        ) -> filt::FiltOutput<Self>;
    }

    pub trait LfoOps: DspTypeAliases + OscOps {
        fn phase_per_smp(context: &Self::Context, frequency: Self::LfoFreq) -> Self::Phase;
        fn calc_lfo(
            phase: Self::Phase,
            wave: lfo::LfoWave,
            rands: &[Self::Sample; 2],
        ) -> Self::Sample;
    }
}

pub trait DspType<_T>:
    Copy + Default + Add<Self, Output = Self> + Sub<Self, Output = Self> + PartialOrd
{
    const PI: Self;
    const TAU: Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn saturating_add(self, rhs: Self) -> Self;
    fn multiply(self, rhs: Self) -> Self;
    fn divide_by_two(self) -> Self;
}

/// A trait encompassing the different sample data types (e.g. 16 bit fixed,
/// 32 bit float, etc).
pub trait DspTypeAliases: Sized {
    /// A type representing a sample of audio data
    type Sample: DspType<Self>;
    /// A type representing a MIDI note number
    type Note: DspType<Self>;
    /// A type representing an offset to a MIDI note number
    type NoteOffset: DspType<Self>;
    /// A type representing a frequency
    type Frequency: DspType<Self>;
    /// A type representing a value between 0 and 1
    type Scalar: DspType<Self>;
    /// A type representing a value between -1 and 1
    type IScalar: DspType<Self>;
    /// A type representing a parameter to an envelope
    type EnvParam: DspType<Self>;
    /// A type representing internal envelope signal levels
    type EnvSignal: detail::EnvType<Self>;
    /// A type representing the phase of a sinusoid
    type Phase: DspType<Self>;
    /// A type representing the frequency of a LFO
    type LfoFreq: DspType<Self>;
    /// Type-specific context information
    type Context;
}

pub trait DspFormat:
    Copy + DspTypeAliases + detail::OscOps + detail::EnvOps + detail::FiltOps + detail::LfoOps
{
    fn default_note() -> Self::Note;
    fn note_to_freq(note: Self::Note) -> Self::Frequency;
    fn scale_sample(smp: Self::Sample, scalar: Self::Scalar) -> Self::Sample;
    fn sample_from_fixed(value: IScalarFxP) -> Self::Sample;
}

impl<T: crate::devices::Float> DspTypeAliases for T {
    type Sample = T;
    type Note = T;
    type NoteOffset = T;
    type Frequency = T;
    type Scalar = T;
    type IScalar = T;
    type EnvParam = T;
    type EnvSignal = T;
    type Phase = T;
    type LfoFreq = T;
    type Context = Context<T>;
}

///Helper trait to make constraint bounds less painful for floating point types
pub trait DspFloat:
    crate::devices::Float
    + DspTypeAliases<
        Sample = Self,
        Note = Self,
        NoteOffset = Self,
        Frequency = Self,
        Scalar = Self,
        IScalar = Self,
        EnvParam = Self,
        EnvSignal = Self,
        Phase = Self,
        LfoFreq = Self,
        Context = Context<Self>,
    >
{
}

impl DspFloat for f32 {}
impl DspFloat for f64 {}

impl<T: DspFloat + detail::OscOps + From<IScalarFxP>> DspFormat for T {
    fn default_note() -> Self::Note {
        Self::from_u16(69)
    }
    fn note_to_freq(note: Self::Note) -> Self::Frequency {
        T::midi_to_freq(note)
    }
    fn scale_sample(smp: Self::Sample, scalar: Self::Scalar) -> Self::Sample {
        smp * scalar
    }
    fn sample_from_fixed(value: IScalarFxP) -> Self::Sample {
        value.into()
    }
}

impl DspFormat for i16 {
    fn default_note() -> Self::Note {
        const DEFAULT: Note = Note::lit("69");
        DEFAULT
    }
    fn note_to_freq(note: Note) -> Frequency {
        crate::fixedmath::midi_note_to_frequency(note)
    }
    fn scale_sample(smp: Self::Sample, scalar: Self::Scalar) -> Self::Sample {
        crate::fixedmath::apply_scalar_i(smp, scalar)
    }
    fn sample_from_fixed(value: IScalarFxP) -> Self::Sample {
        Sample::from_num(value)
    }
}

impl<T: crate::devices::Float> DspType<T> for T {
    const PI: Self = Self::PI;
    const TAU: Self = Self::TAU;
    fn zero() -> Self {
        T::ZERO
    }
    fn one() -> Self {
        T::ONE
    }
    // Don't have to worry about floating point overflow
    fn saturating_add(self, rhs: Self) -> Self {
        self + rhs
    }
    fn multiply(self, rhs: Self) -> Self {
        self * rhs
    }
    fn divide_by_two(self) -> Self {
        self / Self::TWO
    }
}

impl DspTypeAliases for i16 {
    type Sample = Sample;
    type Note = Note;
    type NoteOffset = SignedNoteFxP;
    type Frequency = Frequency;
    type Scalar = Scalar;
    type IScalar = IScalarFxP;
    type EnvParam = EnvParamFxP;
    type EnvSignal = EnvSignalFxP;
    type Phase = PhaseFxP;
    type LfoFreq = LfoFreqFxP;
    type Context = ContextFxP;
}

impl<T: crate::Fixed16> DspType<i16> for T {
    const PI: Self = T::PI;
    const TAU: Self = T::TAU;
    fn zero() -> Self {
        Self::ZERO
    }
    fn one() -> Self {
        Self::ONE_OR_MAX
    }
    fn saturating_add(self, rhs: Self) -> Self {
        <Self as Fixed>::saturating_add(self, rhs)
    }
    fn multiply(self, rhs: Self) -> Self {
        <Self as crate::Fixed16>::multiply(self, rhs)
    }
    fn divide_by_two(self) -> Self {
        self.unwrapped_shr(1)
    }
}

impl DspType<i16> for Frequency {
    const PI: Self = Frequency::PI;
    const TAU: Self = Frequency::TAU;
    fn zero() -> Self {
        Self::ZERO
    }
    fn one() -> Self {
        Self::ONE
    }
    fn saturating_add(self, rhs: Self) -> Self {
        <Self as Fixed>::saturating_add(self, rhs)
    }
    fn multiply(self, rhs: Self) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    fn divide_by_two(self) -> Self {
        self.unwrapped_shr(1)
    }
}

impl DspType<i16> for PhaseFxP {
    const PI: Self = PhaseFxP::PI;
    const TAU: Self = PhaseFxP::TAU;
    fn zero() -> Self {
        Self::ZERO
    }
    fn one() -> Self {
        Self::ONE
    }
    fn saturating_add(self, rhs: Self) -> Self {
        <Self as Fixed>::saturating_add(self, rhs)
    }
    fn multiply(self, rhs: Self) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    fn divide_by_two(self) -> Self {
        self.unwrapped_shr(1)
    }
}

pub trait Source<T> {
    type It<'a>: Iterator<Item = T>
    where
        Self: 'a;
    fn get<'a>(&'a mut self) -> Self::It<'a>;
}

pub struct IteratorSource<I: Iterator> {
    data: I,
}

impl<I: Iterator> Source<I::Item> for IteratorSource<I> {
    type It<'a> = &'a mut I where I: 'a;
    fn get(&mut self) -> &mut I {
        &mut self.data
    }
}

impl<I: Iterator> From<I> for IteratorSource<I> {
    fn from(value: I) -> Self {
        Self { data: value }
    }
}
