use std::{
	cmp::{Eq, Ordering},
	collections::HashMap,
	fmt::Display,
	hash::Hash,
	num::NonZeroU64,
	ops::{Add, Div, Mul, Rem, Sub},
	sync::Mutex,
};

use bilge::arbitrary_int::{u10, u52, Number};
use once_cell::sync::Lazy;
use varlen::array_init::FromIterPrefix;

use crate::{
	error::ArithmeticError,
	value::{value_storage, DataView, InitEmpty, ValType},
	Value,
};

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

impl Value {
	fn new_boxed_int(i: IntVal) -> Value {
		let (b, i) = match i {
			IntVal::InfPos => (true, 1),
			IntVal::InfNeg => (true, -1),
			IntVal::Int(i) => (false, i),
		};
		let init = value_storage::Init {
			ty: ValType::Int,
			len: 0,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			ints: FromIterPrefix([i].into_iter()),
			floats: InitEmpty,
			bool_var: InitEmpty,
			int_var: InitEmpty,
			float_var: InitEmpty,
			int_set_var: InitEmpty,
			bytes: FromIterPrefix([b as u8].into_iter()),
		};
		Self::new_box(init)
	}

	fn new_boxed_float(i: FloatVal) -> Value {
		Self::new_box(value_storage::Init {
			ty: ValType::Float,
			len: 0,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			ints: InitEmpty,
			floats: FromIterPrefix([i].into_iter()),
			bool_var: InitEmpty,
			int_var: InitEmpty,
			float_var: InitEmpty,
			int_set_var: InitEmpty,
			bytes: InitEmpty,
		})
	}

	pub fn as_int(&self) -> IntVal {
		let DataView::Int(i) = self.deref() else {
			panic!("expected int, found {self:?}");
		};
		i
	}
}

static INT_MAP: Lazy<Mutex<HashMap<IntVal, Value>>> = Lazy::new(|| HashMap::new().into());
pub static INT_INF_POS: Lazy<Value> = Lazy::new(|| {
	let mut map = INT_MAP.lock().unwrap();
	let inf = map
		.entry(IntVal::InfPos)
		.or_insert_with(|| Value::new_boxed_int(IntVal::InfPos));
	inf.clone()
});
pub static INT_INF_NEG: Lazy<Value> = Lazy::new(|| {
	let mut map = INT_MAP.lock().unwrap();
	let inf = map
		.entry(IntVal::InfNeg)
		.or_insert_with(|| Value::new_boxed_int(IntVal::InfNeg));
	inf.clone()
});

static FLOAT_MAP: Lazy<Mutex<HashMap<FloatVal, Value>>> = Lazy::new(|| HashMap::new().into());
#[allow(dead_code)] // TODO!
pub static FLOAT_INF_POS: Lazy<Value> = Lazy::new(|| FloatVal::INFINITY.into());
#[allow(dead_code)] // TODO!
pub static FLOAT_INF_NEG: Lazy<Value> = Lazy::new(|| FloatVal::NEG_INFINITY.into());

impl From<bool> for Value {
	fn from(value: bool) -> Self {
		Value::from(if value { 1 } else { 0 })
	}
}
impl TryInto<bool> for &Value {
	type Error = ();

	fn try_into(self) -> Result<bool, Self::Error> {
		let val: i64 = self.try_into()?;
		if val != 0 && val != 1 {
			todo!()
		}
		Ok(val == 1)
	}
}
impl TryInto<bool> for Value {
	type Error = ();

	fn try_into(self) -> Result<bool, Self::Error> {
		(&self).try_into()
	}
}

impl From<IntVal> for Value {
	fn from(value: IntVal) -> Self {
		match value {
			IntVal::InfPos => INT_INF_POS.clone(),
			IntVal::InfNeg => INT_INF_NEG.clone(),
			IntVal::Int(i) if (Self::MIN_INT..=Self::MAX_INT).contains(&i) => {
				// Can box integer (fits in 62 bits)
				let mut x = i.unsigned_abs() << 3;
				if i.is_negative() {
					x |= Self::INT_SIGN_BIT;
				}
				x |= Self::INT_TAG;
				Self {
					raw: NonZeroU64::new(x).unwrap(),
				}
			}
			iv => {
				// Try and find integer in map or allocate new integer
				let mut map = INT_MAP.lock().unwrap();
				let v = map.entry(iv).or_insert_with(|| Value::new_boxed_int(iv));
				v.clone()
			}
		}
	}
}
impl TryInto<IntVal> for &Value {
	type Error = ();

	fn try_into(self) -> Result<IntVal, Self::Error> {
		if let DataView::Int(i) = self.deref() {
			Ok(i)
		} else {
			todo!()
		}
	}
}
impl TryInto<IntVal> for Value {
	type Error = ();

	fn try_into(self) -> Result<IntVal, Self::Error> {
		(&self).try_into()
	}
}
impl TryInto<i64> for &Value {
	type Error = ();

	fn try_into(self) -> Result<i64, Self::Error> {
		if let IntVal::Int(i) = self.try_into()? {
			Ok(i)
		} else {
			Err(())
		}
	}
}
impl TryInto<i64> for Value {
	type Error = ();

	fn try_into(self) -> Result<i64, Self::Error> {
		(&self).try_into()
	}
}
impl From<i64> for Value {
	fn from(value: i64) -> Self {
		IntVal::Int(value).into()
	}
}

impl From<FloatVal> for Value {
	fn from(value: FloatVal) -> Self {
		let f: f64 = value.into();
		const EXPONENT_MASK: u64 = 0x7FF << 52;
		let bits = f.to_bits();
		let mut exponent = (bits & EXPONENT_MASK) >> 52;
		if exponent != 0 {
			if !(513..=1534).contains(&exponent) {
				// Exponent doesn't fit in 10 bits
				let mut map = FLOAT_MAP.lock().unwrap();
				let v = map
					.entry(value)
					.or_insert_with(|| Value::new_boxed_float(value));
				return v.clone();
			}
			exponent -= 512; // Make exponent fit in 10 bits, with bias 511
		}
		debug_assert!(exponent <= <u10 as Number>::MAX.value().into());
		let sign = (bits & (1 << 63)) != 0;

		const FRACTION_MASK: u64 = 0xFFFFFFFFFFFFF;
		let fraction = bits & FRACTION_MASK; // Remove one bit of precision
		debug_assert!(fraction <= <u52 as Number>::MAX.value());
		let mut raw = (fraction << 1) | (exponent << 53) | Self::FLOAT_TAG;
		if sign {
			raw |= Self::FLOAT_SIGN_BIT;
		}
		Value {
			raw: NonZeroU64::new(raw).unwrap(),
		}
	}
}
impl From<f64> for Value {
	fn from(value: f64) -> Self {
		Value::from(FloatVal::from(value))
	}
}
impl TryInto<FloatVal> for &Value {
	type Error = ();

	fn try_into(self) -> Result<FloatVal, Self::Error> {
		if let DataView::Float(f) = self.deref() {
			Ok(f)
		} else {
			todo!()
		}
	}
}
impl TryInto<FloatVal> for Value {
	type Error = ();

	fn try_into(self) -> Result<FloatVal, Self::Error> {
		(&self).try_into()
	}
}
impl TryInto<f64> for &Value {
	type Error = ();

	fn try_into(self) -> Result<f64, Self::Error> {
		let fv: FloatVal = self.try_into()?;
		Ok(fv.into())
	}
}
impl TryInto<f64> for Value {
	type Error = ();

	fn try_into(self) -> Result<f64, Self::Error> {
		(&self).try_into()
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		value::{
			num::{FloatVal, IntVal, FLOAT_INF_NEG, FLOAT_INF_POS},
			RefType,
		},
		Value,
	};

	#[test]
	fn test_bool_value() {
		let f: bool = Value::from(false).try_into().unwrap();
		assert!(!f);
		let t: bool = Value::from(true).try_into().unwrap();
		assert!(t);
	}

	#[test]
	fn test_integer_value() {
		let zero: i64 = Value::from(0i64).try_into().unwrap();
		assert_eq!(zero, 0i64);

		let one: i64 = Value::from(1i64).try_into().unwrap();
		assert_eq!(one, 1i64);
		let minus_one: i64 = Value::from(-1i64).try_into().unwrap();
		assert_eq!(minus_one, -1i64);

		// Unboxed min and max
		let minimum: i64 = Value::from(Value::MIN_INT).try_into().unwrap();
		assert_eq!(minimum, Value::MIN_INT);
		let maximum: i64 = Value::from(Value::MAX_INT).try_into().unwrap();
		assert_eq!(maximum, Value::MAX_INT);

		// Positive and Negative Infinity
		let pos_inf: IntVal = Value::from(IntVal::InfPos).try_into().unwrap();
		assert_eq!(pos_inf, IntVal::InfPos);
		let neg_inf: IntVal = Value::from(IntVal::InfNeg).try_into().unwrap();
		assert_eq!(neg_inf, IntVal::InfNeg);

		// i64 min and max
		let minimum: i64 = Value::from(i64::MAX).try_into().unwrap();
		assert_eq!(minimum, i64::MAX);
		let maximum: i64 = Value::from(i64::MIN).try_into().unwrap();
		assert_eq!(maximum, i64::MIN);
	}

	#[test]
	fn test_float_value() {
		let zero: f64 = Value::from(0.0f64).try_into().unwrap();
		assert_eq!(zero, 0.0);
		let one: f64 = Value::from(1.0f64).try_into().unwrap();
		assert_eq!(one, 1.0);
		let minus_one: f64 = Value::from(-1.0f64).try_into().unwrap();
		assert_eq!(minus_one, -1.0);

		let twodottwo: f64 = Value::from(2.2f64).try_into().unwrap();
		assert_eq!(twodottwo, 2.2);

		// Positive and Negative Infinity
		let pos_inf: f64 = Value::from(f64::INFINITY).try_into().unwrap();
		assert_eq!(pos_inf, f64::INFINITY);
		let neg_inf: f64 = Value::from(f64::NEG_INFINITY).try_into().unwrap();
		assert_eq!(neg_inf, f64::NEG_INFINITY);
		// f64 min and max
		let minimum: f64 = Value::from(f64::MAX).try_into().unwrap();
		assert_eq!(minimum, f64::MAX);
		let maximum: f64 = Value::from(f64::MIN).try_into().unwrap();
		assert_eq!(maximum, f64::MIN);

		assert_eq!(FLOAT_INF_NEG.ref_ty(), RefType::Boxed);
		assert_eq!(FLOAT_INF_POS.ref_ty(), RefType::Boxed);
	}

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
