//! Fixed-Point math functions used internally by the synthesizer.
//!
//! As a quick aside - all of these functions prioritize
//! speed over accuracy.  Don't use for scientific calculations...
//! you have been warned!

use core::ops::Add;
use fixed::types::extra::{IsLessOrEqual, LeEqU16, LeEqU32, True, Unsigned, U16, U31};
pub use fixed::types::*;
use fixed::{FixedI16, FixedI32, FixedU16, FixedU32};

/// A fixed point number representing a sample or otherwise generic piece of data
/// within the synthesizer.  These are 16 bit signed fixed point numbers with 12
/// fractional bits.  Put another way, our reference (0dB) level is set at an
/// amplitude of 2^12, and we have 3 bits (9dB) of headroom before clipping,
/// since we lose a bit for the sign bit.
pub type Sample = I4F12;
/// A unsigned data type with the same number of fractional bits as a sample.
/// Usually used for internal processing to steal an extra bit of precision when
/// we know a value cannot be negative.
pub type USample = U4F12;
/// A unsigned 16 bit fixed point number representing a note/pitch, with 7 integral
/// bits and 9 fractional.  The integral bits correspond to the MIDI note numbers,
/// i.e. a value of 69.0 represents A440 and tuning is 12 tone equal temprament.
///
/// 9 fractional bits provides a resolution of about 0.2 cents, but most of the
/// functions in this library will only be accurate to about 0.5-1 cent.
pub type Note = U7F9;
/// A signed variant of the above.  This does not have the precision to represent
/// the entire range of the MIDI note range, but is helpful for small adjustments
/// (e.g. pitch bend)
pub type SignedNote = I7F9;
/// A unsigned 32 bit fixed point number representing a frequency in Hz.
/// This uses 14 integral bits and 18 fractional bits
pub type Frequency = U14F18; // 14 bits will hold the highest MIDI freq
/// A unsigned 16 bit fixed point number in the interval `[0, 1)`.  Used primarily
/// for "scaling" signals in amplitude, hence the (admittedly not great but useful)
/// name assigned.  Note that 0xFFFF is slightly less than 1.0, so we will lose
/// a (very) small amount of signal when maxed out.
pub type Scalar = U0F16;
/// A signed 16 bit fixed point number in the interval `[-1, 1)`.
pub type IScalar = I1F15;
/// An unsigned 32 bit fixed point number with 16 fractional bits
/// Called a ScaleOutput because it can hold 1/x or 1*x for any Scalar
pub type ScaleOutput = U16F16;

//for the following constants, we'll use as many bits as we can fit
//in a couple cases, that means we'll buy a extra place shifting right
const FRAC_2_3: Scalar = Scalar::lit("0x0.aaab"); //0x0.a repeating

// (4/3)*ln(2)
const FRAC_4LN2_3: Scalar = Scalar::lit("0x0.ec98");
// Frequency of E4 ~= 329.63 Hz
const FREQ_E4: Frequency = Frequency::lit("329.627557");

// The functions below are helpers because the one's in std are not const
// evaluable on stable... when that gets out of nightly these can be removed

// Calculate x!
const fn factorial(mut x: u64) -> u64 {
    let mut ret = 1;
    while x > 1 {
        ret *= x;
        x -= 1;
    }
    ret
}

// Calculate x^n
// should do pow-mod here but it's only to generate the lookup table...
const fn float_powi(x: f32, mut n: u64) -> f32 {
    let mut ret = 1f32;
    while n > 0 {
        ret *= x;
        n -= 1;
    }
    ret
}

// Absolute value
const fn f_abs(x: f32) -> f32 {
    if x < 0f32 {
        -x
    } else {
        x
    }
}

/// Calculate sin(x*pi)
pub fn sin_pi(x: IScalar) -> IScalar {
    const fn calc(theta: IScalar) -> IScalar {
        const DELTA: f32 = 1f32 / (1 << 17) as f32;
        const CONV_FACTOR: f32 = (1u16 << 15) as f32;
        let theta_float = std::f32::consts::PI * (theta.to_bits() as f32) / CONV_FACTOR;
        let mut n = 1u64;
        let mut term = theta_float;
        let mut acc = theta_float;
        while f_abs(term) > DELTA {
            let sign = if n % 2 != 0 { -1f32 } else { 1f32 };
            let exp = 2 * n + 1;
            term = sign * float_powi(theta_float, exp) / factorial(exp) as f32;
            acc += term;
            n += 1;
        }
        IScalar::from_bits((acc * CONV_FACTOR) as i16)
    }
    const fn generate() -> [IScalar; 257] {
        let mut table = [IScalar::ZERO; 257];
        let mut i = i8::MIN as isize;
        while i <= i8::MAX as isize {
            let value = IScalar::from_bits((i as i16) << 8);
            table[(i - (i8::MIN as isize)) as usize] = calc(value);
            i += 1;
        }
        table
    }
    const LOOKUP_TABLE: [IScalar; 257] = generate();
    let x_bits = (x.to_bits() as i32).wrapping_add(1 << 15) as u16;
    let idx = (x_bits >> 8) as usize;
    let frac = Scalar::from_bits((x_bits & 0xFF) << 8);
    LOOKUP_TABLE[idx]
        + IScalar::from_num((LOOKUP_TABLE[idx + 1] - LOOKUP_TABLE[idx]).wide_mul_unsigned(frac))
}

/// Calculate cos(x*pi)
pub fn cos_pi(x: IScalar) -> IScalar {
    sin_pi(IScalar::lit("0.5").wrapping_sub(x))
}

pub fn inverse(x: Scalar) -> ScaleOutput {
    const fn calc(n: Scalar) -> ScaleOutput {
        let n_float = (n.to_bits() as f32) / 65536f32;
        let ret = (65536f32 / n_float) as u32;
        ScaleOutput::from_bits(ret)
    }
    const fn generate() -> [ScaleOutput; 257] {
        let mut table = [ScaleOutput::MAX; 257];
        table[256] = ScaleOutput::ONE;
        let mut i = 1u16;
        while i < 256u16 {
            let value = Scalar::from_bits(i << 8);
            table[i as usize] = calc(value);
            i += 1;
        }
        table
    }
    const LOOKUP_TABLE: [ScaleOutput; 257] = generate();
    let x_bits = x.to_bits();
    let idx = (x_bits >> 8) as usize;
    let frac = Scalar::from_bits((x_bits & 0xFF) << 8);
    let table_diff = LOOKUP_TABLE[idx + 1] - LOOKUP_TABLE[idx];
    LOOKUP_TABLE[idx] + scale_fixedfloat(table_diff, frac)
}

pub fn one_over_one_plus_scalar(x: Scalar) -> Scalar {
    const fn calc(n: Scalar) -> Scalar {
        let n_float = (n.to_bits() as f32) / 65536f32;
        let ret = (65536f32 / (1f32 + n_float)) as u16;
        Scalar::from_bits(ret)
    }
    const fn generate() -> [Scalar; 257] {
        let mut table = [Scalar::MAX; 257];
        table[256] = Scalar::lit("0.5");
        let mut i = 1u16;
        while i < 256u16 {
            let value = Scalar::from_bits(i << 8);
            table[i as usize] = calc(value);
            i += 1;
        }
        table
    }
    const LOOKUP_TABLE: [Scalar; 257] = generate();
    let x_bits = x.to_bits();
    let idx = (x_bits >> 8) as usize;
    let frac = Scalar::from_bits((x_bits & 0xFF) << 8);
    LOOKUP_TABLE[idx] - Scalar::from_num((LOOKUP_TABLE[idx] - LOOKUP_TABLE[idx + 1]).wide_mul(frac))
}

pub fn one_over_one_minus(x: Scalar) -> ScaleOutput {
    const fn calc(n: Scalar) -> ScaleOutput {
        let n_float = (n.to_bits() as f32) / 65536f32;
        let ret = (65536f32 / (1f32 - n_float)) as u32;
        ScaleOutput::from_bits(ret)
    }
    const fn generate() -> [ScaleOutput; 257] {
        let mut table = [ScaleOutput::MAX; 257];
        table[0] = ScaleOutput::ONE;
        let mut i = 1u16;
        while i < 256u16 {
            let value = Scalar::from_bits(i << 8);
            table[i as usize] = calc(value);
            i += 1;
        }
        table
    }
    const LOOKUP_TABLE: [ScaleOutput; 257] = generate();
    let x_bits = x.to_bits();
    let idx = (x_bits >> 8) as usize;
    let frac = Scalar::from_bits((x_bits & 0xFF) << 8);
    let table_diff = LOOKUP_TABLE[idx + 1] - LOOKUP_TABLE[idx];
    LOOKUP_TABLE[idx] + scale_fixedfloat(table_diff, frac)
}

/// Take a 32 bit fixed point number A and a 16 bit fixed point number B, and return
/// a 32 bit fixed point number representing the product of those two numbers with
/// the same number of integral bits as A.  This uses some very, very basic software
/// floating point logic internally to avoid a widening multiply.  Will result in
/// some loss of precision if A has more than 16 significant digits
pub fn scale_fixedfloat<FracA, FracB>(a: FixedU32<FracA>, b: FixedU16<FracB>) -> FixedU32<FracA>
where
    FracA: Unsigned + LeEqU32,
    FracB: Unsigned + LeEqU16 + Add<U16> + IsLessOrEqual<FracA>,
{
    let bbits = FixedU16::<FracB>::INT_NBITS;
    let shift = a.leading_zeros();
    let a_shifted = U0F32::from_bits(a.unwrapped_shl(shift).to_bits());
    let prod = b.wide_mul(U0F16::from_num(a_shifted));
    let res = if shift > bbits {
        prod.unwrapped_shr(shift - bbits)
    } else {
        prod.unwrapped_shl(bbits - shift)
    };
    FixedU32::<FracA>::from_bits(res.to_bits())
}

/// Widen the given 16 bit fixed point number to a 32 bit fixed point number
pub fn widen_i<Frac>(a: FixedI16<Frac>) -> FixedI32<Frac>
where
    Frac: Unsigned + LeEqU16 + LeEqU32,
{
    FixedI32::<Frac>::from_num(a)
}

fn one_over_one_plus_helper<Frac>(n: FixedU32<Frac>) -> (U1F31, u32)
where
    Frac: Unsigned + IsLessOrEqual<U31, Output = True> + LeEqU32,
{
    let nbits = FixedU32::<Frac>::INT_NBITS;
    let x = n + FixedU32::<Frac>::ONE;
    let mut shift = x.leading_zeros();
    let mut x_shifted = U1F31::from_bits(x.to_bits()).unwrapped_shl(shift);
    shift += 1;
    if x_shifted >= U1F31::SQRT_2 {
        shift -= 1;
        x_shifted = x_shifted.unwrapped_shr(1);
    }
    (x_shifted, nbits - shift)
}

/// Calculate 1/(1+x) for a 32 bit fixed point number and return the result as
/// a tuple representing a sixteen bit number in scientific notation - the first
/// element is a sixteen bit fixed point number with 1 integral bit, and the
/// second element represents the negative of the exponent (base 2).
///
/// Internally, this uses a quadratic taylor series expansion about 2^-x for the
/// closest value of 2^-x on a logarithmic basis.
///
/// # Examples
///
/// ```no_compile
/// use culsynth::fixedmath::{one_over_one_plus, U16F16};
/// let x = U16F16::ONE;
/// let (y, exp) = one_over_one_plus(x); // 1 / (1 + 1) == 1 / 2
/// assert!(y.unwrapped_shr(exp) == U16F16::lit("0.5"));
/// ```
pub fn one_over_one_plus<Frac>(x: FixedU32<Frac>) -> (U1F15, u32)
where
    Frac: Unsigned + IsLessOrEqual<U31, Output = True> + LeEqU32,
{
    let (x_shifted, shift) = one_over_one_plus_helper(x);
    let x_shifted_trunc = U1F15::from_num(x_shifted);
    let x2 = I3F29::from_num(x_shifted_trunc.wide_mul(x_shifted_trunc));
    let one_minus_x = I3F29::ONE - I3F29::from_num(x_shifted);
    (
        U1F15::from_num(x2 + one_minus_x + one_minus_x.unwrapped_shl(1)),
        shift,
    )
}

/// A very rough approximation of tan(x) using a quadratic taylor series expansion.
/// Used primarily in filter gain prewarping, where it's accurate enough for angles
/// representing lower frequencies where precise tuning is more important.  Will be
/// somewhat inaccurate at frequencies above about half the Nyquist frequency.
pub fn tan_fixed(x: U0F16) -> U1F15 {
    let x2 = x.wide_mul(x);
    let x2_over3 = U0F16::from_num(x2).wide_mul(FRAC_2_3).unwrapped_shr(1);
    let res_over_x = U1F15::from_num(x2_over3) + U1F15::ONE;
    U1F15::from_num(res_over_x.wide_mul(x))
}

/// calculate e^x in the range [-0.5, 0.5) using an order 4 Taylor series
fn exp_fixed_small(x: I0F16) -> U2F14 {
    // e^x ~= 1 + x + x^2/2! + x^3/3! + x^4/4!
    //     ~= 1 + x * { 1 + x/2 * [ 1 + x/3 * ( 1 + x/4 )]}
    // c_nested = 1 + x/4
    let c_nested = I3F29::ONE + I3F29::from_num(x).unwrapped_shr(2);
    // b = x * (2/3) * (1/2) = x/3
    let b = I0F16::from_num(x.wide_mul_unsigned(FRAC_2_3).unwrapped_shr(1));
    // b_nested = 1 + b*c_nested
    let b_nested = I3F29::ONE + I3F13::from_num(c_nested).wide_mul(b);
    // a = x/2
    let a = I3F13::from_num(I3F29::from_num(x).unwrapped_shr(1));
    // a_nested = 1 + a*b_nested
    let a_nested = I6F26::ONE + I3F13::from_num(b_nested).wide_mul(a);
    // exp(x) ~= 1 + x*a_nested
    U2F14::from_num(x.wide_mul(I3F13::from_num(a_nested)) + I3F29::ONE)
}

/// Calculate e^x of a 16 bit signed fixed point number with 13 fractional bits
/// (that is to say, between -4 and 4), and return it as a unsigned 32 bit
/// number with 24 fractional bits.
pub fn exp_fixed(x: I3F13) -> U8F24 {
    // going to use the fact that our input domain is limited to [-4, 4)
    // to calculate e^x as the product e^(int(x))*e^(frac(x)), then since
    // frac is in the range [0, 1) we'll shift that left by a factor of
    // sqrt(e) to be about the easy Taylor series expansion, i.e.:
    //
    //     e^x ~= e^[int(x) + 1/2] * e^[frac(x) - 1/2]
    //
    // We'll use a taylor series (see exp_fixed_small) for the fractional
    // part, and generate a lookup table (plus some offset shifts) for
    // the integral part (plus 0.5)
    //
    // Lookup Table generated using the following python snippet:
    //
    // v = [x + 0.5 for x in range(-4,4)]
    // for v in values:
    //      h = float.hex(exp(v))
    // 	    pwr = str(int(h.split('p')[1]) + 1)
    // 	    x = int('0x1' + h[4:8], 0)
    // 	    print('(Scalar::lit("0x0.' + hex(x >> 1)[2:] + '"), ' + str(pwr) + '),')
    //
    // We've scaled all the scientific notation representations to be in
    // [0, 1) to maximize the significant bits, so this table also stores
    // how many left/right shifts were performed (and subsequently must be
    // reversed) in the generation of the table.
    const LOOKUP_TABLE: [(Scalar, u32, u32); 8] = [
        (Scalar::lit("0x0.f760"), 5, 0),
        (Scalar::lit("0x0.a81c"), 3, 0),
        (Scalar::lit("0x0.e47c"), 2, 0),
        (Scalar::lit("0x0.9b45"), 0, 0),
        (Scalar::lit("0x0.d309"), 0, 1),
        (Scalar::lit("0x0.8f69"), 0, 3),
        (Scalar::lit("0x0.c2eb"), 0, 4),
        (Scalar::lit("0x0.8476"), 0, 6),
    ];
    const ONE_HALF: I3F13 = I3F13::lit("0.5");
    // Note: x.int() rounds towards -inf, not zero for fixed point numbers
    let x_int = x.int().to_num::<i8>(); //in the range [-4, 4)
    let index = (x_int + 4) as usize;
    let frac_exp = exp_fixed_small(I0F16::from_num(x.frac() - ONE_HALF));
    let (multiplier, right, left) = LOOKUP_TABLE[index];
    // do a 16 bit widening multiply, then we'll store in a 64 bit fixed point number to make
    // sure we retain all precision on shifting.
    let retval = U8F56::from_num(multiplier.wide_mul(frac_exp));
    U8F24::from_num(retval.unwrapped_shl(left).unwrapped_shr(right))
}

// Take a MIDI note representation and put it closer to 1.0/octave
// We'll leave it as 16 semitones/oct here for convenience (so this is
// just bit twiddling)
fn note_to_value(note: Note) -> I3F13 {
    const C64: I19F13 = I19F13::lit("64");
    //subtract 64 to make it centered about zero
    let note_signed = I19F13::from_num(note) - C64;
    //divide down to 16 semitones/octave
    I3F13::from_num(note_signed.unwrapped_shr(4))
}

/// Convert a MIDI note number to a frequency in Hz
pub fn midi_note_to_frequency(note: Note) -> Frequency {
    // f = f0 * e^(note/12)
    let note_xform = note_to_value(note);
    //this change in representation brings us to 1.0 = 16 semitones
    //need to multiply by (4/3) in order to scale it to 1.0=12 semitones
    //we also need to multiply by ln(2) so we'll do that in one step
    let power = I3F13::from_num(note_xform.wide_mul_unsigned(FRAC_4LN2_3));
    //Our note numbers are centered about E4, so that's our f0:
    FREQ_E4 * U14F18::from_num(exp_fixed(power))
}

#[cfg(test)]
mod tests {
    use super::super::util::calculate_cents;
    use super::*;
    use fixed::traits::ToFixed;
    //test for correctness of constants
    #[test]
    fn const_fraction_correctness() {
        assert_eq!(FRAC_2_3, Scalar::from_num(2.0 / 3.0));
    }
    //TODO: Test sin/cos
    #[test]
    fn midi_pitch_calculations() {
        for i in 0..=127 {
            let pitch = 440.0 * f32::powf(2.0, ((i - 69) as f32) / 12.0);
            let pitch_fixed = midi_note_to_frequency(i.to_fixed()).to_num::<f32>();
            let error = calculate_cents(pitch, pitch_fixed);
            assert!(error < 1.0); //less than one cent per note
        }
    }
}
