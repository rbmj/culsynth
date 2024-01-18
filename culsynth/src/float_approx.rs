//! Floating point fast approximations used internally by the synthesizer
//!
//! Benchmark me!

#[cfg(any(test, doc, not(feature = "libm")))]
mod detail {
    use crate::Float;
    use num_traits::AsPrimitive;

    /// Approximate sin(x), using a 7th order taylor series approximation about
    /// x == 0.  This is fairly accurate from -pi to pi.
    pub fn sin_approx<T: Float>(x: T) -> T {
        //small angle approximation.  Faster and removes 0 as an edge case
        if x.abs() < (T::ONE / T::from_u16(0x10)) {
            return x;
        }
        let x2 = x * x;
        // sin(x) = x - x^3/3! + x^5/5! - x^7/7! (+ higher order terms)
        //        = x { 1 - x^2/6 [ 1 - x^2/20 ( 1 - x^2/42 ) ] }
        let c = x2 / T::from_u16(42);
        let c_nested = T::ONE - c;
        let b = x2 / T::from_u16(20);
        let b_nested = T::ONE - (b * c_nested);
        let a = x2 / T::from_u16(6);
        let a_nested = T::ONE - (a * b_nested);
        x * a_nested
    }

    /// Approximate cos(x), using a sixth order taylor series approximation about
    /// x == 0.  This is fairly accurate from -pi/2 to pi/2.
    pub fn cos_approx<T: Float>(x: T) -> T {
        let x2 = x * x;
        //small angle approximation.  Faster and removes 0 as an edge case
        if x.abs() < (T::ONE / T::from_u16(0x10)) {
            return T::ONE - (x2 / T::TWO);
        }
        let c = x2 / T::from_u16(30);
        let c_nested = T::ONE - c;
        let b = x2 / T::from_u16(12);
        let b_nested = T::ONE - (b * c_nested);
        let a_mult_b_nested = (x2 / T::TWO) * b_nested;
        // cos(x) ~= 1 - a*b_nested
        T::ONE - a_mult_b_nested
    }

    /// A very rough approximation of tan(x) using a quadratic taylor series expansion.
    /// Used primarily in filter gain prewarping, where it's accurate enough for angles
    /// representing lower frequencies where precise tuning is more important.  Will be
    /// somewhat inaccurate at frequencies above about half the Nyquist frequency.
    pub fn tan_approx<T: Float>(x: T) -> T {
        let x2_over3 = (x * x) / T::THREE;
        x * (x2_over3 + T::ONE)
    }

    /// calculate e^x in the range [-0.5, 0.5) using an order 4 Taylor series
    fn exp_approx_small<T: Float>(x: T) -> T {
        // e^x ~= 1 + x + x^2/2! + x^3/3! + x^4/4!
        //     ~= 1 + x * { 1 + x/2 * [ 1 + x/3 * ( 1 + x/4 )]}
        // c_nested = 1 + x/4
        let c_nested = T::ONE + (x / (T::TWO * T::TWO));
        // b = x * (2/3) * (1/2) = x/3
        let b = x / T::THREE;
        // b_nested = 1 + b*c_nested
        let b_nested = T::ONE + (b * c_nested);
        // a = x/2
        let a = x / T::TWO;
        // a_nested = 1 + a*b_nested
        let a_nested = T::ONE + (a * b_nested);
        // exp(x) ~= 1 + x*a_nested
        T::ONE + (x * a_nested)
    }

    /// Calculate e^x of fixed-point number in the range `[-4, 4)`
    ///
    /// # Panics
    ///
    /// This function will panic if the number is not in the specified domain
    pub fn exp_approx<T: Float + From<f32> + AsPrimitive<isize>>(x: T) -> T {
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
        const LOOKUP_TABLE: [f32; 8] = [
            0.030_197_383,
            0.082_085,
            0.223_130_16,
            0.606_530_66,
            1.648_721_2,
            4.481_689,
            12.182_493,
            33.115_45,
        ];
        let floor = x.floor();
        let index = (floor.as_() + 4) as usize;
        let frac_exp = exp_approx_small(x - floor - T::ONE_HALF);
        frac_exp * LOOKUP_TABLE[index].into()
    }

    /// Convert a MIDI note number to a frequency in Hz
    pub fn midi_note_to_frequency<T: Float + From<f32> + AsPrimitive<isize>>(note: T) -> T {
        const FRAC_LN2_12: f32 = 0.057_762_265;
        const FREQ_E4: f32 = 329.627_56;
        let f0: T = FREQ_E4.into();
        f0 * exp_approx((note - T::from_u16(64)) * FRAC_LN2_12.into())
    }
}

#[cfg(any(test, doc, not(feature = "libm")))]
pub use detail::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::calculate_cents;

    #[test]
    fn sin_approx_rms_error() {
        let numsteps = 2000;
        let mut error = 0.0;
        for i in 0..=numsteps {
            let x = (core::f32::consts::TAU * i as f32) / (numsteps as f32) - core::f32::consts::PI;
            let approx = sin_approx(x);
            let float = x.sin();
            let this_error = float - approx;
            error += this_error * this_error;
        }
        error /= numsteps as f32;
        error = error.sqrt();
        assert!(error < 0.02); //RMS error on interval (-pi, pi)
    }
    #[test]
    fn cos_fixed_rms_error() {
        let numsteps = 2000;
        let mut error = 0.0;
        for i in 0..=numsteps {
            let x = (core::f32::consts::TAU * i as f32) / (numsteps as f32) - core::f32::consts::PI;
            let approx = cos_approx(x);
            let float = x.cos();
            let this_error = float - approx;
            error += this_error * this_error;
        }
        error /= numsteps as f32;
        error = error.sqrt();
        //TODO: Is this good enough?  RMS error is good on (-pi/2, pi/2)
        assert!(error < 0.06); //RMS error on interval (-pi, pi)
    }
    #[test]
    fn midi_pitch_calculations_float_approx() {
        for i in 0..=127 {
            let pitch = 440.0 * f32::powf(2.0, ((i - 69) as f32) / 12.0);
            let pitch_approx = midi_note_to_frequency(i as f32);
            let error = calculate_cents(pitch, pitch_approx);
            assert!(error < 1.0); //less than one cent per note
        }
    }
}
