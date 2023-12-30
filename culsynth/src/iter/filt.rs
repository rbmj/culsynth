use super::*;
use detail::FiltOps;

#[derive(Default, Clone)]
pub struct FiltOutput<T: DspTypeAliases> {
    pub low: T::Sample,
    pub band: T::Sample,
    pub high: T::Sample,
}

struct FiltState<T: DspFormat> {
    low_z: T::FiltFeedback,
    band_z: T::FiltFeedback,
    context: T::Context,
}

impl<T: DspFormat> FiltState<T> {
    fn new(ctx: T::Context) -> Self {
        Self {
            low_z: Default::default(),
            band_z: Default::default(),
            context: ctx,
        }
    }
}

pub struct Filt<
    T: DspFormat,
    Signal: Source<T::Sample>,
    Cutoff: Source<T::Note>,
    Resonance: Source<T::Scalar>,
> {
    state: FiltState<T>,
    signal: Signal,
    cutoff: Cutoff,
    resonance: Resonance,
}

impl<
        T: DspFormat,
        Signal: Source<T::Sample>,
        Cutoff: Source<T::Note>,
        Resonance: Source<T::Scalar>,
    > Filt<T, Signal, Cutoff, Resonance>
{
    pub fn with_signal<NewSignal: Source<T::Sample>>(
        self,
        new_signal: NewSignal,
    ) -> Filt<T, NewSignal, Cutoff, Resonance> {
        Filt {
            state: self.state,
            signal: new_signal,
            cutoff: self.cutoff,
            resonance: self.resonance,
        }
    }
    pub fn with_cutoff<NewCutoff: Source<T::Note>>(
        self,
        new_cutoff: NewCutoff,
    ) -> Filt<T, Signal, NewCutoff, Resonance> {
        Filt {
            state: self.state,
            signal: self.signal,
            cutoff: new_cutoff,
            resonance: self.resonance,
        }
    }
    pub fn with_resonance<NewResonance: Source<T::Scalar>>(
        self,
        new_resonance: NewResonance,
    ) -> Filt<T, Signal, Cutoff, NewResonance> {
        Filt {
            state: self.state,
            signal: self.signal,
            cutoff: self.cutoff,
            resonance: new_resonance,
        }
    }
    pub fn iter<'a>(&'a mut self) -> FiltIter<'a, T, Signal, Cutoff, Resonance> {
        FiltIter {
            filt_state: &mut self.state,
            signal: self.signal.get(),
            cutoff: self.cutoff.get(),
            resonance: self.resonance.get(),
        }
    }
}

pub struct FiltIter<
    'a,
    T: DspFormat,
    Signal: Source<T::Sample> + 'a,
    Cutoff: Source<T::Note> + 'a,
    Resonance: Source<T::Scalar> + 'a,
> {
    filt_state: &'a mut FiltState<T>,
    signal: Signal::It<'a>,
    cutoff: Cutoff::It<'a>,
    resonance: Resonance::It<'a>,
}

impl<
        'a,
        T: DspFormat,
        Signal: Source<T::Sample> + 'a,
        Cutoff: Source<T::Note> + 'a,
        Resonance: Source<T::Scalar> + 'a,
    > Iterator for FiltIter<'a, T, Signal, Cutoff, Resonance>
{
    type Item = FiltOutput<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let signal = self.signal.next()?;
        let cutoff = self.cutoff.next()?;
        let resonance = self.resonance.next()?;
        let resonance = T::Scalar::one()
            - if resonance < T::RES_MAX {
                resonance
            } else {
                T::RES_MAX
            };
        Some(T::calc_filt(
            &self.filt_state.context,
            signal,
            cutoff,
            resonance,
            &mut self.filt_state.low_z,
            &mut self.filt_state.band_z,
        ))
    }
}

impl<T: DspFloat> FiltOps for T {
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

impl FiltOps for i16 {
    const RES_MAX: Scalar = Scalar::lit("0x0.F000");
    type FiltGain = crate::fixedmath::U1F15;
    type FiltFeedback = crate::fixedmath::I12F20;
    fn prewarped_gain(context: &ContextFxP, cutoff: Note) -> Self::FiltGain {
        use crate::fixedmath::{midi_note_to_frequency, tan_fixed, U14F2};
        let f_c = U14F2::from_num(midi_note_to_frequency(cutoff));
        let omega_d = Scalar::from_num(
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
            U3F13::from_num(gain_plus_2r).wide_mul_signed(Sample::saturating_from_num(*band_z));
        let high_num = Sample::saturating_from_num(
            Self::FiltFeedback::from_num(signal)
                - Self::FiltFeedback::from_num(band_high_feedback)
                - *low_z,
        );
        let high_unshifted: I5F27 = high_num.wide_mul_unsigned(denom_inv);
        let high = Sample::saturating_from_num(high_unshifted.unwrapped_shr(shift));

        let band_gain = Self::FiltFeedback::from_num(gain.wide_mul_signed(high));
        let band = band_gain + *band_z;
        *band_z = band + band_gain;
        let band = Sample::saturating_from_num(band);

        let low_gain = Self::FiltFeedback::from_num(gain.wide_mul_signed(band));
        let low = low_gain + *low_z;
        *low_z = low + low_gain;
        let low = Sample::saturating_from_num(low);

        FiltOutput { low, band, high }
    }
}
