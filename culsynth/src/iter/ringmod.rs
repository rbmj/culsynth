use super::*;

pub struct RingModMixer<
    T: DspFormat,
    Input: Source<(T::Sample, T::Sample)>,
    MixA: Source<T::Scalar>,
    MixB: Source<T::Scalar>,
    MixMod: Source<T::Scalar>,
> {
    input: Input,
    mix_a: MixA,
    mix_b: MixB,
    mix_mod: MixMod,
    phantom: core::marker::PhantomData<T>,
}

impl<
        T: DspFormat,
        Input: Source<(T::Sample, T::Sample)>,
        MixA: Source<T::Scalar>,
        MixB: Source<T::Scalar>,
        MixMod: Source<T::Scalar>,
    > RingModMixer<T, Input, MixA, MixB, MixMod>
{
    pub fn with_input<NewInput: Source<(T::Sample, T::Sample)>>(
        self,
        new_input: NewInput,
    ) -> RingModMixer<T, NewInput, MixA, MixB, MixMod> {
        RingModMixer {
            input: new_input,
            mix_a: self.mix_a,
            mix_b: self.mix_b,
            mix_mod: self.mix_mod,
            phantom: self.phantom,
        }
    }
    pub fn with_mix_a<NewMixA: Source<T::Scalar>>(
        self,
        new_mix_a: NewMixA,
    ) -> RingModMixer<T, Input, NewMixA, MixB, MixMod> {
        RingModMixer {
            input: self.input,
            mix_a: new_mix_a,
            mix_b: self.mix_b,
            mix_mod: self.mix_mod,
            phantom: self.phantom,
        }
    }
    pub fn with_mix_b<NewMixB: Source<T::Scalar>>(
        self,
        new_mix_b: NewMixB,
    ) -> RingModMixer<T, Input, MixA, NewMixB, MixMod> {
        RingModMixer {
            input: self.input,
            mix_a: self.mix_a,
            mix_b: new_mix_b,
            mix_mod: self.mix_mod,
            phantom: self.phantom,
        }
    }
    pub fn with_mix_mod<NewMixMod: Source<T::Scalar>>(
        self,
        new_mix_mod: NewMixMod,
    ) -> RingModMixer<T, Input, MixA, MixB, NewMixMod> {
        RingModMixer {
            input: self.input,
            mix_a: self.mix_a,
            mix_b: self.mix_b,
            mix_mod: new_mix_mod,
            phantom: self.phantom,
        }
    }
}

struct RingModMixerIter<
    'a,
    T: DspFormat,
    Input: Source<(T::Sample, T::Sample)> + 'a,
    MixA: Source<T::Scalar> + 'a,
    MixB: Source<T::Scalar> + 'a,
    MixMod: Source<T::Scalar> + 'a,
> {
    input: Input::It<'a>,
    mix_a: MixA::It<'a>,
    mix_b: MixB::It<'a>,
    mix_mod: MixMod::It<'a>,
}

impl<
        'a,
        T: DspFormat,
        Input: Source<(T::Sample, T::Sample)> + 'a,
        MixA: Source<T::Scalar> + 'a,
        MixB: Source<T::Scalar> + 'a,
        MixMod: Source<T::Scalar> + 'a,
    > Iterator for RingModMixerIter<'a, T, Input, MixA, MixB, MixMod>
{
    type Item = T::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        let (a, b) = self.input.next()?;
        let mix_a = self.mix_a.next()?;
        let mix_b = self.mix_b.next()?;
        let mix_ring = self.mix_mod.next()?;
        let ring = T::Sample::multiply(a, b);
        Some(
            T::scale_sample(a, mix_a)
                .saturating_add(T::scale_sample(b, mix_b))
                .saturating_add(T::scale_sample(ring, mix_ring)),
        )
    }
}

pub fn new<T: DspFormat>(
    _context: T::Context,
) -> RingModMixer<
    T,
    IteratorSource<Repeat<(T::Sample, T::Sample)>>,
    IteratorSource<Repeat<T::Scalar>>,
    IteratorSource<Repeat<T::Scalar>>,
    IteratorSource<Repeat<T::Scalar>>,
> {
    RingModMixer {
        input: repeat((T::Sample::zero(), T::Sample::zero())).into(),
        mix_a: repeat(T::Scalar::zero()).into(),
        mix_b: repeat(T::Scalar::zero()).into(),
        mix_mod: repeat(T::Scalar::zero()).into(),
        phantom: Default::default(),
    }
}
