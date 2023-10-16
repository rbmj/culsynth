pub mod osc;

use super::fixedmath;
use super::fixedmath::Sample as SampleFxP;
use super::fixedmath::USample as USampleFxP;
use super::fixedmath::Note as NoteFxP;

const STATIC_BUFFER_SIZE : usize = 256;
type BufferT<T> = [T; STATIC_BUFFER_SIZE];

