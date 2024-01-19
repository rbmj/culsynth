use super::*;
use core::ops::{Add, Neg, Sub};
use fixed::traits::FromFixed;
use fixedmath::scale_fixedfloat;

/// A trait encompassing the different sample data types (e.g. 16 bit fixed,
/// 32 bit float, etc).
pub trait DspFormat:
    DspFormatBase
    + devices::osc::detail::OscOps
    + devices::env::detail::EnvOps
    + devices::filt::detail::FiltOps
    + devices::lfo::detail::LfoOps
    + voice::modulation::detail::ModulatorOps
{
}

/// Type aliases defining the data types of various internal signals within the
/// synthesizer.  This is primariliy to be generic over fixed/floating point
pub trait DspFormatBase: Sized + Copy + Default + Send {
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
    type EnvSignal: devices::env::detail::EnvType<Self> + Send;
    /// A type representing the phase of a sinusoid
    type Phase: DspType<Self>;
    /// A type representing the frequency of a LFO
    type LfoFreq: DspType<Self>;
    /// A type representing a sample that *may* have higher precision/range
    type WideSample: Copy + Default + Add<Self::WideSample, Output = Self::WideSample>;
    /// Type-specific context information
    type Context: Send + crate::context::GetContext;
    /// Provide a value of the default note, definied as A440 (MIDI NN #69)
    fn default_note() -> Self::Note;
    /// Convert a midi Note into a Frequency
    fn note_to_freq(note: Self::Note) -> Self::Frequency;
    /// Convert a signed scalar to a Sample
    fn sample_from_fixed(value: crate::IScalarFxP) -> Self::Sample;
    /// Convert a sample to a 32 bit float
    fn sample_to_float(value: Self::Sample) -> f32;
    /// Widen a sample to a WideSample
    fn widen_sample(smp: Self::Sample) -> Self::WideSample;
    /// Narrow a WideSample to a Sample
    fn narrow_sample(wide_smp: Self::WideSample) -> Self::Sample;
    /// Convert a Scalar to a Note (where 0 maps to the lowest representable
    /// note, and 1 maps to the highest)
    fn note_from_scalar(scalar: Self::Scalar) -> Self::Note;
    /// Apply a note offset
    fn apply_note_offset(note: Self::Note, offset: Self::NoteOffset) -> Self::Note;
}

///Helper trait to make constraint bounds less painful for floating point types
pub trait DspFloat:
    crate::Float
    + Send
    + FromFixed
    + DspFormatBase<
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
        WideSample = Self,
        Context = context::Context<Self>,
    >
{
}

/// A trait to simplify common operations on DSP Types.  This is used to
/// maximize the amount of code that can be agnostic to fixed and floating point
pub trait DspType<T: DspFormatBase>:
    Copy + Default + Send + Add<Self, Output = Self> + Sub<Self, Output = Self> + PartialOrd
{
    /// A constant representing the value PI (3.14159...)
    const PI: Self;
    /// A constant representing the value 2*[PI]
    const TAU: Self;
    /// Returns zero
    fn zero() -> Self;
    /// Returns one
    fn one() -> Self;
    /// This function will perform a saturating addition for fixed-point types,
    /// and a normal addition for floating-point types
    ///
    /// Used for when saturation is desired to avoid overflows, not correctness
    fn dsp_saturating_add(self, rhs: Self) -> Self;
    /// Multiply this type with itself.  This trait does not provide any
    /// specified behavior for fixed-point overflow.
    fn multiply(self, rhs: Self) -> Self;
    /// Divide a value by two
    fn divide_by_two(self) -> Self;
    /// Multiply this type by a Scalar.  This will never overflow
    /// (by definition, the result will always be smaller)
    fn scale(self, rhs: T::Scalar) -> Self;
}

// Floating-point implementation:

impl<T: DspFloat> DspFormat for T {}

impl<T: Float + Send> DspFormatBase for T
where
    T: From<crate::IScalarFxP> + From<crate::NoteFxP>,
{
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
    type WideSample = T;
    type Context = context::Context<T>;
    fn default_note() -> Self::Note {
        Self::from_u16(69)
    }
    fn note_to_freq(note: Self::Note) -> Self::Frequency {
        T::midi_to_freq(note)
    }
    fn sample_from_fixed(value: IScalarFxP) -> Self::Sample {
        value.into()
    }
    fn sample_to_float(value: Self::Sample) -> f32 {
        value.as_f32()
    }
    fn widen_sample(smp: Self::Sample) -> Self::WideSample {
        smp
    }
    fn narrow_sample(wide_smp: Self::WideSample) -> Self::Sample {
        wide_smp
    }
    fn note_from_scalar(scalar: Self::Scalar) -> Self::Note {
        let note_max: Self = NoteFxP::MAX.into();
        note_max * scalar
    }
    fn apply_note_offset(note: Self::Note, offset: Self::NoteOffset) -> Self::Note {
        note + offset
    }
}

impl DspFloat for f32 {}
impl DspFloat for f64 {}

impl<T: Float + Send> DspType<T> for T
where
    T: From<crate::IScalarFxP> + From<crate::NoteFxP>,
{
    const PI: Self = Self::PI;
    const TAU: Self = Self::TAU;
    fn zero() -> Self {
        T::ZERO
    }
    fn one() -> Self {
        T::ONE
    }
    // Don't have to worry about floating point overflow
    fn dsp_saturating_add(self, rhs: Self) -> Self {
        self + rhs
    }
    fn multiply(self, rhs: Self) -> Self {
        self * rhs
    }
    fn divide_by_two(self) -> Self {
        self / Self::TWO
    }
    fn scale(self, rhs: Self) -> Self {
        self * rhs
    }
}

// 16-bit fixed point:

impl DspFormat for i16 {}

impl DspFormatBase for i16 {
    type Sample = SampleFxP;
    type Note = NoteFxP;
    type NoteOffset = SignedNoteFxP;
    type Frequency = FrequencyFxP;
    type Scalar = ScalarFxP;
    type IScalar = IScalarFxP;
    type EnvParam = EnvParamFxP;
    type EnvSignal = devices::env::detail::EnvSignalFxP;
    type Phase = PhaseFxP;
    type LfoFreq = LfoFreqFxP;
    type WideSample = WideSampleFxP;
    type Context = context::ContextFxP;
    fn default_note() -> Self::Note {
        const DEFAULT: NoteFxP = NoteFxP::lit("69");
        DEFAULT
    }
    fn note_to_freq(note: NoteFxP) -> FrequencyFxP {
        crate::fixedmath::midi_note_to_frequency(note)
    }
    fn sample_from_fixed(value: IScalarFxP) -> Self::Sample {
        SampleFxP::from_num(value)
    }
    fn sample_to_float(value: Self::Sample) -> f32 {
        value.into()
    }
    fn widen_sample(smp: Self::Sample) -> Self::WideSample {
        crate::fixedmath::widen_i(smp)
    }
    fn narrow_sample(wide_smp: Self::WideSample) -> SampleFxP {
        SampleFxP::saturating_from_num(wide_smp)
    }
    fn note_from_scalar(scalar: ScalarFxP) -> NoteFxP {
        NoteFxP::from_bits(scalar.to_bits())
    }
    fn apply_note_offset(note: NoteFxP, offset: SignedNoteFxP) -> NoteFxP {
        note.saturating_add_signed(offset)
    }
}

impl<T: Fixed16 + Send> DspType<i16> for T {
    const PI: Self = T::PI;
    const TAU: Self = T::TAU;
    fn zero() -> Self {
        Self::ZERO
    }
    fn one() -> Self {
        Self::ONE_OR_MAX
    }
    fn dsp_saturating_add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
    fn multiply(self, rhs: Self) -> Self {
        self.multiply_fixed(rhs)
    }
    fn divide_by_two(self) -> Self {
        self.unwrapped_shr(1)
    }
    fn scale(self, rhs: ScalarFxP) -> Self {
        self.scale_fixed(rhs)
    }
}

impl DspType<i16> for FrequencyFxP {
    const PI: Self = FrequencyFxP::PI;
    const TAU: Self = FrequencyFxP::TAU;
    fn zero() -> Self {
        Self::ZERO
    }
    fn one() -> Self {
        Self::ONE
    }
    fn dsp_saturating_add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
    fn multiply(self, rhs: Self) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    fn divide_by_two(self) -> Self {
        self.unwrapped_shr(1)
    }
    fn scale(self, rhs: ScalarFxP) -> Self {
        scale_fixedfloat(self, rhs)
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
    fn dsp_saturating_add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
    fn multiply(self, rhs: Self) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    fn divide_by_two(self) -> Self {
        self.unwrapped_shr(1)
    }
    fn scale(self, rhs: ScalarFxP) -> Self {
        let (abs, neg) = (self.unsigned_abs(), self.is_negative());
        let mut scaled = Self::from_num(scale_fixedfloat(abs, rhs));
        if neg {
            scaled = scaled.neg();
        }
        scaled
    }
}

/*
impl DspType<i16> for WideSampleFxP {
    const PI: Self = WideSampleFxP::PI;
    const TAU: Self = WideSampleFxP::TAU;
    fn zero() -> Self {
        Self::ZERO
    }
    fn one() -> Self {
        Self::ONE
    }
    fn dsp_saturating_add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
    fn multiply(self, rhs: Self) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    fn divide_by_two(self) -> Self {
        self.unwrapped_shr(1)
    }
    fn scale(self, rhs: ScalarFxP) -> Self {
        let (abs, neg) = (self.unsigned_abs(), self.is_negative());
        let mut scaled = Self::from_num(scale_fixedfloat(abs, rhs));
        if neg {
            scaled = scaled.neg();
        }
        scaled
    }
}
*/
