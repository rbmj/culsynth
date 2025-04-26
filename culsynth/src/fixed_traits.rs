use super::ScalarFxP;
use core::ops::Add;
use fixed::types::extra::{LeEqU16, LeEqU32, Sum, Unsigned, U16};
use fixed::{traits::Fixed, FixedI32};
use serde::{Deserialize, Serialize};

/// A trait encompassing 16 bit fixed point numbers along with a couple of
/// convenience methods for the type.
pub trait Fixed16: Fixed + Serialize + for<'a> Deserialize<'a> {
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
    /// Set a fixed point number to a value based on a MIDI U7
    fn set_from_u7(&mut self, value: wmidi::U7) {
        *self = Self::from_u7(value);
    }
    /// Get this type based on a MIDI U7
    fn from_u7(value: wmidi::U7) -> Self;
}

impl<N> Fixed16 for fixed::FixedI16<N>
where
    N: Unsigned + LeEqU16 + LeEqU32 + Add<N> + Add<U16>,
    Sum<N, N>: Unsigned + LeEqU32,
    Sum<N, U16>: Unsigned + LeEqU32,
    Self::Bits: From<i16>,
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
    fn from_u7(cc: wmidi::U7) -> Self {
        let cc: u8 = cc.into();
        let control = (cc as i8) - 64;
        let value = (control as i16) << 9;
        Self::from_bits(value.into())
    }
}

impl<N> Fixed16 for fixed::FixedU16<N>
where
    N: Unsigned + LeEqU16 + LeEqU32 + Add<N> + Add<U16>,
    Sum<N, N>: Unsigned + LeEqU32,
    Sum<N, U16>: Unsigned + LeEqU32,
    Self::Bits: From<u16>,
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
    fn from_u7(cc: wmidi::U7) -> Self {
        let cc: u8 = cc.into();
        let value = (cc as u16) << 9;
        Self::from_bits(value.into())
    }
}
