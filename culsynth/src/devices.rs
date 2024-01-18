//! This module contains definitions of several different DSP primitives.

use crate::{DspFloat, DspFormat, DspFormatBase, DspType};
use core::iter::{repeat, Iterator, Repeat};

pub(crate) mod amp;
pub(crate) mod env;
pub(crate) mod filt;
pub(crate) mod lfo;
pub(crate) mod mixer;
pub(crate) mod mixosc;
pub(crate) mod modfilt;
pub(crate) mod osc;
pub(crate) mod ringmod;

mod iter;

use crate::context::{Context, ContextFxP};
use crate::{fixedmath, EnvParamFxP, NoteFxP, SampleFxP, ScalarFxP, SignedNoteFxP};

/// A DSP Device
///
/// This is one of the central abstractions in this library.  A device is a
/// logical component, or set of components, that takes a set of input signals
/// and applies some logic to these signals, according to a set of parameters,
/// that then produces a set of output signals.
///
/// For example, a filter would take an audio input, a cutoff frequency
/// parameter, and provide an audio output.  If this filter also featured
/// keyboard tracking, the keyboard note signal would be an additional input,
/// while the amount of keyboard tracking to use (or even whether to enable it)
/// would be a parameter.
///
/// In the general case (e.g. many modular setups) the line between these two
/// can be blurred, or even does not exist.  The semantic distinction is used
/// here as a convenience.  For example, one could define one function to get
/// the value of parameters that have been set in a user interface, and another
/// to get the value of the input from other stages of the synthesizer logic,
/// and not need any glue code or to worry about sensible defaults/uninitialized
/// members based on one of the functions having limited information on the
/// synth's state.
pub trait Device<T: DspFormat> {
    /// The input type for this device.  Used to represent signals within the
    /// synthesizer, e.g. a control voltage, audio signal, etc.
    type Input;
    /// The parameter type for this device.  Used to represent a value that
    /// changes how the device acts on its inputs.  A good analogy is any
    /// knob on a synth would logically fall into this category.
    type Params;
    /// The output type for this device.
    type Output;
    /// Within the provided `context`, take one sample of `input` and execute
    /// the design's DSP logic using `params`, then return a sample of output.
    fn next(
        &mut self,
        context: &T::Context,
        input: Self::Input,
        params: Self::Params,
    ) -> Self::Output;
    /// This is similar to [Device::next], but works on iterators and returns
    /// an iterator to the results
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

/// An iterator over a [Device] returned by [Device::process]
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
pub use iter::env::{new_env_param_iter, EnvParamIter};
pub use iter::filt::{new_filt_param_iter, FiltParamIter};
pub use iter::lfo::{new_lfo_param_iter, LfoParamIter};
pub use iter::mixosc::{
    new_mixosc_param_iter, new_synced_mixoscs_param_iter, MixOscParamIter, SyncedMixOscsParamIter,
};
pub use iter::modfilt::{
    new_modfilt_input_iter, new_modfilt_param_iter, ModFiltInputIter, ModFiltParamIter,
};
pub use iter::osc::{
    new_osc_param_iter, new_synced_oscs_param_iter, OscParamIter, SyncedOscsParamIter,
};
pub use iter::ringmod::{
    new_ringmod_input_iter, new_ringmod_param_iter, RingModInputIter, RingModParamIter,
};
pub use lfo::{Lfo, LfoOptions, LfoParams, LfoWave};
pub use mixer::Mixer;
pub use mixosc::{MixOsc, MixOscParams, SyncedMixOscs, SyncedMixOscsOutput, SyncedMixOscsParams};
pub use modfilt::{ModFilt, ModFiltInput, ModFiltParams};
pub use osc::{Osc, OscOutput, OscParams, SyncedOscs, SyncedOscsOutput, SyncedOscsParams};
pub use ringmod::{RingMod, RingModInput, RingModParams};
