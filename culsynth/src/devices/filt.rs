use super::*;

use crate::{DspFloat, DspFormat, DspFormatBase, DspType};

pub(crate) mod detail {
    use super::*;
    pub trait FiltOps: DspFormatBase {
        const RES_MAX: Self::Scalar;
        type FiltGain;
        type FiltFeedback: Default + Clone + Send;
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
}

#[derive(Clone, Default)]
pub struct FiltParams<T: DspFormatBase> {
    pub cutoff: T::Note,
    pub resonance: T::Scalar,
}

impl<T: DspFloat> From<&FiltParams<i16>> for FiltParams<T> {
    fn from(value: &FiltParams<i16>) -> Self {
        FiltParams::<T> {
            cutoff: value.cutoff.to_num(),
            resonance: value.resonance.to_num(),
        }
    }
}

#[derive(Clone, Default)]
pub struct FiltOutput<T: DspFormatBase> {
    pub low: T::Sample,
    pub band: T::Sample,
    pub high: T::Sample,
}

#[derive(Default, Clone)]
pub struct Filt<T: DspFormat> {
    low_z: T::FiltFeedback,
    band_z: T::FiltFeedback,
}

impl<T: DspFormat> Filt<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T: DspFormat> Device<T> for Filt<T> {
    type Input = T::Sample;
    type Params = FiltParams<T>;
    type Output = FiltOutput<T>;
    fn next(
        &mut self,
        context: &T::Context,
        signal: T::Sample,
        params: FiltParams<T>,
    ) -> FiltOutput<T> {
        let resonance = T::Scalar::one()
            - if params.resonance < T::RES_MAX {
                params.resonance
            } else {
                T::RES_MAX
            };
        T::calc_filt(
            context,
            signal,
            params.cutoff,
            resonance,
            &mut self.low_z,
            &mut self.band_z,
        )
    }
}

impl<T: DspFloat> detail::FiltOps for T {
    const RES_MAX: T = T::RES_MAX;
    type FiltGain = T;
    type FiltFeedback = T;
    fn prewarped_gain(context: &Context<Self>, cutoff: T) -> T {
        let f_c = cutoff.midi_to_freq();
        T::ftan(T::PI * f_c / context.sample_rate)
    }
    fn calc_filt(
        context: &Self::Context,
        signal: Self::Sample,
        cutoff: Self::Note,
        res: Self::Scalar,
        low_z: &mut Self::FiltFeedback,
        band_z: &mut Self::FiltFeedback,
    ) -> filt::FiltOutput<T> {
        let gain = Self::prewarped_gain(&context, cutoff);
        let denom = gain * gain + Self::TWO * res * gain + Self::ONE;
        let high = (signal - (Self::TWO * res + gain) * (*band_z) - (*low_z)) / denom;

        let band_gain = gain * high;
        let band = band_gain + *band_z;
        *band_z = band + band_gain;

        let low_gain = gain * band;
        let low = low_gain + *low_z;
        *low_z = low + low_gain;

        FiltOutput { low, band, high }
    }
}

impl detail::FiltOps for i16 {
    const RES_MAX: ScalarFxP = ScalarFxP::lit("0x0.F000");
    type FiltGain = crate::fixedmath::U1F15;
    type FiltFeedback = crate::fixedmath::I12F20;
    fn prewarped_gain(context: &ContextFxP, cutoff: NoteFxP) -> Self::FiltGain {
        use crate::fixedmath::{midi_note_to_frequency, tan_fixed, U14F2};
        let f_c = U14F2::from_num(midi_note_to_frequency(cutoff));
        let omega_d = ScalarFxP::from_num(
            f_c.wide_mul(context.sample_rate.frac_2pi4096_sr())
                .unwrapped_shr(13),
        );
        tan_fixed(omega_d)
    }
    fn calc_filt(
        context: &Self::Context,
        signal: Self::Sample,
        cutoff: Self::Note,
        res: Self::Scalar,
        low_z: &mut Self::FiltFeedback,
        band_z: &mut Self::FiltFeedback,
    ) -> filt::FiltOutput<i16> {
        use crate::fixedmath::{one_over_one_plus, I5F27, I7F25, U3F13, U3F29};

        let gain = Self::prewarped_gain(&context, cutoff);
        let gain2 = U3F29::from_num(gain.wide_mul(gain));
        // resonance * gain is a U1F31, so this will only lose the least
        // significant bit and provides space for the shift left below
        let gain_r = U3F29::from_num(res.wide_mul(gain));
        let k = gain2 + gain_r.unwrapped_shl(1);
        let (denom_inv, shift) = one_over_one_plus(k);

        let gain_plus_2r = U3F29::from_num(res).unwrapped_shl(1) + U3F29::from_num(gain);
        let band_high_feedback: I7F25 =
            U3F13::from_num(gain_plus_2r).wide_mul_signed(SampleFxP::saturating_from_num(*band_z));
        let high_num = SampleFxP::saturating_from_num(
            Self::FiltFeedback::from_num(signal)
                - Self::FiltFeedback::from_num(band_high_feedback)
                - *low_z,
        );
        let high_unshifted: I5F27 = high_num.wide_mul_unsigned(denom_inv);
        let high = SampleFxP::saturating_from_num(high_unshifted.unwrapped_shr(shift));

        let band_gain = Self::FiltFeedback::from_num(gain.wide_mul_signed(high));
        let band = band_gain + *band_z;
        *band_z = band + band_gain;
        let band = SampleFxP::saturating_from_num(band);

        let low_gain = Self::FiltFeedback::from_num(gain.wide_mul_signed(band));
        let low = low_gain + *low_z;
        *low_z = low + low_gain;
        let low = SampleFxP::saturating_from_num(low);

        FiltOutput { low, band, high }
    }
}
