//! # Expressions for constraints and objectives
//!
//! This module defines the expressions used in constraints and objectives.
//! These expressions are represented in textual format in the XCSP3 XML format.
//! The expressions are parsed from strings and can be serialized back to
//! strings.
//!
//! The expressions are generally split using the type of value or decision
//! variable it will result in. An enumerated type [`Exp`] is used to represent
//! expressions in positions that could take multiple or any type.

use std::{fmt::Display, marker::PhantomData, ops::RangeInclusive, str::FromStr};

use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{alphanumeric1, char, digit1, multispace0, multispace1},
	combinator::{all_consuming, map, map_res, opt, recognize, verify},
	multi::{many0, separated_list0, separated_list1},
	sequence::{delimited, pair, preceded, separated_pair, terminated},
	IResult, Parser,
};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use crate::{IntVal, VarRef};

/// List of reserved identifiers used by builtin expressions
pub const RESERVED: &[&str] = &[
	"abs", "add", "and", "card", "convex", "diff", "disjoint", "dist", "div", "eq", "ge", "gt",
	"hull", "if", "iff", "imp", "in", "inter", "le", "lt", "max", "min", "mod", "mul", "ne", "neg",
	"not", "or", "pow", "sdiff", "set", "sqr", "sqrt", "sub", "subseq", "subset", "superseq",
	"superset", "union", "xor",
];

/// Expression resulting in a Boolean value or decision variable
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BoolExp<Identifier> {
	/// Boolean constant
	///
	/// When serialized Boolean values `false` and `true` are represented by
	/// integer values 0 and 1.
	Const(bool),
	/// Reference to a variable or array access
	Var(VarRef<Identifier>),
	/// Logical not (i.e., ¬x)
	Not(Box<BoolExp<Identifier>>),
	/// Logical and (i.e., x1 ∧ ...∧ xn)
	And(Vec<BoolExp<Identifier>>),
	/// Logical or (i.e., x1 ∨ ... ∨ xn)
	Or(Vec<BoolExp<Identifier>>),
	/// Logical xor (i.e., x1 ⊕ ... ⊕ xn)
	Xor(Vec<BoolExp<Identifier>>),
	/// Logical equivalence (i.e., x1 ⇔ ... ⇔ xn)
	Equiv(Vec<BoolExp<Identifier>>),
	/// Logical implication (i.e., x ⇒ y)
	Implies(Box<BoolExp<Identifier>>, Box<BoolExp<Identifier>>),
	/// Less than (i.e., x < y)
	LessThan(Box<IntExp<Identifier>>, Box<IntExp<Identifier>>),
	/// Less than or equal (i.e., x ≤ y)
	LessThanEq(Box<IntExp<Identifier>>, Box<IntExp<Identifier>>),
	///Greater than (i.e., x > y)
	GreaterThan(Box<IntExp<Identifier>>, Box<IntExp<Identifier>>),
	///Greater than or equal (i.e., x ≥ y)
	GreaterThanEq(Box<IntExp<Identifier>>, Box<IntExp<Identifier>>),
	/// Different From (i.e., x ≠ y)
	NotEqual(Box<Exp<Identifier>>, Box<Exp<Identifier>>),
	/// Equal to (i.e., x1 = ... = xr)
	Equal(Vec<Exp<Identifier>>),
	/// Membership (i.e., x ∈ s)
	Member(Box<IntExp<Identifier>>, Box<SetExp<Identifier>>),
	/// Disjoint sets (i.e., s ∩ t = ∅)
	Disjoint(Box<SetExp<Identifier>>, Box<SetExp<Identifier>>),
	/// Strict subset (i.e., s ⊂ t)
	SubSet(Box<SetExp<Identifier>>, Box<SetExp<Identifier>>),
	/// Subset or equal to (i.e., s ⊆ t)
	SubSetEq(Box<SetExp<Identifier>>, Box<SetExp<Identifier>>),
	/// Strict superset (i.e., s ⊃ t)
	SuperSet(Box<SetExp<Identifier>>, Box<SetExp<Identifier>>),
	/// Superset or equal to (i.e., s ⊇ t)
	SuperSetEq(Box<SetExp<Identifier>>, Box<SetExp<Identifier>>),
	/// Convexity (i.e., s = {i : min s ≤ i ≤ max s})
	Convex(Box<SetExp<Identifier>>),
}

/// Expression of any type
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Exp<Identifier> {
	/// A Boolean expression
	Bool(Box<BoolExp<Identifier>>),
	/// An integer expression
	Int(Box<IntExp<Identifier>>),
	/// An set of integers expression
	Set(Box<SetExp<Identifier>>),
	/// Reference to a variable or array access
	Var(VarRef<Identifier>),
}

#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
/// Helper structure used to parse a list of expressions.
pub(crate) struct ExpList<Identifier> {
	#[serde(rename = "$text", deserialize_with = "Exp::parse_vec")]
	pub(crate) elements: Vec<Exp<Identifier>>,
}

/// Expression resulting in an integer value or decision variable
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IntExp<Identifier> {
	/// Constant integer value
	Const(IntVal),
	/// Reference to a variable or array access
	Var(VarRef<Identifier>),
	/// Oposite (i.e., -x)
	Neg(Box<IntExp<Identifier>>),
	/// Absolute value (i.e., |x|)
	Abs(Box<IntExp<Identifier>>),
	/// Addition (i.e., x1 + ... + xn)
	Add(Vec<IntExp<Identifier>>),
	/// Subtraction (i.e., x - y)
	Sub(Box<IntExp<Identifier>>, Box<IntExp<Identifier>>),
	/// Multiplication (i.e., x1 ∗ ... ∗ xn)
	Mul(Vec<IntExp<Identifier>>),
	/// Division (i.e., x / y)
	Div(Box<IntExp<Identifier>>, Box<IntExp<Identifier>>),
	/// Remainder (i.e., x % y)
	Mod(Box<IntExp<Identifier>>, Box<IntExp<Identifier>>),
	/// Square (i.e., x^2)
	Sqr(Box<IntExp<Identifier>>),
	/// Power (i.e., x^y)
	Pow(Box<IntExp<Identifier>>, Box<IntExp<Identifier>>),
	/// Minimum (i.e., min{x1, ..., xn})
	Min(Vec<IntExp<Identifier>>),
	/// Maximum (i.e., max{x1, ..., xn})
	Max(Vec<IntExp<Identifier>>),
	/// Distance (i.e., |x - y|)
	Dist(Box<IntExp<Identifier>>, Box<IntExp<Identifier>>),
	/// Alternative (i.e., value of x, if b is true, value of y, otherwise)
	If(
		BoolExp<Identifier>,
		Box<IntExp<Identifier>>,
		Box<IntExp<Identifier>>,
	),
	/// Boolean expression used as an integer expression
	Bool(BoolExp<Identifier>),
	/// Cardinality (i.e., |s|)
	Card(Box<SetExp<Identifier>>),
}

/// Expression resulting in an set of integers value or decision variable
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SetExp<Identifier> {
	/// Set literal specifying each of its values (i.e., {x1, ..., xn})
	Set(Vec<IntExp<Identifier>>),
	/// Set literal specifying an inclusive range using a lower and upper bound
	/// (i.e., { i : x ≤ i ≤ y})
	Range((IntExp<Identifier>, IntExp<Identifier>)),
	/// Reference to a variable or array access
	Var(VarRef<Identifier>),
	/// Convex hull (i.e., {i : min s ≤ i ≤ max s})
	Hull(Box<SetExp<Identifier>>),
	/// Difference (i.e., s \ t)
	Diff(Box<SetExp<Identifier>>, Box<SetExp<Identifier>>),
	/// Union (i.e., s1 ∪ ... ∪ sn)
	Union(Vec<SetExp<Identifier>>),
	/// Intersection (i.e., s1 ∩ ... ∩ sn)
	Inter(Vec<SetExp<Identifier>>),
	/// Symmetric difference (i.e., s1 ∆ ... ∆ sn)
	SDiff(Vec<SetExp<Identifier>>),
}

/// Parser combinator that parses an identifier from a string
pub(crate) fn identifier<Identifier: FromStr>(input: &str) -> IResult<&str, Identifier> {
	let (input, v) = verify(alphanumeric1, |s: &str| {
		s.chars().next().unwrap().is_ascii_alphabetic() && !RESERVED.contains(&s)
	})
	.parse(input)?;
	Ok((
		input,
		match v.parse() {
			Ok(t) => t,
			Err(_) => panic!("unable to create identifier"),
		},
	))
}

/// Parser combinator that parses an integer from a string
pub(crate) fn int(input: &str) -> IResult<&str, IntVal> {
	let (input, neg) = opt(char('-')).parse(input)?;
	let (input, i): (_, i64) = map_res(recognize(digit1), str::parse).parse(input)?;
	Ok((input, if neg.is_some() { -i } else { i }))
}

/// Parser combinator that parses a range of integers from a string
pub(crate) fn range(input: &str) -> IResult<&str, RangeInclusive<IntVal>> {
	let (input, lb) = int(input)?;
	if let (input, Some(_)) = opt(tag("..")).parse(input)? {
		let (input, ub) = int(input)?;
		Ok((input, lb..=ub))
	} else {
		Ok((input, lb..=lb))
	}
}

/// Parser combinator that repeatedly calls a parser consuming whitespace in
/// between when possible
pub(crate) fn sequence<'a, O>(
	p: impl Parser<&'a str, Output = O>,
) -> impl Parser<&'a str, Output = Vec<O>> {
	terminated(many0(preceded(multispace0, p)), multispace0)
}

/// Parser combinator that expects parentheses with a comma seperated parser
/// rules
pub(crate) fn tuple<'a, O>(
	p: impl Parser<&'a str, Output = O>,
) -> impl Parser<&'a str, Output = Vec<O>> {
	delimited(char('('), separated_list1(char(','), p), char(')'))
}

/// Parser combinator that requires whitespace between parsing rules
pub(crate) fn whitespace_seperated<'a, O>(
	p: impl Parser<&'a str, Output = O>,
) -> impl Parser<&'a str, Output = Vec<O>> {
	separated_list1(multispace1, p)
}

impl<Identifier: FromStr> BoolExp<Identifier> {
	/// Parser combinator for a call Boolean expression with a Boolean argument
	/// from a string.
	fn call_arg1(input: &str) -> IResult<&str, Self> {
		let (input, tag) = tag("not")(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e) = Self::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"not" => BoolExp::Not,
				_ => unreachable!(),
			}(Box::new(e)),
		))
	}

	/// Parser combinator for a call Boolean expression with a set arguments from
	/// a string.
	fn call_arg1_set(input: &str) -> IResult<&str, Self> {
		let (input, tag) = tag("convex")(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e) = SetExp::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"convex" => BoolExp::Convex,
				_ => unreachable!(),
			}(Box::new(e)),
		))
	}

	/// Parser combinator for a call Boolean expression with two Boolean arguments
	/// from a string.
	fn call_arg2(input: &str) -> IResult<&str, Self> {
		let (input, tag) = tag("imp")(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e1) = Self::parse(input)?;
		let (input, _) = char(',')(input)?;
		let (input, e2) = Self::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"imp" => BoolExp::Implies,
				_ => unreachable!(),
			}(Box::new(e1), Box::new(e2)),
		))
	}

	/// Parser combinator for a call Boolean expression with two expression
	/// arguments from a string.
	fn call_arg2_exp(input: &str) -> IResult<&str, Self> {
		let (input, tag) = tag("ne")(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e1) = Exp::parse(input)?;
		let (input, _) = char(',')(input)?;
		let (input, e2) = Exp::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"imp" => BoolExp::NotEqual,
				_ => unreachable!(),
			}(Box::new(e1), Box::new(e2)),
		))
	}

	/// Parser combinator for a call Boolean expression with two integer arguments
	/// from a string.
	fn call_arg2_int(input: &str) -> IResult<&str, Self> {
		let (input, tag) = alt((tag("lt"), tag("le"), tag("gt"), tag("ge"))).parse(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e1) = IntExp::parse(input)?;
		let (input, _) = char(',')(input)?;
		let (input, e2) = IntExp::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"lt" => BoolExp::LessThan,
				"le" => BoolExp::LessThanEq,
				"gt" => BoolExp::GreaterThan,
				"ge" => BoolExp::GreaterThanEq,
				_ => unreachable!(),
			}(Box::new(e1), Box::new(e2)),
		))
	}

	/// Parser combinator for a call Boolean expression with an integer and a set
	/// argument from a string.
	fn call_arg2_int_set(input: &str) -> IResult<&str, Self> {
		let (input, tag) = tag("in")(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e1) = IntExp::parse(input)?;
		let (input, _) = char(',')(input)?;
		let (input, e2) = SetExp::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"in" => BoolExp::Member,
				_ => unreachable!(),
			}(Box::new(e1), Box::new(e2)),
		))
	}

	/// Parser combinator for a call Boolean expression with two set arguments
	/// from a string.
	fn call_arg2_set(input: &str) -> IResult<&str, Self> {
		let (input, tag) = alt((
			tag("disjoint"),
			tag("subset"),
			tag("subseq"),
			tag("supseq"),
			tag("supset"),
		))
		.parse(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e1) = SetExp::parse(input)?;
		let (input, _) = char(',')(input)?;
		let (input, e2) = SetExp::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"disjoint" => BoolExp::Disjoint,
				"subset" => BoolExp::SubSet,
				"subseq" => BoolExp::SubSetEq,
				"supseq" => BoolExp::SuperSetEq,
				"supset" => BoolExp::SuperSet,
				_ => unreachable!(),
			}(Box::new(e1), Box::new(e2)),
		))
	}

	/// Parser combinator for a call Boolean expression with a variadic number of
	/// Boolean arguments from a string.
	fn call_argn(input: &str) -> IResult<&str, Self> {
		let (input, tag) = alt((tag("and"), tag("or"), tag("xor"), tag("iff"))).parse(input)?;
		let (input, _) = char('(')(input)?;
		let (input, es) = separated_list0(char(','), Self::parse).parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"and" => BoolExp::And,
				"or" => BoolExp::Or,
				"xor" => BoolExp::Xor,
				"iff" => BoolExp::Equiv,
				_ => unreachable!(),
			}(es),
		))
	}

	/// Parser combinator for a call Boolean expression with a variadic number of
	/// expression arguments from a string.
	fn call_argn_exp(input: &str) -> IResult<&str, Self> {
		let (input, tag) = tag("eq")(input)?;
		let (input, _) = char('(')(input)?;
		let (input, es) = separated_list1(char(','), Exp::parse).parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"eq" => BoolExp::Equal,
				_ => unreachable!(),
			}(es),
		))
	}

	/// Parser combinator for a Boolean expression from a string.
	pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
		alt((
			map(verify(digit1, |s| matches!(s, "0" | "1")), |s| {
				BoolExp::Const(s == "1")
			}),
			Self::call_arg1,
			Self::call_arg1_set,
			Self::call_arg2,
			Self::call_arg2_int,
			Self::call_arg2_int_set,
			Self::call_arg2_set,
			Self::call_arg2_exp,
			Self::call_argn,
			Self::call_argn_exp,
			map(VarRef::parse, BoolExp::Var),
		))
		.parse(input)
	}
}

impl<'de, Identifier: FromStr> Deserialize<'de> for BoolExp<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<BoolExp<Identifier>, D::Error> {
		/// Visitor for deserializing a `BoolExp`.
		struct V<Ident>(PhantomData<Ident>);
		impl<Ident: FromStr> Visitor<'_> for V<Ident> {
			type Value = BoolExp<Ident>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a Boolean expression")
			}

			fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
				let v = v.trim();
				let (_, v) = Self::Value::parse(v)
					.map_err(|e| E::custom(format!("invalid Boolean expression: {e:?}")))?;
				Ok(v)
			}
		}
		deserializer.deserialize_str(V(PhantomData::<Identifier>))
	}
}

impl<Identifier: Display> Display for BoolExp<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BoolExp::Const(b) => write!(f, "{}", if *b { 1 } else { 0 }),
			BoolExp::Var(id) => write!(f, "{}", id),
			BoolExp::Not(e) => write!(f, "not({})", e),
			BoolExp::And(es) => write!(
				f,
				"and({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			BoolExp::Or(es) => write!(
				f,
				"or({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			BoolExp::Xor(es) => write!(
				f,
				"xor({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			BoolExp::Equiv(es) => write!(
				f,
				"iff({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			BoolExp::Implies(e1, e2) => write!(f, "imp({},{})", e1, e2),
			BoolExp::LessThan(e1, e2) => write!(f, "lt({},{})", e1, e2),
			BoolExp::LessThanEq(e1, e2) => write!(f, "le({},{})", e1, e2),
			BoolExp::GreaterThan(e1, e2) => write!(f, "gt({},{})", e1, e2),
			BoolExp::GreaterThanEq(e1, e2) => write!(f, "ge({},{})", e1, e2),
			BoolExp::NotEqual(e1, e2) => write!(f, "ne({},{})", e1, e2),
			BoolExp::Equal(es) => write!(
				f,
				"eq({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			BoolExp::Member(e1, e2) => write!(f, "in({},{})", e1, e2),
			BoolExp::Disjoint(e1, e2) => write!(f, "disjoint({},{})", e1, e2),
			BoolExp::SubSet(e1, e2) => write!(f, "subset({},{})", e1, e2),
			BoolExp::SubSetEq(e1, e2) => write!(f, "subseq({},{})", e1, e2),
			BoolExp::SuperSet(e1, e2) => write!(f, "supset({},{})", e1, e2),
			BoolExp::SuperSetEq(e1, e2) => write!(f, "supseq({},{})", e1, e2),
			BoolExp::Convex(e) => write!(f, "convex({})", e),
		}
	}
}

impl<Identifier: Display> Serialize for BoolExp<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

impl<Identifier: FromStr> Exp<Identifier> {
	/// Parser combinator for an expression of any type from a string
	pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
		alt((
			map(VarRef::parse, |x| Exp::Var(x)),
			map(SetExp::parse, |x| Exp::Set(Box::new(x))),
			map(BoolExp::parse, |x| Exp::Bool(Box::new(x))),
			map(IntExp::parse, |x| Exp::Int(Box::new(x))),
		))
		.parse(input)
	}

	/// Parse a list of expressions seperated by whitespace
	pub(crate) fn parse_vec<'de, D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<Vec<Self>, D::Error> {
		/// Visitor for parsing a list of expressions
		struct V<X>(PhantomData<X>);
		impl<X: FromStr> Visitor<'_> for V<X> {
			type Value = Vec<Exp<X>>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a list of expressions")
			}

			fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
				let v = v.trim();
				let (_, v) = all_consuming(whitespace_seperated(Exp::parse))
					.parse(v)
					.map_err(|_| E::custom(format!("invalid expressions `{v}'")))?;
				Ok(v)
			}
		}
		let visitor = V::<Identifier>(PhantomData);
		deserializer.deserialize_str(visitor)
	}
}

impl<Identifier: Display> Display for Exp<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Exp::Bool(e) => write!(f, "{}", e),
			Exp::Int(e) => write!(f, "{}", e),
			Exp::Set(e) => write!(f, "{}", e),
			Exp::Var(e) => write!(f, "{}", e),
		}
	}
}

impl<Identifier: Display> Serialize for Exp<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

impl<Identifier: FromStr> IntExp<Identifier> {
	/// Parser combinator for a call integer expression with a integer argument
	/// from string
	fn call_arg1(input: &str) -> IResult<&str, Self> {
		let (input, tag) = alt((tag("neg"), tag("abs"), tag("sqr"))).parse(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e) = Self::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"neg" => IntExp::Neg,
				"abs" => IntExp::Abs,
				"sqr" => IntExp::Sqr,
				_ => unreachable!(),
			}(Box::new(e)),
		))
	}

	/// Parser combinator for a call integer expression with a set argument from
	/// string
	fn call_arg1_set(input: &str) -> IResult<&str, Self> {
		let (input, tag) = tag("card")(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e) = SetExp::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"card" => IntExp::Card,
				_ => unreachable!(),
			}(Box::new(e)),
		))
	}

	/// Parser combinator for a call integer expression with two integer arguments
	/// from string
	fn call_arg2(input: &str) -> IResult<&str, Self> {
		let (input, tag) =
			alt((tag("sub"), tag("div"), tag("mod"), tag("pow"), tag("dist"))).parse(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e1) = Self::parse(input)?;
		let (input, _) = char(',')(input)?;
		let (input, e2) = Self::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"sub" => IntExp::Sub,
				"div" => IntExp::Div,
				"mod" => IntExp::Mod,
				"pow" => IntExp::Pow,
				"dist" => IntExp::Dist,
				_ => unreachable!(),
			}(Box::new(e1), Box::new(e2)),
		))
	}

	/// Parser combinator for a call integer expression with three integer
	/// arguments from string
	fn call_arg3(input: &str) -> IResult<&str, Self> {
		let (input, tag) = alt((tag("if"),)).parse(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e1) = BoolExp::parse(input)?;
		let (input, _) = char(',')(input)?;
		let (input, e2) = Self::parse(input)?;
		let (input, _) = char(',')(input)?;
		let (input, e3) = Self::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"if" => IntExp::If,
				_ => unreachable!(),
			}(e1, Box::new(e2), Box::new(e3)),
		))
	}

	/// Parser combinator for a call integer expression with variadic number of
	/// integer arguments from string
	fn call_argn(input: &str) -> IResult<&str, Self> {
		let (input, tag) = alt((tag("add"), tag("max"), tag("min"), tag("mul"))).parse(input)?;
		let (input, _) = char('(')(input)?;
		let (input, es) = separated_list1(char(','), Self::parse).parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"add" => IntExp::Add,
				"max" => IntExp::Max,
				"min" => IntExp::Min,
				"mul" => IntExp::Mul,
				_ => unreachable!(),
			}(es),
		))
	}

	/// Parser combinator for an integer expression from string
	pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
		alt((
			map(int, IntExp::Const),
			IntExp::call_arg1,
			IntExp::call_arg1_set,
			IntExp::call_arg2,
			IntExp::call_arg3,
			IntExp::call_argn,
			map(VarRef::parse, IntExp::Var),
			map(BoolExp::parse, IntExp::Bool),
		))
		.parse(input)
	}

	/// Parse a list of integer expressions
	pub(crate) fn parse_vec<'de, D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<Vec<Self>, D::Error> {
		/// Visitor for a list of integer expressions
		struct V<X>(PhantomData<X>);
		impl<X: FromStr> Visitor<'_> for V<X> {
			type Value = Vec<IntExp<X>>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a list of integers expressions")
			}

			fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
				let v = v.trim();
				let (_, v) = all_consuming(whitespace_seperated(IntExp::parse))
					.parse(v)
					.map_err(|_| E::custom(format!("invalid integer expressions `{v}'")))?;
				Ok(v)
			}
		}
		let visitor = V::<Identifier>(PhantomData);
		deserializer.deserialize_str(visitor)
	}
}

impl<'de, Identifier: FromStr> Deserialize<'de> for IntExp<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<IntExp<Identifier>, D::Error> {
		/// Visitor for `IntExp`
		struct V<Ident>(PhantomData<Ident>);
		impl<Ident: FromStr> Visitor<'_> for V<Ident> {
			type Value = IntExp<Ident>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("an integer expression")
			}

			fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
				let v = v.trim();
				let (_, v) = Self::Value::parse(v)
					.map_err(|_| E::custom(format!("invalid integer expression `{v}'")))?;
				Ok(v)
			}
		}
		deserializer.deserialize_str(V(PhantomData::<Identifier>))
	}
}

impl<Identifier: Display> Display for IntExp<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			IntExp::Const(i) => write!(f, "{}", i),
			IntExp::Var(id) => write!(f, "{}", id),
			IntExp::Neg(e) => write!(f, "neg({})", e),
			IntExp::Abs(e) => write!(f, "abs({})", e),
			IntExp::Add(es) => write!(
				f,
				"add({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			IntExp::Sub(e1, e2) => write!(f, "sub({},{})", e1, e2),
			IntExp::Mul(es) => write!(
				f,
				"mul({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			IntExp::Div(e1, e2) => write!(f, "div({},{})", e1, e2),
			IntExp::Mod(e1, e2) => write!(f, "mod({},{})", e1, e2),
			IntExp::Sqr(e) => write!(f, "sqr({})", e),
			IntExp::Pow(e1, e2) => write!(f, "pow({},{})", e1, e2),
			IntExp::Min(es) => write!(
				f,
				"min({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			IntExp::Max(es) => write!(
				f,
				"max({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			IntExp::Dist(e1, e2) => write!(f, "dist({},{})", e1, e2),
			IntExp::If(b, e1, e2) => write!(f, "if({},{},{})", b, e1, e2),
			IntExp::Bool(b) => write!(f, "{}", b),
			IntExp::Card(s) => write!(f, "card({})", s),
		}
	}
}

impl<Identifier: Display> Serialize for IntExp<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

impl<Identifier: FromStr> SetExp<Identifier> {
	/// Parser combinator to parse a call set expression with 1 argument from a
	/// string.
	fn call_arg1(input: &str) -> IResult<&str, Self> {
		let (input, tag) = tag("hull")(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e) = Self::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"hull" => SetExp::Hull,
				_ => unreachable!(),
			}(Box::new(e)),
		))
	}

	/// Parser combinator to parse a call set expression with 2 arguments from a
	/// string.
	fn call_arg2(input: &str) -> IResult<&str, Self> {
		let (input, tag) = tag("diff")(input)?;
		let (input, _) = char('(')(input)?;
		let (input, e1) = Self::parse(input)?;
		let (input, _) = char(',')(input)?;
		let (input, e2) = Self::parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"diff" => SetExp::Diff,
				_ => unreachable!(),
			}(Box::new(e1), Box::new(e2)),
		))
	}

	/// Parser combinator to parse a call set expression with 3 arguments from a
	/// string.
	fn call_arg3(input: &str) -> IResult<&str, Self> {
		let (input, tag) = alt((tag("union"), tag("inter"), tag("sdiff"))).parse(input)?;
		let (input, _) = char('(')(input)?;
		let (input, es) = separated_list1(char(','), Self::parse).parse(input)?;
		let (input, _) = char(')')(input)?;
		Ok((
			input,
			match tag {
				"union" => SetExp::Union,
				"inter" => SetExp::Inter,
				"sdiff" => SetExp::SDiff,
				_ => unreachable!(),
			}(es),
		))
	}

	/// Parser combinator to parse a set expression from a string.
	pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
		alt((
			map(
				separated_pair(IntExp::parse, tag(".."), IntExp::parse),
				|(from, to)| SetExp::Range((from, to)),
			),
			Self::call_arg1,
			Self::call_arg2,
			Self::call_arg3,
			map(
				delimited(
					pair(tag("set"), char('(')),
					separated_list0(char(','), IntExp::parse),
					char(')'),
				),
				SetExp::Set,
			),
			map(VarRef::parse, SetExp::Var),
		))
		.parse(input)
	}
}

impl<Identifier: Display> Display for SetExp<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			SetExp::Var(id) => write!(f, "{}", id),
			SetExp::Set(es) => write!(
				f,
				"{{{}}}",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			SetExp::Range((from, to)) => write!(f, "{}..{}", from, to),
			SetExp::Hull(e) => write!(f, "hull({})", e),
			SetExp::Diff(e1, e2) => write!(f, "diff({}, {})", e1, e2),
			SetExp::Union(es) => write!(
				f,
				"union({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			SetExp::Inter(es) => write!(
				f,
				"inter({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
			SetExp::SDiff(es) => write!(
				f,
				"sdiff({})",
				es.iter()
					.map(|e| e.to_string())
					.collect::<Vec<_>>()
					.join(",")
			),
		}
	}
}
