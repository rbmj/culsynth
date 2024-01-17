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
    pub fn to_osc_params(&self) -> OscParams<T> {
        OscParams {
            tune: self.tune,
            shape: self.shape,
        }
    }
}

/// This wraps [Osc], combining the oscillator with a mixer for each of the
/// wave shapes and taking the gain of each wave as a parameter and providing
/// a pre-mixed output
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

#[derive(Clone, Default)]
pub struct SyncedMixOscsParams<T: DspFormatBase> {
    pub primary: MixOscParams<T>,
    pub secondary: MixOscParams<T>,
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

#[derive(Clone, Default)]
pub struct SyncedMixOscsOutput<T: DspFormatBase> {
    pub primary: T::Sample,
    pub secondary: T::Sample,
}

#[derive(Clone, Default)]
pub struct SyncedMixOscs<T: DspFormat> {
    mixer_pri: Mixer<T, 4>,
    mixer_sec: Mixer<T, 4>,
    oscs: SyncedOscs<T>,
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
        let sec_out = self.mixer_pri.next(
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
