use crate::{DspFormat, DspFormatBase, DspFloat};

use super::*;

/// A parameter pack for [MixOsc].
#[derive(Clone, Default)]
pub struct MixOscParams<T: DspFormatBase> {
    /// The tuning offset, in semitones offset from 12TET/A440
    pub tune: T::NoteOffset,
    /// The oscillator shape parameter, ranging from 0-1 and representing the
    /// amount of phase distortion/pulse width ratio for this output
    pub shape: T::Scalar,
    /// Sine wave gain
    pub sin: T::Scalar,
    /// Square wave gain
    pub sq: T::Scalar,
    /// Triangle wave gain
    pub tri: T::Scalar,
    /// Sawtooth wave gain
    pub saw: T::Scalar,
}

impl<T: DspFloat> From<&MixOscParams<i16>> for MixOscParams<T> {
    fn from(value: &MixOscParams<i16>) -> Self {
        MixOscParams::<T> {
            tune: value.tune.to_num(),
            shape: value.shape.to_num(),
            sin: value.sin.to_num(),
            sq: value.sq.to_num(),
            tri: value.tri.to_num(),
            saw: value.saw.to_num(),
        }
    }
}

impl<T: DspFormatBase> MixOscParams<T> {
    /// Extract the basic oscillator parameters from this parameter pack.
    pub fn to_osc_params(&self) -> OscParams<T> {
        OscParams {
            tune: self.tune,
            shape: self.shape,
        }
    }
}

/// This wraps [Osc], combining the oscillator with a mixer for each of the
/// wave shapes and taking the gain of each wave as a parameter.  This provides
/// a pre-mixed output as a single signal.
/// 
/// This implements [Device], taking a Note as input and [MixOscParams] as
/// parameters, and outputs a Sample representing the sum of the different
/// waveforms scaled by their respective gains.
#[derive(Clone, Default)]
pub struct MixOsc<T: DspFormat> {
    mixer: Mixer<T, 4>,
    osc: Osc<T>,
}

impl<T: DspFormat> Device<T> for MixOsc<T> {
    type Input = T::Note;
    type Params = MixOscParams<T>;
    type Output = T::Sample;
    fn next(&mut self, context: &T::Context, note: T::Note, params: MixOscParams<T>) -> T::Sample {
        let osc_out = self.osc.next(context, note, params.to_osc_params());
        self.mixer.next(
            context,
            [osc_out.sin, osc_out.sq, osc_out.tri, osc_out.saw],
            [params.sin, params.sq, params.tri, params.saw],
        )
    }
}

/// This struct contains parameters for a synced oscillator pair
#[derive(Clone, Default)]
pub struct SyncedMixOscsParams<T: DspFormatBase> {
    /// Parameters for the primary oscillator
    pub primary: MixOscParams<T>,
    /// Parameters for the secondary (synced) oscillator
    pub secondary: MixOscParams<T>,
    /// True if oscillator sync has been enabled - when false, both oscillators
    /// will run independently
    pub sync: bool,
}

impl<T: DspFloat> From<&SyncedMixOscsParams<i16>> for SyncedMixOscsParams<T> {
    fn from(value: &SyncedMixOscsParams<i16>) -> Self {
        Self {
            primary: (&value.primary).into(),
            secondary: (&value.secondary).into(),
            sync: value.sync,
        }
    }
}

/// The output of a [SyncedMixOscs] device.
#[derive(Clone, Default)]
pub struct SyncedMixOscsOutput<T: DspFormatBase> {
    /// The output of the primary oscillator
    pub primary: T::Sample,
    /// The output of the secondary oscillator
    pub secondary: T::Sample,
}

/// A synced pair of [MixOsc]s.  The secondary oscillator will be synced
/// to the primary oscillator.
/// 
/// This implements [Device], taking a Note as input and a [SyncedMixOscsParams]
/// as parameters.  It outputs a [SyncedMixOscsOutput], which is just the pair
/// of Sample outputs from the underlying [Osc].
/// 
/// See also: [SyncedOscs], [Osc]
#[derive(Clone, Default)]
pub struct SyncedMixOscs<T: DspFormat> {
    oscs: SyncedOscs<T>,
    mixer_pri: Mixer<T, 4>,
    mixer_sec: Mixer<T, 4>,
}

impl<T: DspFormat> Device<T> for SyncedMixOscs<T> {
    type Input = T::Note;
    type Params = SyncedMixOscsParams<T>;
    type Output = SyncedMixOscsOutput<T>;
    fn next(
        &mut self,
        context: &T::Context,
        note: T::Note,
        params: SyncedMixOscsParams<T>,
    ) -> Self::Output {
        let inputs = SyncedOscsParams {
            primary: params.primary.to_osc_params(),
            secondary: params.secondary.to_osc_params(),
            sync: params.sync,
        };
        let SyncedOscsOutput {
            primary: p,
            secondary: s,
        } = self.oscs.next(context, note, inputs);
        let pri_out = self.mixer_pri.next(
            context,
            [p.sin, p.sq, p.tri, p.saw],
            [
                params.primary.sin,
                params.primary.sq,
                params.primary.tri,
                params.primary.saw,
            ],
        );
        let sec_out = self.mixer_sec.next(
            context,
            [s.sin, s.sq, s.tri, s.saw],
            [
                params.secondary.sin,
                params.secondary.sq,
                params.secondary.tri,
                params.secondary.saw,
            ],
        );
        SyncedMixOscsOutput {
            primary: pri_out,
            secondary: sec_out,
        }
    }
}
