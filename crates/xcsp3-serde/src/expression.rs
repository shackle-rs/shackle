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

use std::{
	collections::HashMap, fmt::Display, hash::Hash, marker::PhantomData, ops::RangeInclusive,
};

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

use crate::{error::UnrollError, IntVal, IntoVar, Placeholder, SimpleRef, VarRef};

/// List of reserved identifiers used by builtin expressions
pub const RESERVED: &[&str] = &[
	"abs", "add", "and", "card", "convex", "diff", "disjoint", "dist", "div", "eq", "ge", "gt",
	"hull", "if", "iff", "imp", "in", "inter", "le", "lt", "max", "min", "mod", "mul", "ne", "neg",
	"not", "or", "pow", "sdiff", "set", "sqr", "sqrt", "sub", "subseq", "subset", "superseq",
	"superset", "union", "xor",
];

/// Expression resulting in a Boolean value or decision variable
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BoolExp<Var = VarRef> {
	/// Boolean constant
	///
	/// When serialized Boolean values `false` and `true` are represented by
	/// integer values 0 and 1.
	Const(bool),
	/// Reference to a variable or array access
	Var(Var),
	/// Logical not (i.e., ¬x)
	Not(Box<BoolExp<Var>>),
	/// Logical and (i.e., x1 ∧ ...∧ xn)
	And(Vec<BoolExp<Var>>),
	/// Logical or (i.e., x1 ∨ ... ∨ xn)
	Or(Vec<BoolExp<Var>>),
	/// Logical xor (i.e., x1 ⊕ ... ⊕ xn)
	Xor(Vec<BoolExp<Var>>),
	/// Logical equivalence (i.e., x1 ⇔ ... ⇔ xn)
	Equiv(Vec<BoolExp<Var>>),
	/// Logical implication (i.e., x ⇒ y)
	Implies(Box<BoolExp<Var>>, Box<BoolExp<Var>>),
	/// Less than (i.e., x < y)
	LessThan(Box<IntExp<Var>>, Box<IntExp<Var>>),
	/// Less than or equal (i.e., x ≤ y)
	LessThanEq(Box<IntExp<Var>>, Box<IntExp<Var>>),
	///Greater than (i.e., x > y)
	GreaterThan(Box<IntExp<Var>>, Box<IntExp<Var>>),
	///Greater than or equal (i.e., x ≥ y)
	GreaterThanEq(Box<IntExp<Var>>, Box<IntExp<Var>>),
	/// Different From (i.e., x ≠ y)
	NotEqual(Box<Exp<Var>>, Box<Exp<Var>>),
	/// Equal to (i.e., x1 = ... = xr)
	Equal(Vec<Exp<Var>>),
	/// Membership (i.e., x ∈ s)
	Member(Box<IntExp<Var>>, Box<SetExp<Var>>),
	/// Disjoint sets (i.e., s ∩ t = ∅)
	Disjoint(Box<SetExp<Var>>, Box<SetExp<Var>>),
	/// Strict subset (i.e., s ⊂ t)
	SubSet(Box<SetExp<Var>>, Box<SetExp<Var>>),
	/// Subset or equal to (i.e., s ⊆ t)
	SubSetEq(Box<SetExp<Var>>, Box<SetExp<Var>>),
	/// Strict superset (i.e., s ⊃ t)
	SuperSet(Box<SetExp<Var>>, Box<SetExp<Var>>),
	/// Superset or equal to (i.e., s ⊇ t)
	SuperSetEq(Box<SetExp<Var>>, Box<SetExp<Var>>),
	/// Convexity (i.e., s = {i : min s ≤ i ≤ max s})
	Convex(Box<SetExp<Var>>),
}

/// Expression of any type
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Exp<Var = VarRef> {
	/// A Boolean expression
	Bool(Box<BoolExp<Var>>),
	/// An integer expression
	Int(Box<IntExp<Var>>),
	/// An set of integers expression
	Set(Box<SetExp<Var>>),
	/// Reference to a variable or array access
	Var(Var),
}

#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Var: IntoVar", serialize = "Var: Display"))]
/// Helper structure used to parse a list of expressions.
pub(crate) struct ExpList<Var> {
	#[serde(rename = "$text", deserialize_with = "Exp::parse_vec")]
	pub(crate) elements: Vec<Exp<Var>>,
}

/// Expression resulting in an integer value or decision variable
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IntExp<Var = VarRef> {
	/// Constant integer value
	Const(IntVal),
	/// Reference to a variable or array access
	Var(Var),
	/// Oposite (i.e., -x)
	Neg(Box<IntExp<Var>>),
	/// Absolute value (i.e., |x|)
	Abs(Box<IntExp<Var>>),
	/// Addition (i.e., x1 + ... + xn)
	Add(Vec<IntExp<Var>>),
	/// Subtraction (i.e., x - y)
	Sub(Box<IntExp<Var>>, Box<IntExp<Var>>),
	/// Multiplication (i.e., x1 ∗ ... ∗ xn)
	Mul(Vec<IntExp<Var>>),
	/// Division (i.e., x / y)
	Div(Box<IntExp<Var>>, Box<IntExp<Var>>),
	/// Remainder (i.e., x % y)
	Mod(Box<IntExp<Var>>, Box<IntExp<Var>>),
	/// Square (i.e., x^2)
	Sqr(Box<IntExp<Var>>),
	/// Power (i.e., x^y)
	Pow(Box<IntExp<Var>>, Box<IntExp<Var>>),
	/// Minimum (i.e., min{x1, ..., xn})
	Min(Vec<IntExp<Var>>),
	/// Maximum (i.e., max{x1, ..., xn})
	Max(Vec<IntExp<Var>>),
	/// Distance (i.e., |x - y|)
	Dist(Box<IntExp<Var>>, Box<IntExp<Var>>),
	/// Alternative (i.e., value of x, if b is true, value of y, otherwise)
	If(BoolExp<Var>, Box<IntExp<Var>>, Box<IntExp<Var>>),
	/// Boolean expression used as an integer expression
	Bool(BoolExp<Var>),
	/// Cardinality (i.e., |s|)
	Card(Box<SetExp<Var>>),
}

/// Expression resulting in an set of integers value or decision variable
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SetExp<Var = VarRef> {
	/// Set literal specifying each of its values (i.e., {x1, ..., xn})
	Set(Vec<IntExp<Var>>),
	/// Set literal specifying an inclusive range using a lower and upper bound
	/// (i.e., { i : x ≤ i ≤ y})
	Range((IntExp<Var>, IntExp<Var>)),
	/// Reference to a variable or array access
	Var(Var),
	/// Convex hull (i.e., {i : min s ≤ i ≤ max s})
	Hull(Box<SetExp<Var>>),
	/// Difference (i.e., s \ t)
	Diff(Box<SetExp<Var>>, Box<SetExp<Var>>),
	/// Union (i.e., s1 ∪ ... ∪ sn)
	Union(Vec<SetExp<Var>>),
	/// Intersection (i.e., s1 ∩ ... ∩ sn)
	Inter(Vec<SetExp<Var>>),
	/// Symmetric difference (i.e., s1 ∆ ... ∆ sn)
	SDiff(Vec<SetExp<Var>>),
}

/// Parser combinator that parses an identifier from a string
pub(crate) fn identifier<Identifier: From<String>>(input: &str) -> IResult<&str, Identifier> {
	let (input, v) = verify(alphanumeric1, |s: &str| {
		s.chars().next().unwrap().is_ascii_alphabetic() && !RESERVED.contains(&s)
	})
	.parse(input)?;
	Ok((input, v.to_owned().into()))
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

impl<Identifier> BoolExp<VarRef<Identifier>> {
	/// Returns the placeholder with the highest number, or None if there are no
	/// placeholders.
	pub(crate) fn max_placeholder(&self) -> Option<usize> {
		match self {
			BoolExp::Const(_) => None,
			&BoolExp::Var(VarRef::Placeholder(Placeholder::Position(i))) => Some(i),
			BoolExp::Var(_) => None,
			BoolExp::Not(e) => e.max_placeholder(),
			BoolExp::And(exps) | BoolExp::Equiv(exps) | BoolExp::Or(exps) | BoolExp::Xor(exps) => {
				exps.iter().flat_map(|e| e.max_placeholder()).max()
			}
			BoolExp::Equal(exps) => exps.iter().flat_map(|e| e.max_placeholder()).max(),
			BoolExp::Implies(e1, e2) => {
				[e1, e2].into_iter().flat_map(|e| e.max_placeholder()).max()
			}
			BoolExp::GreaterThan(e1, e2)
			| BoolExp::GreaterThanEq(e1, e2)
			| BoolExp::LessThan(e1, e2)
			| BoolExp::LessThanEq(e1, e2) => [e1, e2].into_iter().flat_map(|e| e.max_placeholder()).max(),
			BoolExp::NotEqual(e1, e2) => {
				[e1, e2].into_iter().flat_map(|e| e.max_placeholder()).max()
			}
			BoolExp::Member(e1, e2) => [e1.max_placeholder(), e2.max_placeholder()]
				.into_iter()
				.flatten()
				.max(),
			BoolExp::Disjoint(e1, e2)
			| BoolExp::SubSet(e1, e2)
			| BoolExp::SubSetEq(e1, e2)
			| BoolExp::SuperSet(e1, e2)
			| BoolExp::SuperSetEq(e1, e2) => [e1, e2].into_iter().flat_map(|e| e.max_placeholder()).max(),
			BoolExp::Convex(e) => e.max_placeholder(),
		}
	}
}

impl<Identifier: Clone + Hash + Eq + ToString> BoolExp<VarRef<Identifier>> {
	pub(crate) fn unroll_single(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Exp<SimpleRef<Identifier>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<BoolExp<SimpleRef<Identifier>>, UnrollError> {
		match self {
			BoolExp::Var(v) => v.unroll_single(arrays, args, remainder)?.try_into(),
			&BoolExp::Const(b) => Ok(BoolExp::Const(b)),
			BoolExp::Not(b) => b
				.unroll_single(arrays, args, remainder)
				.map(|i| BoolExp::Not(i.into())),
			BoolExp::And(exps) | BoolExp::Equiv(exps) | BoolExp::Or(exps) | BoolExp::Xor(exps) => {
				let mut res = Vec::new();
				for exp in exps {
					res.extend(exp.unroll(arrays, args, remainder)?);
				}
				Ok(match self {
					BoolExp::And(_) => BoolExp::And,
					BoolExp::Equiv(_) => BoolExp::Equiv,
					BoolExp::Or(_) => BoolExp::Or,
					BoolExp::Xor(_) => BoolExp::Xor,
					_ => unreachable!(),
				}(res))
			}
			BoolExp::Implies(b1, b2) => {
				let b1 = b1.unroll_single(arrays, args, remainder)?;
				let b2 = b2.unroll_single(arrays, args, remainder)?;
				Ok(BoolExp::Implies(b1.into(), b2.into()))
			}
			BoolExp::LessThan(i1, i2)
			| BoolExp::LessThanEq(i1, i2)
			| BoolExp::GreaterThan(i1, i2)
			| BoolExp::GreaterThanEq(i1, i2) => {
				let i1 = i1.unroll_single(arrays, args, remainder)?;
				let i2 = i2.unroll_single(arrays, args, remainder)?;
				Ok(match self {
					BoolExp::LessThan(_, _) => BoolExp::LessThan,
					BoolExp::LessThanEq(_, _) => BoolExp::LessThanEq,
					BoolExp::GreaterThan(_, _) => BoolExp::GreaterThan,
					BoolExp::GreaterThanEq(_, _) => BoolExp::GreaterThanEq,
					_ => unreachable!(),
				}(i1.into(), i2.into()))
			}
			BoolExp::NotEqual(e1, e2) => {
				let e1 = e1.unroll_single(arrays, args, remainder)?;
				let e2 = e2.unroll_single(arrays, args, remainder)?;
				Ok(BoolExp::NotEqual(e1.into(), e2.into()))
			}
			BoolExp::Equal(exps) => {
				let mut res = Vec::new();
				for exp in exps {
					res.extend(exp.unroll(arrays, args, remainder)?);
				}
				Ok(BoolExp::Equal(res))
			}
			BoolExp::Member(i, s) => {
				let i = i.unroll_single(arrays, args, remainder)?;
				let s = s.unroll_single(arrays, args, remainder)?;
				Ok(BoolExp::Member(i.into(), s.into()))
			}
			BoolExp::Disjoint(s1, s2)
			| BoolExp::SubSet(s1, s2)
			| BoolExp::SubSetEq(s1, s2)
			| BoolExp::SuperSet(s1, s2)
			| BoolExp::SuperSetEq(s1, s2) => {
				let s1 = s1.unroll_single(arrays, args, remainder)?;
				let s2 = s2.unroll_single(arrays, args, remainder)?;
				Ok(match self {
					BoolExp::Disjoint(_, _) => BoolExp::Disjoint,
					BoolExp::SubSet(_, _) => BoolExp::SubSet,
					BoolExp::SubSetEq(_, _) => BoolExp::SubSetEq,
					BoolExp::SuperSet(_, _) => BoolExp::SuperSet,
					BoolExp::SuperSetEq(_, _) => BoolExp::SuperSetEq,
					_ => unreachable!(),
				}(s1.into(), s2.into()))
			}
			BoolExp::Convex(set) => set
				.unroll_single(arrays, args, remainder)
				.map(|s| BoolExp::Convex(s.into())),
		}
	}

	pub(crate) fn unroll(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Exp<SimpleRef<Identifier>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Vec<BoolExp<SimpleRef<Identifier>>>, UnrollError> {
		if let BoolExp::Var(v) = self {
			v.unroll(arrays, args, remainder)?
				.into_iter()
				.map(|e| e.try_into())
				.collect()
		} else {
			Ok(vec![self.unroll_single(arrays, args, remainder)?])
		}
	}
}

impl<Var: IntoVar> BoolExp<Var> {
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
			map(VarRef::parse, |v| BoolExp::Var(Var::into_var(v))),
		))
		.parse(input)
	}
}

impl<'de, Var: IntoVar> Deserialize<'de> for BoolExp<Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Visitor for deserializing a `BoolExp`.
		struct V<Var>(PhantomData<Var>);
		impl<Var: IntoVar> Visitor<'_> for V<Var> {
			type Value = BoolExp<Var>;

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
		deserializer.deserialize_str(V(PhantomData::<Var>))
	}
}

impl<Identifier: Display> Display for BoolExp<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BoolExp::Const(b) => write!(f, "{}", if *b { 1 } else { 0 }),
			BoolExp::Var(id) => write!(f, "{}", id.to_string()),
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

impl<Var> TryFrom<Exp<Var>> for BoolExp<Var> {
	type Error = UnrollError;

	fn try_from(exp: Exp<Var>) -> Result<Self, UnrollError> {
		match exp {
			Exp::Bool(b) => Ok(*b),
			Exp::Int(_) => Err(UnrollError::InvalidType {
				placeholder_ty: "int",
				arg_ty: "bool",
			}),
			Exp::Set(_) => Err(UnrollError::InvalidType {
				placeholder_ty: "set",
				arg_ty: "bool",
			}),
			Exp::Var(var) => Ok(BoolExp::Var(var)),
		}
	}
}

impl<Var> Exp<Var> {
	pub(crate) fn into_var(self) -> Result<Var, UnrollError> {
		match self {
			Exp::Var(v) => Ok(v),
			Exp::Bool(b) => {
				if let BoolExp::Var(v) = *b {
					Ok(v)
				} else {
					Err(UnrollError::InvalidType {
						placeholder_ty: "var_ref",
						arg_ty: "bool",
					})
				}
			}
			Exp::Int(i) => {
				if let IntExp::Var(v) = *i {
					Ok(v)
				} else {
					Err(UnrollError::InvalidType {
						placeholder_ty: "var_ref",
						arg_ty: "int",
					})
				}
			}
			Exp::Set(s) => {
				if let SetExp::Var(v) = *s {
					Ok(v)
				} else {
					Err(UnrollError::InvalidType {
						placeholder_ty: "var_ref",
						arg_ty: "set",
					})
				}
			}
		}
	}
}

impl<Identifier> Exp<VarRef<Identifier>> {
	/// Returns the placeholder with the highest number, or None if there are no
	/// placeholders.
	pub(crate) fn max_placeholder(&self) -> Option<usize> {
		match self {
			&Exp::Var(VarRef::Placeholder(Placeholder::Position(i))) => Some(i),
			Exp::Var(_) => None,
			Exp::Set(s) => s.max_placeholder(),
			Exp::Bool(b) => b.max_placeholder(),
			Exp::Int(i) => i.max_placeholder(),
		}
	}
}

impl<Identifier: Clone + Hash + Eq + ToString> Exp<VarRef<Identifier>> {
	pub(crate) fn unroll_single(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Exp<SimpleRef<Identifier>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Exp<SimpleRef<Identifier>>, UnrollError> {
		match self {
			Exp::Var(v) => v.unroll_single(arrays, args, remainder),
			Exp::Set(s) => s.unroll_single(arrays, args, remainder).map(Into::into),
			Exp::Bool(b) => b.unroll_single(arrays, args, remainder).map(Into::into),
			Exp::Int(i) => i.unroll_single(arrays, args, remainder).map(Into::into),
		}
	}

	pub(crate) fn unroll(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Exp<SimpleRef<Identifier>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Vec<Exp<SimpleRef<Identifier>>>, UnrollError> {
		match self {
			Exp::Var(v) => v.unroll(arrays, args, remainder),
			Exp::Set(s) => Ok(s
				.unroll(arrays, args, remainder)?
				.into_iter()
				.map(Into::into)
				.collect()),
			Exp::Bool(b) => Ok(b
				.unroll(arrays, args, remainder)?
				.into_iter()
				.map(Into::into)
				.collect()),
			Exp::Int(i) => Ok(i
				.unroll(arrays, args, remainder)?
				.into_iter()
				.map(Into::into)
				.collect()),
		}
	}
}

impl<Var: IntoVar> Exp<Var> {
	/// Parser combinator for an expression of any type from a string
	pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
		alt((
			map(VarRef::parse, |x| Exp::Var(Var::into_var(x))),
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
		impl<X: IntoVar> Visitor<'_> for V<X> {
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
		let visitor = V::<Var>(PhantomData);
		deserializer.deserialize_str(visitor)
	}
}

impl<Identifier: Display> Display for Exp<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Exp::Bool(e) => write!(f, "{}", e),
			Exp::Int(e) => write!(f, "{}", e),
			Exp::Set(e) => write!(f, "{}", e),
			Exp::Var(e) => write!(f, "{}", e.to_string()),
		}
	}
}

impl<Identifier> From<BoolExp<Identifier>> for Exp<Identifier> {
	fn from(i: BoolExp<Identifier>) -> Exp<Identifier> {
		Exp::Bool(i.into())
	}
}

impl<Identifier> From<IntExp<Identifier>> for Exp<Identifier> {
	fn from(i: IntExp<Identifier>) -> Exp<Identifier> {
		Exp::Int(i.into())
	}
}

impl<Identifier> From<SetExp<Identifier>> for Exp<Identifier> {
	fn from(i: SetExp<Identifier>) -> Exp<Identifier> {
		Exp::Set(i.into())
	}
}

impl<Identifier: Display> Serialize for Exp<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

impl<Identifier> IntExp<VarRef<Identifier>> {
	/// Returns the placeholder with the highest number, or None if there are no
	/// placeholders.
	pub(crate) fn max_placeholder(&self) -> Option<usize> {
		match self {
			IntExp::Const(_) => None,
			&IntExp::Var(VarRef::Placeholder(Placeholder::Position(i))) => Some(i),
			IntExp::Var(_) => None,
			IntExp::Abs(e) | IntExp::Neg(e) | IntExp::Sqr(e) => e.max_placeholder(),
			IntExp::Add(exps) | IntExp::Max(exps) | IntExp::Min(exps) | IntExp::Mul(exps) => {
				exps.iter().filter_map(|e| e.max_placeholder()).max()
			}
			IntExp::Dist(e1, e2)
			| IntExp::Div(e1, e2)
			| IntExp::Mod(e1, e2)
			| IntExp::Pow(e1, e2)
			| IntExp::Sub(e1, e2) => [e1, e2].iter().filter_map(|e| e.max_placeholder()).max(),
			IntExp::If(e1, e2, e3) => [
				e1.max_placeholder(),
				e2.max_placeholder(),
				e3.max_placeholder(),
			]
			.into_iter()
			.flatten()
			.max(),
			IntExp::Bool(e) => e.max_placeholder(),
			IntExp::Card(e) => e.max_placeholder(),
		}
	}
}

impl<Identifier: Clone + Hash + Eq + ToString> IntExp<VarRef<Identifier>> {
	pub(crate) fn unroll_single(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Exp<SimpleRef<Identifier>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<IntExp<SimpleRef<Identifier>>, UnrollError> {
		match self {
			IntExp::Var(v) => v.unroll_single(arrays, args, remainder)?.try_into(),
			&IntExp::Const(c) => Ok(IntExp::Const(c)),
			IntExp::Abs(i) | IntExp::Neg(i) | IntExp::Sqr(i) => {
				let i = i.unroll_single(arrays, args, remainder)?;
				Ok(match self {
					IntExp::Abs(_) => IntExp::Abs,
					IntExp::Neg(_) => IntExp::Neg,
					IntExp::Sqr(_) => IntExp::Sqr,
					_ => unreachable!(),
				}(i.into()))
			}
			IntExp::Add(exps) | IntExp::Mul(exps) | IntExp::Min(exps) | IntExp::Max(exps) => {
				let mut res = Vec::new();
				for e in exps {
					res.extend(e.unroll(arrays, args, remainder)?);
				}
				Ok(match self {
					IntExp::Add(_) => IntExp::Add,
					IntExp::Mul(_) => IntExp::Mul,
					IntExp::Min(_) => IntExp::Min,
					IntExp::Max(_) => IntExp::Max,
					_ => unreachable!(),
				}(res))
			}
			IntExp::Dist(i1, i2)
			| IntExp::Div(i1, i2)
			| IntExp::Mod(i1, i2)
			| IntExp::Pow(i1, i2)
			| IntExp::Sub(i1, i2) => {
				let i1 = i1.unroll_single(arrays, args, remainder)?;
				let i2 = i2.unroll_single(arrays, args, remainder)?;
				Ok(match self {
					IntExp::Dist(_, _) => IntExp::Dist,
					IntExp::Div(_, _) => IntExp::Div,
					IntExp::Mod(_, _) => IntExp::Mod,
					IntExp::Pow(_, _) => IntExp::Pow,
					IntExp::Sub(_, _) => IntExp::Sub,
					_ => unreachable!(),
				}(i1.into(), i2.into()))
			}
			IntExp::If(b, i1, i2) => {
				let b = b.unroll_single(arrays, args, remainder)?;
				let i1 = i1.unroll_single(arrays, args, remainder)?;
				let i2 = i2.unroll_single(arrays, args, remainder)?;
				Ok(IntExp::If(b, i1.into(), i2.into()))
			}
			IntExp::Bool(b) => Ok(IntExp::Bool(b.unroll_single(arrays, args, remainder)?)),
			IntExp::Card(s) => Ok(IntExp::Card(
				s.unroll_single(arrays, args, remainder)?.into(),
			)),
		}
	}

	pub(crate) fn unroll(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Exp<SimpleRef<Identifier>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Vec<IntExp<SimpleRef<Identifier>>>, UnrollError> {
		if let IntExp::Var(v) = self {
			v.unroll(arrays, args, remainder)?
				.into_iter()
				.map(|e| e.try_into())
				.collect()
		} else {
			Ok(vec![self.unroll_single(arrays, args, remainder)?])
		}
	}
}

impl<Var: IntoVar> IntExp<Var> {
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
			map(VarRef::parse, |v| IntExp::Var(Var::into_var(v))),
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
		impl<X: IntoVar> Visitor<'_> for V<X> {
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
		let visitor = V::<Var>(PhantomData);
		deserializer.deserialize_str(visitor)
	}
}

impl<'de, Var: IntoVar> Deserialize<'de> for IntExp<Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<IntExp<Var>, D::Error> {
		/// Visitor for `IntExp`
		struct V<Ident>(PhantomData<Ident>);
		impl<Ident: IntoVar> Visitor<'_> for V<Ident> {
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
		deserializer.deserialize_str(V(PhantomData::<Var>))
	}
}

impl<Identifier: Display> Display for IntExp<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			IntExp::Const(i) => write!(f, "{}", i),
			IntExp::Var(id) => write!(f, "{}", id.to_string()),
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

impl<Var> TryFrom<Exp<Var>> for IntExp<Var> {
	type Error = UnrollError;

	fn try_from(exp: Exp<Var>) -> Result<Self, UnrollError> {
		match exp {
			Exp::Int(i) => Ok(*i),
			Exp::Bool(b) => Ok(IntExp::Bool(*b)),
			Exp::Set(_) => Err(UnrollError::InvalidType {
				placeholder_ty: "set",
				arg_ty: "int",
			}),
			Exp::Var(var) => Ok(IntExp::Var(var)),
		}
	}
}

impl<Identifier> SetExp<VarRef<Identifier>> {
	/// Returns the placeholder with the highest number, or None if there are no
	/// placeholders.
	pub(crate) fn max_placeholder(&self) -> Option<usize> {
		match self {
			&SetExp::Var(VarRef::Placeholder(Placeholder::Position(i))) => Some(i),
			SetExp::Var(_) => None,
			SetExp::Set(exps) => exps.iter().filter_map(|e| e.max_placeholder()).max(),
			SetExp::Range((e1, e2)) => [e1, e2].iter().filter_map(|e| e.max_placeholder()).max(),
			SetExp::Hull(e) => e.max_placeholder(),
			SetExp::Diff(e1, e2) => [e1, e2].iter().filter_map(|e| e.max_placeholder()).max(),
			SetExp::Inter(exps) | SetExp::SDiff(exps) | SetExp::Union(exps) => {
				exps.iter().filter_map(|e| e.max_placeholder()).max()
			}
		}
	}
}

impl<Identifier: Clone + Hash + Eq + ToString> SetExp<VarRef<Identifier>> {
	pub(crate) fn unroll_single(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Exp<SimpleRef<Identifier>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<SetExp<SimpleRef<Identifier>>, UnrollError> {
		match self {
			SetExp::Var(v) => v.unroll_single(arrays, args, remainder)?.try_into(),
			SetExp::Range((i1, i2)) => {
				let i1 = i1.unroll_single(arrays, args, remainder)?;
				let i2 = i2.unroll_single(arrays, args, remainder)?;
				Ok(SetExp::Range((i1, i2)))
			}
			SetExp::Hull(s) => Ok(SetExp::Hull(
				s.unroll_single(arrays, args, remainder)?.into(),
			)),
			SetExp::Diff(s1, s2) => {
				let s1 = s1.unroll_single(arrays, args, remainder)?;
				let s2 = s2.unroll_single(arrays, args, remainder)?;
				Ok(SetExp::Diff(s1.into(), s2.into()))
			}
			SetExp::Inter(set_exps) | SetExp::SDiff(set_exps) | SetExp::Union(set_exps) => {
				let mut result = Vec::new();
				for exp in set_exps {
					result.push(exp.unroll_single(arrays, args, remainder)?);
				}
				Ok(match self {
					SetExp::Union(_) => SetExp::Union,
					SetExp::Inter(_) => SetExp::Inter,
					SetExp::SDiff(_) => SetExp::SDiff,
					_ => unreachable!(),
				}(result))
			}
			SetExp::Set(int_exps) => {
				let mut result = Vec::new();
				for exp in int_exps {
					result.push(exp.unroll_single(arrays, args, remainder)?);
				}
				Ok(SetExp::Set(result))
			}
		}
	}

	pub(crate) fn unroll(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Exp<SimpleRef<Identifier>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Vec<SetExp<SimpleRef<Identifier>>>, UnrollError> {
		if let SetExp::Var(v) = self {
			v.unroll(arrays, args, remainder)?
				.into_iter()
				.map(|e| e.try_into())
				.collect()
		} else {
			Ok(vec![self.unroll_single(arrays, args, remainder)?])
		}
	}
}

impl<Var: IntoVar> SetExp<Var> {
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
			map(VarRef::parse, |v| SetExp::Var(Var::into_var(v))),
		))
		.parse(input)
	}
}

impl<Identifier: Display> Display for SetExp<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			SetExp::Var(id) => write!(f, "{}", id.to_string()),
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

impl<Var> TryFrom<Exp<Var>> for SetExp<Var> {
	type Error = UnrollError;

	fn try_from(exp: Exp<Var>) -> Result<Self, UnrollError> {
		match exp {
			Exp::Set(set) => Ok(*set),
			Exp::Int(_) => Err(UnrollError::InvalidType {
				placeholder_ty: "int",
				arg_ty: "set",
			}),
			Exp::Bool(_) => Err(UnrollError::InvalidType {
				placeholder_ty: "bool",
				arg_ty: "set",
			}),
			Exp::Var(var) => Ok(SetExp::Var(var)),
		}
	}
}
