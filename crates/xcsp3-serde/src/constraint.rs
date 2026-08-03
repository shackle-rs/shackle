//! # Constraints
//!
//! This module contains the definition of the constraints that can be used in a
//! XCSP3 instance. Each constraint is represented by a struct that contains the
//! necessary information to represent the constraint in the XCSP3 format. The
//! enumerated type [`Constraint`] is used to represent any of constraint.

use std::{borrow::Cow, collections::HashMap, fmt::Display, hash::Hash, marker::PhantomData};

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
	error::UnrollError,
	expression::{
		identifier, int, range, sequence, tuple, whitespace_seperated, BoolExp, Exp, ExpList,
		IntExp,
	},
	from_string, serialize_list, Instantiation, IntVal, IntoVar, MetaInfo, Placeholder, SimpleRef,
	VarRef,
};

macro_rules! constraints_enum {
	($(#[$attr:meta])* $vis:vis $name:ident, $basic:meta, $meta:meta, $args:meta, $slide_list:meta) => {
		$(#[$attr])*
		$vis enum $name<Identifier = String, Var = VarRef<Identifier>> {
			#[cfg($basic)]
			/// [`AllDifferent`] constraint
			AllDifferent(AllDifferent<Identifier, Var>),
			#[cfg($basic)]
			/// [`AllEqual`] constraint
			AllEqual(AllEqual<Identifier, Var>),
			#[cfg($basic)]
			/// [`BinPacking`] constraint
			BinPacking(BinPacking<Identifier, Var>),
			#[cfg($basic)]
			/// [`Cardinality`] constraint
			Cardinality(Cardinality<Identifier, Var>),
			#[cfg($basic)]
			/// [`Channel`] constraint
			Channel(Channel<Identifier, Var>),
			#[cfg($basic)]
			/// [`Circuit`] constraint
			Circuit(Circuit<Identifier, Var>),
			#[cfg($basic)]
			/// [`Clause`] constraint
			Clause(Clause<Identifier, Var>),
			#[cfg($basic)]
			/// [`Count`] constraint
			Count(Count<Identifier, Var>),
			#[cfg($basic)]
			/// [`Cumulative`] constraint
			Cumulative(Cumulative<Identifier, Var>),
			#[cfg($basic)]
			/// [`Element`] constraint
			Element(Element<Identifier, Var>),
			#[cfg($basic)]
			/// [`Extension`] constraint
			Extension(Extension<Identifier, Var>),
			#[cfg($basic)]
			/// [`Instantiation`] constraint
			Instantiation(Instantiation<Identifier, Var>),
			#[cfg($basic)]
			/// [`Intension`] constraint
			Intension(Intension<Identifier, Var>),
			#[cfg($basic)]
			/// [`Knapsack`] constraint
			Knapsack(Knapsack<Identifier, Var>),
			#[cfg($basic)]
			/// [`Lex`] constraint
			Lex(Lex<Identifier, Var>),
			#[cfg($basic)]
			/// [`Maximum`] constraint
			Maximum(Maximum<Identifier, Var>),
			#[cfg($basic)]
			/// [`Mdd`] constraint
			Mdd(Mdd<Identifier, Var>),
			#[cfg($basic)]
			/// [`Minimum`] constraint
			Minimum(Minimum<Identifier, Var>),
			#[cfg($basic)]
			/// [`NValues`] constraint
			NValues(NValues<Identifier, Var>),
			#[cfg($basic)]
			/// [`NoOverlap`] constraint
			NoOverlap(NoOverlap<Identifier, Var>),
			#[cfg($basic)]
			/// [`Ordered`] constraint
			Ordered(Ordered<Identifier, Var>),
			#[cfg($basic)]
			/// [`Precedence`] constraint
			Precedence(Precedence<Identifier, Var>),
			#[cfg($basic)]
			/// [`Regular`] constraint
			Regular(Regular<Identifier, Var>),
			#[cfg($basic)]
			/// [`Sum`] constraint
			Sum(Sum<Identifier, Var>),
			#[cfg($meta)]
			/// Constraint [`Group`], that can serve as a template
			Group(Group<Identifier, Var>),
			#[cfg($meta)]
			/// Constraint [`Group`], that can serve as a template
			Block(Block<Identifier, Var>),
			#[cfg($meta)]
			/// Meta-constraint [`Slide`], that instantiates a template over
			/// successive sub-lists
			Slide(Slide<Identifier, Var>),
			#[cfg(not($basic))]
			/// Simple [`Constraint`] type
			Constraint(Constraint<Identifier, Var>),
			#[cfg($args)]
			/// Constraint [`Group`], that can serve as a template
			Args(ExpList<Var>),
			#[cfg($slide_list)]
			/// A `<list>` element of a [`Slide`] meta-constraint
			List(SlideList<Var>),
		}
	};
}

constraints_enum!(
	#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
	#[serde(
				rename_all = "camelCase",
				bound(deserialize = "Identifier: From<String>, Var: IntoVar", serialize = "Identifier: Display, Var: Display")
	)]
	/// Enumerated type to represent basic (instantiated) constraints
	pub Constraint,
	/* expand_basic = true */ all(),
	/* meta = false */ any(),
	/* template_args = false */ any(),
	/* slide_list = false */ any()
);
constraints_enum!(
	#[derive(Clone, Debug, PartialEq, Hash)]
	/// Enumerated type that contains meta-constraints, such as [`Group`] and
	/// [`Block`], in addition to standard [`Constraint`].
	pub MetaConstraint,
	/* expand_basic = false */ any(),
	/* meta = true */ all(),
	/* template_args = false */ any(),
	/* slide_list = false */ any()
);
constraints_enum!(
	#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
	#[serde(
		rename_all = "camelCase",
		bound(
			deserialize = "Identifier: From<String>, Var: IntoVar",
			serialize = "Identifier: Display, Var: Display"
		)
	)]
	/// Internal constraint enum used to capture [`MetaConstraint`].
	CaptureConstraint,
	/* expand_basic = true */ all(),
	/* meta = true */ all(),
	/* template_args = false */ any(),
	/* slide_list = false */ any()
);
constraints_enum!(
	#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
	#[serde(
		rename_all = "camelCase",
		bound(
			deserialize = "Identifier: From<String>, Var: IntoVar",
			serialize = "Identifier: Display, Var: Display"
		)
	)]
	/// Internal constraint enum used to capture basic constraints and <args> elements
	TemplateCapture,
	/* expand_basic = true */ all(),
	/* meta = true */ any(),
	/* template_args = false */ all(),
	/* slide_list = false */ any()
);
constraints_enum!(
	#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
	#[serde(
		rename_all = "camelCase",
		bound(
			deserialize = "Identifier: From<String>, Var: IntoVar",
			serialize = "Identifier: Display, Var: Display"
		)
	)]
	/// Internal constraint enum used to capture basic constraints and the <list>
	/// elements of a [`Slide`] meta-constraint
	SlideCapture,
	/* expand_basic = true */ all(),
	/* meta = true */ any(),
	/* template_args = false */ any(),
	/* slide_list = true */ all()
);

/// Constraint forcing a set of expressions to take distinct values
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct AllDifferent<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	#[serde(
		alias = "$text",
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	/// List of expressions that must take distinct values
	pub list: Vec<IntExp<Var>>,
	/// Matrix of expressions of which every row and every column must take
	/// distinct values
	///
	/// This is the `allDifferent-matrix` variant, which is used instead of
	/// [`Self::list`].
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_exp_tuples",
		serialize_with = "serialize_exp_tuples"
	)]
	pub matrix: Matrix<Var>,
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
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct AllEqual<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions that must take the same value
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
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
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct BinPacking<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions representing the bin in which each item is placed
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// List of expressions representing the size of each item
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub sizes: Vec<IntExp<Var>>,
	/// Condition that must be respected by the total size of the items in each bin
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub condition: Option<Condition<Var>>,
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	/// List of expressions representing the limit for the total size of the items
	/// in each bin
	pub limits: Vec<IntExp<Var>>,
	/// List of expressions representing the load of each bin
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub loads: Vec<IntExp<Var>>,
}

#[derive(Clone, Debug, PartialEq, Hash, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
/// A set of constraints that is linked together semantically
pub struct Block<Identifier = String, Var = VarRef<Identifier>> {
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
	pub constraints: Vec<MetaConstraint<Identifier, Var>>,
	#[serde(flatten)]
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
}

/// A two-dimensional list of expressions, as used by the "matrix" variants of
/// [`AllDifferent`] and [`Element`].
///
/// Note that when a matrix is given as a single array slice (e.g. `x[][]`), it
/// is stored as a single row containing that one expression. The row structure
/// is only recovered when the constraint is unrolled, since that is where the
/// dimensions of the array are known.
pub type Matrix<Var> = Vec<Vec<IntExp<Var>>>;

/// Constraint enforcing the amount of times certain values are taken by a set
/// of expressions.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Cardinality<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
	/// List of expressions of which the values are observed
	pub list: Vec<IntExp<Var>>,
	/// List of values that are observed
	pub values: Vec<IntExp<Var>>,
	/// Whether the expressions are allowed to take values not in the list of
	/// observed values
	pub closed: bool,
	/// List of expressions representing the number of times each value is taken
	pub occurs: Vec<Exp<Var>>,
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
pub struct Channel<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
	/// List of expressions that is being channelled
	pub list: Vec<IntExp<Var>>,
	/// Inverse list of expressions that is being channelled
	pub inverse_list: Vec<IntExp<Var>>,
	/// Expression representing the index of the only expression in [`Self::list`]
	/// that is allowed to take the value 1.
	pub value: Option<IntExp<Var>>,
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
#[serde(bound(serialize = "Identifier: Display, Var: Display"))]
pub struct Circuit<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions that must form a circuit
	pub list: OffsetList<Var>,
	/// Size of the circuit
	#[serde(skip_serializing_if = "Option::is_none")]
	pub size: Option<IntExp<Var>>,
}

/// Constraint that enforces that at least one of the literals in
/// [`Self::list`] takes the value `true`.
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Clause<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// Literals of the clause
	#[serde(
		alias = "$text",
		deserialize_with = "deserialize_literals",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<BoolExp<Var>>,
}

/// Condition to be enforced
///
/// This type is used as part of a larger constraint type
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Condition<Var> {
	/// Operator of the condition
	pub operator: Operator,
	/// Right-side operand of the condition
	pub operand: Exp<Var>,
}

/// Constraint that enforced that the number of times expressions in
/// [`Self::list`] take a value from [`Self::values`] abides by the given
/// [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Count<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions of which the values are observed
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// List of values that are counted
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub values: Vec<IntExp<Var>>,
	/// Condition to be enforced on the count
	pub condition: Condition<Var>,
}

/// Constraint that enforces that at each point in time, the cumulated height of
/// tasks that overlap that point, respects the given [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Cumulative<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of starting time-points of the tasks
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub origins: Vec<IntExp<Var>>,
	/// List of durations of the tasks
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub lengths: Vec<IntExp<Var>>,
	/// List of heights of the tasks
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub heights: Vec<IntExp<Var>>,
	/// Condition to be enforced on the cumulated height at each time point
	pub condition: Condition<Var>,
}

/// Constraint that enforces that the value of the expression at
/// [`Self::index`] abides by the given [`Self::condition`], or alternatively is
/// equal the expression [`Self::value`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Element<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// Indexed list of values
	#[serde(default, skip_serializing_if = "OffsetList::is_empty")]
	pub list: OffsetList<Var>,
	/// Indexed matrix of values
	///
	/// This is the `element-matrix` variant, which is used instead of
	/// [`Self::list`] and is indexed by two expressions.
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_exp_tuples",
		serialize_with = "serialize_exp_tuples"
	)]
	pub matrix: Matrix<Var>,
	/// Index of the value to be constrained
	///
	/// The `element-matrix` variant is indexed by a row and a column, and
	/// therefore has two index expressions.
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub index: Vec<IntExp<Var>>,
	/// Value to be assigned to the indexed expression
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub value: Option<IntExp<Var>>,
	/// Condition to be enforced on the indexed expression
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub condition: Option<Condition<Var>>,
}

/// Constraint that enforces that the expressions in [`Self::list`] either take
/// the values of one of the rows in [`Self::supports`], or alternatively do not
/// match any of the rows in [`Self::conflicts`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Extension<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// Combinations of values that the expressions are allowed to take
	///
	/// A [`None`] entry represents the star `*`, marking a position that is
	/// allowed to take any value (a "short", or "starred", tuple).
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_tuples",
		serialize_with = "serialize_int_tuples"
	)]
	pub supports: Vec<Vec<Option<IntVal>>>,
	/// Combinations of values that the expressions are not allowed to take
	///
	/// A [`None`] entry represents the star `*`, see [`Self::supports`].
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_tuples",
		serialize_with = "serialize_int_tuples"
	)]
	pub conflicts: Vec<Vec<Option<IntVal>>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
/// Groups of constraints that are instantiated using the given arguments.
pub struct Group<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
	/// List of constraints
	pub constraints: Vec<Constraint<Identifier, Var>>,
	/// List of arguments to instantiate the constraints
	pub args: Vec<Vec<Exp<Var>>>,
}

/// Constraint that enforces that the Boolean Expression [`Self::function`] must
/// be satisfied.
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Intension<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// Boolean expression to be satisfied
	#[serde(alias = "$text")]
	pub function: BoolExp<Var>,
}

/// Constraint where the expressions in [`Self::list`] depict the amount of an
/// item chosen. The constraint enforces that the sum of the [`Self::weights`]
/// abides by the first [`Self::condition`] and the sum of the [`Self::profits`]
/// abides by the second [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Knapsack<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions that depict the amount of an item chosen
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
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
	pub condition: [Condition<Var>; 2],
}

/// Constraint that enforces that the maximum value taken by the expression in
/// [`Self::list`] abides by the given [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Maximum<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions considered
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// Condition to be enforced on the maximum value
	pub condition: Condition<Var>,
}

/// Constraint that enforces that the values of the [`Self::list`] follow a
/// valid path according to the [`Self::transitions`] that form an Multi-valued
/// Decision Diagram (MDD).
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Mdd<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// List of transitions that form the Multi-valued Decision Diagram (MDD)
	#[serde(
		deserialize_with = "Transition::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub transitions: Vec<Transition<Identifier>>,
}

/// Constraint that enforces that the minimum value taken by the expression in
/// [`Self::list`] abides by the given [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Minimum<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions considered
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// Condition to be enforced on the minimum value
	pub condition: Condition<Var>,
}

/// Cosntraint that enforces a [`Self::condition`] on the number of different
/// values taken by the expressions in [`Self::list`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct NValues<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions considered
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// Values that are not counted
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub except: Vec<IntVal>,
	/// Condition to be enforced on the number of different values
	pub condition: Condition<Var>,
}

/// Constraint that enforces that the boxes defined by the [`Self::origins`] and
/// [`Self::lengths`] do not overlap.
///
/// Each entry of [`Self::origins`] and [`Self::lengths`] is one box, given as
/// its coordinate in each dimension. In the common one-dimensional case (where
/// boxes are tasks on a timeline) every entry holds a single expression.
///
/// When [`Self::zero_ignored`] field is set to `false`, it indicates that
/// zero-length tasks cannot be packed anywhere (cannot overlap with other
/// tasks).
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct NoOverlap<Identifier = String, Var = VarRef<Identifier>> {
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
	/// List of starting points of the boxes, one coordinate per dimension
	#[serde(
		deserialize_with = "deserialize_exp_tuples",
		serialize_with = "serialize_exp_tuples"
	)]
	pub origins: Vec<Vec<IntExp<Var>>>,
	/// List of lengths of the boxes, one length per dimension
	#[serde(
		deserialize_with = "deserialize_exp_tuples",
		serialize_with = "serialize_exp_tuples"
	)]
	pub lengths: Vec<Vec<IntExp<Var>>>,
}

/// List of expressions where the index is considered to start at
/// [`Self::start_index`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = " Var: IntoVar", serialize = "Var: Display"))]
pub struct OffsetList<Var> {
	/// List of expressions
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// Index of the first element in the list
	#[serde(rename = "@startIndex", default, skip_serializing_if = "is_default")]
	pub start_index: IntVal,
}

impl<Var> OffsetList<Var> {
	/// Whether the list contains no expressions
	fn is_empty(&self) -> bool {
		self.list.is_empty()
	}
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
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Ordered<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// Minimum distances between any two successive variables
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub lengths: Vec<IntExp<Var>>,
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
pub struct Precedence<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
	/// List of expressions considered
	pub list: Vec<IntExp<Var>>,
	/// Ordered values considered
	pub values: Vec<IntVal>,
	/// Whether the expressions must take one of the values in [`Self::values`]
	pub covered: bool,
}

/// Constraint that enforces that the values of the expressions in
/// [`Self::list`] follow a valid sequence of [`Self::transitions`], starting
/// from tje [`Self::start`] state and ending at the [`Self::finish`] state.
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Regular<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// List of transitions between states
	#[serde(
		deserialize_with = "Transition::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub transitions: Vec<Transition<Identifier>>,
	/// Starting state
	#[serde(deserialize_with = "from_string", serialize_with = "as_str")]
	pub start: Identifier,
	/// Final state
	#[serde(
		rename = "final",
		deserialize_with = "from_string",
		serialize_with = "as_str"
	)]
	pub finish: Identifier,
}

/// Constraint that enforces that lists of expressions are lexicographically
/// ordered according to [`Self::operator`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Lex<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// Lists that must be ordered with respect to each other
	#[serde(
		rename = "list",
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_exp_lists",
		serialize_with = "serialize_exp_lists"
	)]
	pub lists: Matrix<Var>,
	/// Matrix of which both the rows and the columns must be ordered
	///
	/// This is the `lex-matrix` variant, which is used instead of
	/// [`Self::lists`].
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_exp_tuples",
		serialize_with = "serialize_exp_tuples"
	)]
	pub matrix: Matrix<Var>,
	/// The operator used to order the lists
	///
	/// The operator must be either [`Operator::Lt`], [`Operator::Le`],
	/// [`Operator::Ge`], or [`Operator::Gt`].
	pub operator: Operator,
}

/// Meta-constraint that instantiates a constraint template over successive
/// sub-lists of one or more lists of expressions.
///
/// For each iteration, [`SlideList::collect`] expressions are taken from every
/// list, and together they instantiate the placeholders of
/// [`Self::constraint`]. Successive iterations start [`SlideList::offset`]
/// expressions further along each list.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Slide<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	pub info: MetaInfo<Identifier>,
	/// Whether the sliding wraps around the end of the list
	pub circular: bool,
	/// Lists of expressions over which the template is slid
	pub lists: Vec<SlideList<Var>>,
	/// Constraint template that is instantiated for every sub-list
	pub constraint: Box<Constraint<Identifier, Var>>,
}

/// A list of expressions over which a [`Slide`] meta-constraint is slid.
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Var: IntoVar", serialize = "Var: Display"))]
pub struct SlideList<Var> {
	/// Expressions contained in the list
	#[serde(
		rename = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// Distance between the starts of two successive sub-lists
	#[serde(
		rename = "@offset",
		default = "usize_one",
		skip_serializing_if = "is_one"
	)]
	pub offset: usize,
	/// Number of expressions taken from this list for each instantiation
	///
	/// When absent, this defaults to 1 if the [`Slide`] has multiple lists, and
	/// to the arity of the constraint template otherwise.
	#[serde(rename = "@collect", default, skip_serializing_if = "Option::is_none")]
	pub collect: Option<usize>,
}

/// Constraint that enforces that the sum of the values of the expressions in
/// [`Self::list`], optionally multiplied by [`Self::coeffs`], abides by the
/// [`Self::condition`].
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Sum<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// List of expressions to be constrained
	#[serde(
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// Coefficient for each expression
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub coeffs: Vec<IntVal>,
	/// Condition to be enforced
	pub condition: Condition<Var>,
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
///
/// Both the general syntax `(1,2)(3,*)` and the unary syntax `1 2 4 8..10` (used
/// when the table has arity one) are accepted. The star `*` marks a position
/// that can take any value, and is represented by [`None`].
fn deserialize_int_tuples<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<Vec<Vec<Option<IntVal>>>, D::Error> {
	/// Visitor to parse a list of integer tuples
	struct V;
	impl Visitor<'_> for V {
		type Value = Vec<Vec<Option<IntVal>>>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("an integer")
		}

		fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let v = v.trim();
			// Each item yields one or more tuples: a range in the unary syntax
			// stands for one tuple per value it contains.
			let (_, tuples) = all_consuming(sequence(alt((
				map(tuple(table_entry), |t| vec![t]),
				map(char('*'), |_| vec![vec![None]]),
				map(range, |r| r.map(|i| vec![Some(i)]).collect()),
			))))
			.parse(v)
			.map_err(|_| E::custom(format!("invalid integer `{v}'")))?;
			Ok(tuples.into_iter().flatten().collect())
		}
	}
	deserializer.deserialize_str(V)
}

/// Parser combinator for a single value within a table, where `*` denotes that
/// any value is allowed
fn table_entry(input: &str) -> IResult<&str, Option<IntVal>> {
	alt((map(char('*'), |_| None), map(int, Some))).parse(input)
}

/// Deserialize a list of literals visiting a string
///
/// In addition to the regular `not(x)` syntax, the older `@not@(x)` notation for
/// a negated literal is accepted.
fn deserialize_literals<'de, D: Deserializer<'de>, Var: IntoVar>(
	deserializer: D,
) -> Result<Vec<BoolExp<Var>>, D::Error> {
	/// Visitor to parse a list of literals
	struct V<X>(PhantomData<X>);
	impl<X: IntoVar> Visitor<'_> for V<X> {
		type Value = Vec<BoolExp<X>>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("a list of literals")
		}

		fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let v = v.trim();
			let (_, lits) = all_consuming(whitespace_seperated(alt((
				map(
					delimited(tag("@not@("), VarRef::parse, char(')')),
					|v: VarRef<String>| BoolExp::Not(Box::new(BoolExp::Var(X::into_var(v)))),
				),
				BoolExp::parse,
			))))
			.parse(v)
			.map_err(|_| E::custom(format!("invalid literals `{v}'")))?;
			Ok(lits)
		}
	}
	deserializer.deserialize_str(V::<Var>(PhantomData))
}

/// Deserialize repeated `<list>` elements as the rows of a matrix
fn deserialize_exp_lists<'de, D: Deserializer<'de>, Var: IntoVar>(
	deserializer: D,
) -> Result<Matrix<Var>, D::Error> {
	/// A single `<list>` element
	#[derive(Deserialize)]
	#[serde(bound(deserialize = "Var: IntoVar"))]
	struct ListElement<Var> {
		/// Expressions contained in the list
		#[serde(rename = "$text", deserialize_with = "IntExp::parse_vec")]
		list: Vec<IntExp<Var>>,
	}
	Ok(Vec::<ListElement<Var>>::deserialize(deserializer)?
		.into_iter()
		.map(|l| l.list)
		.collect())
}

/// Serialize the rows of a matrix as repeated `<list>` elements
fn serialize_exp_lists<S: Serializer, Var: Display>(
	lists: &Matrix<Var>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	/// A single `<list>` element
	#[derive(Serialize)]
	#[serde(bound(serialize = "Var: Display"))]
	struct ListElement<'a, Var> {
		/// Expressions contained in the list
		#[serde(rename = "$text", serialize_with = "serialize_list")]
		list: &'a Vec<IntExp<Var>>,
	}
	lists
		.iter()
		.map(|list| ListElement { list })
		.collect::<Vec<_>>()
		.serialize(serializer)
}

/// Deserialize a list of integer expression tuples visiting a string
///
/// Both the k-dimensional syntax `(x0,y0)(x1,y1)` and the one-dimensional
/// syntax `x0 x1` are accepted; the latter results in singleton tuples.
fn deserialize_exp_tuples<'de, D: Deserializer<'de>, Var: IntoVar>(
	deserializer: D,
) -> Result<Vec<Vec<IntExp<Var>>>, D::Error> {
	/// Visitor to parse a list of integer expression tuples
	struct V<X>(PhantomData<X>);
	impl<X: IntoVar> Visitor<'_> for V<X> {
		type Value = Vec<Vec<IntExp<X>>>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("a list of integer expression tuples")
		}

		fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let (_, rows) = all_consuming(sequence(alt((
				tuple(IntExp::parse),
				map(IntExp::parse, |e| vec![e]),
			))))
			.parse(v)
			.map_err(|_| E::custom(format!("invalid integer expressions `{v}'")))?;
			Ok(rows)
		}
	}
	deserializer.deserialize_str(V::<Var>(PhantomData))
}

/// Serialize a list of integer expression tuples as a string
///
/// One-dimensional tuples are written using the plain `x0 x1` syntax.
fn serialize_exp_tuples<S: Serializer, T: Display>(
	vals: &[Vec<T>],
	serializer: S,
) -> Result<S::Ok, S::Error> {
	if vals.iter().all(|t| t.len() == 1) {
		return serialize_list(&vals.iter().flatten().collect::<Vec<_>>(), serializer);
	}
	serializer.serialize_str(
		&vals
			.iter()
			.map(|t| {
				format!(
					"({})",
					t.iter()
						.map(|e| e.to_string())
						.collect::<Vec<_>>()
						.join(",")
				)
			})
			.collect::<Vec<_>>()
			.join(""),
	)
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

/// Whether the value is one
fn is_one(x: &usize) -> bool {
	*x == 1
}

/// The default step of a [`SlideList`]
fn usize_one() -> usize {
	1
}

/// Serialize a list of integer tuples as a string
fn serialize_int_tuples<S: Serializer>(
	vals: &[Vec<Option<IntVal>>],
	serializer: S,
) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(
		&vals
			.iter()
			.map(|e| {
				format!(
					"({})",
					e.iter()
						.map(|e| match e {
							Some(e) => e.to_string(),
							None => "*".to_owned(),
						})
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
impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de> for Block<Identifier, Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Deserialize a <block> element
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: From<String>, Var: IntoVar"))]
		/// A set of constraints that is linked together semantically
		struct Block<Identifier, Var> {
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
			pub constraints: Vec<MetaConstraint<Identifier, Var>>,
		}
		let c = Block::deserialize(deserializer)?;
		let class: Vec<_> = c.class.into_iter().map(Into::into).collect();

		Ok(Self {
			info: MetaInfo {
				identifier: c.identifier,
				note: c.note,
			},
			class,
			constraints: c.constraints,
		})
	}
}

impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de>
	for Cardinality<Identifier, Var>
{
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Helper struct to deserialize the <values> element of the cardinality constraint
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Var: IntoVar"))]
		struct Values<Var> {
			/// closed attribute
			#[serde(default, rename = "@closed")]
			closed: Option<bool>,
			/// content of the <values> element
			#[serde(rename = "$text", deserialize_with = "IntExp::parse_vec")]
			list: Vec<IntExp<Var>>,
		}
		/// Helper struct to deserialize the <cardinality> element of the cardinality constraint
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: From<String>, Var: IntoVar"))]
		struct Cardinality<Identifier, Var> {
			/// Metadata for the constraint
			#[serde(flatten)]
			info: MetaInfo<Identifier>,
			/// <list> element
			#[serde(deserialize_with = "IntExp::parse_vec")]
			list: Vec<IntExp<Var>>,
			/// <values> element
			values: Values<Var>,
			/// <occurs> element
			#[serde(deserialize_with = "Exp::parse_vec")]
			occurs: Vec<Exp<Var>>,
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

impl<Identifier: Display, Var: Display> Serialize for Cardinality<Identifier, Var> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		/// Serialize a <values> element
		#[derive(Serialize)]
		#[serde(bound(serialize = "Var: Display"))]
		struct Values<'a, Var> {
			/// closed attribute
			#[serde(rename = "@closed", skip_serializing_if = "is_false")]
			closed: bool,
			/// content of the <values> element
			#[serde(rename = "$text", serialize_with = "serialize_list")]
			list: &'a Vec<IntExp<Var>>,
		}
		/// Serialize a <cardinality> element
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display, Var: Display"))]
		struct Cardinality<'a, Identifier, Var> {
			/// meta information
			#[serde(flatten)]
			info: &'a MetaInfo<Identifier>,
			/// <list> element
			#[serde(serialize_with = "serialize_list")]
			list: &'a Vec<IntExp<Var>>,
			/// <values> element
			values: Values<'a, Var>,
			/// <occurs> element
			#[serde(serialize_with = "serialize_list")]
			occurs: &'a Vec<Exp<Var>>,
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

impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de> for Channel<Identifier, Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Deserialize a <channel> element
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "I: From<String>, V: IntoVar"))]
		struct Channel<'a, I, V> {
			/// meta information
			#[serde(flatten)]
			info: MetaInfo<I>,
			/// <list> element(s)
			list: Vec<Cow<'a, str>>,
			/// <value> element
			#[serde(default)]
			value: Option<IntExp<V>>,
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

impl<Identifier: Display, Var: Display> Serialize for Channel<Identifier, Var> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		/// Serialize a <channel> element
		#[derive(Serialize)]
		#[serde(bound(serialize = "I: Display, V: Display"))]
		struct Channel<'a, I, V> {
			/// meta information
			#[serde(flatten)]
			info: &'a MetaInfo<I>,
			/// <list> element(s)
			list: Vec<String>,
			/// <value> element
			#[serde(skip_serializing_if = "Option::is_none")]
			value: &'a Option<IntExp<V>>,
		}

		let p = |i: &Vec<IntExp<Var>>| -> String {
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

impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de> for Circuit<Identifier, Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Deserializes a <circuit> element
		#[derive(Deserialize)]
		#[serde(bound = "Identifier: From<String>, Var: IntoVar")]
		struct Circuit<Identifier, Var> {
			/// meta information
			#[serde(flatten)]
			info: MetaInfo<Identifier>,
			/// textual content of the element
			#[serde(default, deserialize_with = "IntExp::parse_vec", alias = "$text")]
			simple: Vec<IntExp<Var>>,
			/// <list> element
			#[serde(default)]
			list: OffsetList<Var>,
			/// <size> element
			#[serde(default)]
			size: Option<IntExp<Var>>,
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

impl<'de, Var: IntoVar> Deserialize<'de> for Condition<Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Condition<Var>, D::Error> {
		/// Visitor for parsing a condition.
		struct V<X>(PhantomData<X>);
		impl<X: IntoVar> Visitor<'_> for V<X> {
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
		deserializer.deserialize_str(V(PhantomData::<Var>))
	}
}

impl<Identifier: Clone + Hash + Eq + ToString> Condition<VarRef<Identifier>> {
	fn unroll(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Vec<Exp<SimpleRef<Identifier>>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Condition<SimpleRef<Identifier>>, UnrollError> {
		Ok(Condition {
			operator: self.operator.clone(),
			operand: self.operand.unroll_single(arrays, args, remainder)?,
		})
	}
}

impl<Var: Display> Display for Condition<Var> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({},{})", self.operator, self.operand)
	}
}

impl<Var: Display> Serialize for Condition<Var> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

impl<Identifier, I> Constraint<Identifier, VarRef<I>> {
	fn max_placeholder(&self) -> Option<usize> {
		match self {
			Constraint::AllDifferent(AllDifferent { list, matrix, .. }) => list
				.iter()
				.chain(matrix.iter().flatten())
				.filter_map(|exp| exp.max_placeholder())
				.max(),
			Constraint::AllEqual(AllEqual { list, .. })
			| Constraint::Extension(Extension { list, .. })
			| Constraint::Mdd(Mdd { list, .. })
			| Constraint::Regular(Regular { list, .. })
			| Constraint::Precedence(Precedence { list, .. }) => {
				list.iter().filter_map(|exp| exp.max_placeholder()).max()
			}
			Constraint::BinPacking(BinPacking {
				list,
				sizes,
				condition,
				limits,
				loads,
				..
			}) => list
				.iter()
				.chain(sizes)
				.chain(limits)
				.chain(loads)
				.filter_map(|exp| exp.max_placeholder())
				.chain(condition.as_ref().and_then(|c| c.operand.max_placeholder()))
				.max(),
			Constraint::Cardinality(Cardinality {
				list,
				values,
				occurs,
				..
			}) => list
				.iter()
				.chain(values)
				.filter_map(|exp| exp.max_placeholder())
				.chain(occurs.iter().filter_map(|exp| exp.max_placeholder()))
				.max(),
			Constraint::Channel(Channel {
				list,
				inverse_list,
				value,
				..
			}) => list
				.iter()
				.chain(inverse_list)
				.chain(value)
				.filter_map(|exp| exp.max_placeholder())
				.max(),
			Constraint::Circuit(Circuit {
				list: OffsetList { list, .. },
				size,
				..
			}) => list
				.iter()
				.chain(size)
				.filter_map(|exp| exp.max_placeholder())
				.max(),
			Constraint::Clause(Clause { list, .. }) => {
				list.iter().filter_map(|exp| exp.max_placeholder()).max()
			}
			Constraint::Count(Count {
				list,
				values,
				condition,
				..
			}) => list
				.iter()
				.chain(values)
				.filter_map(|exp| exp.max_placeholder())
				.chain(condition.operand.max_placeholder())
				.max(),
			Constraint::Cumulative(Cumulative {
				origins,
				lengths,
				heights,
				condition,
				..
			}) => origins
				.iter()
				.chain(lengths)
				.chain(heights)
				.filter_map(|exp| exp.max_placeholder())
				.chain(condition.operand.max_placeholder())
				.max(),
			Constraint::Element(Element {
				list: OffsetList { list, .. },
				matrix,
				index,
				value,
				condition,
				..
			}) => list
				.iter()
				.chain(matrix.iter().flatten())
				.chain(index)
				.chain(value)
				.filter_map(|exp| exp.max_placeholder())
				.chain(condition.as_ref().and_then(|c| c.operand.max_placeholder()))
				.max(),
			Constraint::Instantiation(Instantiation { list, .. }) => list
				.iter()
				.filter_map(|v| {
					if let &VarRef::Placeholder(Placeholder::Position(i)) = v {
						Some(i)
					} else {
						None
					}
				})
				.max(),
			Constraint::Intension(Intension { function, .. }) => function.max_placeholder(),
			Constraint::Knapsack(Knapsack {
				list, condition, ..
			}) => list
				.iter()
				.filter_map(|e| e.max_placeholder())
				.chain(condition.iter().filter_map(|c| c.operand.max_placeholder()))
				.max(),
			Constraint::Lex(Lex { lists, matrix, .. }) => lists
				.iter()
				.chain(matrix)
				.flatten()
				.filter_map(|e| e.max_placeholder())
				.max(),
			Constraint::Maximum(Maximum {
				list, condition, ..
			})
			| Constraint::Minimum(Minimum {
				list, condition, ..
			})
			| Constraint::Sum(Sum {
				list, condition, ..
			})
			| Constraint::NValues(NValues {
				list, condition, ..
			}) => list
				.iter()
				.filter_map(|e| e.max_placeholder())
				.chain(condition.operand.max_placeholder())
				.max(),
			Constraint::NoOverlap(NoOverlap {
				origins, lengths, ..
			}) => origins
				.iter()
				.chain(lengths)
				.flatten()
				.flat_map(|e| e.max_placeholder())
				.max(),
			Constraint::Ordered(Ordered { list, lengths, .. }) => list
				.iter()
				.chain(lengths)
				.filter_map(|e| e.max_placeholder())
				.max(),
		}
	}
}

impl<Identifier: Clone + Hash + Eq + ToString> Constraint<Identifier, VarRef<Identifier>> {
	pub(crate) fn unroll(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Vec<Exp<SimpleRef<Identifier>>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Constraint<Identifier, SimpleRef<Identifier>>, UnrollError> {
		let instantiate_exps = |list: &[Exp<_>]| {
			let mut nlist = Vec::new();
			for e in list {
				nlist.extend(e.unroll(arrays, args, remainder)?);
			}
			Ok(nlist)
		};
		let instantiate_ints = |list: &[IntExp<_>]| {
			let mut nlist = Vec::new();
			for e in list {
				nlist.extend(e.unroll(arrays, args, remainder)?);
			}
			Ok(nlist)
		};
		let instantiate_int_rows = |rows: &[Vec<IntExp<_>>]| {
			rows.iter()
				.map(|row| instantiate_ints(row))
				.collect::<Result<Vec<_>, _>>()
		};
		// A matrix given as a single array slice only gains its row structure
		// here, where the dimensions of the array are known.
		let instantiate_matrix = |rows: &[Vec<IntExp<_>>]| {
			if let [row] = rows {
				if let [IntExp::Var(v @ VarRef::ArrayAccess(_, _))] = &row[..] {
					return v
						.unroll_matrix(arrays, args, remainder)?
						.into_iter()
						.map(|row| row.into_iter().map(TryInto::try_into).collect())
						.collect();
				}
			}
			instantiate_int_rows(rows)
		};
		let instantiate_vars = |list: &[VarRef<_>]| {
			let mut nlist = Vec::new();
			for e in list {
				nlist.extend(
					e.unroll(arrays, args, remainder)?
						.into_iter()
						.map(|v| v.into_var())
						.collect::<Result<Vec<_>, _>>()?,
				);
			}
			Ok(nlist)
		};

		match self {
			Constraint::AllDifferent(AllDifferent {
				info,
				list,
				matrix,
				except,
			}) => Ok(Constraint::AllDifferent(AllDifferent {
				info: info.clone(),
				list: instantiate_ints(list)?,
				matrix: instantiate_matrix(matrix)?,
				except: except.clone(),
			})),
			Constraint::AllEqual(AllEqual { info, list, except }) => {
				Ok(Constraint::AllEqual(AllEqual {
					info: info.clone(),
					list: instantiate_ints(list)?,
					except: except.clone(),
				}))
			}
			Constraint::BinPacking(BinPacking {
				info,
				list,
				sizes,
				condition,
				limits,
				loads,
			}) => {
				let condition = if let Some(condition) = condition {
					Some(condition.unroll(arrays, args, remainder)?)
				} else {
					None
				};
				Ok(Constraint::BinPacking(BinPacking {
					info: info.clone(),
					list: instantiate_ints(list)?,
					sizes: instantiate_ints(sizes)?,
					condition,
					limits: instantiate_ints(limits)?,
					loads: instantiate_ints(loads)?,
				}))
			}
			Constraint::Cardinality(Cardinality {
				info,
				list,
				values,
				closed,
				occurs,
			}) => Ok(Constraint::Cardinality(Cardinality {
				info: info.clone(),
				list: instantiate_ints(list)?,
				values: instantiate_ints(values)?,
				closed: *closed,
				occurs: instantiate_exps(occurs)?,
			})),
			Constraint::Channel(Channel {
				info,
				list,
				inverse_list,
				value,
			}) => {
				let value = if let Some(value) = value {
					Some(value.unroll_single(arrays, args, remainder)?)
				} else {
					None
				};
				Ok(Constraint::Channel(Channel {
					info: info.clone(),
					list: instantiate_ints(list)?,
					inverse_list: instantiate_ints(inverse_list)?,
					value,
				}))
			}
			Constraint::Circuit(Circuit {
				info,
				list: OffsetList { list, start_index },
				size,
			}) => Ok(Constraint::Circuit(Circuit {
				info: info.clone(),
				list: OffsetList {
					list: instantiate_ints(list)?,
					start_index: *start_index,
				},
				size: if let Some(size) = size {
					Some(size.unroll_single(arrays, args, remainder)?)
				} else {
					None
				},
			})),
			Constraint::Clause(Clause { info, list }) => {
				let mut nlist = Vec::with_capacity(list.len());
				for e in list {
					nlist.extend(e.unroll(arrays, args, remainder)?);
				}
				Ok(Constraint::Clause(Clause {
					info: info.clone(),
					list: nlist,
				}))
			}
			Constraint::Count(Count {
				info,
				list,
				values,
				condition,
			}) => Ok(Constraint::Count(Count {
				info: info.clone(),
				list: instantiate_ints(list)?,
				values: instantiate_ints(values)?,
				condition: condition.unroll(arrays, args, remainder)?,
			})),
			Constraint::Cumulative(Cumulative {
				info,
				origins,
				lengths,
				heights,
				condition,
			}) => Ok(Constraint::Cumulative(Cumulative {
				info: info.clone(),
				origins: instantiate_ints(origins)?,
				lengths: instantiate_ints(lengths)?,
				heights: instantiate_ints(heights)?,
				condition: condition.unroll(arrays, args, remainder)?,
			})),
			Constraint::Element(Element {
				info,
				list: OffsetList { list, start_index },
				matrix,
				index,
				value,
				condition,
			}) => {
				let index = instantiate_ints(index)?;
				let value = if let Some(value) = value {
					Some(value.unroll_single(arrays, args, remainder)?)
				} else {
					None
				};
				let condition = if let Some(condition) = condition {
					Some(condition.unroll(arrays, args, remainder)?)
				} else {
					None
				};
				Ok(Constraint::Element(Element {
					info: info.clone(),
					list: OffsetList {
						list: instantiate_ints(list)?,
						start_index: *start_index,
					},
					matrix: instantiate_matrix(matrix)?,
					index,
					value,
					condition,
				}))
			}
			Constraint::Extension(Extension {
				info,
				list,
				supports,
				conflicts,
			}) => Ok(Constraint::Extension(Extension {
				info: info.clone(),
				list: instantiate_ints(list)?,
				supports: supports.clone(),
				conflicts: conflicts.clone(),
			})),
			Constraint::Instantiation(Instantiation {
				info,
				ty,
				cost,
				list,
				values,
			}) => Ok(Constraint::Instantiation(Instantiation {
				info: info.clone(),
				ty: ty.clone(),
				cost: *cost,
				list: instantiate_vars(list)?,
				values: values.clone(),
			})),
			Constraint::Intension(Intension { info, function }) => {
				Ok(Constraint::Intension(Intension {
					info: info.clone(),
					function: function.unroll_single(arrays, args, remainder)?,
				}))
			}
			Constraint::Knapsack(Knapsack {
				info,
				list,
				weights,
				profits,
				condition: [c1, c2],
			}) => Ok(Constraint::Knapsack(Knapsack {
				info: info.clone(),
				list: instantiate_ints(list)?,
				weights: weights.clone(),
				profits: profits.clone(),
				condition: [
					c1.unroll(arrays, args, remainder)?,
					c2.unroll(arrays, args, remainder)?,
				],
			})),
			Constraint::Lex(Lex {
				info,
				lists,
				matrix,
				operator,
			}) => Ok(Constraint::Lex(Lex {
				info: info.clone(),
				lists: instantiate_int_rows(lists)?,
				matrix: instantiate_matrix(matrix)?,
				operator: operator.clone(),
			})),
			Constraint::Maximum(Maximum {
				info,
				list,
				condition,
			}) => Ok(Constraint::Maximum(Maximum {
				info: info.clone(),
				list: instantiate_ints(list)?,
				condition: condition.unroll(arrays, args, remainder)?,
			})),
			Constraint::Mdd(Mdd {
				info,
				list,
				transitions,
			}) => Ok(Constraint::Mdd(Mdd {
				info: info.clone(),
				list: instantiate_ints(list)?,
				transitions: transitions.clone(),
			})),
			Constraint::Minimum(Minimum {
				info,
				list,
				condition,
			}) => Ok(Constraint::Minimum(Minimum {
				info: info.clone(),
				list: instantiate_ints(list)?,
				condition: condition.unroll(arrays, args, remainder)?,
			})),
			Constraint::NValues(NValues {
				info,
				list,
				except,
				condition,
			}) => Ok(Constraint::NValues(NValues {
				info: info.clone(),
				list: instantiate_ints(list)?,
				except: except.clone(),
				condition: condition.unroll(arrays, args, remainder)?,
			})),
			Constraint::NoOverlap(NoOverlap {
				info,
				zero_ignored,
				origins,
				lengths,
			}) => Ok(Constraint::NoOverlap(NoOverlap {
				info: info.clone(),
				zero_ignored: *zero_ignored,
				origins: instantiate_int_rows(origins)?,
				lengths: instantiate_int_rows(lengths)?,
			})),
			Constraint::Ordered(Ordered {
				info,
				list,
				lengths,
				operator,
			}) => Ok(Constraint::Ordered(Ordered {
				info: info.clone(),
				list: instantiate_ints(list)?,
				lengths: instantiate_ints(lengths)?,
				operator: operator.clone(),
			})),
			Constraint::Precedence(Precedence {
				info,
				list,
				values,
				covered,
			}) => Ok(Constraint::Precedence(Precedence {
				info: info.clone(),
				list: instantiate_ints(list)?,
				values: values.clone(),
				covered: *covered,
			})),
			Constraint::Regular(Regular {
				info,
				list,
				transitions,
				start,
				finish,
			}) => Ok(Constraint::Regular(Regular {
				info: info.clone(),
				list: instantiate_ints(list)?,
				transitions: transitions.clone(),
				start: start.clone(),
				finish: finish.clone(),
			})),
			Constraint::Sum(Sum {
				info,
				list,
				coeffs,
				condition,
			}) => Ok(Constraint::Sum(Sum {
				info: info.clone(),
				list: instantiate_ints(list)?,
				coeffs: coeffs.clone(),
				condition: condition.unroll(arrays, args, remainder)?,
			})),
		}
	}
}

impl<Identifier, Var> TryFrom<TemplateCapture<Identifier, Var>> for Constraint<Identifier, Var> {
	type Error = ();

	fn try_from(value: TemplateCapture<Identifier, Var>) -> Result<Self, Self::Error> {
		match value {
			TemplateCapture::AllDifferent(all_different) => {
				Ok(Constraint::AllDifferent(all_different))
			}
			TemplateCapture::AllEqual(all_equal) => Ok(Constraint::AllEqual(all_equal)),
			TemplateCapture::BinPacking(bin_packing) => Ok(Constraint::BinPacking(bin_packing)),
			TemplateCapture::Cardinality(cardinality) => Ok(Constraint::Cardinality(cardinality)),
			TemplateCapture::Channel(channel) => Ok(Constraint::Channel(channel)),
			TemplateCapture::Circuit(circuit) => Ok(Constraint::Circuit(circuit)),
			TemplateCapture::Clause(clause) => Ok(Constraint::Clause(clause)),
			TemplateCapture::Count(count) => Ok(Constraint::Count(count)),
			TemplateCapture::Cumulative(cumulative) => Ok(Constraint::Cumulative(cumulative)),
			TemplateCapture::Element(element) => Ok(Constraint::Element(element)),
			TemplateCapture::Extension(extension) => Ok(Constraint::Extension(extension)),
			TemplateCapture::Instantiation(instantiation) => {
				Ok(Constraint::Instantiation(instantiation))
			}
			TemplateCapture::Intension(intension) => Ok(Constraint::Intension(intension)),
			TemplateCapture::Knapsack(knapsack) => Ok(Constraint::Knapsack(knapsack)),
			TemplateCapture::Lex(lex) => Ok(Constraint::Lex(lex)),
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

impl<Identifier, Var> TryFrom<SlideCapture<Identifier, Var>> for Constraint<Identifier, Var> {
	type Error = ();

	fn try_from(value: SlideCapture<Identifier, Var>) -> Result<Self, Self::Error> {
		match value {
			SlideCapture::AllDifferent(all_different) => {
				Ok(Constraint::AllDifferent(all_different))
			}
			SlideCapture::AllEqual(all_equal) => Ok(Constraint::AllEqual(all_equal)),
			SlideCapture::BinPacking(bin_packing) => Ok(Constraint::BinPacking(bin_packing)),
			SlideCapture::Cardinality(cardinality) => Ok(Constraint::Cardinality(cardinality)),
			SlideCapture::Channel(channel) => Ok(Constraint::Channel(channel)),
			SlideCapture::Circuit(circuit) => Ok(Constraint::Circuit(circuit)),
			SlideCapture::Clause(clause) => Ok(Constraint::Clause(clause)),
			SlideCapture::Count(count) => Ok(Constraint::Count(count)),
			SlideCapture::Cumulative(cumulative) => Ok(Constraint::Cumulative(cumulative)),
			SlideCapture::Element(element) => Ok(Constraint::Element(element)),
			SlideCapture::Extension(extension) => Ok(Constraint::Extension(extension)),
			SlideCapture::Instantiation(instantiation) => {
				Ok(Constraint::Instantiation(instantiation))
			}
			SlideCapture::Intension(intension) => Ok(Constraint::Intension(intension)),
			SlideCapture::Knapsack(knapsack) => Ok(Constraint::Knapsack(knapsack)),
			SlideCapture::Lex(lex) => Ok(Constraint::Lex(lex)),
			SlideCapture::Maximum(maximum) => Ok(Constraint::Maximum(maximum)),
			SlideCapture::Mdd(mdd) => Ok(Constraint::Mdd(mdd)),
			SlideCapture::Minimum(minimum) => Ok(Constraint::Minimum(minimum)),
			SlideCapture::NValues(nvalues) => Ok(Constraint::NValues(nvalues)),
			SlideCapture::NoOverlap(no_overlap) => Ok(Constraint::NoOverlap(no_overlap)),
			SlideCapture::Ordered(ordered) => Ok(Constraint::Ordered(ordered)),
			SlideCapture::Precedence(precedence) => Ok(Constraint::Precedence(precedence)),
			SlideCapture::Regular(regular) => Ok(Constraint::Regular(regular)),
			SlideCapture::Sum(sum) => Ok(Constraint::Sum(sum)),
			SlideCapture::List(_) => Err(()),
		}
	}
}

impl<Identifier, I> Group<Identifier, VarRef<I>> {
	/// Returns the placeholder with the highest number, or None if there are no
	/// placeholders.
	fn max_placeholder(&self) -> Option<usize> {
		self.constraints
			.iter()
			.filter_map(|c| c.max_placeholder())
			.max()
	}
}

impl<Identifier: Clone + Hash + Eq + ToString> Group<Identifier, VarRef<Identifier>> {
	/// Create the instantiated versions of the group of constriants
	pub fn unroll(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
	) -> Result<Vec<Constraint<Identifier, SimpleRef<Identifier>>>, UnrollError> {
		if self.args.is_empty() {
			return self
				.constraints
				.iter()
				.map(|c| c.unroll(arrays, &[], &[]))
				.collect();
		}
		let rem_start = self.max_placeholder().map(|x| x + 1).unwrap_or(0);

		let mut flat = Vec::with_capacity(self.constraints.len() * self.args.len());
		for args in &self.args {
			// Each `<args>` token is kept separate, since a single token (e.g.
			// `x[0][]`) can stand for a whole list of expressions.
			let expanded = args
				.iter()
				.map(|arg| arg.unroll(arrays, &[], &[]))
				.collect::<Result<Vec<_>, _>>()?;
			// Tokens beyond the highest placeholder are matched by `%...`.
			let remainder: Vec<_> = expanded[rem_start.min(expanded.len())..]
				.iter()
				.flatten()
				.cloned()
				.collect();
			for constraint in &self.constraints {
				flat.push(constraint.unroll(arrays, &expanded, &remainder)?);
			}
		}
		Ok(flat)
	}
}

// Note: flatten of MetaInfo does not seem to work here
impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de> for Group<Identifier, Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: From<String>, Var: IntoVar"))]
		struct Group<Identifier, Var> {
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
			constraints: Vec<TemplateCapture<Identifier, Var>>,
		}
		let grp: Group<Identifier, Var> = Deserialize::deserialize(deserializer)?;
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
impl<Identifier: Display, Var: Display> Serialize for Group<Identifier, Var> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		#[serde(bound(serialize = "Var: Display"))]
		/// Helper structure used to parse a list of expressions.
		struct ExpList<'a, Var> {
			#[serde(rename = "$text", serialize_with = "serialize_list")]
			pub(crate) elements: &'a Vec<Exp<Var>>,
		}
		#[derive(Serialize)]
		#[serde(bound(serialize = "Var: Display"), rename_all = "camelCase")]
		enum ExpListE<'a, Var> {
			Args(ExpList<'a, Var>),
		}
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display, Var: Display"))]
		/// Helper struct to serialize the instantiation element
		struct Group<'a, Identifier, Var> {
			/// Optional metadata for the constraint
			#[serde(flatten)]
			info: &'a MetaInfo<Identifier>,
			/// List of constraints
			#[serde(rename = "$value")]
			constraints: &'a Vec<Constraint<Identifier, Var>>,
			/// Arguments to instantiate the constraints
			#[serde(rename = "$value")]
			args: Vec<ExpListE<'a, Var>>,
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

impl<Identifier: Clone + Hash + Eq + ToString> Slide<Identifier, VarRef<Identifier>> {
	/// Create the instantiated versions of the constraint template
	pub fn unroll(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
	) -> Result<Vec<Constraint<Identifier, SimpleRef<Identifier>>>, UnrollError> {
		// The arity of the template determines how many expressions a single
		// list contributes to each instantiation.
		let arity = self.constraint.max_placeholder().map_or(0, |i| i + 1);
		let lists = self
			.lists
			.iter()
			.map(|l| {
				let list = l
					.list
					.iter()
					.map(|e| e.unroll(arrays, &[], &[]))
					.collect::<Result<Vec<_>, _>>()?
					.into_iter()
					.flatten()
					.map(|e| Exp::Int(Box::new(e)))
					.collect::<Vec<_>>();
				let collect = l
					.collect
					.unwrap_or(if self.lists.len() == 1 { arity } else { 1 });
				Ok((list, l.offset, collect))
			})
			.collect::<Result<Vec<_>, UnrollError>>()?;

		// Every list must be able to supply a sub-list for each iteration.
		let iterations = lists
			.iter()
			.map(|(list, offset, collect)| {
				if list.len() < *collect {
					0
				} else if self.circular {
					list.len().div_ceil(*offset)
				} else {
					(list.len() - collect) / offset + 1
				}
			})
			.min()
			.unwrap_or(0);

		let mut flat = Vec::with_capacity(iterations);
		for i in 0..iterations {
			let mut args = Vec::with_capacity(arity);
			for (list, offset, collect) in &lists {
				args.extend(
					(0..*collect).map(|j| vec![list[(i * offset + j) % list.len()].clone()]),
				);
			}
			flat.push(self.constraint.unroll(arrays, &args, &[])?);
		}
		Ok(flat)
	}
}

// Note: flatten of MetaInfo does not seem to work here, see the note on the
// implementation for `Group`
impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de> for Slide<Identifier, Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: From<String>, Var: IntoVar"))]
		struct Slide<Identifier, Var> {
			/// Name assigned to the element
			#[serde(rename = "@id", default, deserialize_with = "crate::deserialize_ident")]
			identifier: Option<Identifier>,
			/// Comment from the user about the element
			#[serde(rename = "@note", default)]
			note: Option<String>,
			/// Whether the sliding wraps around the end of the list
			#[serde(rename = "@circular", default)]
			circular: bool,
			/// Lists and the constraint template
			#[serde(default, rename = "$value")]
			content: Vec<SlideCapture<Identifier, Var>>,
		}
		let slide: Slide<Identifier, Var> = Deserialize::deserialize(deserializer)?;
		let mut lists = Vec::new();
		let mut constraint = None;
		for c in slide.content {
			match c {
				SlideCapture::List(l) => lists.push(l),
				c => {
					constraint = Some(
						c.try_into()
							.map_err(|_| de::Error::custom("invalid slide constraint template"))?,
					)
				}
			}
		}
		let Some(constraint) = constraint else {
			return Err(de::Error::missing_field("constraint template"));
		};
		Ok(Self {
			info: MetaInfo {
				identifier: slide.identifier,
				note: slide.note,
			},
			circular: slide.circular,
			lists,
			constraint: Box::new(constraint),
		})
	}
}

// Note: flatten of MetaInfo does not seem to work here, see the note on the
// implementation for `Group`
impl<Identifier: Display, Var: Display> Serialize for Slide<Identifier, Var> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		#[serde(bound(serialize = "Var: Display"), rename_all = "camelCase")]
		enum SlideListE<'a, Var> {
			List(&'a SlideList<Var>),
		}
		#[derive(Serialize)]
		#[serde(bound(serialize = "Identifier: Display, Var: Display"))]
		/// Helper struct to serialize the slide element
		struct Slide<'a, Identifier, Var> {
			/// Optional metadata for the constraint
			#[serde(flatten)]
			info: &'a MetaInfo<Identifier>,
			/// Whether the sliding wraps around the end of the list
			#[serde(rename = "@circular", skip_serializing_if = "is_false")]
			circular: bool,
			/// Lists over which the template is slid
			#[serde(rename = "$value")]
			lists: Vec<SlideListE<'a, Var>>,
			/// Constraint template
			#[serde(rename = "$value")]
			constraint: &'a Constraint<Identifier, Var>,
		}
		Slide {
			info: &self.info,
			circular: self.circular,
			lists: self.lists.iter().map(SlideListE::List).collect(),
			constraint: &self.constraint,
		}
		.serialize(serializer)
	}
}

impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de>
	for MetaConstraint<Identifier, Var>
{
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let con: CaptureConstraint<Identifier, Var> = Deserialize::deserialize(deserializer)?;
		Ok(con.into())
	}
}

impl<Identifier, Var> From<CaptureConstraint<Identifier, Var>> for MetaConstraint<Identifier, Var> {
	fn from(value: CaptureConstraint<Identifier, Var>) -> Self {
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
			CaptureConstraint::Clause(clause) => {
				MetaConstraint::Constraint(Constraint::Clause(clause))
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
			CaptureConstraint::Lex(lex) => MetaConstraint::Constraint(Constraint::Lex(lex)),
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
			CaptureConstraint::Slide(slide) => MetaConstraint::Slide(slide),
		}
	}
}

impl<Identifier: Display, Var: Display> Serialize for MetaConstraint<Identifier, Var> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		#[serde(
			bound(serialize = "Identifier: Display, Var: Display"),
			rename_all = "camelCase"
		)]
		enum OutputConstraint<'a, Identifier, Var> {
			/// [`AllDifferent`] constraint
			AllDifferent(&'a AllDifferent<Identifier, Var>),
			/// [`AllEqual`] constraint
			AllEqual(&'a AllEqual<Identifier, Var>),
			/// [`BinPacking`] constraint
			BinPacking(&'a BinPacking<Identifier, Var>),
			/// [`Cardinality`] constraint
			Cardinality(&'a Cardinality<Identifier, Var>),
			/// [`Channel`] constraint
			Channel(&'a Channel<Identifier, Var>),
			/// [`Circuit`] constraint
			Circuit(&'a Circuit<Identifier, Var>),
			/// [`Clause`] constraint
			Clause(&'a Clause<Identifier, Var>),
			/// [`Count`] constraint
			Count(&'a Count<Identifier, Var>),
			/// [`Cumulative`] constraint
			Cumulative(&'a Cumulative<Identifier, Var>),
			/// [`Element`] constraint
			Element(&'a Element<Identifier, Var>),
			/// [`Extension`] constraint
			Extension(&'a Extension<Identifier, Var>),
			/// [`Instantiation`] constraint
			Instantiation(&'a Instantiation<Identifier, Var>),
			/// [`Intension`] constraint
			Intension(&'a Intension<Identifier, Var>),
			/// [`Knapsack`] constraint
			Knapsack(&'a Knapsack<Identifier, Var>),
			/// [`Lex`] constraint
			Lex(&'a Lex<Identifier, Var>),
			/// [`Maximum`] constraint
			Maximum(&'a Maximum<Identifier, Var>),
			/// [`Mdd`] constraint
			Mdd(&'a Mdd<Identifier, Var>),
			/// [`Minimum`] constraint
			Minimum(&'a Minimum<Identifier, Var>),
			/// [`NValues`] constraint
			NValues(&'a NValues<Identifier, Var>),
			/// [`NoOverlap`] constraint
			NoOverlap(&'a NoOverlap<Identifier, Var>),
			/// [`Ordered`] constraint
			Ordered(&'a Ordered<Identifier, Var>),
			/// [`Precedence`] constraint
			Precedence(&'a Precedence<Identifier, Var>),
			/// [`Regular`] constraint
			Regular(&'a Regular<Identifier, Var>),
			/// [`Sum`] constraint
			Sum(&'a Sum<Identifier, Var>),
			/// Constraint [`Group`], that can serve as a template
			Group(&'a Group<Identifier, Var>),
			/// Constraint [`Group`], that can serve as a template
			Block(&'a Block<Identifier, Var>),
			/// Meta-constraint [`Slide`]
			Slide(&'a Slide<Identifier, Var>),
		}

		let c = match self {
			MetaConstraint::Group(group) => OutputConstraint::Group(group),
			MetaConstraint::Block(block) => OutputConstraint::Block(block),
			MetaConstraint::Slide(slide) => OutputConstraint::Slide(slide),
			MetaConstraint::Constraint(con) => match con {
				Constraint::AllDifferent(all_different) => {
					OutputConstraint::AllDifferent(all_different)
				}
				Constraint::AllEqual(all_equal) => OutputConstraint::AllEqual(all_equal),
				Constraint::BinPacking(bin_packing) => OutputConstraint::BinPacking(bin_packing),
				Constraint::Cardinality(cardinality) => OutputConstraint::Cardinality(cardinality),
				Constraint::Channel(channel) => OutputConstraint::Channel(channel),
				Constraint::Circuit(circuit) => OutputConstraint::Circuit(circuit),
				Constraint::Clause(clause) => OutputConstraint::Clause(clause),
				Constraint::Count(count) => OutputConstraint::Count(count),
				Constraint::Cumulative(cumulative) => OutputConstraint::Cumulative(cumulative),
				Constraint::Element(element) => OutputConstraint::Element(element),
				Constraint::Extension(extension) => OutputConstraint::Extension(extension),
				Constraint::Instantiation(instantiation) => {
					OutputConstraint::Instantiation(instantiation)
				}
				Constraint::Intension(intension) => OutputConstraint::Intension(intension),
				Constraint::Knapsack(knapsack) => OutputConstraint::Knapsack(knapsack),
				Constraint::Lex(lex) => OutputConstraint::Lex(lex),
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

impl<Var> Default for OffsetList<Var> {
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

impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de> for Precedence<Identifier, Var> {
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
		#[serde(bound(
			deserialize = "Identifier: From<String>, Var: IntoVar",
			serialize = "Identifier: Display"
		))]
		struct Precedence<Identifier, Var> {
			/// Meta information
			#[serde(flatten)]
			info: MetaInfo<Identifier>,
			/// <list> element or content of the element
			#[serde(
				alias = "$text",
				deserialize_with = "IntExp::parse_vec",
				serialize_with = "serialize_list"
			)]
			list: Vec<IntExp<Var>>,
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

impl<Identifier: Display, Var: Display> Serialize for Precedence<Identifier, Var> {
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
		#[serde(bound(serialize = "Identifier: Display, Var: Display"))]
		struct Precedence<'a, Identifier, Var> {
			/// Optional meta information
			#[serde(flatten)]
			info: &'a MetaInfo<Identifier>,
			/// <list> element or string content
			#[serde(alias = "$text", serialize_with = "serialize_list")]
			list: &'a Vec<IntExp<Var>>,
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
			Constraint::Clause(clause) => TemplateCapture::Clause(clause),
			Constraint::Count(count) => TemplateCapture::Count(count),
			Constraint::Cumulative(cumulative) => TemplateCapture::Cumulative(cumulative),
			Constraint::Element(element) => TemplateCapture::Element(element),
			Constraint::Extension(extension) => TemplateCapture::Extension(extension),
			Constraint::Instantiation(instantiation) => {
				TemplateCapture::Instantiation(instantiation)
			}
			Constraint::Intension(intension) => TemplateCapture::Intension(intension),
			Constraint::Knapsack(knapsack) => TemplateCapture::Knapsack(knapsack),
			Constraint::Lex(lex) => TemplateCapture::Lex(lex),
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

impl<Identifier: From<String>> Transition<Identifier> {
	/// Parse a list of transitions.
	fn parse_vec<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Self>, D::Error> {
		/// Visitor for parsing a list of transitions.
		struct V<X>(PhantomData<X>);
		impl<X: From<String>> Visitor<'_> for V<X> {
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
