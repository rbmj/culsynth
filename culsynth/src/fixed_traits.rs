use super::ScalarFxP;
use core::ops::Add;
use fixed::types::extra::{LeEqU16, LeEqU32, Sum, Unsigned, U16};
use fixed::{traits::Fixed, FixedI32};

/// A trait encompassing 16 bit fixed point numbers along with a couple of
/// convenience methods for the type.
pub trait Fixed16: Fixed {
    /// The value one, or if one is not representable, the maximum representable
    /// by the type
    const ONE_OR_MAX: Self = if let Some(val) = Self::TRY_ONE {
        val
    } else {
        Self::MAX
    };
    /// Multiply two fixed point numbers
    fn multiply_fixed(self, rhs: Self) -> Self;
    /// Scale a fixed point number
    fn scale_fixed(self, rhs: ScalarFxP) -> Self;
    /// A 32 bit fixed point number with the same number of fractional bits
    type Widened: Fixed;
    /// Widen this to a Self::Widened
    fn widen(self) -> Self::Widened;
    /// Create a widened from bits
    fn widened_from_bits(bits: i32) -> Self::Widened;
}

impl<N> Fixed16 for fixed::FixedI16<N>
where
    N: Unsigned + LeEqU16 + LeEqU32 + Add<N> + Add<U16>,
    Sum<N, N>: Unsigned + LeEqU32,
    Sum<N, U16>: Unsigned + LeEqU32,
{
    fn multiply_fixed(self, rhs: Self) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    fn scale_fixed(self, rhs: ScalarFxP) -> Self {
        Self::from_num(self.wide_mul_unsigned(rhs))
    }
    type Widened = FixedI32<Self::Frac>;
    fn widen(self) -> FixedI32<Self::Frac> {
        FixedI32::<Self::Frac>::from_num(self)
    }
    fn widened_from_bits(bits: i32) -> Self::Widened {
        Self::Widened::from_bits(bits)
    }
}

impl<N> Fixed16 for fixed::FixedU16<N>
where
    N: Unsigned + LeEqU16 + LeEqU32 + Add<N> + Add<U16>,
    Sum<N, N>: Unsigned + LeEqU32,
    Sum<N, U16>: Unsigned + LeEqU32,
{
    fn multiply_fixed(self, rhs: Self) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    fn scale_fixed(self, rhs: ScalarFxP) -> Self {
        Self::from_num(self.wide_mul(rhs))
    }
    type Widened = FixedI32<Self::Frac>;
    fn widen(self) -> FixedI32<Self::Frac> {
        FixedI32::<Self::Frac>::from_num(self)
    }
    fn widened_from_bits(bits: i32) -> Self::Widened {
        Self::Widened::from_bits(bits)
    }
}
