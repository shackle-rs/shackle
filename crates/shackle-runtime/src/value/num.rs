use std::{
	cmp::{Eq, Ordering},
	fmt::Display,
	hash::Hash,
	ops::{Add, Div, Mul, Rem, Sub},
};

use crate::error::ArithmeticError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntVal {
	InfPos,
	InfNeg,
	Int(i64),
}
impl IntVal {
	pub fn is_finite(&self) -> bool {
		matches!(self, IntVal::Int(_))
	}
}
impl From<i64> for IntVal {
	fn from(value: i64) -> Self {
		IntVal::Int(value)
	}
}
impl Display for IntVal {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			IntVal::InfPos => write!(f, "+∞"),
			IntVal::InfNeg => write!(f, "-∞"),
			IntVal::Int(i) => write!(f, "{i}"),
		}
	}
}
impl PartialOrd for IntVal {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for IntVal {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self, other) {
			(IntVal::InfPos, IntVal::InfPos) => Ordering::Equal,
			(IntVal::InfNeg, IntVal::InfNeg) => Ordering::Equal,
			(IntVal::InfPos, _) => Ordering::Greater,
			(IntVal::InfNeg, _) => Ordering::Less,
			(_, IntVal::InfPos) => Ordering::Less,
			(_, IntVal::InfNeg) => Ordering::Greater,
			(IntVal::Int(l), IntVal::Int(r)) => l.cmp(r),
		}
	}
}

const ERROR_DIV_ZERO: &str = "integer division by zero";
const ERROR_FLT_OF: &str = "overflow in floating point operation";
const ERROR_INF: &str = "arithmetic operation on infinite value";
const ERROR_INT_OF: &str = "integer overflow";
impl Add for IntVal {
	type Output = Result<IntVal, ArithmeticError>;
	fn add(self, rhs: Self) -> Self::Output {
		let (IntVal::Int(x), IntVal::Int(y)) = (self, rhs) else {
			return Err(ArithmeticError { reason: ERROR_INF });
		};
		let Some(z) = x.checked_add(y) else {
			return Err(ArithmeticError {
				reason: ERROR_INT_OF,
			});
		};
		Ok(z.into())
	}
}
impl Sub for IntVal {
	type Output = Result<IntVal, ArithmeticError>;
	fn sub(self, rhs: Self) -> Self::Output {
		let (IntVal::Int(x), IntVal::Int(y)) = (self, rhs) else {
			return Err(ArithmeticError { reason: ERROR_INF });
		};
		let Some(z) = x.checked_sub(y) else {
			return Err(ArithmeticError {
				reason: ERROR_INT_OF,
			});
		};
		Ok(z.into())
	}
}
impl Mul for IntVal {
	type Output = Result<IntVal, ArithmeticError>;
	fn mul(self, rhs: Self) -> Self::Output {
		let (IntVal::Int(x), IntVal::Int(y)) = (self, rhs) else {
			return Err(ArithmeticError { reason: ERROR_INF });
		};
		let Some(z) = x.checked_mul(y) else {
			return Err(ArithmeticError {
				reason: ERROR_INT_OF,
			});
		};
		Ok(z.into())
	}
}
impl Div for IntVal {
	type Output = Result<IntVal, ArithmeticError>;
	fn div(self, rhs: Self) -> Self::Output {
		let (IntVal::Int(x), IntVal::Int(y)) = (self, rhs) else {
			return Err(ArithmeticError { reason: ERROR_INF });
		};
		let Some(z) = x.checked_div(y) else {
			return Err(ArithmeticError {
				reason: ERROR_DIV_ZERO,
			});
		};
		Ok(z.into())
	}
}
impl Rem for IntVal {
	type Output = Result<IntVal, ArithmeticError>;
	fn rem(self, rhs: Self) -> Self::Output {
		let (IntVal::Int(x), IntVal::Int(y)) = (self, rhs) else {
			return Err(ArithmeticError { reason: ERROR_INF });
		};
		if let Some(z) = x.checked_rem(y) {
			Ok(z.into())
		} else {
			Err(ArithmeticError {
				reason: ERROR_DIV_ZERO,
			})
		}
	}
}

impl IntVal {
	pub fn pow(self, rhs: Self) -> Result<IntVal, ArithmeticError> {
		let (IntVal::Int(x), IntVal::Int(y)) = (self, rhs) else {
			return Err(ArithmeticError { reason: ERROR_INF });
		};
		match y {
			0 => Ok(1.into()),
			1 => Ok(self),
			_ if y.is_negative() => match x {
				0 => Err(ArithmeticError {
					reason: "negative power of zero",
				}),
				1 => Ok(1.into()),
				-1 => Ok(if y % 2 == 0 { 1 } else { -1 }.into()),
				_ => Ok(0.into()),
			},
			_ => {
				if y > u32::MAX.into() {
					return Err(ArithmeticError {
						reason: ERROR_INT_OF,
					});
				}
				let Some(z) = x.checked_pow(y as u32) else {
					return Err(ArithmeticError {
						reason: ERROR_INT_OF,
					});
				};
				Ok(z.into())
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatVal(f64);

impl FloatVal {
	pub const INFINITY: FloatVal = FloatVal(f64::INFINITY);
	pub const NEG_INFINITY: FloatVal = FloatVal(f64::NEG_INFINITY);
	pub fn is_finite(&self) -> bool {
		self.0.is_finite()
	}
}
impl From<f64> for FloatVal {
	fn from(value: f64) -> Self {
		assert!(!value.is_nan(), "NaN is not a valid FloatVal");
		FloatVal(value)
	}
}
impl From<FloatVal> for f64 {
	fn from(val: FloatVal) -> Self {
		val.0
	}
}
impl Display for FloatVal {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.0.is_infinite() {
			if self.0.is_sign_negative() {
				write!(f, "-∞")
			} else {
				write!(f, "+∞")
			}
		} else {
			write!(f, "{:?}", self.0)
		}
	}
}

impl Add for FloatVal {
	type Output = Result<FloatVal, ArithmeticError>;
	fn add(self, rhs: Self) -> Self::Output {
		if !(self.0.is_finite() && rhs.0.is_finite()) {
			return Err(ArithmeticError { reason: ERROR_INF });
		}
		let z = self.0 + rhs.0;
		if z.is_infinite() {
			return Err(ArithmeticError {
				reason: ERROR_FLT_OF,
			});
		}
		Ok(z.into())
	}
}
impl Sub for FloatVal {
	type Output = Result<FloatVal, ArithmeticError>;
	fn sub(self, rhs: Self) -> Self::Output {
		if !(self.0.is_finite() && rhs.0.is_finite()) {
			return Err(ArithmeticError { reason: ERROR_INF });
		}
		let z = self.0 - rhs.0;
		if z.is_infinite() {
			return Err(ArithmeticError {
				reason: ERROR_FLT_OF,
			});
		}
		Ok(z.into())
	}
}
impl Mul for FloatVal {
	type Output = Result<FloatVal, ArithmeticError>;
	fn mul(self, rhs: Self) -> Self::Output {
		if !(self.0.is_finite() && rhs.0.is_finite()) {
			return Err(ArithmeticError { reason: ERROR_INF });
		}
		let z = self.0 * rhs.0;
		if z.is_infinite() {
			return Err(ArithmeticError {
				reason: ERROR_FLT_OF,
			});
		}
		Ok(z.into())
	}
}
impl Div for FloatVal {
	type Output = Result<FloatVal, ArithmeticError>;
	fn div(self, rhs: Self) -> Self::Output {
		if !(self.0.is_finite() && rhs.0.is_finite()) {
			return Err(ArithmeticError { reason: ERROR_INF });
		}
		let z = self.0 / rhs.0;
		if z.is_infinite() {
			return Err(ArithmeticError {
				reason: ERROR_DIV_ZERO,
			});
		}
		Ok(z.into())
	}
}
impl Rem for FloatVal {
	type Output = Result<FloatVal, ArithmeticError>;
	fn rem(self, rhs: Self) -> Self::Output {
		if !(self.0.is_finite() && rhs.0.is_finite()) {
			return Err(ArithmeticError { reason: ERROR_INF });
		}
		let z = self.0 % rhs.0;
		if z.is_nan() {
			return Err(ArithmeticError {
				reason: ERROR_DIV_ZERO,
			});
		}
		Ok(z.into())
	}
}

impl Hash for FloatVal {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		state.write_u64(self.0.to_bits());
	}
}
impl Eq for FloatVal {}

impl Ord for FloatVal {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.partial_cmp(&other.0).unwrap()
	}
}
impl PartialOrd for FloatVal {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl FloatVal {
	pub fn pow(self, other: FloatVal) -> Result<FloatVal, ArithmeticError> {
		if !(self.0.is_finite() && other.0.is_finite()) {
			return Err(ArithmeticError { reason: ERROR_INF });
		}
		let z = self.0.powf(other.0);
		if z.is_infinite() {
			return Err(ArithmeticError {
				reason: ERROR_FLT_OF,
			});
		}
		Ok(z.into())
	}
}

#[cfg(test)]
mod tests {
	use super::{FloatVal, IntVal};

	#[test]
	fn test_int_operations() {
		assert!(!IntVal::InfNeg.is_finite());
		assert!(!IntVal::InfPos.is_finite());
		assert!(IntVal::Int(0).is_finite());

		assert_eq!(IntVal::InfNeg.to_string(), "-∞");
		assert_eq!(IntVal::InfPos.to_string(), "+∞");
		assert_eq!(IntVal::Int(-5).to_string(), "-5");

		assert!(IntVal::InfPos > IntVal::InfNeg);
		assert!(IntVal::InfNeg <= IntVal::InfPos);
		assert!(IntVal::InfNeg <= IntVal::InfNeg);
		assert!(IntVal::InfPos == IntVal::InfPos);
		assert!(IntVal::InfNeg < IntVal::Int(i64::MIN));
		assert!(IntVal::InfPos > IntVal::Int(i64::MIN));
		assert!(IntVal::Int(-1) < IntVal::Int(0));
		assert!(IntVal::Int(1) >= IntVal::Int(1));

		assert_eq!(IntVal::Int(1) + IntVal::Int(5), Ok(IntVal::Int(6)));
		assert!((IntVal::InfNeg + IntVal::Int(1)).is_err());
		assert!((IntVal::Int(0) + IntVal::InfPos).is_err());
		assert!((IntVal::Int(i64::MAX) + IntVal::Int(1)).is_err());

		assert_eq!(IntVal::Int(1) - IntVal::Int(5), Ok(IntVal::Int(-4)));
		assert!((IntVal::InfNeg - IntVal::Int(1)).is_err());
		assert!((IntVal::Int(0) - IntVal::InfPos).is_err());
		assert!((IntVal::Int(i64::MIN) - IntVal::Int(1)).is_err());

		assert_eq!(IntVal::Int(1) * IntVal::Int(5), Ok(IntVal::Int(5)));
		assert!((IntVal::InfNeg * IntVal::Int(1)).is_err());
		assert!((IntVal::Int(0) * IntVal::InfPos).is_err());
		assert!((IntVal::Int(i64::MAX) * IntVal::Int(2)).is_err());

		assert_eq!(IntVal::Int(10) / IntVal::Int(5), Ok(IntVal::Int(2)));
		assert_eq!(IntVal::Int(0) / IntVal::Int(5), Ok(IntVal::Int(0)));
		assert!((IntVal::InfNeg / IntVal::Int(1)).is_err());
		assert!((IntVal::Int(100) / IntVal::InfPos).is_err());
		assert!((IntVal::Int(10) / IntVal::Int(0)).is_err());

		assert_eq!(IntVal::Int(10) % IntVal::Int(4), Ok(IntVal::Int(2)));
		assert_eq!(IntVal::Int(10) % IntVal::Int(-1), Ok(IntVal::Int(0)));
		assert!((IntVal::InfNeg % IntVal::Int(2)).is_err());
		assert!((IntVal::Int(100) % IntVal::InfPos).is_err());
		assert!((IntVal::Int(10) % IntVal::Int(0)).is_err());

		assert_eq!(IntVal::Int(10).pow(IntVal::Int(4)), Ok(IntVal::Int(10_000)));
		assert_eq!(IntVal::Int(10).pow(IntVal::Int(0)), Ok(IntVal::Int(1)));
		assert_eq!(IntVal::Int(10).pow(IntVal::Int(1)), Ok(IntVal::Int(10)));
		assert!(IntVal::Int(0).pow(IntVal::Int(-1)).is_err());
		assert_eq!(IntVal::Int(1).pow(IntVal::Int(-2)), Ok(IntVal::Int(1)));
		assert_eq!(IntVal::Int(-1).pow(IntVal::Int(-2)), Ok(IntVal::Int(1)));
		assert_eq!(IntVal::Int(-1).pow(IntVal::Int(-3)), Ok(IntVal::Int(-1)));
		assert_eq!(IntVal::Int(10).pow(IntVal::Int(-1)), Ok(IntVal::Int(0)));
		assert!(IntVal::InfNeg.pow(IntVal::Int(2)).is_err());
		assert!(IntVal::Int(100).pow(IntVal::InfPos).is_err());
		assert!(IntVal::Int(100)
			.pow(IntVal::Int(u32::MAX as i64 + 1))
			.is_err());
		assert!(IntVal::Int(i64::MAX).pow(IntVal::Int(2)).is_err());
	}

	#[test]
	#[should_panic(expected = "NaN is not a valid FloatVal")]
	fn test_floatval_nan() {
		let _: FloatVal = f64::NAN.into();
	}

	#[test]
	fn test_float_operations() {
		assert!(!FloatVal(f64::NEG_INFINITY).is_finite());
		assert!(!FloatVal(f64::INFINITY).is_finite());
		assert!(FloatVal(0.0).is_finite());

		assert_eq!(FloatVal(f64::NEG_INFINITY).to_string(), "-∞");
		assert_eq!(FloatVal(f64::INFINITY).to_string(), "+∞");
		assert_eq!(FloatVal(-5.0).to_string(), "-5.0");

		assert!(FloatVal(f64::INFINITY) > FloatVal(f64::NEG_INFINITY));
		assert!(FloatVal(f64::NEG_INFINITY) <= FloatVal(f64::INFINITY));
		assert!(FloatVal(f64::NEG_INFINITY) <= FloatVal(f64::NEG_INFINITY));
		assert!(FloatVal(f64::INFINITY) == FloatVal(f64::INFINITY));
		assert!(FloatVal(f64::NEG_INFINITY) < FloatVal(f64::MIN));
		assert!(FloatVal(f64::INFINITY) > FloatVal(f64::MIN));
		assert!(FloatVal(-1.0) < FloatVal(0.0));
		assert!(FloatVal(1.0) >= FloatVal(1.0));

		assert_eq!(FloatVal(1.0) + FloatVal(5.0), Ok(FloatVal(6.0)));
		assert!((FloatVal(f64::NEG_INFINITY) + FloatVal(1.0)).is_err());
		assert!((FloatVal(0.0) + FloatVal(f64::INFINITY)).is_err());
		assert!((FloatVal(f64::MAX) + FloatVal(f64::MAX)).is_err());

		assert_eq!(FloatVal(1.0) - FloatVal(5.0), Ok(FloatVal(-4.0)));
		assert!((FloatVal(f64::NEG_INFINITY) - FloatVal(1.0)).is_err());
		assert!((FloatVal(0.0) - FloatVal(f64::INFINITY)).is_err());
		assert!((FloatVal(f64::MIN) - FloatVal(f64::MAX)).is_err());

		assert_eq!(FloatVal(1.0) * FloatVal(5.0), Ok(FloatVal(5.0)));
		assert!((FloatVal(f64::NEG_INFINITY) * FloatVal(1.0)).is_err());
		assert!((FloatVal(0.0) * FloatVal(f64::INFINITY)).is_err());
		assert!((FloatVal(f64::MAX) * FloatVal(2.0)).is_err());

		assert_eq!(FloatVal(10.0) / FloatVal(5.0), Ok(FloatVal(2.0)));
		assert_eq!(FloatVal(0.0) / FloatVal(5.0), Ok(FloatVal(0.0)));
		assert!((FloatVal(f64::NEG_INFINITY) / FloatVal(1.0)).is_err());
		assert!((FloatVal(100.0) / FloatVal(f64::INFINITY)).is_err());
		assert!((FloatVal(10.0) / FloatVal(0.0)).is_err());

		assert_eq!(FloatVal(10.0) % FloatVal(4.0), Ok(FloatVal(2.0)));
		assert_eq!(FloatVal(10.0) % FloatVal(-1.0), Ok(FloatVal(0.0)));
		assert!((FloatVal(f64::NEG_INFINITY) % FloatVal(2.0)).is_err());
		assert!((FloatVal(100.0) % FloatVal(f64::INFINITY)).is_err());
		assert!((FloatVal(10.0) % FloatVal(0.0)).is_err());

		assert_eq!(FloatVal(10.0).pow(FloatVal(4.0)), Ok(FloatVal(10_000.0)));
		assert_eq!(FloatVal(10.0).pow(FloatVal(0.0)), Ok(FloatVal(1.0)));
		assert_eq!(FloatVal(10.0).pow(FloatVal(1.0)), Ok(FloatVal(10.0)));
		assert!(FloatVal(0.0).pow(FloatVal(-1.0)).is_err());
		assert_eq!(FloatVal(1.0).pow(FloatVal(-2.0)), Ok(FloatVal(1.0)));
		assert_eq!(FloatVal(-1.0).pow(FloatVal(-2.0)), Ok(FloatVal(1.0)));
		assert_eq!(FloatVal(-1.0).pow(FloatVal(-3.0)), Ok(FloatVal(-1.0)));
		assert_eq!(FloatVal(10.0).pow(FloatVal(-1.0)), Ok(FloatVal(0.1)));
		assert!(FloatVal(f64::NEG_INFINITY).pow(FloatVal(2.0)).is_err());
		assert!(FloatVal(100.0).pow(FloatVal(f64::INFINITY)).is_err());
		assert!(FloatVal(f64::MAX).pow(FloatVal(2.0)).is_err());
	}
}
