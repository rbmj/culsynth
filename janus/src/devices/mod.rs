pub mod osc;

use super::fixedmath;
use super::fixedmath::Sample as SampleFxP;

const STATIC_BUFFER_SIZE : usize = 256;
type BufferT<T> = [T; STATIC_BUFFER_SIZE];

