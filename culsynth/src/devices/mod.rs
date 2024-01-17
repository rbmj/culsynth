//! This module contains definitions of several different DSP primitives.

use core::iter::Iterator;

pub(crate) mod amp;
pub(crate) mod env;
pub(crate) mod filt;
pub(crate) mod lfo;
pub(crate) mod mixer;
pub(crate) mod mixosc;
pub(crate) mod modfilt;
pub(crate) mod osc;
pub(crate) mod ringmod;

use crate::context::{Context, ContextFxP};
use crate::{fixedmath, DspFormat, EnvParamFxP, NoteFxP, SampleFxP, ScalarFxP, SignedNoteFxP};

pub trait Device<T: DspFormat> {
    type Input;
    type Params;
    type Output;
    fn next(
        &mut self,
        context: &T::Context,
        input: Self::Input,
        params: Self::Params,
    ) -> Self::Output;
    fn process<'a, InputIt: Iterator<Item = Self::Input>, ParamIt: Iterator<Item = Self::Params>>(
        &'a mut self,
        context: &'a T::Context,
        input: InputIt,
        params: ParamIt,
    ) -> DeviceIter<'a, T, Self, InputIt, ParamIt>
    where
        Self: Sized,
    {
        DeviceIter {
            dev: self,
            ctx: context,
            input,
            params,
        }
    }
}

// FIXME: D ?Sized bound??
pub struct DeviceIter<
    'a,
    T: DspFormat,
    D: Device<T>,
    InputIt: Iterator<Item = D::Input>,
    ParamIt: Iterator<Item = D::Params>,
> {
    dev: &'a mut D,
    ctx: &'a T::Context,
    input: InputIt,
    params: ParamIt,
}

impl<
        'a,
        T: DspFormat,
        D: Device<T>,
        InputIt: Iterator<Item = D::Input>,
        ParamIt: Iterator<Item = D::Params>,
    > Iterator for DeviceIter<'a, T, D, InputIt, ParamIt>
{
    type Item = D::Output;
    fn next(&mut self) -> Option<D::Output> {
        Some(
            self.dev
                .next(self.ctx, self.input.next()?, self.params.next()?),
        )
    }
}

pub use amp::Amp;
pub use env::{Env, EnvParams};
pub use filt::{Filt, FiltOutput, FiltParams};
pub use lfo::{Lfo, LfoOptions, LfoParams, LfoWave};
pub use mixer::Mixer;
pub use mixosc::{MixOsc, MixOscParams, SyncedMixOscs, SyncedMixOscsOutput, SyncedMixOscsParams};
pub use modfilt::{ModFilt, ModFiltInput, ModFiltParams};
pub use osc::{Osc, OscOutput, OscParams, SyncedOscs, SyncedOscsOutput, SyncedOscsParams};
pub use ringmod::{RingMod, RingModInput, RingModParams};
