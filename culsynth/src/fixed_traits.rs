use super::ScalarFxP;
use core::ops::Add;
use fixed::traits::Fixed;
use fixed::types::extra::{LeEqU16, LeEqU32, Sum, Unsigned, U16};

/// A trait encompassing 16 bit fixed point numbers
pub trait Fixed16: fixed::traits::Fixed {
    const ONE_OR_MAX: Self = if let Some(val) = Self::TRY_ONE {
        val
    } else {
        Self::MAX
    };
    fn multiply_fixed(self, rhs: Self) -> Self;
    fn scale_fixed(self, rhs: ScalarFxP) -> Self;
}

impl<N> Fixed16 for fixed::FixedI16<N>
where
    N: Unsigned + LeEqU16 + Add<N> + Add<U16>,
    Sum<N, N>: Unsigned + LeEqU32,
    Sum<N, U16>: Unsigned + LeEqU32,
{
    fn multiply_fixed(self, rhs: Self) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    fn scale_fixed(self, rhs: ScalarFxP) -> Self {
        Self::from_num(self.wide_mul_unsigned(rhs))
    }
}

impl<N> Fixed16 for fixed::FixedU16<N>
where
    N: Unsigned + LeEqU16 + Add<N> + Add<U16>,
    Sum<N, N>: Unsigned + LeEqU32,
    Sum<N, U16>: Unsigned + LeEqU32,
{
    fn multiply_fixed(self, rhs: Self) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    fn scale_fixed(self, rhs: ScalarFxP) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
}
