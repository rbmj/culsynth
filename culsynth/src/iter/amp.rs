use super::*;

pub struct Amp<T: DspFormat, Signal: Source<T::Sample>, Gain: Source<T::Sample>> {
    signal: Signal,
    gain: Gain,
    phantom: core::marker::PhantomData<T>,
}

impl<T: DspFormat, Signal: Source<T::Sample>, Gain: Source<T::Sample>> Amp<T, Signal, Gain> {
    pub fn with_signal<NewSignal: Source<T::Sample>>(
        self,
        new_signal: NewSignal,
    ) -> Amp<T, NewSignal, Gain> {
        Amp {
            signal: new_signal,
            gain: self.gain,
            phantom: self.phantom,
        }
    }
    pub fn with_gain<NewGain: Source<T::Sample>>(
        self,
        new_gain: NewGain,
    ) -> Amp<T, Signal, NewGain> {
        Amp {
            signal: self.signal,
            gain: new_gain,
            phantom: self.phantom,
        }
    }
}

struct AmpIter<'a, T: DspFormat, Signal: Source<T::Sample> + 'a, Gain: Source<T::Sample> + 'a> {
    signal: Signal::It<'a>,
    gain: Gain::It<'a>,
}

impl<'a, T: DspFormat, Signal: Source<T::Sample> + 'a, Gain: Source<T::Sample> + 'a> Iterator
    for AmpIter<'a, T, Signal, Gain>
{
    type Item = T::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        Some(T::Sample::multiply(self.signal.next()?, self.gain.next()?))
    }
}

pub fn new<T: DspFormat>(
    _context: T::Context,
) -> Amp<T, IteratorSource<Repeat<T::Sample>>, IteratorSource<Repeat<T::Sample>>> {
    Amp {
        signal: repeat(T::Sample::zero()).into(),
        gain: repeat(T::Sample::one()).into(),
        phantom: Default::default(),
    }
}
