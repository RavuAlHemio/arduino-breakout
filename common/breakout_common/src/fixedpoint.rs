//! Implementation of fixed-point arithmetic.


use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};


pub type FixedPointValue = i16;
pub type FixedPointIntegerValue = i8; // half size of FixedPointValue
pub type FixedPointMulResult = i32; // double size from FixedPointValue

pub const EXPONENT: u8 = 8;
const MUL_RESULT_MASK: FixedPointMulResult = 0xFFFF; // mask after right-shift by EXPONENT


#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FixedPoint {
    value: FixedPointValue,
}
impl FixedPoint {
    #[inline]
    pub const fn new_integer(int: FixedPointIntegerValue) -> Self {
        Self {
            value: (int as FixedPointValue) << EXPONENT
        }
    }

    #[inline]
    pub const fn new_raw(value: FixedPointValue) -> Self {
        Self {
            value,
        }
    }

    #[inline]
    pub const fn zero() -> FixedPoint { FixedPoint { value: 0 } }

    #[inline]
    pub const fn one() -> FixedPoint { FixedPoint::new_integer(1) }

    #[inline]
    pub const fn as_integer(&self) -> FixedPointIntegerValue {
        (self.value >> EXPONENT) as FixedPointIntegerValue
    }

    #[inline]
    pub const fn as_raw(&self) -> FixedPointValue { self.value }

    #[inline]
    pub const fn is_integer(&self) -> bool {
        const FRAC_MASK: FixedPointValue = (1 << EXPONENT) - 1;
        (self.value & FRAC_MASK) == 0
    }
}
impl Add for FixedPoint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        FixedPoint { value: self.value + rhs.value }
    }
}
impl Sub for FixedPoint {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        FixedPoint { value: self.value - rhs.value }
    }
}
impl Mul for FixedPoint {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        // here it becomes a bit juicy
        // IIIIIIII_FFFFFFFF * IIIIIIII_FFFFFFFF -> IIIIIIII_IIIIIIII_FFFFFFFF_FFFFFFFF
        // => shift down by an exponent and mask to the regular amount of bits
        let left = self.value as FixedPointMulResult;
        let right = rhs.value as FixedPointMulResult;
        let product = left * right;
        let result_product = ((product >> EXPONENT) & MUL_RESULT_MASK) as FixedPointValue;
        FixedPoint { value: result_product }
    }
}
impl Div for FixedPoint {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        // here it becomes even more juicy
        // 4/3 = 1.333... but also 4000/3000 = 1.333...
        // however, 4000/3 = 1333.333...
        // => to ensure we do not lose precision, we must shift the numerator left by the exponent
        let dividend = (self.value as FixedPointMulResult) << EXPONENT;
        let divisor = rhs.value as FixedPointMulResult;
        let quotient = dividend/divisor;
        let result_quotient = (quotient & MUL_RESULT_MASK) as FixedPointValue;
        FixedPoint { value: result_quotient }
    }
}
impl Neg for FixedPoint {
    type Output = Self;
    fn neg(self) -> Self::Output {
        FixedPoint { value: -self.value }
    }
}
impl AddAssign for FixedPoint {
    fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; }
}
impl DivAssign for FixedPoint {
    fn div_assign(&mut self, rhs: Self) { *self = *self / rhs; }
}
impl MulAssign for FixedPoint {
    fn mul_assign(&mut self, rhs: Self) { *self = *self * rhs; }
}
impl SubAssign for FixedPoint {
    fn sub_assign(&mut self, rhs: Self) { *self = *self - rhs; }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn i(int: FixedPointIntegerValue) -> FixedPoint { FixedPoint::new_integer(int) }

    #[test]
    fn test_add() {
        assert_eq!(i(0) + i(0), i(0));
        assert_eq!(i(4) + i(3), i(7));
        assert_eq!(i(23) + i(42), i(65));
    }

    #[test]
    fn test_sub() {
        assert_eq!(i(0) - i(0), i(0));
        assert_eq!(i(7) - i(3), i(4));
        assert_eq!(i(4) - i(3), i(1));
        assert_eq!(i(3) - i(4), i(-1));
        assert_eq!(i(23) - i(42), i(-19));
    }

    #[test]
    fn test_mul() {
        assert_eq!(i(0) * i(0), i(0));
        assert_eq!(i(7) * i(3), i(21));
        assert_eq!(i(4) * i(3), i(12));
        assert_eq!(i(4) * i(-3), i(-12));
        assert_eq!(i(-4) * i(3), i(-12));
        assert_eq!(i(-4) * i(-3), i(12));
    }

    #[test]
    fn test_div() {
        assert_eq!(i(4) / i(2), i(2));
        assert_eq!(i(10) / i(2), i(5));
        assert_eq!(i(32) / i(8), i(4));

        assert!(!(i(4) / i(8)).is_integer());
        assert_eq!((i(4) / i(8)) * i(2), i(1));
    }
}
