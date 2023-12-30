use super::*;
use detail::OscSync;

#[derive(Clone)]
pub struct OscOutput<T: DspTypeAliases> {
    pub sin: T::Sample,
    pub sq: T::Sample,
    pub tri: T::Sample,
    pub saw: T::Sample,
}

impl<T: DspTypeAliases> Default for OscOutput<T> {
    fn default() -> Self {
        OscOutput {
            sin: T::Sample::default(),
            sq: T::Sample::default(),
            tri: T::Sample::default(),
            saw: T::Sample::default(),
        }
    }
}

struct OscState<T: DspFormat> {
    phase: T::Phase,
    context: T::Context,
}

impl<T: DspFormat> OscState<T> {
    fn new(context: T::Context) -> Self {
        Self {
            phase: T::Phase::zero(),
            context,
        }
    }
}

pub struct Osc<
    T: DspFormat,
    Tune: Source<T::NoteOffset>,
    Shape: Source<T::Scalar>,
    Note: Source<T::Note>,
> {
    tune: Tune,
    shape: Shape,
    note: Note,
    state: OscState<T>,
}

impl<
        T: DspFormat,
        Tune: Source<T::NoteOffset>,
        Shape: Source<T::Scalar>,
        Note: Source<T::Note>,
    > Osc<T, Tune, Shape, Note>
{
    pub fn with_tune<NewTune: Source<T::NoteOffset>>(
        self,
        new_tune: NewTune,
    ) -> Osc<T, NewTune, Shape, Note> {
        Osc {
            tune: new_tune,
            shape: self.shape,
            note: self.note,
            state: self.state,
        }
    }
    pub fn with_shape<NewShape: Source<T::Scalar>>(
        self,
        new_shape: NewShape,
    ) -> Osc<T, Tune, NewShape, Note> {
        Osc {
            tune: self.tune,
            shape: new_shape,
            note: self.note,
            state: self.state,
        }
    }
    pub fn sync_to<
        TuneB: Source<T::NoteOffset>,
        ShapeB: Source<T::Scalar>,
        NoteB: Source<T::Note>,
        SyncSrc: Source<T::Scalar>,
    >(
        self,
        master: Osc<T, TuneB, ShapeB, NoteB>,
        sync_src: SyncSrc,
    ) -> SyncedOscs<T, TuneB, ShapeB, NoteB, Tune, Shape, Note, SyncSrc> {
        SyncedOscs {
            master,
            slave: self,
            sync: sync_src,
        }
    }
    pub fn iter<'a>(&'a mut self) -> OscIter<'a, T, Tune, Shape, Note> {
        OscIter {
            osc_state: &mut self.state,
            tune: self.tune.get(),
            shape: self.shape.get(),
            note: self.note.get(),
        }
    }
}

pub fn new<T: DspFormat>(
    context: T::Context,
) -> Osc<
    T,
    IteratorSource<Repeat<T::NoteOffset>>,
    IteratorSource<Repeat<T::Scalar>>,
    IteratorSource<Repeat<T::Note>>,
> {
    Osc {
        tune: repeat(T::NoteOffset::zero()).into(),
        shape: repeat(T::Scalar::zero()).into(),
        note: repeat(T::default_note()).into(),
        state: OscState::new(context),
    }
}

struct OscIter<
    'a,
    T: DspFormat,
    Tune: Source<T::NoteOffset> + 'a,
    Shape: Source<T::Scalar> + 'a,
    Note: Source<T::Note> + 'a,
> {
    osc_state: &'a mut OscState<T>,
    tune: Tune::It<'a>,
    shape: Shape::It<'a>,
    note: Note::It<'a>,
}

impl<
        'a,
        T: DspFormat,
        Tune: Source<T::NoteOffset>,
        Shape: Source<T::Scalar>,
        Note: Source<T::Note>,
    > OscIter<'a, T, Tune, Shape, Note>
{
    fn next_with_sync(
        &mut self,
        mut sync: T::Scalar,
        sync_mode: OscSync,
    ) -> Option<(OscOutput<T>, T::Scalar)> {
        let tune = self.tune.next()?;
        let note = self.note.next()?;
        let shape = self.shape.next()?;
        let freq = T::note_to_freq(T::apply_note_offset(note, tune));
        let out = T::calc_osc(
            &self.osc_state.context,
            freq,
            &mut self.osc_state.phase,
            shape,
            &mut sync,
            sync_mode,
        );
        Some((out, sync))
    }
}

impl<
        'a,
        T: DspFormat,
        Tune: Source<T::NoteOffset>,
        Shape: Source<T::Scalar>,
        Note: Source<T::Note>,
    > Iterator for OscIter<'a, T, Tune, Shape, Note>
{
    type Item = OscOutput<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let (out, _) = self.next_with_sync(T::Scalar::zero(), OscSync::Off)?;
        Some(out)
    }
}

pub struct SyncedOscs<
    T: DspFormat,
    TuneA: Source<T::NoteOffset>,
    ShapeA: Source<T::Scalar>,
    NoteA: Source<T::Note>,
    TuneB: Source<T::NoteOffset>,
    ShapeB: Source<T::Scalar>,
    NoteB: Source<T::Note>,
    SyncSrc: Source<T::Scalar>,
> {
    master: Osc<T, TuneA, ShapeA, NoteA>,
    slave: Osc<T, TuneB, ShapeB, NoteB>,
    sync: SyncSrc,
}

impl<
        T: DspFormat,
        TuneA: Source<T::NoteOffset>,
        ShapeA: Source<T::Scalar>,
        NoteA: Source<T::Note>,
        TuneB: Source<T::NoteOffset>,
        ShapeB: Source<T::Scalar>,
        NoteB: Source<T::Note>,
        SyncSrc: Source<T::Scalar>,
    > SyncedOscs<T, TuneA, ShapeA, NoteA, TuneB, ShapeB, NoteB, SyncSrc>
{
    fn iter<'a>(
        &'a mut self,
    ) -> SyncedOscsIter<'a, T, TuneA, ShapeA, NoteA, TuneB, ShapeB, NoteB, SyncSrc> {
        SyncedOscsIter {
            master: self.master.iter(),
            slave: self.slave.iter(),
            sync: self.sync.get(),
        }
    }
}

struct SyncedOscsIter<
    'a,
    T: DspFormat,
    TuneA: Source<T::NoteOffset>,
    ShapeA: Source<T::Scalar>,
    NoteA: Source<T::Note>,
    TuneB: Source<T::NoteOffset>,
    ShapeB: Source<T::Scalar>,
    NoteB: Source<T::Note>,
    SyncSrc: Source<T::Scalar> + 'a,
> {
    master: OscIter<'a, T, TuneA, ShapeA, NoteA>,
    slave: OscIter<'a, T, TuneB, ShapeB, NoteB>,
    sync: SyncSrc::It<'a>,
}

impl<
        'a,
        T: DspFormat,
        TuneA: Source<T::NoteOffset>,
        ShapeA: Source<T::Scalar>,
        NoteA: Source<T::Note>,
        TuneB: Source<T::NoteOffset>,
        ShapeB: Source<T::Scalar>,
        NoteB: Source<T::Note>,
        SyncSrc: Source<T::Scalar>,
    > Iterator for SyncedOscsIter<'a, T, TuneA, ShapeA, NoteA, TuneB, ShapeB, NoteB, SyncSrc>
{
    type Item = (OscOutput<T>, OscOutput<T>);
    fn next(&mut self) -> Option<Self::Item> {
        let (master_out, sync) = self
            .master
            .next_with_sync(self.sync.next()?, OscSync::Master)?;
        let (slave_out, _) = self.slave.next_with_sync(sync, OscSync::Slave)?;
        Some((master_out, slave_out))
    }
}

// This section contains the actual DSP logic for both fixed and floating point

impl<T: DspFloat> detail::OscOps for T {
    const FRAC_2_PI: T = <T as crate::devices::Float>::FRAC_2_PI;
    fn apply_note_offset(note: Self::Note, offset: Self::NoteOffset) -> Self::Note {
        note + offset
    }
    fn calc_osc(
        ctx: &Self::Context,
        freq: Self::Frequency,
        phase: &mut Self::Phase,
        shape: Self::Scalar,
        sync: &mut Self::Scalar,
        sync_mode: osc::OscSync,
    ) -> osc::OscOutput<Self> {
        let mut out = osc::OscOutput::<T>::default();
        //generate waveforms (piecewise defined)
        let frac_2phase_pi = *phase * <Self as detail::OscOps>::FRAC_2_PI;
        out.saw = frac_2phase_pi / T::TWO;
        if *phase < T::ZERO {
            out.sq = T::ONE.neg();
            if *phase < T::FRAC_PI_2.neg() {
                // phase in [-pi, pi/2)
                // sin(x) = -cos(x+pi/2)
                out.sin = (*phase + T::FRAC_PI_2).fcos().neg();
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
            if *phase < T::FRAC_PI_2 {
                // phase in [0, pi/2)
                out.sin = phase.fsin();
                out.tri = frac_2phase_pi;
            } else {
                // phase in [pi/2, pi)
                // sin(x) = cos(x-pi/2)
                out.sin = (*phase - T::FRAC_PI_2).fcos();
                out.tri = T::TWO - frac_2phase_pi;
            }
        }
        //calculate the next phase
        let phase_per_sample = freq * T::TAU / ctx.sample_rate;
        let shp = if shape < T::SHAPE_CLIP {
            shape
        } else {
            T::SHAPE_CLIP
        };
        // Handle slave oscillator resetting phase if master crosses:
        if sync_mode == OscSync::Slave {
            if *sync != T::ZERO {
                *phase = T::ZERO;
            }
        }
        let phase_per_smp_adj = if *phase < T::ZERO {
            phase_per_sample * (T::ONE / (T::ONE + shp))
        } else {
            phase_per_sample * (T::ONE / (T::ONE - shp))
        };
        let old_phase = *phase;
        match sync_mode {
            OscSync::Off => {
                *phase = *phase + phase_per_smp_adj;
            }
            OscSync::Master => {
                *phase = *phase + phase_per_smp_adj;
                // calculate what time in this sampling period the phase crossed zero:
                *sync = if *sync != T::ZERO && old_phase < T::ZERO && *phase >= T::ZERO {
                    T::ONE - (*phase / phase_per_smp_adj)
                } else {
                    T::ZERO
                };
            }
            OscSync::Slave => {
                *phase = *phase
                    + if *sync != T::ZERO {
                        phase_per_smp_adj * (T::ONE - *sync)
                    } else {
                        phase_per_smp_adj
                    };
            }
        }
        // make sure we calculate the correct new phase on transitions for assymmetric waves:
        // check if we've crossed from negative to positive phase
        if old_phase < T::ZERO && *phase > T::ZERO && shp != T::ZERO {
            // need to multiply residual phase i.e. (phase - 0) by (1+k)/(1-k)
            // where k is the shape, so no work required if shape is 0
            *phase = *phase * (T::ONE + shp) / (T::ONE - shp);
        }
        // Check if we've crossed from positive phase back to negative:
        if *phase >= T::PI {
            // if we're a symmetric wave this is as simple as just subtract 2pi
            if shp == T::ZERO {
                *phase = *phase - T::TAU;
            } else {
                // if assymmetric we have to multiply residual phase i.e. phase - pi
                // by (1-k)/(1+k) where k is the shape:
                let delta = (*phase - T::PI) * (T::ONE - shp) / (T::ONE + shp);
                // add new change in phase to our baseline, -pi:
                *phase = delta - T::PI;
            }
        }
        out
    }
}

impl detail::OscOps for i16 {
    const FRAC_2_PI: Scalar = Scalar::lit("0x0.a2fa");
    fn apply_note_offset(note: Note, offset: SignedNoteFxP) -> Note {
        note.saturating_add_signed(offset)
    }
    fn calc_osc(
        ctx: &ContextFxP,
        freq: Frequency,
        phase: &mut PhaseFxP,
        shape: Scalar,
        sync: &mut Scalar,
        sync_mode: osc::OscSync,
    ) -> osc::OscOutput<i16> {
        use crate::fixedmath;
        use fixedmath::{
            apply_scalar_i, cos_fixed, one_over_one_plus_highacc, scale_fixedfloat, sin_fixed,
        };
        const TWO: Sample = Sample::lit("2");
        //generate waveforms (piecewise defined)
        let frac_2phase_pi = apply_scalar_i(Sample::from_num(*phase), Self::FRAC_2_PI);
        let mut ret = OscOutput::<i16>::default();
        //Sawtooth wave does not have to be piecewise-defined
        ret.saw = frac_2phase_pi.unwrapped_shr(1);
        //All other functions are piecewise-defined:
        if *phase < 0 {
            ret.sq = Sample::NEG_ONE;
            if *phase < PhaseFxP::FRAC_PI_2.unwrapped_neg() {
                // phase in [-pi, pi/2)
                // Use the identity sin(x) = -cos(x+pi/2) since our taylor series
                // approximations are centered about zero and this will be more accurate
                ret.sin = cos_fixed(Sample::from_num(*phase + PhaseFxP::FRAC_PI_2)).unwrapped_neg();
                ret.tri = frac_2phase_pi.unwrapped_neg() - TWO;
            } else {
                // phase in [-pi/2, 0)
                ret.sin = sin_fixed(Sample::from_num(*phase));
                ret.tri = frac_2phase_pi;
            }
        } else {
            ret.sq = Sample::ONE;
            if *phase < PhaseFxP::FRAC_PI_2 {
                // phase in [0, pi/2)
                ret.sin = sin_fixed(Sample::from_num(*phase));
                ret.tri = frac_2phase_pi;
            } else {
                // phase in [pi/2, pi)
                // sin(x) = cos(x-pi/2)
                ret.sin = cos_fixed(Sample::from_num(*phase - PhaseFxP::FRAC_PI_2));
                ret.tri = TWO - frac_2phase_pi;
            }
        }
        // we need to divide by 2^12 here, but we're increasing the fractional part by 10
        // bits so we'll only actually shift by 2 places and then use a bitcast for the
        // remaining logical 10 bits:
        let phase_per_sample = fixedmath::U4F28::from_bits(
            scale_fixedfloat(freq, ctx.sample_rate.frac_2pi4096_sr())
                .unwrapped_shr(2)
                .to_bits(),
        );
        // Handle slave oscillator resetting phase if master crosses:
        if sync_mode == OscSync::Slave {
            if *sync != Scalar::ZERO {
                *phase = PhaseFxP::ZERO;
            }
        }
        // Adjust phase per sample for the shape parameter:
        let phase_per_smp_adj = PhaseFxP::from_num(if *phase < PhaseFxP::ZERO {
            let (x, s) = one_over_one_plus_highacc(clip_shape(shape));
            fixedmath::scale_fixedfloat(phase_per_sample, x).unwrapped_shr(s)
        } else {
            fixedmath::scale_fixedfloat(phase_per_sample, one_over_one_minus_x(shape))
        });
        // Advance the oscillator's phase, and handle oscillator sync logic:
        let old_phase = *phase;
        match sync_mode {
            OscSync::Off => {
                *phase += phase_per_smp_adj;
            }
            OscSync::Master => {
                *phase += phase_per_smp_adj;
                // calculate what time in this sampling period the phase crossed zero:
                *sync = if *sync != Scalar::ZERO
                    && old_phase < PhaseFxP::ZERO
                    && *phase >= PhaseFxP::ZERO
                {
                    // we need to calculate 1 - (phase / phase_per_sample_adj)
                    let adj_s = Scalar::from_num(phase_per_smp_adj.unwrapped_shr(2));
                    let x = fixedmath::U3F13::from_num(*phase).wide_mul(inverse(adj_s));
                    let proportion = Scalar::saturating_from_num(x.unwrapped_shr(2));
                    if proportion == Scalar::MAX {
                        Scalar::DELTA
                    } else {
                        Scalar::MAX - proportion
                    }
                } else {
                    Scalar::ZERO
                }
            }
            OscSync::Slave => {
                *phase += if *sync != Scalar::ZERO {
                    // Only advance phase for the portion of time after master crossed zero:
                    let scale = Scalar::MAX - *sync;
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
        if old_phase < PhaseFxP::ZERO && *phase > PhaseFxP::ZERO && shape != Scalar::ZERO {
            // need to multiply residual phase i.e. (phase - 0) by (1+k)/(1-k)
            // where k is the shape, so no work required if shape is 0
            let scaled = scale_fixedfloat(
                fixedmath::U4F28::from_num(*phase),
                one_over_one_minus_x(shape),
            );
            let one_plus_shape =
                fixedmath::U1F15::from_num(clip_shape(shape)) + fixedmath::U1F15::ONE;
            *phase = PhaseFxP::from_num(scale_fixedfloat(scaled, one_plus_shape));
        }
        // Check if we've crossed from positive phase back to negative:
        if *phase >= PhaseFxP::PI {
            // if we're a symmetric wave this is as simple as just subtract 2pi
            if shape == Scalar::ZERO {
                *phase -= PhaseFxP::TAU;
            } else {
                // if assymmetric we have to multiply residual phase i.e. phase - pi
                // by (1-k)/(1+k) where k is the shape:
                let one_minus_shape = (Scalar::MAX - clip_shape(shape)) + Scalar::DELTA;
                // scaled = residual_phase * (1-k)
                let scaled = scale_fixedfloat(
                    fixedmath::U4F28::from_num(*phase - PhaseFxP::PI),
                    one_minus_shape,
                );
                // new change in phase = scaled * 1/(1 + k)
                let (x, s) = one_over_one_plus_highacc(clip_shape(shape));
                let delta = scale_fixedfloat(scaled, x).unwrapped_shr(s);
                // add new change in phase to our baseline, -pi:
                *phase = PhaseFxP::from_num(delta) - PhaseFxP::PI;
            }
        }
        ret
    }
}

fn clip_shape(x: Scalar) -> Scalar {
    const CLIP_MAX: Scalar = Scalar::lit("0x0.F");
    if x > CLIP_MAX {
        CLIP_MAX
    } else {
        x
    }
}

fn inverse(x: Scalar) -> crate::fixedmath::U8F8 {
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

fn one_over_one_minus_x(x: Scalar) -> crate::fixedmath::USample {
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
