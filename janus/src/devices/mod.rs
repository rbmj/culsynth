pub mod osc;
pub mod env;
pub mod filt;

use super::fixedmath;
use super::fixedmath::Sample as SampleFxP;
use super::fixedmath::USample as USampleFxP;
use super::fixedmath::Note as NoteFxP;
use super::fixedmath::Scalar as ScalarFxP;

const STATIC_BUFFER_SIZE : usize = 256;
type BufferT<T> = [T; STATIC_BUFFER_SIZE];

//TODO: Support multiple sample rates
const SAMPLE_RATE : u16 = 44100;
const FRAC_4096_2PI_SR : fixedmath::U0F32 = fixedmath::U0F32::lit("0x0.9565925d");

pub trait Float : num_traits::Float + num_traits::FloatConst {
    //TODO
}

impl Float for f32 {}
impl Float for f64 {}

fn midi_note_to_frequency<T: Float>(note: T) -> T {
    let c69 = T::from(69).unwrap();
    let c12 = T::from(12).unwrap();
    let c440 = T::from(440).unwrap();
    c440 * ((note - c69) / c12).exp2()
}
