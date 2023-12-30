use super::*;

pub struct MixOsc<
    T: DspFormat,
    OscT: Source<osc::OscOutput<T>>,
    Sin: Source<T::Scalar>,
    Sq: Source<T::Scalar>,
    Tri: Source<T::Scalar>,
    Saw: Source<T::Scalar>,
> {
    osc: OscT,
    sin: Sin,
    sq: Sq,
    tri: Tri,
    saw: Saw,
    phantom: core::marker::PhantomData<T>,
}

impl<
        T: DspFormat,
        OscT: Source<osc::OscOutput<T>>,
        Sin: Source<T::Scalar>,
        Sq: Source<T::Scalar>,
        Tri: Source<T::Scalar>,
        Saw: Source<T::Scalar>,
    > MixOsc<T, OscT, Sin, Sq, Tri, Saw>
{
    pub fn with_input<NewOscT: Source<osc::OscOutput<T>>>(
        self,
        new_osc: NewOscT,
    ) -> MixOsc<T, NewOscT, Sin, Sq, Tri, Saw> {
        MixOsc {
            osc: new_osc,
            sin: self.sin,
            sq: self.sq,
            tri: self.tri,
            saw: self.saw,
            phantom: self.phantom,
        }
    }
    pub fn with_sin<NewSin: Source<T::Scalar>>(
        self,
        new_sin: NewSin,
    ) -> MixOsc<T, OscT, NewSin, Sq, Tri, Saw> {
        MixOsc {
            osc: self.osc,
            sin: new_sin,
            sq: self.sq,
            tri: self.tri,
            saw: self.saw,
            phantom: self.phantom,
        }
    }
    pub fn with_sq<NewSq: Source<T::Scalar>>(
        self,
        new_sq: NewSq,
    ) -> MixOsc<T, OscT, Sin, NewSq, Tri, Saw> {
        MixOsc {
            osc: self.osc,
            sin: self.sin,
            sq: new_sq,
            tri: self.tri,
            saw: self.saw,
            phantom: self.phantom,
        }
    }
    pub fn with_tri<NewTri: Source<T::Scalar>>(
        self,
        new_tri: NewTri,
    ) -> MixOsc<T, OscT, Sin, Sq, NewTri, Saw> {
        MixOsc {
            osc: self.osc,
            sin: self.sin,
            sq: self.sq,
            tri: new_tri,
            saw: self.saw,
            phantom: self.phantom,
        }
    }
    pub fn with_saw<NewSaw: Source<T::Scalar>>(
        self,
        new_saw: NewSaw,
    ) -> MixOsc<T, OscT, Sin, Sq, Tri, NewSaw> {
        MixOsc {
            osc: self.osc,
            sin: self.sin,
            sq: self.sq,
            tri: self.tri,
            saw: new_saw,
            phantom: self.phantom,
        }
    }
    pub fn sync_to<
        OscA: Source<osc::OscOutput<T>>,
        SinA: Source<T::Scalar>,
        SqA: Source<T::Scalar>,
        TriA: Source<T::Scalar>,
        SawA: Source<T::Scalar>,
        SyncedSrc: Source<(osc::OscOutput<T>, osc::OscOutput<T>)>,
    >(
        self,
        mix0: MixOsc<T, OscA, SinA, SqA, TriA, SawA>,
        src: SyncedSrc,
    ) -> SyncedMixOscs<T, SyncedSrc, SinA, SqA, TriA, SawA, Sin, Sq, Tri, Saw> {
        SyncedMixOscs {
            osc: src,
            sin0: mix0.sin,
            sin1: self.sin,
            sq0: mix0.sq,
            sq1: self.sq,
            tri0: mix0.tri,
            tri1: self.tri,
            saw0: mix0.saw,
            saw1: self.saw,
            phantom: self.phantom,
        }
    }
}

struct MixOscIter<
    'a,
    T: DspFormat,
    Osc: Source<osc::OscOutput<T>> + 'a,
    Sin: Source<T::Scalar> + 'a,
    Sq: Source<T::Scalar> + 'a,
    Tri: Source<T::Scalar> + 'a,
    Saw: Source<T::Scalar> + 'a,
> {
    osc_out: Osc::It<'a>,
    sin: Sin::It<'a>,
    sq: Sq::It<'a>,
    tri: Tri::It<'a>,
    saw: Saw::It<'a>,
}

impl<
        'a,
        T: DspFormat,
        Osc: Source<osc::OscOutput<T>>,
        Sin: Source<T::Scalar>,
        Sq: Source<T::Scalar>,
        Tri: Source<T::Scalar>,
        Saw: Source<T::Scalar>,
    > Iterator for MixOscIter<'a, T, Osc, Sin, Sq, Tri, Saw>
{
    type Item = T::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        let out: osc::OscOutput<T> = self.osc_out.next()?;
        Some(
            T::scale_sample(out.sin, self.sin.next()?)
                + T::scale_sample(out.sq, self.sq.next()?)
                + T::scale_sample(out.tri, self.tri.next()?)
                + T::scale_sample(out.saw, self.saw.next()?),
        )
    }
}

pub fn new<T: DspFormat>() -> MixOsc<
    T,
    IteratorSource<Repeat<osc::OscOutput<T>>>,
    IteratorSource<Repeat<T::Scalar>>,
    IteratorSource<Repeat<T::Scalar>>,
    IteratorSource<Repeat<T::Scalar>>,
    IteratorSource<Repeat<T::Scalar>>,
> {
    MixOsc {
        osc: repeat(osc::OscOutput::<T>::default()).into(),
        sin: repeat(T::Scalar::zero()).into(),
        sq: repeat(T::Scalar::zero()).into(),
        tri: repeat(T::Scalar::zero()).into(),
        saw: repeat(T::Scalar::zero()).into(),
        phantom: Default::default(),
    }
}

pub struct SyncedMixOscs<
    T: DspFormat,
    OscT: Source<(osc::OscOutput<T>, osc::OscOutput<T>)>,
    Sin0: Source<T::Scalar>,
    Sq0: Source<T::Scalar>,
    Tri0: Source<T::Scalar>,
    Saw0: Source<T::Scalar>,
    Sin1: Source<T::Scalar>,
    Sq1: Source<T::Scalar>,
    Tri1: Source<T::Scalar>,
    Saw1: Source<T::Scalar>,
> {
    osc: OscT,
    sin0: Sin0,
    sin1: Sin1,
    sq0: Sq0,
    sq1: Sq1,
    tri0: Tri0,
    tri1: Tri1,
    saw0: Saw0,
    saw1: Saw1,
    phantom: core::marker::PhantomData<T>,
}

struct SyncedMixOscsIter<
    'a,
    T: DspFormat,
    OscT: Source<(osc::OscOutput<T>, osc::OscOutput<T>)> + 'a,
    Sin0: Source<T::Scalar> + 'a,
    Sq0: Source<T::Scalar> + 'a,
    Tri0: Source<T::Scalar> + 'a,
    Saw0: Source<T::Scalar> + 'a,
    Sin1: Source<T::Scalar> + 'a,
    Sq1: Source<T::Scalar> + 'a,
    Tri1: Source<T::Scalar> + 'a,
    Saw1: Source<T::Scalar> + 'a,
> {
    osc_out: OscT::It<'a>,
    sin0: Sin0::It<'a>,
    sin1: Sin1::It<'a>,
    sq0: Sq0::It<'a>,
    sq1: Sq1::It<'a>,
    tri0: Tri0::It<'a>,
    tri1: Tri1::It<'a>,
    saw0: Saw0::It<'a>,
    saw1: Saw1::It<'a>,
}

impl<
        'a,
        T: DspFormat,
        OscT: Source<(osc::OscOutput<T>, osc::OscOutput<T>)>,
        Sin0: Source<T::Scalar>,
        Sq0: Source<T::Scalar>,
        Tri0: Source<T::Scalar>,
        Saw0: Source<T::Scalar>,
        Sin1: Source<T::Scalar>,
        Sq1: Source<T::Scalar>,
        Tri1: Source<T::Scalar>,
        Saw1: Source<T::Scalar>,
    > Iterator for SyncedMixOscsIter<'a, T, OscT, Sin0, Sin1, Sq0, Sq1, Tri0, Tri1, Saw0, Saw1>
{
    type Item = (T::Sample, T::Sample);
    fn next(&mut self) -> Option<Self::Item> {
        let out = self.osc_out.next()?;
        Some((
            T::scale_sample(out.0.sin, self.sin0.next()?)
                + T::scale_sample(out.0.sq, self.sq0.next()?)
                + T::scale_sample(out.0.tri, self.tri0.next()?)
                + T::scale_sample(out.0.saw, self.saw0.next()?),
            T::scale_sample(out.1.sin, self.sin1.next()?)
                + T::scale_sample(out.1.sq, self.sq1.next()?)
                + T::scale_sample(out.1.tri, self.tri1.next()?)
                + T::scale_sample(out.1.saw, self.saw1.next()?),
        ))
    }
}
