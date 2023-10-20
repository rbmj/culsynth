use fixed::types::*;
use fixed::types::extra::{Unsigned, IsLessOrEqual, LeEqU32, U31, True};
use fixed::FixedU32;
use janus::util::calculate_cents;



fn osc_implementation(x_arg: U0F16) -> I4F28 {
    if x_arg == U0F16::ZERO {
        return I4F28::ONE;
    }
    let x = I1F15::from_num(x_arg);
    let x_prime = x - I1F15::lit("0.333333333");
    let mut acc = I2F14::ONE;
    for _ in 0..4 {
        let mut prod = I16F16::from_num(acc.wide_mul(x_prime));
        prod += prod.unwrapped_shl(1); // acc *= 3
        prod = prod.unwrapped_shr(2);  // acc /= 4
        acc = I2F14::ONE - I2F14::from_num(prod);
    }
    acc = acc.unwrapped_shr(2);  // acc / 4
    acc += acc.unwrapped_shl(1); // acc *= 3
    //fixedmath::U1F15::from_num(acc)
    I4F28::from_num(acc)
}

fn main() {
    let mut err_generic = 0f64;
    let mut err_generic_highacc = 0f64;
    let mut err_osc = 0f64;
    for i in 0..65535u16 {
        let x = U0F16::from_bits(i);
        let x_wide = U16F16::from_num(x);
        let f = x.to_num::<f32>();
        let res = 1f32 / (1f32 + f);
        let (g1, g1s) = one_over_one_plus(x_wide);
        let (g2, g2s) = one_over_one_plus_highacc(x);
        let generic = calculate_cents(g1.unwrapped_shr(g1s).to_num::<f32>(), res);
        let generic_highacc = calculate_cents(g2.unwrapped_shr(g2s).to_num::<f32>(), res);
        let osc = calculate_cents(osc_implementation(x).to_num::<f32>(), res);
        err_generic += (generic*generic) as f64;
        err_generic_highacc += (generic_highacc*generic_highacc) as f64;
        err_osc += (osc*osc) as f64;
    }
    err_generic /= 65536f64;
    err_generic_highacc /= 65536f64;
    err_osc /= 65536f64;
    println!("Generic:\t{}\nHigh Accuracy:\t{}\nOsc:\t{}", err_generic.sqrt(), err_generic_highacc.sqrt(), err_osc.sqrt());
}