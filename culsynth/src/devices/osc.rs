use super::*;

use crate::Float;
use crate::{FrequencyFxP, PhaseFxP};

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
    phase: T::Phase,
}

impl<T: DspFormat> Osc<T> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            phase: T::Phase::zero(),
        }
    }
    fn next_with_sync(
        &mut self,
        context: &T::Context,
        note: T::Note,
        params: OscParams<T>,
        mut sync_val: T::Scalar,
        sync_mode: detail::OscSync,
    ) -> (OscOutput<T>, T::Scalar) {
        let freq = T::note_to_freq(T::apply_note_offset(note, params.tune));
        let out = T::calc_waveforms(self.phase);
        (self.phase, sync_val) =
            T::advance_phase(context, freq, self.phase, params.shape, sync_val, sync_mode);
        (out, sync_val)
    }
}

impl<T: DspFormat> Device<T> for Osc<T> {
    type Input = T::Note;
    type Params = OscParams<T>;
    type Output = OscOutput<T>;
    fn next(&mut self, context: &T::Context, note: T::Note, params: OscParams<T>) -> Self::Output {
        let (out, _) = self.next_with_sync(
            context,
            note,
            params,
            T::Scalar::zero(),
            detail::OscSync::Off,
        );
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
        let sync_val = if params.sync {
            T::Scalar::one()
        } else {
            T::Scalar::zero()
        };
        let (pri_out, sync_val) =
            self.primary
                .next_with_sync(context, note, params.primary, sync_val, OscSync::Primary);
        let (sec_out, _) = self.secondary.next_with_sync(
            context,
            note,
            params.secondary,
            sync_val,
            OscSync::Secondary,
        );
        SyncedOscsOutput {
            primary: pri_out,
            secondary: sec_out,
        }
    }
}

pub(crate) mod detail {
    use super::*;

    #[derive(PartialEq, Clone, Copy)]
    pub enum OscSync {
        /// No sync behavior - do not calculate
        Off,
        /// This is the master oscillator
        Primary,
        /// This is the slave oscillator
        Secondary,
    }

    pub trait OscOps: crate::DspFormatBase {
        const FRAC_2_PI: Self::Scalar;
        fn advance_phase(
            context: &Self::Context,
            freq: Self::Frequency,
            phase: Self::Phase,
            shape: Self::Scalar,
            sync: Self::Scalar,
            sync_mode: OscSync,
        ) -> (Self::Phase, Self::Scalar);
        fn calc_waveforms(phase: Self::Phase) -> OscOutput<Self>;
    }
}

use detail::OscSync;

// This section contains the actual DSP logic for both fixed and floating point

impl<T: DspFloat> detail::OscOps for T {
    const FRAC_2_PI: T = <T as Float>::FRAC_2_PI;
    fn calc_waveforms(phase: Self::Phase) -> OscOutput<Self> {
        let mut out = osc::OscOutput::<T>::default();
        //generate waveforms (piecewise defined)
        let frac_2phase_pi = phase * <Self as detail::OscOps>::FRAC_2_PI;
        out.saw = frac_2phase_pi / T::TWO;
        if phase < T::ZERO {
            out.sq = T::ONE.neg();
            if phase < T::FRAC_PI_2.neg() {
                // phase in [-pi, pi/2)
                // sin(x) = -cos(x+pi/2)
                out.sin = (phase + T::FRAC_PI_2).fcos().neg();
                // Subtract (1+1) because traits :eyeroll:
                out.tri = frac_2phase_pi.neg() - T::TWO;
            } else {
                // phase in [-pi/2, 0)
                out.sin = phase.fsin();
                //triangle
                out.tri = frac_2phase_pi;
            }
        } else {
            out.sq = T::ONE;
            if phase < T::FRAC_PI_2 {
                // phase in [0, pi/2)
                out.sin = phase.fsin();
                out.tri = frac_2phase_pi;
            } else {
                // phase in [pi/2, pi)
                // sin(x) = cos(x-pi/2)
                out.sin = (phase - T::FRAC_PI_2).fcos();
                out.tri = T::TWO - frac_2phase_pi;
            }
        }
        out
    }
    fn advance_phase(
        ctx: &Self::Context,
        freq: Self::Frequency,
        mut phase: Self::Phase,
        shape: Self::Scalar,
        mut sync: Self::Scalar,
        sync_mode: osc::OscSync,
    ) -> (Self::Phase, Self::Scalar) {
        let phase_per_sample = freq * T::TAU / ctx.sample_rate;
        let shp = if shape < T::SHAPE_CLIP {
            shape
        } else {
            T::SHAPE_CLIP
        };
        // Handle slave oscillator resetting phase if master crosses:
        if sync_mode == OscSync::Secondary && sync != T::ZERO {
            phase = T::ZERO;
        }
        let phase_per_smp_adj = if phase < T::ZERO {
            phase_per_sample * (T::ONE / (T::ONE + shp))
        } else {
            phase_per_sample * (T::ONE / (T::ONE - shp))
        };
        let old_phase = phase;
        match sync_mode {
            OscSync::Off => {
                phase = phase + phase_per_smp_adj;
            }
            OscSync::Primary => {
                phase = phase + phase_per_smp_adj;
                // calculate what time in this sampling period the phase crossed zero:
                sync = if sync != T::ZERO && old_phase < T::ZERO && phase >= T::ZERO {
                    T::ONE - (phase / phase_per_smp_adj)
                } else {
                    T::ZERO
                };
            }
            OscSync::Secondary => {
                phase = phase
                    + if sync != T::ZERO {
                        phase_per_smp_adj * (T::ONE - sync)
                    } else {
                        phase_per_smp_adj
                    };
            }
        }
        // make sure we calculate the correct new phase on transitions for assymmetric waves:
        // check if we've crossed from negative to positive phase
        if old_phase < T::ZERO && phase > T::ZERO && shp != T::ZERO {
            // need to multiply residual phase i.e. (phase - 0) by (1+k)/(1-k)
            // where k is the shape, so no work required if shape is 0
            phase = phase * (T::ONE + shp) / (T::ONE - shp);
        }
        // Check if we've crossed from positive phase back to negative:
        if phase >= T::PI {
            // if we're a symmetric wave this is as simple as just subtract 2pi
            if shp == T::ZERO {
                phase = phase - T::TAU;
            } else {
                // if assymmetric we have to multiply residual phase i.e. phase - pi
                // by (1-k)/(1+k) where k is the shape:
                let delta = (phase - T::PI) * (T::ONE - shp) / (T::ONE + shp);
                // add new change in phase to our baseline, -pi:
                phase = delta - T::PI;
            }
        }
        (phase, sync)
    }
}

impl detail::OscOps for i16 {
    const FRAC_2_PI: ScalarFxP = ScalarFxP::lit("0x0.a2fa");
    fn calc_waveforms(phase: Self::Phase) -> OscOutput<Self> {
        use crate::fixed_traits::Fixed16;
        use fixedmath::{cos_fixed, sin_fixed};
        const TWO: SampleFxP = SampleFxP::lit("2");
        let frac_2phase_pi = SampleFxP::from_num(phase).scale_fixed(Self::FRAC_2_PI);
        let mut ret = OscOutput::<i16> {
            saw: frac_2phase_pi.unwrapped_shr(1), // not piecewise-defined
            ..Default::default()
        };
        //All other functions are piecewise-defined:
        if phase < 0 {
            ret.sq = SampleFxP::NEG_ONE;
            if phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {
                // phase in [-pi, pi/2)
                // Use the identity sin(x) = -cos(x+pi/2) since our taylor series
                // approximations are centered about zero and this will be more accurate
                ret.sin =
                    cos_fixed(SampleFxP::from_num(phase + PhaseFxP::FRAC_PI_2)).unwrapped_neg();
                ret.tri = frac_2phase_pi.unwrapped_neg() - TWO;
            } else {
                // phase in [-pi/2, 0)
                ret.sin = sin_fixed(SampleFxP::from_num(phase));
                ret.tri = frac_2phase_pi;
            }
        } else {
            ret.sq = SampleFxP::ONE;
            if phase < PhaseFxP::FRAC_PI_2 {
                // phase in [0, pi/2)
                ret.sin = sin_fixed(SampleFxP::from_num(phase));
                ret.tri = frac_2phase_pi;
            } else {
                // phase in [pi/2, pi)
                // sin(x) = cos(x-pi/2)
                ret.sin = cos_fixed(SampleFxP::from_num(phase - PhaseFxP::FRAC_PI_2));
                ret.tri = TWO - frac_2phase_pi;
            }
        }
        ret
    }
    fn advance_phase(
        ctx: &ContextFxP,
        freq: FrequencyFxP,
        mut phase: PhaseFxP,
        shape: ScalarFxP,
        mut sync: ScalarFxP,
        sync_mode: osc::OscSync,
    ) -> (PhaseFxP, ScalarFxP) {
        use fixedmath::{one_over_one_plus_highacc, scale_fixedfloat};
        // we need to divide by 2^12 here, but we're increasing the fractional part by 10
        // bits so we'll only actually shift by 2 places and then use a bitcast for the
        // remaining logical 10 bits:
        let phase_per_sample = fixedmath::U4F28::from_bits(
            scale_fixedfloat(freq, ctx.sample_rate.frac_2pi4096_sr())
                .unwrapped_shr(2)
                .to_bits(),
        );
        // Handle slave oscillator resetting phase if master crosses:
        if sync_mode == OscSync::Secondary && sync != ScalarFxP::ZERO {
            phase = PhaseFxP::ZERO;
        }
        // Adjust phase per sample for the shape parameter:
        let phase_per_smp_adj = PhaseFxP::from_num(if phase < PhaseFxP::ZERO {
            let (x, s) = one_over_one_plus_highacc(clip_shape(shape));
            fixedmath::scale_fixedfloat(phase_per_sample, x).unwrapped_shr(s)
        } else {
            fixedmath::scale_fixedfloat(phase_per_sample, one_over_one_minus_x(shape))
        });
        // Advance the oscillator's phase, and handle oscillator sync logic:
        let old_phase = phase;
        match sync_mode {
            OscSync::Off => {
                phase += phase_per_smp_adj;
            }
            OscSync::Primary => {
                phase += phase_per_smp_adj;
                // calculate what time in this sampling period the phase crossed zero:
                sync = if sync != ScalarFxP::ZERO
                    && old_phase < PhaseFxP::ZERO
                    && phase >= PhaseFxP::ZERO
                {
                    // we need to calculate 1 - (phase / phase_per_sample_adj)
                    let adj_s = ScalarFxP::from_num(phase_per_smp_adj.unwrapped_shr(2));
                    let x = fixedmath::U3F13::from_num(phase).wide_mul(inverse(adj_s));
                    let proportion = ScalarFxP::saturating_from_num(x.unwrapped_shr(2));
                    if proportion == ScalarFxP::MAX {
                        ScalarFxP::DELTA
                    } else {
                        ScalarFxP::MAX - proportion
                    }
                } else {
                    ScalarFxP::ZERO
                }
            }
            OscSync::Secondary => {
                phase += if sync != ScalarFxP::ZERO {
                    // Only advance phase for the portion of time after master crossed zero:
                    let scale = ScalarFxP::MAX - sync;
                    PhaseFxP::from_num(scale_fixedfloat(
                        fixedmath::U4F28::from_num(phase_per_smp_adj),
                        scale,
                    ))
                } else {
                    phase_per_smp_adj
                }
            }
        }
        // check if we've crossed from negative to positive phase
        if old_phase < PhaseFxP::ZERO && phase > PhaseFxP::ZERO && shape != ScalarFxP::ZERO {
            // need to multiply residual phase i.e. (phase - 0) by (1+k)/(1-k)
            // where k is the shape, so no work required if shape is 0
            let scaled = scale_fixedfloat(
                fixedmath::U4F28::from_num(phase),
                one_over_one_minus_x(shape),
            );
            let one_plus_shape =
                fixedmath::U1F15::from_num(clip_shape(shape)) + fixedmath::U1F15::ONE;
            phase = PhaseFxP::from_num(scale_fixedfloat(scaled, one_plus_shape));
        }
        // Check if we've crossed from positive phase back to negative:
        if phase >= PhaseFxP::PI {
            // if we're a symmetric wave this is as simple as just subtract 2pi
            if shape == ScalarFxP::ZERO {
                phase -= PhaseFxP::TAU;
            } else {
                // if assymmetric we have to multiply residual phase i.e. phase - pi
                // by (1-k)/(1+k) where k is the shape:
                let one_minus_shape = (ScalarFxP::MAX - clip_shape(shape)) + ScalarFxP::DELTA;
                // scaled = residual_phase * (1-k)
                let scaled = scale_fixedfloat(
                    fixedmath::U4F28::from_num(phase - PhaseFxP::PI),
                    one_minus_shape,
                );
                // new change in phase = scaled * 1/(1 + k)
                let (x, s) = one_over_one_plus_highacc(clip_shape(shape));
                let delta = scale_fixedfloat(scaled, x).unwrapped_shr(s);
                // add new change in phase to our baseline, -pi:
                phase = PhaseFxP::from_num(delta) - PhaseFxP::PI;
            }
        }
        (phase, sync)
    }
}

fn clip_shape(x: ScalarFxP) -> ScalarFxP {
    const CLIP_MAX: ScalarFxP = ScalarFxP::lit("0x0.F");
    if x > CLIP_MAX {
        CLIP_MAX
    } else {
        x
    }
}

fn inverse(x: ScalarFxP) -> crate::fixedmath::U8F8 {
    // For brevity in defining the lookup table:
    const fn lit(x: &str) -> crate::fixedmath::U8F8 {
        crate::fixedmath::U8F8::lit(x)
    }
    #[rustfmt::skip]
    const LOOKUP_TABLE: [crate::fixedmath::U8F8; 256] = [
        lit("0xff.ff"), lit("0xff.ff"), lit("0x80.00"), lit("0x55.55"),
        lit("0x40.00"), lit("0x33.33"), lit("0x2a.aa"), lit("0x24.92"),
        lit("0x20.00"), lit("0x1c.71"), lit("0x19.99"), lit("0x17.45"),
        lit("0x15.55"), lit("0x13.b1"), lit("0x12.49"), lit("0x11.11"),
        lit("0x10.00"), lit("0xf.0f"), lit("0xe.38"), lit("0xd.79"),
        lit("0xc.cc"), lit("0xc.30"), lit("0xb.a2"), lit("0xb.21"),
        lit("0xa.aa"), lit("0xa.3d"), lit("0x9.d8"), lit("0x9.7b"),
        lit("0x9.24"), lit("0x8.d3"), lit("0x8.88"), lit("0x8.42"),
        lit("0x8.00"), lit("0x7.c1"), lit("0x7.87"), lit("0x7.50"),
        lit("0x7.1c"), lit("0x6.eb"), lit("0x6.bc"), lit("0x6.90"),
        lit("0x6.66"), lit("0x6.3e"), lit("0x6.18"), lit("0x5.f4"),
        lit("0x5.d1"), lit("0x5.b0"), lit("0x5.90"), lit("0x5.72"),
        lit("0x5.55"), lit("0x5.39"), lit("0x5.1e"), lit("0x5.05"),
        lit("0x4.ec"), lit("0x4.d4"), lit("0x4.bd"), lit("0x4.a7"),
        lit("0x4.92"), lit("0x4.7d"), lit("0x4.69"), lit("0x4.56"),
        lit("0x4.44"), lit("0x4.32"), lit("0x4.21"), lit("0x4.10"),
        lit("0x4.00"), lit("0x3.f0"), lit("0x3.e0"), lit("0x3.d2"),
        lit("0x3.c3"), lit("0x3.b5"), lit("0x3.a8"), lit("0x3.9b"),
        lit("0x3.8e"), lit("0x3.81"), lit("0x3.75"), lit("0x3.69"),
        lit("0x3.5e"), lit("0x3.53"), lit("0x3.48"), lit("0x3.3d"),
        lit("0x3.33"), lit("0x3.29"), lit("0x3.1f"), lit("0x3.15"),
        lit("0x3.0c"), lit("0x3.03"), lit("0x2.fa"), lit("0x2.f1"),
        lit("0x2.e8"), lit("0x2.e0"), lit("0x2.d8"), lit("0x2.d0"),
        lit("0x2.c8"), lit("0x2.c0"), lit("0x2.b9"), lit("0x2.b1"),
        lit("0x2.aa"), lit("0x2.a3"), lit("0x2.9c"), lit("0x2.95"),
        lit("0x2.8f"), lit("0x2.88"), lit("0x2.82"), lit("0x2.7c"),
        lit("0x2.76"), lit("0x2.70"), lit("0x2.6a"), lit("0x2.64"),
        lit("0x2.5e"), lit("0x2.59"), lit("0x2.53"), lit("0x2.4e"),
        lit("0x2.49"), lit("0x2.43"), lit("0x2.3e"), lit("0x2.39"),
        lit("0x2.34"), lit("0x2.30"), lit("0x2.2b"), lit("0x2.26"),
        lit("0x2.22"), lit("0x2.1d"), lit("0x2.19"), lit("0x2.14"),
        lit("0x2.10"), lit("0x2.0c"), lit("0x2.08"), lit("0x2.04"),
        lit("0x2.00"), lit("0x1.fc"), lit("0x1.f8"), lit("0x1.f4"),
        lit("0x1.f0"), lit("0x1.ec"), lit("0x1.e9"), lit("0x1.e5"),
        lit("0x1.e1"), lit("0x1.de"), lit("0x1.da"), lit("0x1.d7"),
        lit("0x1.d4"), lit("0x1.d0"), lit("0x1.cd"), lit("0x1.ca"),
        lit("0x1.c7"), lit("0x1.c3"), lit("0x1.c0"), lit("0x1.bd"),
        lit("0x1.ba"), lit("0x1.b7"), lit("0x1.b4"), lit("0x1.b2"),
        lit("0x1.af"), lit("0x1.ac"), lit("0x1.a9"), lit("0x1.a6"),
        lit("0x1.a4"), lit("0x1.a1"), lit("0x1.9e"), lit("0x1.9c"),
        lit("0x1.99"), lit("0x1.97"), lit("0x1.94"), lit("0x1.92"),
        lit("0x1.8f"), lit("0x1.8d"), lit("0x1.8a"), lit("0x1.88"),
        lit("0x1.86"), lit("0x1.83"), lit("0x1.81"), lit("0x1.7f"),
        lit("0x1.7d"), lit("0x1.7a"), lit("0x1.78"), lit("0x1.76"),
        lit("0x1.74"), lit("0x1.72"), lit("0x1.70"), lit("0x1.6e"),
        lit("0x1.6c"), lit("0x1.6a"), lit("0x1.68"), lit("0x1.66"),
        lit("0x1.64"), lit("0x1.62"), lit("0x1.60"), lit("0x1.5e"),
        lit("0x1.5c"), lit("0x1.5a"), lit("0x1.58"), lit("0x1.57"),
        lit("0x1.55"), lit("0x1.53"), lit("0x1.51"), lit("0x1.50"),
        lit("0x1.4e"), lit("0x1.4c"), lit("0x1.4a"), lit("0x1.49"),
        lit("0x1.47"), lit("0x1.46"), lit("0x1.44"), lit("0x1.42"),
        lit("0x1.41"), lit("0x1.3f"), lit("0x1.3e"), lit("0x1.3c"),
        lit("0x1.3b"), lit("0x1.39"), lit("0x1.38"), lit("0x1.36"),
        lit("0x1.35"), lit("0x1.33"), lit("0x1.32"), lit("0x1.30"),
        lit("0x1.2f"), lit("0x1.2e"), lit("0x1.2c"), lit("0x1.2b"),
        lit("0x1.29"), lit("0x1.28"), lit("0x1.27"), lit("0x1.25"),
        lit("0x1.24"), lit("0x1.23"), lit("0x1.21"), lit("0x1.20"),
        lit("0x1.1f"), lit("0x1.1e"), lit("0x1.1c"), lit("0x1.1b"),
        lit("0x1.1a"), lit("0x1.19"), lit("0x1.18"), lit("0x1.16"),
        lit("0x1.15"), lit("0x1.14"), lit("0x1.13"), lit("0x1.12"),
        lit("0x1.11"), lit("0x1.0f"), lit("0x1.0e"), lit("0x1.0d"),
        lit("0x1.0c"), lit("0x1.0b"), lit("0x1.0a"), lit("0x1.09"),
        lit("0x1.08"), lit("0x1.07"), lit("0x1.06"), lit("0x1.05"),
        lit("0x1.04"), lit("0x1.03"), lit("0x1.02"), lit("0x1.01"),
    ];
    LOOKUP_TABLE[(x.to_bits() >> 8) as usize]
}

fn one_over_one_minus_x(x: ScalarFxP) -> crate::fixedmath::USample {
    // For brevity in defining the lookup table:
    const fn lit(x: &str) -> crate::fixedmath::USample {
        crate::fixedmath::USample::lit(x)
    }
    let x_bits = clip_shape(x).to_bits();
    // Table generated with python:
    //
    // table = [1/(1-(x/256.0)) for x in range(0,256)][:0xF1]
    // shifted = [int(x*256*16) for x in table]
    // shifted[-1] = shifted[-1] - 1 # Prevent overflow
    // hexvals = [hex(x) for x in shifted]
    // for i in range(len(hexvals)):
    //     val = hexvals[i]
    //     print('lit("' + val[:3] + '.' + val[3:] + '"), ', end='')
    //     if i % 4 == 3:
    //         print('')
    #[rustfmt::skip]
    const LOOKUP_TABLE: [crate::fixedmath::USample; 0xF2] = [
        lit("0x1.000"), lit("0x1.010"), lit("0x1.020"), lit("0x1.030"),
        lit("0x1.041"), lit("0x1.051"), lit("0x1.062"), lit("0x1.073"),
        lit("0x1.084"), lit("0x1.095"), lit("0x1.0a6"), lit("0x1.0b7"),
        lit("0x1.0c9"), lit("0x1.0db"), lit("0x1.0ec"), lit("0x1.0fe"),
        lit("0x1.111"), lit("0x1.123"), lit("0x1.135"), lit("0x1.148"),
        lit("0x1.15b"), lit("0x1.16e"), lit("0x1.181"), lit("0x1.194"),
        lit("0x1.1a7"), lit("0x1.1bb"), lit("0x1.1cf"), lit("0x1.1e2"),
        lit("0x1.1f7"), lit("0x1.20b"), lit("0x1.21f"), lit("0x1.234"),
        lit("0x1.249"), lit("0x1.25e"), lit("0x1.273"), lit("0x1.288"),
        lit("0x1.29e"), lit("0x1.2b4"), lit("0x1.2c9"), lit("0x1.2e0"),
        lit("0x1.2f6"), lit("0x1.30d"), lit("0x1.323"), lit("0x1.33a"),
        lit("0x1.352"), lit("0x1.369"), lit("0x1.381"), lit("0x1.399"),
        lit("0x1.3b1"), lit("0x1.3c9"), lit("0x1.3e2"), lit("0x1.3fb"),
        lit("0x1.414"), lit("0x1.42d"), lit("0x1.446"), lit("0x1.460"),
        lit("0x1.47a"), lit("0x1.495"), lit("0x1.4af"), lit("0x1.4ca"),
        lit("0x1.4e5"), lit("0x1.501"), lit("0x1.51d"), lit("0x1.539"),
        lit("0x1.555"), lit("0x1.571"), lit("0x1.58e"), lit("0x1.5ac"),
        lit("0x1.5c9"), lit("0x1.5e7"), lit("0x1.605"), lit("0x1.623"),
        lit("0x1.642"), lit("0x1.661"), lit("0x1.681"), lit("0x1.6a1"),
        lit("0x1.6c1"), lit("0x1.6e1"), lit("0x1.702"), lit("0x1.724"),
        lit("0x1.745"), lit("0x1.767"), lit("0x1.78a"), lit("0x1.7ad"),
        lit("0x1.7d0"), lit("0x1.7f4"), lit("0x1.818"), lit("0x1.83c"),
        lit("0x1.861"), lit("0x1.886"), lit("0x1.8ac"), lit("0x1.8d3"),
        lit("0x1.8f9"), lit("0x1.920"), lit("0x1.948"), lit("0x1.970"),
        lit("0x1.999"), lit("0x1.9c2"), lit("0x1.9ec"), lit("0x1.a16"),
        lit("0x1.a41"), lit("0x1.a6d"), lit("0x1.a98"), lit("0x1.ac5"),
        lit("0x1.af2"), lit("0x1.b20"), lit("0x1.b4e"), lit("0x1.b7d"),
        lit("0x1.bac"), lit("0x1.bdd"), lit("0x1.c0e"), lit("0x1.c3f"),
        lit("0x1.c71"), lit("0x1.ca4"), lit("0x1.cd8"), lit("0x1.d0c"),
        lit("0x1.d41"), lit("0x1.d77"), lit("0x1.dae"), lit("0x1.de5"),
        lit("0x1.e1e"), lit("0x1.e57"), lit("0x1.e91"), lit("0x1.ecc"),
        lit("0x1.f07"), lit("0x1.f44"), lit("0x1.f81"), lit("0x1.fc0"),
        lit("0x2.000"), lit("0x2.040"), lit("0x2.082"), lit("0x2.0c4"),
        lit("0x2.108"), lit("0x2.14d"), lit("0x2.192"), lit("0x2.1d9"),
        lit("0x2.222"), lit("0x2.26b"), lit("0x2.2b6"), lit("0x2.302"),
        lit("0x2.34f"), lit("0x2.39e"), lit("0x2.3ee"), lit("0x2.43f"),
        lit("0x2.492"), lit("0x2.4e6"), lit("0x2.53c"), lit("0x2.593"),
        lit("0x2.5ed"), lit("0x2.647"), lit("0x2.6a4"), lit("0x2.702"),
        lit("0x2.762"), lit("0x2.7c4"), lit("0x2.828"), lit("0x2.88d"),
        lit("0x2.8f5"), lit("0x2.95f"), lit("0x2.9cb"), lit("0x2.a3a"),
        lit("0x2.aaa"), lit("0x2.b1d"), lit("0x2.b93"), lit("0x2.c0b"),
        lit("0x2.c85"), lit("0x2.d02"), lit("0x2.d82"), lit("0x2.e05"),
        lit("0x2.e8b"), lit("0x2.f14"), lit("0x2.fa0"), lit("0x3.030"),
        lit("0x3.0c3"), lit("0x3.159"), lit("0x3.1f3"), lit("0x3.291"),
        lit("0x3.333"), lit("0x3.3d9"), lit("0x3.483"), lit("0x3.531"),
        lit("0x3.5e5"), lit("0x3.69d"), lit("0x3.759"), lit("0x3.81c"),
        lit("0x3.8e3"), lit("0x3.9b0"), lit("0x3.a83"), lit("0x3.b5c"),
        lit("0x3.c3c"), lit("0x3.d22"), lit("0x3.e0f"), lit("0x3.f03"),
        lit("0x4.000"), lit("0x4.104"), lit("0x4.210"), lit("0x4.325"),
        lit("0x4.444"), lit("0x4.56c"), lit("0x4.69e"), lit("0x4.7dc"),
        lit("0x4.924"), lit("0x4.a79"), lit("0x4.bda"), lit("0x4.d48"),
        lit("0x4.ec4"), lit("0x5.050"), lit("0x5.1eb"), lit("0x5.397"),
        lit("0x5.555"), lit("0x5.726"), lit("0x5.90b"), lit("0x5.b05"),
        lit("0x5.d17"), lit("0x5.f41"), lit("0x6.186"), lit("0x6.3e7"),
        lit("0x6.666"), lit("0x6.906"), lit("0x6.bca"), lit("0x6.eb3"),
        lit("0x7.1c7"), lit("0x7.507"), lit("0x7.878"), lit("0x7.c1f"),
        lit("0x8.000"), lit("0x8.421"), lit("0x8.888"), lit("0x8.d3d"),
        lit("0x9.249"), lit("0x9.7b4"), lit("0x9.d89"), lit("0xa.3d7"),
        lit("0xa.aaa"), lit("0xb.216"), lit("0xb.a2e"), lit("0xc.30c"),
        lit("0xc.ccc"), lit("0xd.794"), lit("0xe.38e"), lit("0xf.0f0"),
        lit("0xf.fff"), lit("0xf.fff") //throw 2x maxs at the end to avoid out-of-bounds on CLIP_MAX
    ];
    let index = x_bits >> 8;
    let lookup_val = LOOKUP_TABLE[index as usize];
    let interp = (LOOKUP_TABLE[index as usize + 1] - lookup_val)
        .wide_mul(crate::fixedmath::U8F8::from_bits(x_bits & 0xFF));
    lookup_val + crate::fixedmath::USample::from_num(interp)
}
