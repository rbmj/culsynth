pub use fixed::types::*;
use fixed::types::extra::{Unsigned, IsLessOrEqual, LeEqU32, LeEqU16, U16, U31, True};
use fixed::{FixedU32, FixedU16};
use fixed::traits::ToFixed;
use core::ops::Add;

pub type Sample = I4F12;       // Provides 3 bits (9dB) of headroom
pub type USample = U4F12;      // (Unsigned)
pub type Note = U7F9;          // 7 bits for note # plus 9 bits for bends
pub type Frequency = U14F18;   // 14 bits will hold the highest MIDI freq
pub type Scalar = U0F16;       // A number in [0, 1) for multiplication

//for the following constants, we'll use as many bits as we can fit
//in a couple cases, that means we'll buy a extra place shifting right
const FRAC_16_21 : Scalar = Scalar::lit("0x0.c30c"); //0x0.c30 repeating
const FRAC_4_5 : Scalar = Scalar::lit("0x0.cccd");   //0x0.c repeating
const FRAC_2_3 : Scalar = Scalar::lit("0x0.aaab");   //0x0.a repeating
const FRAC_8_15 : Scalar = Scalar::lit("0x0.8889");  //0x0.8 repeating

//when to apply a small angle approximation
const SMALL_ANGLE_LESS : Sample = Sample::lit("0x0.1");

// (4/3)*ln(2)
const FRAC_4LN2_3 : Scalar = Scalar::lit("0x0.ec98");
// Frequency of E4 ~= 329.63 Hz
const FREQ_E4 : Frequency = Frequency::lit("329.627557");
// 256 / [the freq above]
const FRAC_256_FREQ_E4 : U8F24 = U8F24::lit("0x0.c6d17e");


pub fn scale_fixedfloat<FracA, FracB>(a: FixedU32<FracA>, b: FixedU16<FracB>) -> FixedU32<FracA>
    where FracA: Unsigned + LeEqU32,
          FracB: Unsigned + LeEqU16 + Add<U16> + IsLessOrEqual<FracA>,
{
    let bbits = FixedU16::<FracB>::INT_NBITS;
    let shift = a.leading_zeros();
    //ALL WRONG - logical shift by shift - abits
    let a_shifted = U0F32::from_bits(a.unwrapped_shl(shift).to_bits());
    let prod = b.wide_mul(U0F16::from_num(a_shifted));
    let res = if shift > bbits {
        prod.unwrapped_shr(shift - bbits)
    }
    else {
        prod.unwrapped_shl(bbits - shift)
    };
    FixedU32::<FracA>::from_bits(res.to_bits())
}

// As a quick aside - all of these functions prioritize
// speed over accuracy.  Don't use for scientific calculations...
// you have been warned!

fn one_over_one_plus_helper<Frac>(n: FixedU32<Frac>) -> (U1F31, u32)
    where Frac: Unsigned + IsLessOrEqual<U31, Output = True> + LeEqU32
{
    let nbits = FixedU32::<Frac>::INT_NBITS;
    let x = n + FixedU32::<Frac>::ONE;
    let mut shift = x.leading_zeros();
    let mut x_shifted = U1F31::from_bits(x.to_bits())
        .unwrapped_shl(shift);
    shift = shift + 1;
    if x_shifted >= U1F31::SQRT_2 {
        shift -= 1;
        x_shifted = x_shifted.unwrapped_shr(1);
    }
    (x_shifted, nbits - shift)
}

pub fn one_over_one_plus<Frac>(x: FixedU32<Frac>) -> (U1F15, u32)
    where Frac: Unsigned + IsLessOrEqual<U31, Output = True> + LeEqU32
{
    let (x_shifted, shift) = one_over_one_plus_helper(x);
    let x_shifted_trunc = U1F15::from_num(x_shifted);
    let x2 = I3F29::from_num(x_shifted_trunc.wide_mul(x_shifted_trunc));
    let one_minus_x = I3F29::ONE - I3F29::from_num(x_shifted);
    (U1F15::from_num(x2 + one_minus_x + one_minus_x.unwrapped_shl(1)), shift)
}

pub fn one_over_one_plus_highacc(x: U0F16) -> (U1F15, u32) {
    let (x_shifted, shift) = one_over_one_plus_helper(U16F16::from_num(x));
    const FIVE_NAR : U3F13 = U3F13::lit("5");
    const FIVE : U4F28 = U4F28::lit("5");
    const TEN : U4F28 = U4F28::lit("10");
    let x_shifted_trunc = U1F15::from_num(x_shifted);
    let p1 = x_shifted_trunc.wide_mul(FIVE_NAR - U3F13::from_num(x_shifted_trunc));
    let p2 = x_shifted_trunc.wide_mul(U3F13::from_num(TEN - p1));
    let p3 = x_shifted_trunc.wide_mul(U3F13::from_num(TEN - p2));
    (U1F15::from_num(FIVE - p3), shift)
}

pub fn sin_fixed(x : Sample) -> Sample {
    //small angle approximation.  Faster and removes 0 as an edge case
    if x.abs() < SMALL_ANGLE_LESS {
        return x;
    }
    //x^2.  Always >0, so use unsigned to avoid overflow with 4 bits
    //perhaps use wrapping_from_num if !debug? 
    let x2 = USample::from_num(x.wide_mul(x)); 
    // sin(x) = x - x^3/3! + x^5/5! - x^7/7! (+ higher order terms)
    //        = x { 1 - x^2/6 [ 1 - x^2/20 ( 1 - x^2/42 ) ] }
    //
    // let c = x^2 * (16/21) * (1/32) = x^2/42
    let c = FRAC_16_21.wide_mul(x2).unwrapped_shr(5);
    // let c_nested = 1 - c
    let c_nested = Scalar::from_num(U4F28::ONE - c);
    // let b = x^2 * (4/5) * (1/16) = x^2 / 20
    let b = Scalar::from_num(FRAC_4_5.wide_mul(x2).unwrapped_shr(4));
    // let b_nested = 1 - b*c_nested
    let b_nested = Scalar::from_num(
        U16F16::ONE - U16F16::from_num(b.wide_mul(c_nested)));
    // let a = x^2 * (2/3) * (1/4) = x^2 / 6
    let a = U2F14::from_num(FRAC_2_3.wide_mul(x2).unwrapped_shr(2));
    let a_nested = I1F15::from_num(
        I2F30::ONE - I2F30::from_num(a.wide_mul(b_nested)));
    // sin(x) ~= x*a_nested
    Sample::from_num(x.wide_mul(a_nested))
}

pub fn cos_fixed(x: Sample) -> Sample {
    let x2 = USample::from_num(x.wide_mul(x));
    //small angle approximation.  Faster and removes 0 as an edge case
    if x.abs() < SMALL_ANGLE_LESS {
        return Sample::ONE - Sample::from_num(x2.unwrapped_shr(1));
    }
    // c = x^2 * (8/15) * (1/16) = x^2/30
    let c = FRAC_8_15.wide_mul(x2).unwrapped_shr(4);
    let c_nested = Scalar::from_num(U4F28::ONE - c);
    // b = x^2 * (2/3) * (1/8) = x^2/12
    let b = Scalar::from_num(FRAC_2_3.wide_mul(x2).unwrapped_shr(3));
    let b_nested = Scalar::from_num(
        U16F16::ONE - U16F16::from_num(b.wide_mul(c_nested)));
    // let a = x^2/2, a*b_nested:
    let a_mult_b_nested = x2.wide_mul(b_nested).unwrapped_shr(1);
    // cos(x) ~= 1 - a*b_nested
    Sample::ONE - Sample::from_num(a_mult_b_nested)
}

pub fn tan_fixed(x: U0F16) -> U1F15 {
    let x2 = x.wide_mul(x);
    let x2_over3 = U0F16::from_num(x2).wide_mul(FRAC_2_3).unwrapped_shr(1);
    let res_over_x = U1F15::from_num(x2_over3) + U1F15::ONE;
    U1F15::from_num(res_over_x.wide_mul(x))
}

// calculate e^x in the range [-0.5, 0.5) using an order 4 Taylor series
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
    U2F14::from_num(x.wide_mul(I3F13::from_num(a_nested)) +  I3F29::ONE)
}

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
    const LOOKUP_TABLE : [(Scalar, u32, u32); 8] = [
        (Scalar::lit("0x0.f760"), 5, 0),
        (Scalar::lit("0x0.a81c"), 3, 0),
        (Scalar::lit("0x0.e47c"), 2, 0),
        (Scalar::lit("0x0.9b45"), 0, 0),
        (Scalar::lit("0x0.d309"), 0, 1),
        (Scalar::lit("0x0.8f69"), 0, 3),
        (Scalar::lit("0x0.c2eb"), 0, 4),
        (Scalar::lit("0x0.8476"), 0, 6)];
    let x_int = x.int().to_num::<i8>(); //in the range [-4, 4)
    let index = (x_int + 4) as usize;
    let frac_exp = exp_fixed_small(I0F16::from_num(x.frac() - I3F13::lit("0.5")));
    let (multiplier, right, left) = LOOKUP_TABLE[index];
    let retval = U8F56::from_num(multiplier.wide_mul(frac_exp));
    U8F24::from_num(retval.unwrapped_shl(left).unwrapped_shr(right))
}

// Take a MIDI note representation and put it closer to 1.0/octave
// We'll leave it as 16 semitones/oct here for convenience (so this is
// just bit twiddling)
fn note_to_value(note: Note) -> I3F13 {
    //subtract 64 to make it centered about zero
    let note_signed = I19F13::from_num(note) - I19F13::lit("64");
    //divide down to 16 semitones/octave
    I3F13::from_num(note_signed.unwrapped_shr(4))
}

// Convert a MIDI note number to a frequency in Hz
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

// Convert a MIDI note number to a period in seconds
pub fn midi_note_to_period(note: Note) -> U0F32 {
    let note_xform = note_to_value(note);
    //same logic as midi_note_to_frequency, but we the negative exponent this time
    let power = I3F13::from_num(note_xform.wide_mul_unsigned(FRAC_4LN2_3));
    //this will be the period times 256 (note numerator of the constant)
    let x = FRAC_256_FREQ_E4 * exp_fixed(power.unwrapped_neg());
    //now we just need to divide by 256 without losing precision
    //this should be optimized to a no-op
    U32F32::from_num(x).unwrapped_shr(8).to_fixed()
    //note there will be minimum 3 leading zeros since midi note 0 is ~8.2Hz
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::util::calculate_cents;
    //test for correctness of constants
    #[test]
    fn const_fraction_correctness() {
        assert_eq!(FRAC_16_21, Scalar::from_num(16.0/21.0));
        assert_eq!(FRAC_4_5, Scalar::from_num(4.0/5.0));
        assert_eq!(FRAC_2_3, Scalar::from_num(2.0/3.0));
        assert_eq!(FRAC_8_15, Scalar::from_num(8.0/15.0));
    }
    //
    //SIN TESTS:
    //
    #[test]
    fn sin_zero_and_small_angles() {
        //make sure we don't panic on the edge case
        let _ = sin_fixed(SMALL_ANGLE_LESS);
        assert_eq!(sin_fixed(Sample::ZERO), Sample::ZERO);
    }
    #[test]
    fn sin_fixed_rms_error() {
        let numsteps = 2000;
        let mut error = 0.0;
        for i in 0..=numsteps {
            let x = (std::f32::consts::TAU*i as f32)/(numsteps as f32)
                - std::f32::consts::PI;
            let fixed = sin_fixed(Sample::from_num(x));
            let float = x.sin();
            let this_error = float - fixed.to_num::<f32>();
            error += this_error*this_error;
        }
        error /= numsteps as f32;
        error = error.sqrt();
        assert!(error < 0.02); //RMS error on interval (-pi, pi)
    }
    #[test]
    fn sin_slightly_over_bounds_no_overflows() {
        let _a = sin_fixed(Sample::lit("3.2"));
        let _b = sin_fixed(Sample::lit("-3.2"));
    }
    //
    //COS TESTS:
    //
    #[test]
    fn cos_zero_and_small_angles() {
        //make sure we don't panic on the edge case
        let _ = cos_fixed(SMALL_ANGLE_LESS);
        assert_eq!(cos_fixed(Sample::ZERO), Sample::ONE);
    }
    #[test]
    fn cos_fixed_rms_error() {
        let numsteps = 2000;
        let mut error = 0.0;
        for i in 0..=numsteps {
            let x = (std::f32::consts::TAU*i as f32)/(numsteps as f32)
                - std::f32::consts::PI;
            let fixed = cos_fixed(Sample::from_num(x));
            let float = x.cos();
            let this_error = float - fixed.to_num::<f32>();
            error += this_error*this_error;
        }
        error /= numsteps as f32;
        error = error.sqrt();
        //TODO: Is this good enough?  RMS error is good on (-pi/2, pi/2)
        assert!(error < 0.06); //RMS error on interval (-pi, pi)
    }
    #[test]
    fn cos_slightly_over_bounds_no_overflows() {
        let _a = cos_fixed(Sample::lit("3.2"));
        let _b = cos_fixed(Sample::lit("-3.2"));
    }
    #[test]
    fn midi_pitch_calculations() {
        for i in 0..=127 {
            let pitch = 440.0*f32::powf(2.0, ((i-69) as f32)/12.0);
            let pitch_fixed = midi_note_to_frequency(i.to_fixed()).to_num::<f32>();
            let period = 1.0/pitch;
            let period_fixed = midi_note_to_period(i.to_fixed()).to_num::<f32>();
            let error = calculate_cents(pitch, pitch_fixed);
            assert!(error < 1.0); //less than one cent per note
            let period_error = calculate_cents(period, period_fixed);
            assert!(period_error < 1.0);
        }
    }
}


