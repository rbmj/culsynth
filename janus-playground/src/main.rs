use fixed::{FixedU32, FixedU16};
use fixed::types::*;
use fixed::types::extra::*;
use core::ops::{Add, Sub};

fn scale_fixedfloat<FracA, FracB>(a: FixedU32<FracA>, b: FixedU16<FracB>) -> FixedU32<FracA>
    where FracA: Unsigned + LeEqU32 + Sub<U16>,
          FracB: Unsigned + LeEqU16 + Add<U16> + IsLessOrEqual<FracA>,
          <FracA as Sub::<U16>>::Output: LeEqU16
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

fn main() {
    let mut buf = String::new();
    loop {
        buf.clear();
        if std::io::stdin().read_line(&mut buf).is_err() {
            continue;
        }
        match U4F28::from_str(&buf[0..buf.len()-2]) {
            Ok(x) => {
                //let mut f = x.to_num::<f32>();
                let y = U1F15::lit("0.25");
                println!("{} * 0.25 = {}", x, scale_fixedfloat(x, y));
            },
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}