use super::*;

use crate::fixedmath::{one_over_one_minus, one_over_one_plus_scalar, scale_fixedfloat, sin_pi};
use crate::{FrequencyFxP, IScalarFxP, PhaseFxP, ScaleOutputFxP};

/// Parameters for an [Osc]
#[derive(Clone, Default)]
pub struct OscParams<T: DspFormatBase> {
    /// Tuning, as an offset in MIDI note number
    pub tune: T::NoteOffset,
    /// The amount of phase distortion to apply to the waveform, from 0 to 1
    pub shape: T::Scalar,
}

impl<T: DspFloat> From<&OscParams<i16>> for OscParams<T> {
    fn from(value: &OscParams<i16>) -> Self {
        Self {
            tune: value.tune.to_num(),
            shape: value.shape.to_num(),
        }
    }
}

/// Parameters for [SyncedOscs]
#[derive(Clone, Default)]
pub struct SyncedOscsParams<T: DspFormatBase> {
    /// Parameters for the primary oscillator
    pub primary: OscParams<T>,
    /// Parameters for the secondary oscillator
    pub secondary: OscParams<T>,
    /// True if oscillator sync has been enabled - when false, both oscillators
    /// will run independently
    pub sync: bool,
}

impl<T: DspFloat> From<&SyncedOscsParams<i16>> for SyncedOscsParams<T> {
    fn from(value: &SyncedOscsParams<i16>) -> Self {
        Self {
            primary: (&value.primary).into(),
            secondary: (&value.secondary).into(),
            sync: value.sync,
        }
    }
}
/// The output of an oscillator.
#[derive(Clone, Default)]
pub struct OscOutput<T: DspFormatBase> {
    /// The sine wave output
    pub sin: T::Sample,
    /// The square wave output
    pub sq: T::Sample,
    /// The triangle wave output
    pub tri: T::Sample,
    /// The sawtooth wave output
    pub saw: T::Sample,
}

/// Output from [SyncedOscs]
#[derive(Clone, Default)]
pub struct SyncedOscsOutput<T: DspFormatBase> {
    /// Output from the primary oscillator
    pub primary: OscOutput<T>,
    /// Output from the secondary oscillator
    pub secondary: OscOutput<T>,
}

/// A variable-frequency, audio-rate oscillator
///
/// This models an oscillator with Sine, Square, Triangle, and Sawtooth wave
/// outputs.  It is tunable across the entire range of MIDI note numbers and
/// includes code to support oscillator sync (see [SyncedOscs]).  The oscillator
/// also supports phase distortion via the `shape` parameter
/// (see [OscParams::shape]), which will modify the balance between the
/// positive and negative phase portions of the waveform while maintaining the
/// same overall fundamental frequency.
///
/// This device returns each individual waveform as a separate output.  For
/// convenience, devices are provided that premix these waveforms into a single
/// output with parameterized gains (see [MixOsc] and [SyncedMixOscs]).
///
/// This struct implements [Device], taking a Note as input and [OscParams] as
/// parameters and returning a [OscOutput], which contains all of the output
/// waveforms from the oscillator.
#[derive(Clone, Default)]
pub struct Osc<T: DspFormat> {
    phase_over_pi: T::Phase,
}

impl<T: DspFormat> Osc<T> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            phase_over_pi: T::Phase::zero(),
        }
    }
    fn next_with_sync(
        &mut self,
        context: &T::Context,
        note: T::Note,
        params: OscParams<T>,
        mut sync: OscSync<T>,
    ) -> (OscOutput<T>, OscSync<T>) {
        let freq = T::note_to_freq(T::apply_note_offset(note, params.tune));
        let warped_phase = T::warp_phase(self.phase_over_pi, params.shape);
        let out = T::calc_waveforms(warped_phase);
        (self.phase_over_pi, sync) = T::advance_phase(context, freq, self.phase_over_pi, sync);
        (out, sync)
    }
}

impl<T: DspFormat> Device<T> for Osc<T> {
    type Input = T::Note;
    type Params = OscParams<T>;
    type Output = OscOutput<T>;
    fn next(&mut self, context: &T::Context, note: T::Note, params: OscParams<T>) -> Self::Output {
        let (out, _) = self.next_with_sync(context, note, params, OscSync::Off);
        out
    }
}

/// A synced pair of [Osc]s.  The secondary oscillator will be synced
/// to the primary oscillator.
///
/// This implements [Device], taking a Note as input and a [SyncedOscsParams]
/// as parameters.  It outputs a [SyncedOscsOutput], which contains the output
/// signals from both underlying oscillators.
#[derive(Clone, Default)]
pub struct SyncedOscs<T: DspFormat> {
    primary: Osc<T>,
    secondary: Osc<T>,
}

impl<T: DspFormat> SyncedOscs<T> {
    /// Constructor
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T: DspFormat> Device<T> for SyncedOscs<T> {
    type Input = T::Note;
    type Params = SyncedOscsParams<T>;
    type Output = SyncedOscsOutput<T>;
    fn next(
        &mut self,
        context: &T::Context,
        note: T::Note,
        params: SyncedOscsParams<T>,
    ) -> Self::Output {
        let sync = if params.sync {
            OscSync::<T>::Primary
        } else {
            OscSync::<T>::Off
        };
        let (pri_out, sync) = self.primary.next_with_sync(context, note, params.primary, sync);
        let (sec_out, _) = self.secondary.next_with_sync(context, note, params.secondary, sync);
        SyncedOscsOutput {
            primary: pri_out,
            secondary: sec_out,
        }
    }
}

pub(crate) mod detail {
    use super::*;

    #[derive(PartialEq, Clone, Copy)]
    pub enum OscSync<T: DspFormatBase> {
        /// No sync behavior - do not calculate
        Off,
        /// This is the primary oscillator, and sync is enabled
        Primary,
        /// This is the secondary oscillator, sync is enabled, and the master
        /// completed a full phase at some portion through this sample
        Secondary(T::Scalar),
    }

    pub trait OscOps: crate::DspFormatBase {
        fn advance_phase(
            context: &Self::Context,
            freq: Self::Frequency,
            phase_over_pi: Self::Phase,
            sync: OscSync<Self>,
        ) -> (Self::Phase, OscSync<Self>);
        fn calc_waveforms(phase_over_pi: Self::Phase) -> OscOutput<Self>;
        fn warp_phase(phase_over_pi: Self::Phase, shape: Self::Scalar) -> Self::Phase;
    }
}

use detail::OscSync;

// This section contains the actual DSP logic for both fixed and floating point

impl<T: DspFloat> detail::OscOps for T {
    fn calc_waveforms(phase: Self::Phase) -> OscOutput<Self> {
        OscOutput {
            sin: (phase * T::PI).fsin(),
            sq: if phase > T::ZERO {
                T::ONE
            } else {
                T::ONE.neg()
            },
            saw: phase,
            tri: T::ONE - (phase.abs() * T::TWO),
        }
    }
    fn warp_phase(phase_over_pi: Self::Phase, shape: Self::Scalar) -> Self::Phase {
        if phase_over_pi <= shape {
            ((T::ONE + phase_over_pi) / (T::ONE + shape)) - T::ONE
        } else {
            (phase_over_pi - shape) / (T::ONE - shape)
        }
    }
    fn advance_phase(
        ctx: &Self::Context,
        freq: Self::Frequency,
        mut phase: Self::Phase,
        sync: OscSync<T>,
    ) -> (Self::Phase, OscSync<T>) {
        let phase_per_sample = freq * T::TWO / ctx.sample_rate;
        let mut sync_out = osc::OscSync::<T>::Off;
        match sync {
            OscSync::Off => {
                phase = phase + phase_per_sample;
            }
            OscSync::Primary => {
                let old_phase = phase;
                phase = phase + phase_per_sample;
                // calculate what time in this sampling period the phase crossed zero:
                if old_phase < T::ZERO && phase >= T::ZERO {
                    sync_out = OscSync::Secondary(phase / phase_per_sample);
                }
            }
            OscSync::Secondary(primary_xpt) => {
                phase = phase_per_sample * primary_xpt;
            }
        }
        if phase >= T::ONE {
            phase = phase - T::TWO;
        }
        (phase, sync_out)
    }
}

impl detail::OscOps for i16 {
    fn calc_waveforms(phase: PhaseFxP) -> OscOutput<Self> {
        OscOutput {
            saw: SampleFxP::from_num(phase),
            sq: if phase < PhaseFxP::ZERO {
                SampleFxP::NEG_ONE
            } else {
                SampleFxP::ONE
            },
            tri: SampleFxP::ONE - (SampleFxP::from_num(phase).abs().unwrapped_shl(1)),
            sin: SampleFxP::from_num(sin_pi(IScalarFxP::from_num(phase))),
        }
    }
    fn warp_phase(phase_over_pi: PhaseFxP, shape: ScalarFxP) -> PhaseFxP {
        use crate::fixedmath::{U1F15, U1F31};
        let phase_over_pi = IScalarFxP::from_num(phase_over_pi);
        if phase_over_pi <= IScalarFxP::from_num(shape) {
            let phase_plus_one = U1F15::ONE.add_signed(phase_over_pi);
            let warped_offset = one_over_one_plus_scalar(shape).wide_mul(phase_plus_one);
            PhaseFxP::NEG_ONE.add_unsigned(U1F31::from_num(warped_offset))
        } else {
            let phase_minus_shape = ScalarFxP::from_num(phase_over_pi) - shape;
            PhaseFxP::from_num(scale_fixedfloat(
                one_over_one_minus(shape),
                phase_minus_shape,
            ))
        }
    }
    fn advance_phase(
        ctx: &ContextFxP,
        freq: FrequencyFxP,
        phase: PhaseFxP,
        sync: OscSync<i16>,
    ) -> (PhaseFxP, OscSync<i16>) {
        let mut sync_out = OscSync::<i16>::Off;
        let phase_per_smp = PhaseFxP::from_num(
            scale_fixedfloat(freq, ctx.sample_rate.frac_32768_sr()).unwrapped_shr(14),
        );
        let mut new_phase = phase.wrapping_add(phase_per_smp);
        match sync {
            OscSync::Off => {}
            OscSync::Primary => {
                // calculate what time in this sampling period the phase crossed zero:
                if new_phase > PhaseFxP::ZERO && phase <= PhaseFxP::ZERO {
                    // we need to calculate 1 - (new_phase/ phase_per_sample)
                    let proportion = scale_fixedfloat(
                        fixedmath::inverse(ScalarFxP::from_num(phase_per_smp)),
                        ScalarFxP::from_num(new_phase),
                    );
                    sync_out = OscSync::Secondary(ScalarFxP::saturating_from_num(
                        ScaleOutputFxP::ONE - proportion,
                    ));
                }
            }
            OscSync::Secondary(primary_xpt) => {
                let per_smp = phase_per_smp.unsigned_abs();
                new_phase = PhaseFxP::from_num(scale_fixedfloat(per_smp, primary_xpt));
            }
        }
        (new_phase, sync_out)
    }
}
