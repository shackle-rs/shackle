//! # Constraints
//!
//! This module contains the definition of the constraints that can be used in a
//! XCSP3 instance. Each constraint is represented by a struct that contains the
//! necessary information to represent the constraint in the XCSP3 format. The
//! enumerated type [`Constraint`] is used to represent any of constraint.

use std::{borrow::Cow, fmt::Display, marker::PhantomData, str::FromStr};

use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::char,
	combinator::{all_consuming, map},
	sequence::{delimited, separated_pair},
	IResult, Parser,
};
use serde::{
	de::{self, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{
	as_str, deserialize_int_vals,
	expression::{
		identifier, int, sequence, tuple, whitespace_seperated, BoolExp, Exp, ExpList, IntExp,
	},
	from_str, serialize_list, Instantiation, IntVal, MetaInfo,
};

macro_rules! constraints_enum {
	($(#[$attr:meta])* $vis:vis $name:ident, $basic:meta, $meta:meta, $args:meta) => {
		$(#[$attr])*
		$vis enum $name<Identifier = String> {
			#[cfg($basic)]
			/// [`AllDifferent`] constraint
			AllDifferent(AllDifferent<Identifier>),
			#[cfg($basic)]
			/// [`AllEqual`] constraint
			AllEqual(AllEqual<Identifier>),
			#[cfg($basic)]
			/// [`BinPacking`] constraint
			BinPacking(BinPacking<Identifier>),
			#[cfg($basic)]
			/// [`Cardinality`] constraint
			Cardinality(Cardinality<Identifier>),
			#[cfg($basic)]
			/// [`Channel`] constraint
			Channel(Channel<Identifier>),
			#[cfg($basic)]
			/// [`Circuit`] constraint
			Circuit(Circuit<Identifier>),
			#[cfg($basic)]
			/// [`Count`] constraint
			Count(Count<Identifier>),
			#[cfg($basic)]
			/// [`Cumulative`] constraint
			Cumulative(Cumulative<Identifier>),
			#[cfg($basic)]
			/// [`Element`] constraint
			Element(Element<Identifier>),
			#[cfg($basic)]
			/// [`Extension`] constraint
			Extension(Extension<Identifier>),
			#[cfg($basic)]
			/// [`Instantiation`] constraint
			Instantiation(Instantiation<Identifier>),
			#[cfg($basic)]
			/// [`Intension`] constraint
			Intension(Intension<Identifier>),
			#[cfg($basic)]
			/// [`Knapsack`] constraint
			Knapsack(Knapsack<Identifier>),
			#[cfg($basic)]
			/// [`Maximum`] constraint
			Maximum(Maximum<Identifier>),
			#[cfg($basic)]
			/// [`Mdd`] constraint
			Mdd(Mdd<Identifier>),
			#[cfg($basic)]
			/// [`Minimum`] constraint
			Minimum(Minimum<Identifier>),
			#[cfg($basic)]
			/// [`NValues`] constraint
			NValues(NValues<Identifier>),
			#[cfg($basic)]
			/// [`NoOverlap`] constraint
			NoOverlap(NoOverlap<Identifier>),
			#[cfg($basic)]
			/// [`Ordered`] constraint
			Ordered(Ordered<Identifier>),
			#[cfg($basic)]
			/// [`Precedence`] constraint
			Precedence(Precedence<Identifier>),
			#[cfg($basic)]
			/// [`Regular`] constraint
			Regular(Regular<Identifier>),
			#[cfg($basic)]
			/// [`Sum`] constraint
			Sum(Sum<Identifier>),
			#[cfg($meta)]
			/// Constraint [`Group`], that can serve as a template
			Group(Group<Identifier>),
			#[cfg($meta)]
			/// Constraint [`Group`], that can serve as a template
			Block(Block<Identifier>),
			#[cfg(not($basic))]
			/// Simple [`Constraint`] type
			Constraint(Constraint<Identifier>),
			#[cfg($args)]
			/// Constraint [`Group`], that can serve as a template
			Args(ExpList<Identifier>),
		}
	};
}

constraints_enum!(
	#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
	#[serde(
				rename_all = "camelCase",
				bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display")
	)]
	/// Enumerated type to represent basic (instantiated) constraints
	pub Constraint,
	/* expand_basic = true */ all(),
	/* meta = false */ any(),
	/* template_args = false */ any()
);
constraints_enum!(
	#[derive(Clone, Debug, PartialEq, Hash)]
	/// Enumerated type that contains meta-constraints, such as [`Group`] and
	/// [`Block`], in addition to standard [`Constraint`].
	pub MetaConstraint,
	/* expand_basic = false */ any(),
	/* meta = true */ all(),
	/* template_args = false */ any()
);
constraints_enum!(
	#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
	#[serde(
		rename_all = "camelCase",
		bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display")
	)]
	/// Internal constraint enum used to capture [`MetaConstraint`].
	CaptureConstraint,
	/* expand_basic = true */ all(),
	/* meta = true */ all(),
	/* template_args = false */ any()
);
constraints_enum!(
	#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
	#[serde(
		rename_all = "camelCase",
		bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display")
	)]
	/// Internal constraint enum used to capture basic constraints and <args> elements
	TemplateCapture,
	/* expand_basic = true */ all(),
	/* meta = true */ any(),
	/* template_args = false */ all()
);

/// Constraint forcing a set of expressions to take distinct values
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct AllDifferent<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	/// List of expressions that must take distinct values
	pub list: Vec<IntExp<Identifier>>,
	/// List of values that are excluded from the constraint and can be taken by
	/// multiple expressions
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub except: Vec<IntVal>,
}

/// Constraint forcing a set of expressions to take the same value
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct AllEqual<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions that must take the same value
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// List of values that are excluded from the constraint and can be taken by
	/// expressions not matching other expressions
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub except: Vec<IntVal>,
}

// TODO: Should `condition`, `limits` and `loads` be made mutually exclusive in
// the struct?
/// Constraint forcing a list of items, whose sizes are given, are put in
/// different bins in such a way that the total size of the items in each bin
/// respects a numerical condition.
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct BinPacking<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions representing the bin in which each item is placed
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// List of expressions representing the size of each item
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub sizes: Vec<IntExp<Identifier>>,
	/// Condition that must be respected by the total size of the items in each bin
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub condition: Option<Condition<Identifier>>,
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	/// List of expressions representing the limit for the total size of the items
	/// in each bin
	pub limits: Vec<IntExp<Identifier>>,
	/// List of expressions representing the load of each bin
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub loads: Vec<IntExp<Identifier>>,
}

#[derive(Clone, Debug, PartialEq, Hash, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
/// A set of constraints that is linked together semantically
pub struct Block<Identifier = String> {
	#[serde(
		default,
		rename = "@class",
		skip_serializing_if = "Vec::is_empty",
		serialize_with = "serialize_list"
	)]
	/// Optional class designation for the contained constraints
	pub class: Vec<Identifier>,
	#[serde(default, rename = "$value")]
	/// List of constraints
	pub constraints: Vec<MetaConstraint<Identifier>>,
	#[serde(flatten)]
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
}

/// Constraint enforcing the amount of times certain values are taken by a set
/// of expressions.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Cardinality<Identifier = String> {
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
	/// List of expressions of which the values are observed
	pub list: Vec<IntExp<Identifier>>,
	/// List of values that are observed
	pub values: Vec<IntExp<Identifier>>,
	/// Whether the expressions are allowed to take values not in the list of
	/// observed values
	pub closed: bool,
	/// List of expressions representing the number of times each value is taken
	pub occurs: Vec<Exp<Identifier>>,
}

/// Constraint that enforces that if the ith expression takes the value j, then
/// the jth expression takes the value i.
///
/// If [`Self::inverse_list`] is not empty, then the constraint enforces that if
/// the ith expression in [`Self::list`] takes the value j, then the jth
/// expression in [`Self::inverse_list`] takes the value i.
///
/// If [`Self::value`] is not empty, then the constraint enforces that the ith
/// expression in [`Self::list`] takes the value 1 iff the expression in
/// [`Self::value`] takes the value i.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Channel<Identifier = String> {
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
	/// List of expressions that is being channelled
	pub list: Vec<IntExp<Identifier>>,
	/// Inverse list of expressions that is being channelled
	pub inverse_list: Vec<IntExp<Identifier>>,
	/// Expression representing the index of the only expression in [`Self::list`]
	/// that is allowed to take the value 1.
	pub value: Option<IntExp<Identifier>>,
}

/// Constraint that ensures that the values of the expressions in [`Self::list`]
/// form a circuit.
///
/// That is to say, each expression takes the value of an list index,
/// representing an arc in the circuit. The values of the expressions must form
/// a cycle. Expressions are allowed to take the value of their own index,
/// effectively making excluding them from the cycle. When [`Self::size`] is
/// given, then the circuit must have the length of [`Self::size`]. Otherwise,
/// the circuit must be at least 2 in length.
#[derive(Clone, Debug, PartialEq, Hash, Serialize)]
#[serde(bound(serialize = "Identifier: Display"))]
pub struct Circuit<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions that must form a circuit
	pub list: OffsetList<Identifier>,
	/// Size of the circuit
	#[serde(skip_serializing_if = "Option::is_none")]
	pub size: Option<IntExp<Identifier>>,
}

/// Condition to be enforced
///
/// This type is used as part of a larger constraint type
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Condition<Identifier> {
	/// Operator of the condition
	pub operator: Operator,
	/// Right-side operand of the condition
	pub operand: Exp<Identifier>,
}

/// Constraint that enforced that the number of times expressions in
/// [`Self::list`] take a value from [`Self::values`] abides by the given
/// [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Count<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions of which the values are observed
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// List of values that are counted
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub values: Vec<IntExp<Identifier>>,
	/// Condition to be enforced on the count
	pub condition: Condition<Identifier>,
}

/// Constraint that enforces that at each point in time, the cumulated height of
/// tasks that overlap that point, respects the given [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Cumulative<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of starting time-points of the tasks
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub origins: Vec<IntExp<Identifier>>,
	/// List of durations of the tasks
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub lengths: Vec<IntExp<Identifier>>,
	/// List of heights of the tasks
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub heights: Vec<IntExp<Identifier>>,
	/// Condition to be enforced on the cumulated height at each time point
	pub condition: Condition<Identifier>,
}

/// Constraint that enforces that the value of the expression at
/// [`Self::index`] abides by the given [`Self::condition`], or alternatively is
/// equal the expression [`Self::value`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Element<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// Indexed list of values
	pub list: OffsetList<Identifier>,
	/// Index of the value to be constrained
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub index: Option<IntExp<Identifier>>,
	/// Value to be assigned to the indexed expression
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub value: Option<IntExp<Identifier>>,
	/// Condition to be enforced on the indexed expression
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub condition: Option<Condition<Identifier>>,
}

// TODO: Support for "smart" extension
/// Constraint that enforces that the expressions in [`Self::list`] either take
/// the values of one of the rows in [`Self::supports`], or alternatively do not
/// match any of the rows in [`Self::conflicts`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Extension<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// Combinations of values that the expressions are allowed to take
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_tuples",
		serialize_with = "serialize_int_tuples"
	)]
	pub supports: Vec<Vec<IntVal>>,
	/// Combinations of values that the expressions are not allowed to take
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_tuples",
		serialize_with = "serialize_int_tuples"
	)]
	pub conflicts: Vec<Vec<IntVal>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
/// Groups of constraints that are instantiated using the given arguments.
pub struct Group<Identifier = String> {
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
	/// List of constraints
	pub constraints: Vec<Constraint<Identifier>>,
	/// List of arguments to instantiate the constraints
	pub args: Vec<Vec<Exp<Identifier>>>,
}

/// Constraint that enforces that the Boolean Expression [`Self::function`] must
/// be satisfied.
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Intension<Identifier> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// Boolean expression to be satisfied
	#[serde(alias = "$text")]
	pub function: BoolExp<Identifier>,
}

/// Constraint where the expressions in [`Self::list`] depict the amount of an
/// item chosen. The constraint enforces that the sum of the [`Self::weights`]
/// abides by the first [`Self::condition`] and the sum of the [`Self::profits`]
/// abides by the second [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Knapsack<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions that depict the amount of an item chosen
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// List of weights of the items
	#[serde(
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub weights: Vec<IntVal>,
	/// List of profits of the items
	#[serde(
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub profits: Vec<IntVal>,
	/// The first `Condition` element is related to weights whereas the second
	/// [`Condition`] element is related to profits.
	pub condition: [Condition<Identifier>; 2],
}

/// Constraint that enforces that the maximum value taken by the expression in
/// [`Self::list`] abides by the given [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Maximum<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions considered
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// Condition to be enforced on the maximum value
	pub condition: Condition<Identifier>,
}

/// Constraint that enforces that the values of the [`Self::list`] follow a
/// valid path according to the [`Self::transitions`] that form an Multi-valued
/// Decision Diagram (MDD).
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Mdd<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// List of transitions that form the Multi-valued Decision Diagram (MDD)
	#[serde(
		deserialize_with = "Transition::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub transitions: Vec<Transition<Identifier>>,
}

// #[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
// #[serde(
// 	bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"),
// 	rename_all = "camelCase"
// )]
// #[serde(tag = "type")]
// pub enum MetaConstraint<Identifier = String> {
// 	Group(Group<Identifier>),
// }

/// Constraint that enforces that the minimum value taken by the expression in
/// [`Self::list`] abides by the given [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Minimum<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions considered
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// Condition to be enforced on the minimum value
	pub condition: Condition<Identifier>,
}

/// Cosntraint that enforces a [`Self::condition`] on the number of different
/// values taken by the expressions in [`Self::list`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct NValues<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions considered
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// Values that are not counted
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub except: Vec<IntVal>,
	/// Condition to be enforced on the number of different values
	pub condition: Condition<Identifier>,
}

// TODO: k-dimensional no-overlap constraint
/// Constraint that enforces that the tasks defined by the [`Self::origins`] and
/// [`Self::lengths`] do not overlap.
///
/// When [`Self::zero_ignored`] field is set to `false`, it indicates that
/// zero-length tasks cannot be packed anywhere (cannot overlap with other
/// tasks).
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct NoOverlap<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	#[serde(
		default = "bool_true",
		skip_serializing_if = "is_true",
		rename = "@zeroIgnored"
	)]
	/// Indicates whether zero-length tasks can be placed anywhere
	pub zero_ignored: bool,
	/// List of starting points of the tasks
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub origins: Vec<IntExp<Identifier>>,
	/// List of lengths of the tasks
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub lengths: Vec<IntExp<Identifier>>,
}

/// List of expressions where the index is considered to start at
/// [`Self::start_index`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct OffsetList<Identifier> {
	/// List of expressions
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// Index of the first element in the list
	#[serde(rename = "@startIndex", default, skip_serializing_if = "is_default")]
	pub start_index: IntVal,
}

/// Operator used as part of the [`Condition`] struct or a constraint.
#[derive(Clone, Debug, PartialEq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Operator {
	/// Less than
	Lt,
	/// Less than or equal
	Le,
	/// Equal
	Eq,
	/// Greater than or equal
	Ge,
	/// Greater than
	Gt,
	/// Not equal
	Ne,
	/// Element of
	In,
}

/// Constraint that enforces that values of the expressions in [`Self::list`]
/// are ordered according to the [`Self::operator`].
///
/// The [`Self::lengths`] field indicates the minimum distances between any two
/// successive variables of [`Self::list`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Ordered<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// Minimum distances between any two successive variables
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub lengths: Vec<IntExp<Identifier>>,
	/// The operator used to order the expressions
	///
	/// The operator must be either [`Operator::Lt`], [`Operator::Le`],
	/// [`Operator::Ge`], or [`Operator::Gt`].
	pub operator: Operator,
}

/// Cosntraint that enforces that first occurence of each values of the
/// expressions in [`Self::list`] occur in the same order as the values in
/// [`Self::values`].
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Precedence<Identifier = String> {
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
	/// List of expressions considered
	pub list: Vec<IntExp<Identifier>>,
	/// Ordered values considered
	pub values: Vec<IntVal>,
	/// Whether the expressions must take one of the values in [`Self::values`]
	pub covered: bool,
}

/// Constraint that enforces that the values of the expressions in
/// [`Self::list`] follow a valid sequence of [`Self::transitions`], starting
/// from tje [`Self::start`] state and ending at the [`Self::finish`] state.
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Regular<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// List of transitions between states
	#[serde(
		deserialize_with = "Transition::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub transitions: Vec<Transition<Identifier>>,
	/// Starting state
	#[serde(deserialize_with = "from_str", serialize_with = "as_str")]
	pub start: Identifier,
	/// Final state
	#[serde(
		rename = "final",
		deserialize_with = "from_str",
		serialize_with = "as_str"
	)]
	pub finish: Identifier,
}

/// Constraint that enforces that the sum of the values of the expressions in
/// [`Self::list`], optionally multiplied by [`Self::coeffs`], abides by the
/// [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Sum<Identifier = String> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Identifier>>,
	/// Coefficient for each expression
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub coeffs: Vec<IntVal>,
	/// Condition to be enforced
	pub condition: Condition<Identifier>,
}

/// Transition between two state for the regular and MDD constraints.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Transition<Identifier> {
	/// Identifier of the source state
	pub from: Identifier,
	/// Value to be taken by the expression
	pub val: IntVal,
	/// Identifier of the destination state
	pub to: Identifier,
}

/// Function returning `true`
fn bool_true() -> bool {
	true
}

/// Deserialize a list of integer tuples visiting a string
fn deserialize_int_tuples<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<Vec<Vec<IntVal>>, D::Error> {
	/// Visitor to parse a list of integer tuples
	struct V;
	impl Visitor<'_> for V {
		type Value = Vec<Vec<IntVal>>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("an integer")
		}

		fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let v = v.trim();
			let (_, v) = all_consuming(sequence(tuple(int)))
				.parse(v)
				.map_err(|_| E::custom(format!("invalid integer `{v}'")))?;
			Ok(v)
		}
	}
	deserializer.deserialize_str(V)
}

/// Whether the value is the default value
fn is_default<T: Default + PartialEq>(val: &T) -> bool {
	val == &T::default()
}

/// Whether the value is `false`
fn is_false(x: &bool) -> bool {
	!x
}

/// Whether the value is `true`
fn is_true(x: &bool) -> bool {
	*x
}

/// Serialize a list of integer tuples as a string
fn serialize_int_tuples<S: Serializer>(
	vals: &[Vec<IntVal>],
	serializer: S,
) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(
		&vals
			.iter()
			.map(|e| {
				format!(
					"({})",
					e.iter()
						.map(|e| format!("{}", e))
						.collect::<Vec<_>>()
						.join(",")
				)
			})
			.collect::<Vec<_>>()
			.join(""),
	)
}

// Note: flatten of MetaInfo does not seem to work for Block
// (https://github.com/tafia/quick-xml/issues/326)
impl<'de, Identifier: FromStr> Deserialize<'de> for Block<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Deserialize a <block> element
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: FromStr"))]
		/// A set of constraints that is linked together semantically
		struct Block<Identifier> {
			/// Name assigned to the element
			#[serde(default, rename = "@id", deserialize_with = "crate::deserialize_ident")]
			pub identifier: Option<Identifier>,
			/// Comment from the user about the element
			#[serde(default, rename = "@note")]
			pub note: Option<String>,
			#[serde(default, rename = "@class")]
			/// Optional class designation for the contained constraints
			pub class: Vec<String>,
			#[serde(default, rename = "$value")]
			/// List of constraints
			pub constraints: Vec<MetaConstraint<Identifier>>,
		}
		let c = Block::deserialize(deserializer)?;
		let class: Result<_, _> = c
			.class
			.into_iter()
			.map(|v| {
				FromStr::from_str(&v)
					.map_err(|_| de::Error::custom("unable to create identifier from string"))
			})
			.collect();

		Ok(Self {
			info: MetaInfo {
				identifier: c.identifier,
				note: c.note,
			},
			class: class?,
			constraints: c.constraints,
		})
	}
}

impl<'de, Identifier: FromStr> Deserialize<'de> for Cardinality<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Helper struct to deserialize the <values> element of the cardinality constraint
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: FromStr"))]
		struct Values<Identifier> {
			/// closed attribute
			#[serde(default, rename = "@closed")]
			closed: Option<bool>,
			/// content of the <values> element
			#[serde(rename = "$text", deserialize_with = "IntExp::parse_vec")]
			list: Vec<IntExp<Identifier>>,
		}
		/// Helper struct to deserialize the <cardinality> element of the cardinality constraint
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: FromStr"))]
		struct Cardinality<Identifier = String> {
			/// Metadata for the constraint
			#[serde(flatten)]
			info: MetaInfo<Identifier>,
			/// <list> element
			#[serde(deserialize_with = "IntExp::parse_vec")]
			list: Vec<IntExp<Identifier>>,
			/// <values> element
			values: Values<Identifier>,
			/// <occurs> element
			#[serde(deserialize_with = "Exp::parse_vec")]
			occurs: Vec<Exp<Identifier>>,
		}
		let x = Cardinality::deserialize(deserializer)?;
		Ok(Self {
			info: x.info,
			list: x.list,
			values: x.values.list,
			closed: x.values.closed.unwrap_or(false),
			occurs: x.occurs,
		})
	}
}

impl<Identifier: Display> Serialize for Cardinality<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		/// Serialize a <values> element
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display"))]
		struct Values<'a, Identifier> {
			/// closed attribute
			#[serde(rename = "@closed", skip_serializing_if = "is_false")]
			closed: bool,
			/// content of the <values> element
			#[serde(rename = "$text", serialize_with = "serialize_list")]
			list: &'a Vec<IntExp<Identifier>>,
		}
		/// Serialize a <cardinality> element
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display"))]
		struct Cardinality<'a, Identifier = String> {
			/// meta information
			#[serde(flatten)]
			info: &'a MetaInfo<Identifier>,
			/// <list> element
			#[serde(serialize_with = "serialize_list")]
			list: &'a Vec<IntExp<Identifier>>,
			/// <values> element
			values: Values<'a, Identifier>,
			/// <occurs> element
			#[serde(serialize_with = "serialize_list")]
			occurs: &'a Vec<Exp<Identifier>>,
		}
		let x = Cardinality {
			info: &self.info,
			list: &self.list,
			values: Values {
				closed: self.closed,
				list: &self.values,
			},
			occurs: &self.occurs,
		};
		x.serialize(serializer)
	}
}

impl<'de, Identifier: FromStr> Deserialize<'de> for Channel<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Channel<Identifier>, D::Error> {
		/// Deserialize a <channel> element
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "I: FromStr"))]
		struct Channel<'a, I: FromStr> {
			/// meta information
			#[serde(flatten)]
			info: MetaInfo<I>,
			/// <list> element(s)
			list: Vec<Cow<'a, str>>,
			/// <value> element
			#[serde(default)]
			value: Option<IntExp<I>>,
		}
		let c = Channel::deserialize(deserializer)?;
		if c.list.is_empty() {
			return Err(de::Error::missing_field("list"));
		}
		let (_, list) = all_consuming(whitespace_seperated(IntExp::parse))
			.parse(c.list[0].trim())
			.map_err(|_| {
				de::Error::custom(format!(
					"invalid integer expressions `{}'",
					c.list[0].trim()
				))
			})?;
		let inverse_list = if let Some(inverse_list) = c.list.get(1) {
			let inverse_list = inverse_list.trim();
			all_consuming(whitespace_seperated(IntExp::parse))
				.parse(inverse_list)
				.map_err(|_| {
					de::Error::custom(format!("invalid integer expressions `{inverse_list}'"))
				})?
				.1
		} else {
			Vec::new()
		};

		Ok(Self {
			info: c.info,
			list,
			inverse_list,
			value: c.value,
		})
	}
}

impl<Identifier: Display> Serialize for Channel<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		/// Serialize a <channel> element
		#[derive(Serialize)]
		#[serde(bound(serialize = "I: Display"))]
		struct Channel<'a, I: Display> {
			/// meta information
			#[serde(flatten)]
			info: &'a MetaInfo<I>,
			/// <list> element(s)
			list: Vec<String>,
			/// <value> element
			#[serde(skip_serializing_if = "Option::is_none")]
			value: &'a Option<IntExp<I>>,
		}

		let p = |i: &Vec<IntExp<Identifier>>| -> String {
			i.iter()
				.map(|e| format!("{}", e))
				.collect::<Vec<_>>()
				.join(" ")
		};

		let mut c = Channel {
			info: &self.info,
			list: vec![p(&self.list)],
			value: &self.value,
		};
		if !self.inverse_list.is_empty() {
			c.list.push(p(&self.inverse_list))
		}
		c.serialize(serializer)
	}
}

impl<'de, Identifier: FromStr> Deserialize<'de> for Circuit<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Deserializes a <circuit> element
		#[derive(Deserialize)]
		#[serde(bound = "Identifier: FromStr")]
		struct Circuit<Identifier> {
			/// meta information
			#[serde(flatten)]
			info: MetaInfo<Identifier>,
			/// textual content of the element
			#[serde(default, deserialize_with = "IntExp::parse_vec", alias = "$text")]
			simple: Vec<IntExp<Identifier>>,
			/// <list> element
			#[serde(default)]
			list: OffsetList<Identifier>,
			/// <size> element
			#[serde(default)]
			size: Option<IntExp<Identifier>>,
		}
		let mut x = Circuit::deserialize(deserializer)?;
		if !x.simple.is_empty() {
			x.list = OffsetList {
				list: x.simple,
				start_index: 0,
			};
		}
		Ok(Self {
			info: x.info,
			list: x.list,
			size: x.size,
		})
	}
}

impl<'de, Identifier: FromStr> Deserialize<'de> for Condition<Identifier> {
	fn deserialize<D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<Condition<Identifier>, D::Error> {
		/// Visitor for parsing a condition.
		struct V<X>(PhantomData<X>);
		impl<X: FromStr> Visitor<'_> for V<X> {
			type Value = Condition<X>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a condition")
			}

			fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
				let v = v.trim();
				let mut parser = delimited(
					char('('),
					separated_pair(Operator::parse, char(','), Exp::parse),
					char(')'),
				);
				let (_, (operator, operand)) = parser
					.parse(v)
					.map_err(|e| E::custom(format!("invalid condition {e:?}")))?;
				Ok(Condition { operator, operand })
			}
		}
		deserializer.deserialize_str(V(PhantomData::<Identifier>))
	}
}

impl<Identifier: Display> Display for Condition<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({},{})", self.operator, self.operand)
	}
}

impl<Identifier: Display> Serialize for Condition<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

impl<Identifier> TryFrom<TemplateCapture<Identifier>> for Constraint<Identifier> {
	type Error = ();

	fn try_from(value: TemplateCapture<Identifier>) -> Result<Self, Self::Error> {
		match value {
			TemplateCapture::AllDifferent(all_different) => {
				Ok(Constraint::AllDifferent(all_different))
			}
			TemplateCapture::AllEqual(all_equal) => Ok(Constraint::AllEqual(all_equal)),
			TemplateCapture::BinPacking(bin_packing) => Ok(Constraint::BinPacking(bin_packing)),
			TemplateCapture::Cardinality(cardinality) => Ok(Constraint::Cardinality(cardinality)),
			TemplateCapture::Channel(channel) => Ok(Constraint::Channel(channel)),
			TemplateCapture::Circuit(circuit) => Ok(Constraint::Circuit(circuit)),
			TemplateCapture::Count(count) => Ok(Constraint::Count(count)),
			TemplateCapture::Cumulative(cumulative) => Ok(Constraint::Cumulative(cumulative)),
			TemplateCapture::Element(element) => Ok(Constraint::Element(element)),
			TemplateCapture::Extension(extension) => Ok(Constraint::Extension(extension)),
			TemplateCapture::Instantiation(instantiation) => {
				Ok(Constraint::Instantiation(instantiation))
			}
			TemplateCapture::Intension(intension) => Ok(Constraint::Intension(intension)),
			TemplateCapture::Knapsack(knapsack) => Ok(Constraint::Knapsack(knapsack)),
			TemplateCapture::Maximum(maximum) => Ok(Constraint::Maximum(maximum)),
			TemplateCapture::Mdd(mdd) => Ok(Constraint::Mdd(mdd)),
			TemplateCapture::Minimum(minimum) => Ok(Constraint::Minimum(minimum)),
			TemplateCapture::NValues(nvalues) => Ok(Constraint::NValues(nvalues)),
			TemplateCapture::NoOverlap(no_overlap) => Ok(Constraint::NoOverlap(no_overlap)),
			TemplateCapture::Ordered(ordered) => Ok(Constraint::Ordered(ordered)),
			TemplateCapture::Precedence(precedence) => Ok(Constraint::Precedence(precedence)),
			TemplateCapture::Regular(regular) => Ok(Constraint::Regular(regular)),
			TemplateCapture::Sum(sum) => Ok(Constraint::Sum(sum)),
			TemplateCapture::Args(_) => Err(()),
		}
	}
}

// Note: flatten of MetaInfo does not seem to work here
impl<'de, Identifier: FromStr> Deserialize<'de> for Group<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: FromStr"))]
		struct Group<Identifier = String> {
			/// Name assigned to the element
			#[serde(
				rename = "@id",
				default,
				skip_serializing_if = "Option::is_none",
				deserialize_with = "crate::deserialize_ident",
				serialize_with = "serialize_ident"
			)]
			pub identifier: Option<Identifier>,
			/// Comment from the user about the element
			#[serde(rename = "@note", default, skip_serializing_if = "Option::is_none")]
			pub note: Option<String>,

			/// List of constraints
			#[serde(default, rename = "$value")]
			constraints: Vec<TemplateCapture<Identifier>>,
		}
		let grp: Group<Identifier> = Deserialize::deserialize(deserializer)?;
		let mut args = Vec::new();
		let constraints = grp
			.constraints
			.into_iter()
			.filter_map(|c| match c {
				TemplateCapture::Args(x) => {
					args.push(x.elements);
					None
				}
				_ => Some(c.try_into().unwrap()),
			})
			.collect();
		Ok(Self {
			info: MetaInfo {
				identifier: grp.identifier,
				note: grp.note,
			},
			constraints,
			args,
		})
	}
}

// Note: flatten of MetaInfo does not seem to work here
// (https://github.com/tafia/quick-xml/issues/761)
impl<Identifier: Display> Serialize for Group<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display"))]
		/// Helper structure used to parse a list of expressions.
		struct ExpList<'a, Identifier> {
			#[serde(rename = "$text", serialize_with = "serialize_list")]
			pub(crate) elements: &'a Vec<Exp<Identifier>>,
		}
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display"), rename_all = "camelCase")]
		enum ExpListE<'a, Identifier> {
			Args(ExpList<'a, Identifier>),
		}
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display"))]
		/// Helper struct to serialize the instantiation element
		struct Group<'a, Identifier = String> {
			/// Optional metadata for the constraint
			#[serde(flatten)]
			info: &'a MetaInfo<Identifier>,
			/// List of constraints
			#[serde(rename = "$value")]
			constraints: &'a Vec<Constraint<Identifier>>,
			/// Arguments to instantiate the constraints
			#[serde(rename = "$value")]
			args: Vec<ExpListE<'a, Identifier>>,
		}
		Group {
			info: &self.info,
			constraints: &self.constraints,
			args: self
				.args
				.iter()
				.map(|v| ExpListE::Args(ExpList { elements: v }))
				.collect(),
		}
		.serialize(serializer)
	}
}

impl<'de, Identifier: FromStr> Deserialize<'de> for MetaConstraint<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let con: CaptureConstraint<Identifier> = Deserialize::deserialize(deserializer)?;
		Ok(con.into())
	}
}

impl<Identifier> From<CaptureConstraint<Identifier>> for MetaConstraint<Identifier> {
	fn from(value: CaptureConstraint<Identifier>) -> Self {
		match value {
			CaptureConstraint::AllDifferent(all_different) => {
				MetaConstraint::Constraint(Constraint::AllDifferent(all_different))
			}
			CaptureConstraint::AllEqual(all_equal) => {
				MetaConstraint::Constraint(Constraint::AllEqual(all_equal))
			}
			CaptureConstraint::BinPacking(bin_packing) => {
				MetaConstraint::Constraint(Constraint::BinPacking(bin_packing))
			}
			CaptureConstraint::Cardinality(cardinality) => {
				MetaConstraint::Constraint(Constraint::Cardinality(cardinality))
			}
			CaptureConstraint::Channel(channel) => {
				MetaConstraint::Constraint(Constraint::Channel(channel))
			}
			CaptureConstraint::Circuit(circuit) => {
				MetaConstraint::Constraint(Constraint::Circuit(circuit))
			}
			CaptureConstraint::Count(count) => MetaConstraint::Constraint(Constraint::Count(count)),
			CaptureConstraint::Cumulative(cumulative) => {
				MetaConstraint::Constraint(Constraint::Cumulative(cumulative))
			}
			CaptureConstraint::Element(element) => {
				MetaConstraint::Constraint(Constraint::Element(element))
			}
			CaptureConstraint::Extension(extension) => {
				MetaConstraint::Constraint(Constraint::Extension(extension))
			}
			CaptureConstraint::Instantiation(instantiation) => {
				MetaConstraint::Constraint(Constraint::Instantiation(instantiation))
			}
			CaptureConstraint::Intension(intension) => {
				MetaConstraint::Constraint(Constraint::Intension(intension))
			}
			CaptureConstraint::Knapsack(knapsack) => {
				MetaConstraint::Constraint(Constraint::Knapsack(knapsack))
			}
			CaptureConstraint::Maximum(maximum) => {
				MetaConstraint::Constraint(Constraint::Maximum(maximum))
			}
			CaptureConstraint::Mdd(mdd) => MetaConstraint::Constraint(Constraint::Mdd(mdd)),
			CaptureConstraint::Minimum(minimum) => {
				MetaConstraint::Constraint(Constraint::Minimum(minimum))
			}
			CaptureConstraint::NValues(nvalues) => {
				MetaConstraint::Constraint(Constraint::NValues(nvalues))
			}
			CaptureConstraint::NoOverlap(no_overlap) => {
				MetaConstraint::Constraint(Constraint::NoOverlap(no_overlap))
			}
			CaptureConstraint::Ordered(ordered) => {
				MetaConstraint::Constraint(Constraint::Ordered(ordered))
			}
			CaptureConstraint::Precedence(precedence) => {
				MetaConstraint::Constraint(Constraint::Precedence(precedence))
			}
			CaptureConstraint::Regular(regular) => {
				MetaConstraint::Constraint(Constraint::Regular(regular))
			}
			CaptureConstraint::Sum(sum) => MetaConstraint::Constraint(Constraint::Sum(sum)),
			CaptureConstraint::Group(group) => MetaConstraint::Group(group),
			CaptureConstraint::Block(block) => MetaConstraint::Block(block),
		}
	}
}

impl<Identifier: Display> Serialize for MetaConstraint<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display"), rename_all = "camelCase")]
		enum OutputConstraint<'a, Identifier> {
			/// [`AllDifferent`] constraint
			AllDifferent(&'a AllDifferent<Identifier>),
			/// [`AllEqual`] constraint
			AllEqual(&'a AllEqual<Identifier>),
			/// [`BinPacking`] constraint
			BinPacking(&'a BinPacking<Identifier>),
			/// [`Cardinality`] constraint
			Cardinality(&'a Cardinality<Identifier>),
			/// [`Channel`] constraint
			Channel(&'a Channel<Identifier>),
			/// [`Circuit`] constraint
			Circuit(&'a Circuit<Identifier>),
			/// [`Count`] constraint
			Count(&'a Count<Identifier>),
			/// [`Cumulative`] constraint
			Cumulative(&'a Cumulative<Identifier>),
			/// [`Element`] constraint
			Element(&'a Element<Identifier>),
			/// [`Extension`] constraint
			Extension(&'a Extension<Identifier>),
			/// [`Instantiation`] constraint
			Instantiation(&'a Instantiation<Identifier>),
			/// [`Intension`] constraint
			Intension(&'a Intension<Identifier>),
			/// [`Knapsack`] constraint
			Knapsack(&'a Knapsack<Identifier>),
			/// [`Maximum`] constraint
			Maximum(&'a Maximum<Identifier>),
			/// [`Mdd`] constraint
			Mdd(&'a Mdd<Identifier>),
			/// [`Minimum`] constraint
			Minimum(&'a Minimum<Identifier>),
			/// [`NValues`] constraint
			NValues(&'a NValues<Identifier>),
			/// [`NoOverlap`] constraint
			NoOverlap(&'a NoOverlap<Identifier>),
			/// [`Ordered`] constraint
			Ordered(&'a Ordered<Identifier>),
			/// [`Precedence`] constraint
			Precedence(&'a Precedence<Identifier>),
			/// [`Regular`] constraint
			Regular(&'a Regular<Identifier>),
			/// [`Sum`] constraint
			Sum(&'a Sum<Identifier>),
			/// Constraint [`Group`], that can serve as a template
			Group(&'a Group<Identifier>),
			/// Constraint [`Group`], that can serve as a template
			Block(&'a Block<Identifier>),
		}

		let c = match self {
			MetaConstraint::Group(group) => OutputConstraint::Group(group),
			MetaConstraint::Block(block) => OutputConstraint::Block(block),
			MetaConstraint::Constraint(con) => match con {
				Constraint::AllDifferent(all_different) => {
					OutputConstraint::AllDifferent(all_different)
				}
				Constraint::AllEqual(all_equal) => OutputConstraint::AllEqual(all_equal),
				Constraint::BinPacking(bin_packing) => OutputConstraint::BinPacking(bin_packing),
				Constraint::Cardinality(cardinality) => OutputConstraint::Cardinality(cardinality),
				Constraint::Channel(channel) => OutputConstraint::Channel(channel),
				Constraint::Circuit(circuit) => OutputConstraint::Circuit(circuit),
				Constraint::Count(count) => OutputConstraint::Count(count),
				Constraint::Cumulative(cumulative) => OutputConstraint::Cumulative(cumulative),
				Constraint::Element(element) => OutputConstraint::Element(element),
				Constraint::Extension(extension) => OutputConstraint::Extension(extension),
				Constraint::Instantiation(instantiation) => {
					OutputConstraint::Instantiation(instantiation)
				}
				Constraint::Intension(intension) => OutputConstraint::Intension(intension),
				Constraint::Knapsack(knapsack) => OutputConstraint::Knapsack(knapsack),
				Constraint::Maximum(maximum) => OutputConstraint::Maximum(maximum),
				Constraint::Mdd(mdd) => OutputConstraint::Mdd(mdd),
				Constraint::Minimum(minimum) => OutputConstraint::Minimum(minimum),
				Constraint::NValues(nvalues) => OutputConstraint::NValues(nvalues),
				Constraint::NoOverlap(no_overlap) => OutputConstraint::NoOverlap(no_overlap),
				Constraint::Ordered(ordered) => OutputConstraint::Ordered(ordered),
				Constraint::Precedence(precedence) => OutputConstraint::Precedence(precedence),
				Constraint::Regular(regular) => OutputConstraint::Regular(regular),
				Constraint::Sum(sum) => OutputConstraint::Sum(sum),
			},
		};
		Serialize::serialize(&c, serializer)
	}
}

impl<Identifier> Default for OffsetList<Identifier> {
	fn default() -> Self {
		Self {
			list: Vec::new(),
			start_index: IntVal::default(),
		}
	}
}

impl Operator {
	fn parse(input: &str) -> IResult<&str, Self> {
		map(
			alt((
				tag("lt"),
				tag("le"),
				tag("eq"),
				tag("ge"),
				tag("gt"),
				tag("ne"),
				tag("in"),
			)),
			|op| match op {
				"lt" => Self::Lt,
				"le" => Self::Le,
				"eq" => Self::Eq,
				"ge" => Self::Ge,
				"gt" => Self::Gt,
				"ne" => Self::Ne,
				"in" => Self::In,
				_ => unreachable!(),
			},
		)
		.parse(input)
	}
}

impl<'de> Deserialize<'de> for Operator {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Operator, D::Error> {
		/// Visitor for parsing a condition.
		struct V;
		impl Visitor<'_> for V {
			type Value = Operator;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("an operator")
			}

			fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
				let v = v.trim();
				Ok(all_consuming(Operator::parse)
					.parse(v)
					.map_err(|e| E::custom(format!("invalid condition {e:?}")))?
					.1)
			}
		}
		deserializer.deserialize_str(V)
	}
}

impl Display for Operator {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Operator::Lt => write!(f, "lt"),
			Operator::Le => write!(f, "le"),
			Operator::Eq => write!(f, "eq"),
			Operator::Ge => write!(f, "ge"),
			Operator::Gt => write!(f, "gt"),
			Operator::Ne => write!(f, "ne"),
			Operator::In => write!(f, "in"),
		}
	}
}

impl<'de, Identifier: FromStr> Deserialize<'de> for Precedence<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Deserialize the <values> element
		#[derive(Default, Deserialize)]
		struct Values {
			/// covered attribute
			#[serde(default, rename = "@covered")]
			covered: Option<bool>,
			/// content of the element
			#[serde(rename = "$text", deserialize_with = "deserialize_int_vals")]
			list: Vec<IntVal>,
		}
		/// Deserialize the <precedence> element
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
		struct Precedence<Identifier = String> {
			/// Meta information
			#[serde(flatten)]
			info: MetaInfo<Identifier>,
			/// <list> element or content of the element
			#[serde(
				alias = "$text",
				deserialize_with = "IntExp::parse_vec",
				serialize_with = "serialize_list"
			)]
			list: Vec<IntExp<Identifier>>,
			/// <values> element
			#[serde(default)]
			values: Values,
		}
		let x = Precedence::deserialize(deserializer)?;
		Ok(Self {
			info: x.info,
			list: x.list,
			values: x.values.list,
			covered: x.values.covered.unwrap_or(false),
		})
	}
}

impl<Identifier: Display> Serialize for Precedence<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		/// Serialize the <values> element
		#[derive(Serialize)]
		struct Values<'a> {
			/// covered attribute
			#[serde(rename = "@covered", skip_serializing_if = "is_false")]
			covered: bool,
			/// content of the <values> element
			#[serde(rename = "$text", serialize_with = "serialize_list")]
			list: &'a Vec<IntVal>,
		}
		impl Values<'_> {
			/// Whether serializing the <values> element can be skipped
			fn skip(&self) -> bool {
				!self.covered && self.list.is_empty()
			}
		}
		/// Serialize the <precedence> element
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display"))]
		struct Precedence<'a, Identifier = String> {
			/// Optional meta information
			#[serde(flatten)]
			info: &'a MetaInfo<Identifier>,
			/// <list> element or string content
			#[serde(alias = "$text", serialize_with = "serialize_list")]
			list: &'a Vec<IntExp<Identifier>>,
			/// <values> element
			#[serde(skip_serializing_if = "Values::skip")]
			values: Values<'a>,
		}
		let x = Precedence {
			info: &self.info,
			list: &self.list,
			values: Values {
				covered: self.covered,
				list: &self.values,
			},
		};
		x.serialize(serializer)
	}
}

impl<Identifier> From<Constraint<Identifier>> for TemplateCapture<Identifier> {
	fn from(value: Constraint<Identifier>) -> Self {
		match value {
			Constraint::AllDifferent(all_different) => TemplateCapture::AllDifferent(all_different),
			Constraint::AllEqual(all_equal) => TemplateCapture::AllEqual(all_equal),
			Constraint::BinPacking(bin_packing) => TemplateCapture::BinPacking(bin_packing),
			Constraint::Cardinality(cardinality) => TemplateCapture::Cardinality(cardinality),
			Constraint::Channel(channel) => TemplateCapture::Channel(channel),
			Constraint::Circuit(circuit) => TemplateCapture::Circuit(circuit),
			Constraint::Count(count) => TemplateCapture::Count(count),
			Constraint::Cumulative(cumulative) => TemplateCapture::Cumulative(cumulative),
			Constraint::Element(element) => TemplateCapture::Element(element),
			Constraint::Extension(extension) => TemplateCapture::Extension(extension),
			Constraint::Instantiation(instantiation) => {
				TemplateCapture::Instantiation(instantiation)
			}
			Constraint::Intension(intension) => TemplateCapture::Intension(intension),
			Constraint::Knapsack(knapsack) => TemplateCapture::Knapsack(knapsack),
			Constraint::Maximum(maximum) => TemplateCapture::Maximum(maximum),
			Constraint::Mdd(mdd) => TemplateCapture::Mdd(mdd),
			Constraint::Minimum(minimum) => TemplateCapture::Minimum(minimum),
			Constraint::NValues(nvalues) => TemplateCapture::NValues(nvalues),
			Constraint::NoOverlap(no_overlap) => TemplateCapture::NoOverlap(no_overlap),
			Constraint::Ordered(ordered) => TemplateCapture::Ordered(ordered),
			Constraint::Precedence(precedence) => TemplateCapture::Precedence(precedence),
			Constraint::Regular(regular) => TemplateCapture::Regular(regular),
			Constraint::Sum(sum) => TemplateCapture::Sum(sum),
		}
	}
}

impl<Identifier: FromStr> Transition<Identifier> {
	/// Parse a list of transitions.
	fn parse_vec<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Self>, D::Error> {
		/// Visitor for parsing a list of transitions.
		struct V<X>(PhantomData<X>);
		impl<X: FromStr> Visitor<'_> for V<X> {
			type Value = Vec<Transition<X>>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a list of transitions")
			}

			fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
				let v = v.trim();
				let transition = map(
					(
						char('('),
						identifier,
						char(','),
						int,
						char(','),
						identifier,
						char(')'),
					),
					|(_, from, _, val, _, to, _)| Transition { from, val, to },
				);
				let (_, v) = all_consuming(sequence(transition))
					.parse(v)
					.map_err(|_| E::custom(format!("invalid transitions `{v}'")))?;
				Ok(v)
			}
		}
		let visitor = V::<Identifier>(PhantomData);
		deserializer.deserialize_str(visitor)
	}
}

impl<Identifier: Display> Display for Transition<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({},{},{})", self.from, self.val, self.to)
	}
}
