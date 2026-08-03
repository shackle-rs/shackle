//! Serialization of the XCSP3 (core) format
//!
//! XCSP3 is an integrated format for representing combinatorial constrained
//! problems, which can deal with mono/multi optimization, many types of
//! variables, cost functions, reification, views, annotations, variable
//! quantification, distributed, probabilistic and qualitative reasoning. It is
//! also compact, and easy to read and to parse. The objective of XCSP3 is to
//! ease the effort required to test and compare different algorithms by
//! providing a common test-bed of combinatorial constrained instances.
//!
//! This crate focuses on the (de-)serializeation of the XCSP3 format. It can be
//! used to parse an XCSP3 XML file into the provided rust types, or writing the
//! provided rust types to an XCSP3 XML file.
//!
//! # Getting Started
//!
//! Install `xcsp3-serde` and `quick-xml` for your package:
//!
//! ```bash
//! cargo add xcsp3-serde quick-xml
//! ```
//!
//! Once these dependencies have been installed to your crate, you could
//! deserialize a XCSP3 XML file as follows:
//!
//! ```
//! # use xcsp3_serde::Instance;
//! # use std::{fs::File, io::BufReader, path::Path};
//! # let path = Path::new("corpus/xcsp3_ex_001.xml");
//! // let path = Path::new("/lorem/ipsum/instance.xml");
//! let rdr = BufReader::new(File::open(path).unwrap());
//! let instance: Instance = quick_xml::de::from_reader(rdr).unwrap();
//! // ... process XCSP3 ...
//! ```
//!
//! If, however, you want to serialize a XCSP3 instance you could follow the
//! following fragment:
//!
//! ```
//! # use xcsp3_serde::Instance;
//! let instance = Instance::<String>::default();
//! // ... create XCSP3 instance ...
//! let xml_str = quick_xml::se::to_string(&instance).unwrap();
//! ```
//! Note that `quick_xml::se::to_writer`, using a buffered file writer, would be
//! preferred when writing larger instances.
//!
//! # Limitations
//!
//! Not all XCSP3 features are currently implemented, the functionality of
//! XCSP3-core is generally implemented and supported. This allows users to work
//! with the most common constraint types and representations. Future updates
//! will focus on expanding the range of supported XCSP3 features.

pub mod constraint;
pub mod error;
pub mod expression;

use std::{
	borrow::Cow,
	collections::{HashMap, VecDeque},
	fmt::{self, Display},
	hash::Hash,
	marker::PhantomData,
	ops::RangeInclusive,
};

use itertools::Itertools;
use nom::{
	branch::alt,
	bytes::streaming::tag,
	character::complete::{char, digit1},
	combinator::{all_consuming, map, map_res, opt, recognize},
	multi::many0,
	sequence::{delimited, pair, preceded},
	IResult, Parser,
};
pub use rangelist::RangeList;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
	constraint::{Constraint, MetaConstraint},
	error::UnrollError,
	expression::{identifier, int, range, sequence, whitespace_seperated, Exp, IntExp},
};

/// Definition of a k-dimensional arrays of variables
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Array<Identifier = String, Var = VarRef<Identifier>> {
	/// Name used to refer to the array
	pub identifier: Identifier,
	/// Comment by the user
	pub note: Option<String>,
	/// Dimensions of the array
	pub size: Vec<usize>,
	/// Domains of the variables contained within the array
	///
	/// Note that when several subsets of variables of an array have different
	/// domains, a rangelist is provided for each of these subsets. The first
	/// member of the tuple indicates the list of variables to which the domain
	/// definition applies. The special identifier `others` is used to declare a
	/// default domain for all other variables contained in the array.
	pub domains: Vec<(Vec<Var>, RangeList<IntVal>)>,
}

/// The way in which combinations of objectives are to be evaluated
#[derive(Clone, Debug, Default, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CombinationType {
	/// Objectives are lexicographically ordered
	///
	/// A solution is superceeded if it is better in the first objective, or if it
	/// is equal in the first objective and better in the second objective, and so
	/// on.
	#[default]
	Lexico,
	/// No objective is more important than another one
	///
	/// A solution is better than another if it is better in at least one
	/// objective and not worse in any other objective.
	Pareto,
}

/// The framework of an XCSP3 instance
///
/// The framework of an XCSP3 instance is used to determine the types of
/// constraints and variables that can be used in the instance. Different
/// frameworks correspond to different types of problems that can be expressed
/// in XCSP3.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FrameworkType {
	/// Constraint Satisfaction Problem
	///
	/// A discrete Constraint Network that constains a finite set of variables and
	/// a finite set of constraints.
	#[default]
	Csp,
	/// Constraint Optimization Problem
	///
	/// An instance is defined by a set of variables, a set of constraints, as for
	/// [`FrameworkType::Csp`], together with a set of objective functions.
	/// Mono-objective optimization is when only one objective function is
	/// present. Otherwise, this is multi-objective optimization.
	Cop,
	/// Weighted Constraint Satisfaction Problem
	///
	/// An extension to [`FrameworkType::Csp`] that relies on a valuation
	/// structure using weighted constraints.
	Wcsp,
	/// Fuzzy Constraint Satisfaction Problem
	///
	/// An extension of [`FrameworkType::Csp`] with fuzzy constraints. Each fuzzy
	/// constraint represents a fuzzy relation on its scope: it associates a value
	/// in \[0,1\], called membership degree, with each constraint tuple,
	/// indicating to what extent the tuple belongs to the relation and therefore
	/// satisfies the constraint.
	Fcsp,
	/// Quantified Constraint Satisfaction Problem
	///
	/// An extension of [`FrameworkType::Csp`] in which variables may be
	/// quantified universally or existentially.
	Qcsp,
	/// Extended Quantified Constraint Optimization Problem
	///
	/// An extension of [`FrameworkType::Qcsp`] to overcome some difficulties that
	/// may occur when modeling real problems with classical QCSP.
	QcspPlus,
	/// Quantified Constraint Optimization Problem
	///
	/// An extesion of [`FrameworkType::Qcsp`] that allows us to formally express
	/// preferences over [`FrameworkType::Qcsp`] strategies
	Qcop,
	/// Extended Quantified Constraint Optimization Problem
	///
	/// An extesion of [`FrameworkType::QcspPlus`] that allows us to formally
	/// express preferences over [`FrameworkType::QcspPlus`] strategies
	QcopPlus,
	/// Stochastic Constraint Satisfaction Problem
	Scsp,
	/// Stochastic Constraint Optimization Problem
	Scop,
	/// Qualitative Spatial Temporal Reasoning
	Qstr,
	/// Temporal Constraint Satisfaction Problem
	///
	/// In this framework, variables represent time points and temporal
	/// information is represented by a set of unary and binary constraints, each
	/// specifying a set of permitted intervals.
	Tcsp,
	/// Numerical Constraint Satisfaction Problem
	///
	/// An extension of [`FrameworkType::Csp`] in which variables are real numbers
	/// and constraints are relations between these variables.
	Ncsp,
	/// Numerical Constraint Optimization Problem
	///
	/// An extension of [`FrameworkType::Ncsp`] that includes objective functions.
	Ncop,
	/// Distributed Constraint Satisfaction Problem
	DisCsp,
	/// Distributed Weighted Constraint Satisfaction Problem
	DisWcsp,
}

/// An expression used to access a single element or a larger part of an array
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum Indexing {
	/// Accessing a single index of a dimension in an array
	Single(usize),
	/// Accessing a slice of a dimension in an array
	Range(usize, usize),
	/// Accessing the full range of an array
	Full,
}

/// XCSP3 problem instance
#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Instance<Identifier = String, Var = VarRef<Identifier>> {
	/// The type of the framework used to express the instance.
	pub ty: FrameworkType,
	/// Definitions of the single decision variables
	pub variables: Vec<Variable<Identifier>>,
	/// Definitions of the arrays of decision variables
	pub arrays: Vec<Array<Identifier, Var>>,
	/// Constraints that must be satisfied for a solution to be valid
	pub constraints: Vec<MetaConstraint<Identifier, Var>>,
	/// The objectives to be optimized
	pub objectives: Objectives<Identifier, Var>,
}

/// An assignment from a list of variables to a list of values
///
/// This structure is used both to represent an elementary constraint in an
/// instance, and to represent the solution to an instance.
#[derive(Clone, Debug, PartialEq, Hash, Deserialize)]
#[serde(bound(deserialize = "Identifier: From<String>, Var: IntoVar"))]
pub struct Instantiation<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the constraint
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// The type of instantiation
	///
	/// This field is used to distinguish between different types of solutions,
	/// and signal whether the solution is optimal or not. When this type is used
	/// as a constraint, then this field is ignore and can be set to `None`.
	#[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
	pub ty: Option<InstantiationType>,
	/// The objective cost of the instantiation
	///
	/// This field is used to represent the cost of a solution, and is only used
	/// when the instantiation type is used to represent a solution. When this
	/// type is used as a constraint, then this field is ignore and can be set to
	/// `None`.
	#[serde(rename = "@cost", default, skip_serializing_if = "Option::is_none")]
	pub cost: Option<IntVal>,
	#[serde(
		deserialize_with = "VarRef::parse_vec",
		serialize_with = "serialize_list"
	)]
	/// List of variables that are assigned values
	pub list: Vec<Var>,
	/// List of values assigned to the variables
	///
	/// A [`None`] entry represents the star `*`, marking a variable that is
	/// allowed to take any value.
	#[serde(
		deserialize_with = "deserialize_opt_int_vals",
		serialize_with = "serialize_opt_list"
	)]
	pub values: Vec<Option<IntVal>>,
}

/// The type of instantiation
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstantiationType {
	/// A solution that satisfies all constraints
	Solution,
	/// A solution that satisfies all constraints and is optimal with regards to
	/// the objective function(s)
	Optimum,
}

/// Trait used to construct variable references during deserialization
pub trait IntoVar {
	/// Constructs a variable reference from a string-based representation
	fn into_var(var: VarRef) -> Self;
}

/// Type used to represent integer values
pub type IntVal = i64;

/// Type used to capture optional metadata that can be attached to most XCSP3
/// elements
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>",
	serialize = "Identifier: Display"
))]
pub struct MetaInfo<Identifier> {
	/// Name assigned to the element
	#[serde(
		rename = "@id",
		default,
		skip_serializing_if = "Option::is_none",
		deserialize_with = "deserialize_ident",
		serialize_with = "serialize_ident"
	)]
	pub identifier: Option<Identifier>,
	/// Comment from the user about the element
	#[serde(rename = "@note", default, skip_serializing_if = "Option::is_none")]
	pub note: Option<String>,
}

/// Objective function
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(
	rename_all = "camelCase",
	bound(
		deserialize = "Identifier: From<String>, Var: IntoVar",
		serialize = "Identifier: Display, Var: Display"
	)
)]
pub enum Objective<Identifier = String, Var = VarRef<Identifier>> {
	/// An objective function where the goal is to find the smallest possible
	/// value.
	#[serde(rename = "minimize")]
	Minimize(ObjExp<Identifier, Var>),
	/// An objective function where the goal is to find the largest possible
	/// value.
	#[serde(rename = "maximize")]
	Maximize(ObjExp<Identifier, Var>),
}

/// Collection of objective functions
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct Objectives<Identifier = String, Var = VarRef<Identifier>> {
	/// Combinator to aggregate multiple objectives
	#[serde(default, rename = "@combination")]
	pub combination: CombinationType,
	/// List of objectives functions
	#[serde(rename = "$value")]
	pub objectives: Vec<Objective<Identifier, Var>>,
}

/// Expression used to represent an objective function
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>, Var: IntoVar",
	serialize = "Identifier: Display, Var: Display"
))]
pub struct ObjExp<Identifier = String, Var = VarRef<Identifier>> {
	/// Optional metadata for the objective
	#[serde(flatten)]
	pub info: MetaInfo<Identifier>,
	/// Evaluation method for the list of expressions
	#[serde(alias = "@type", default)]
	pub ty: ObjType,
	/// List of expressions
	#[serde(
		alias = "$text",
		deserialize_with = "IntExp::parse_vec",
		serialize_with = "serialize_list"
	)]
	pub list: Vec<IntExp<Var>>,
	/// List of coefficients to apply to the expressions
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub coeffs: Vec<IntVal>,
}

/// Evaluation method for the list of expressions in an objective function
#[derive(Clone, Debug, Default, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ObjType {
	/// Sum of the expressions
	#[default]
	Sum,
	/// Minimum value of the expressions
	Minimum,
	/// Maximum value of the expressions
	Maximum,
	/// Number of different values among the expressions
	NValues,
	/// Lexico order of the expressions
	Lex,
}

/// Representation of a placeholder to be replaced in a meta-constraint.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Placeholder {
	/// Placeholder replaced by the argument at the given position.
	Position(usize),
	/// Placeholder replaced by all arguments larger than the largest given
	/// position.
	Remainder,
}

/// Reference to a variable or array element
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum SimpleRef<Identifier> {
	/// Reference to a variable
	Ident(Identifier),
	/// Reference to an array element
	ArrayAccess(Identifier, Vec<usize>),
}

/// Definition of a variable
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(
	deserialize = "Identifier: From<String>",
	serialize = "Identifier: Display"
))]
pub struct Variable<Identifier = String> {
	/// Name of the variable
	#[serde(
		rename = "@id",
		deserialize_with = "from_string",
		serialize_with = "as_str"
	)]
	pub identifier: Identifier,
	/// Comment by the user about the variable
	#[serde(rename = "@note", default, skip_serializing_if = "Option::is_none")]
	pub note: Option<String>,
	/// List of possible values the variable can take
	#[serde(
		rename = "$text",
		deserialize_with = "deserialize_range_list",
		serialize_with = "serialize_range_list"
	)]
	pub domain: RangeList<IntVal>,
}

/// Reference to a variable, array element, array slice, or placeholder in a
/// group.
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum VarRef<Identifier = String> {
	/// Reference to a variable
	Ident(Identifier),
	/// Reference to an array element or slice
	ArrayAccess(Identifier, Vec<Indexing>),
	/// Placeholders to be replaced by other references
	Placeholder(Placeholder),
}

/// Serialize the value by converting it to a string
fn as_str<S: Serializer, I: Display>(value: &I, serializer: S) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(&value.to_string())
}

/// Combine a list of integer ranges into a single range list
fn collect_range_list<I: IntoIterator<Item = RangeInclusive<IntVal>>>(
	iter: I,
) -> RangeList<IntVal> {
	let mut r: Vec<_> = iter.into_iter().collect();
	r.sort_by_key(|i| *i.start());
	let mut it = r.into_iter();
	let mut ranges = Vec::new();
	let mut cur = it.next().unwrap();
	for next in it {
		if *cur.end() >= (next.start() - 1) {
			cur = *cur.start()..=*next.end()
		} else {
			ranges.push(cur);
			cur = next;
		}
	}
	ranges.push(cur);
	ranges.into_iter().collect()
}

/// Deserialize a string as an identifier
fn deserialize_ident<'de, D: Deserializer<'de>, Identifier: From<String>>(
	deserializer: D,
) -> Result<Option<Identifier>, D::Error> {
	/// Visitor to deserialize a string as an identifier
	struct V<X>(PhantomData<X>);
	impl<X: From<String>> Visitor<'_> for V<X> {
		type Value = Option<X>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("an identfier")
		}

		fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
			Ok(Some(s.trim().to_owned().into()))
		}
	}
	let visitor = V::<Identifier>(PhantomData);
	deserializer.deserialize_str(visitor)
}

/// Deserialize a string as a list of integers
fn deserialize_int_vals<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<Vec<IntVal>, D::Error> {
	/// Visitor to parse a list of integers
	struct V;
	impl Visitor<'_> for V {
		type Value = Vec<IntVal>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a list of integers")
		}

		fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let v = v.trim();
			let (_, vals) = all_consuming(whitespace_seperated(repeated(int)))
				.parse(v)
				.map_err(|_| E::custom(format!("invalid list of integers {v}")))?;
			Ok(vals.into_iter().flatten().collect())
		}
	}
	deserializer.deserialize_str(V)
}

/// Deserialize a string as a list of integers, where `*` denotes that any value
/// is allowed
fn deserialize_opt_int_vals<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<Vec<Option<IntVal>>, D::Error> {
	/// Visitor to parse a list of integers
	struct V;
	impl Visitor<'_> for V {
		type Value = Vec<Option<IntVal>>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a list of integers")
		}

		fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let v = v.trim();
			let (_, vals) = all_consuming(whitespace_seperated(repeated(alt((
				map(char('*'), |_| None),
				map(int, Some),
			)))))
			.parse(v)
			.map_err(|_| E::custom(format!("invalid list of integers {v}")))?;
			Ok(vals.into_iter().flatten().collect())
		}
	}
	deserializer.deserialize_str(V)
}

/// Parser combinator for a value that can be followed by `x<count>` to indicate
/// that it occurs `count` times in a row
fn repeated<'a, O: Clone>(
	p: impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
) -> impl Parser<&'a str, Output = Vec<O>> {
	map(pair(p, opt(preceded(char('x'), idx_int))), |(v, n)| {
		vec![v; n.unwrap_or(1)]
	})
}

/// Deserialize a string as a range list
fn deserialize_range_list<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<RangeList<IntVal>, D::Error> {
	/// Visitor for deserializing a range list
	struct V;
	impl Visitor<'_> for V {
		type Value = RangeList<IntVal>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a list of ranges")
		}

		fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let v = v.trim();
			let (_, r) = all_consuming(whitespace_seperated(range))
				.parse(v)
				.map_err(|_| E::custom(format!("invalid list of ranges `{v}")))?;
			Ok(collect_range_list(r))
		}
	}
	let visitor = V;
	deserializer.deserialize_str(visitor)
}

/// Deserialize a string as a size expression
fn deserialize_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<usize>, D::Error> {
	/// Visitor for deserializing a size expression
	struct V;
	impl Visitor<'_> for V {
		type Value = Vec<usize>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("an array size expression")
		}

		fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let v = v.trim();
			let (_, r) = all_consuming(sequence(delimited(
				char::<_, nom::error::Error<&str>>('['),
				map_res(recognize(digit1), str::parse),
				char(']'),
			)))
			.parse(v)
			.map_err(|_| E::custom(format!("invalid array size expression `{v}'")))?;
			Ok(r)
		}
	}
	let visitor = V;
	deserializer.deserialize_str(visitor)
}

/// Deserialize a string and call the `FromStr` implementation
fn from_string<'de, D: Deserializer<'de>, I: From<String>>(deserializer: D) -> Result<I, D::Error> {
	let s: Cow<'_, str> = Deserialize::deserialize(deserializer)?;
	Ok(s.trim().to_owned().into())
}

/// Parser combinator that parses an integer from a string
fn idx_int(input: &str) -> IResult<&str, usize> {
	let (input, i): (_, usize) = map_res(recognize(digit1), str::parse).parse(input)?;
	Ok((input, i))
}

/// Parser combinator that parses a range of integers from a string
fn idx_range(input: &str) -> IResult<&str, RangeInclusive<usize>> {
	let (input, lb) = idx_int(input)?;
	if let (input, Some(_)) = opt(tag("..")).parse(input)? {
		let (input, ub) = idx_int(input)?;
		Ok((input, lb..=ub))
	} else {
		Ok((input, lb..=lb))
	}
}

/// Serialize a list of values by printing them to strings and joining them with
/// spaces
fn serialize_list<S: Serializer, T: Display>(exps: &[T], serializer: S) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(
		&exps
			.iter()
			.map(|e| format!("{}", e))
			.collect::<Vec<_>>()
			.join(" "),
	)
}

/// Serialize a list of optional values as a string, writing [`None`] as `*`
fn serialize_opt_list<S: Serializer, T: Display>(
	exps: &[Option<T>],
	serializer: S,
) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(
		&exps
			.iter()
			.map(|e| match e {
				Some(e) => e.to_string(),
				None => "*".to_owned(),
			})
			.collect::<Vec<_>>()
			.join(" "),
	)
}

/// Serialize an optional identifier as a string
fn serialize_ident<S: Serializer, Identifier: Display>(
	identifier: &Option<Identifier>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(&format!("{}", identifier.as_ref().unwrap()))
}

/// Serialize a list of integers as a string of ranges separated by spaces
fn serialize_range_list<S: Serializer>(
	exps: &RangeList<IntVal>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(
		&exps
			.into_iter()
			.map(|e| {
				if e.start() == e.end() {
					e.start().to_string()
				} else {
					format!("{}..{}", e.start(), e.end())
				}
			})
			.collect::<Vec<_>>()
			.join(" "),
	)
}

/// Serialize a list of dimensions as a string size expression
fn serialize_size<S: Serializer>(exps: &[usize], serializer: S) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(
		&exps
			.iter()
			.map(|e| format!("[{}]", e))
			.collect::<Vec<_>>()
			.join(""),
	)
}

impl<Identifier: Clone + Hash + Eq + ToString> Array<Identifier, VarRef<Identifier>> {
	/// Expand the domain definitions of the array domain defintiions into
	/// [`SimpleRef`].
	pub fn unroll(&self) -> Result<Array<Identifier, SimpleRef<Identifier>>, UnrollError> {
		let size_wrap: HashMap<_, _> = Some((self.identifier.clone(), &self.size[..]))
			.into_iter()
			.collect();
		let mut domains = Vec::with_capacity(self.domains.len());
		for (v, d) in &self.domains {
			let mut res: Vec<SimpleRef<_>> = Vec::new();
			for x in v {
				res.extend(
					x.unroll(&size_wrap, &[], &[])?
						.into_iter()
						.map(|x| match x {
							Exp::Var(v) => v,
							_ => unreachable!(),
						}),
				);
			}
			domains.push((res, d.clone()));
		}
		Ok(Array {
			identifier: self.identifier.clone(),
			note: self.note.clone(),
			size: self.size.clone(),
			domains,
		})
	}
}

impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de> for Array<Identifier, Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Helper struct to deserialize the content of the <domain> element
		#[derive(Deserialize)]
		#[serde(bound = "Var: IntoVar")]
		struct DomainStruct<Var> {
			/// for attribute
			#[serde(rename = "@for", deserialize_with = "VarRef::parse_vec")]
			vars: Vec<Var>,
			/// content of element
			#[serde(rename = "$text", deserialize_with = "deserialize_range_list")]
			domain: RangeList<IntVal>,
		}
		/// Helper enum to deserialize the content of the <array> element
		#[derive(Deserialize)]
		#[serde(bound = " Var: IntoVar")]
		enum Domain<'a, Var> {
			/// multiple <domain> elements
			#[serde(rename = "domain")]
			Domain(Vec<DomainStruct<Var>>),
			/// single string content
			#[serde(rename = "$text")]
			Direct(Cow<'a, str>),
		}
		/// Helper struct to deserialize an <array> element
		#[derive(Deserialize)]
		#[serde(bound = "Identifier: From<String>, Var: IntoVar")]
		struct Array<'a, Identifier, Var> {
			/// id attribute
			#[serde(rename = "@id", deserialize_with = "from_string")]
			identifier: Identifier,
			/// optional note attribute
			#[serde(rename = "@note", default, skip_serializing_if = "Option::is_none")]
			note: Option<String>,
			/// size attribute
			#[serde(rename = "@size", deserialize_with = "deserialize_size")]
			size: Vec<usize>,
			/// content of the element
			#[serde(rename = "$value")]
			domain: Domain<'a, Var>,
		}
		let x = Array::deserialize(deserializer)?;
		let domains = match x.domain {
			Domain::Domain(v) => v.into_iter().map(|d| (d.vars, d.domain)).collect(),
			Domain::Direct(s) => {
				let s = s.trim();
				let s = all_consuming(whitespace_seperated(range))
					.parse(s.as_ref())
					.map_err(|_| {
						serde::de::Error::custom(format!("unable to parse ranges from `{s}'"))
					})?;
				vec![(
					vec![Var::into_var(VarRef::Ident("others".to_owned()))],
					collect_range_list(s.1),
				)]
			}
		};
		Ok(Self {
			identifier: x.identifier,
			note: x.note,
			size: x.size,
			domains,
		})
	}
}

impl<Identifier: Display> Serialize for Array<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		/// Helper struct to serialize the domain expression
		#[derive(Serialize)]
		#[serde(bound = "Identifier: Display")]
		struct DomainStruct<'a, Identifier: Display> {
			/// Variable references serialized as the for attribute
			#[serde(rename = "@for", serialize_with = "serialize_list")]
			vars: &'a Vec<VarRef<Identifier>>,
			/// RangeList serialized as the string content of the element
			#[serde(rename = "$text", serialize_with = "serialize_range_list")]
			domain: &'a RangeList<IntVal>,
		}
		/// Domain expression serialized as the <domain> elements
		#[derive(Serialize)]
		#[serde(bound = "Identifier: Display")]
		enum Domain<'a, Identifier: Display> {
			/// Domain expression serialized as the <domain> elements
			#[serde(rename = "domain")]
			Domain(DomainStruct<'a, Identifier>),
		}
		#[derive(Serialize)]
		#[serde(bound = "Identifier: Display")]
		struct Array<'a, Identifier: Display> {
			/// Identifier serialized as the id attribute
			#[serde(rename = "@id", serialize_with = "as_str")]
			identifier: &'a Identifier,
			/// String serialized as the note attribute
			#[serde(rename = "@note", default, skip_serializing_if = "Option::is_none")]
			note: &'a Option<String>,
			/// Size expression serialized as the size attribute
			#[serde(rename = "@size", serialize_with = "serialize_size")]
			size: &'a Vec<usize>,
			/// Domain expressions serialized as the element content
			#[serde(rename = "$value")]
			domain: Vec<Domain<'a, Identifier>>,
		}
		let domain = self
			.domains
			.iter()
			.map(|(v, d)| Domain::Domain(DomainStruct { vars: v, domain: d }))
			.collect();
		let x = Array {
			identifier: &self.identifier,
			note: &self.note,
			size: &self.size,
			domain,
		};
		x.serialize(serializer)
	}
}

impl<Identifier: Clone + Eq + Hash + ToString> Instance<Identifier, VarRef<Identifier>> {
	/// Create a flat list of constraints, instantiating all
	/// [`Group`](constraint::Group)s and [`Slide`](constraint::Slide)s,
	/// extracting constraints from [`Block`](constraint::Block)s, and expanding
	/// all slicing operations.
	pub fn unroll_constraints(
		&self,
	) -> Result<Vec<Constraint<Identifier, SimpleRef<Identifier>>>, UnrollError> {
		let arrays: HashMap<Identifier, &[usize]> = self
			.arrays
			.iter()
			.map(|arr| (arr.identifier.clone(), &arr.size[..]))
			.collect();

		let mut flat = Vec::new();
		let mut metas = VecDeque::new();
		metas.push_back(&self.constraints);
		while let Some(cons) = metas.pop_front() {
			for con in cons {
				match con {
					MetaConstraint::Group(group) => flat.extend(group.unroll(&arrays)?),
					MetaConstraint::Slide(slide) => flat.extend(slide.unroll(&arrays)?),
					MetaConstraint::Block(block) => metas.push_back(&block.constraints),
					MetaConstraint::Constraint(c) => flat.push(c.unroll(&arrays, &[], &[])?),
				}
			}
		}
		Ok(flat)
	}
}

impl<Identifier> Default for Instance<Identifier> {
	fn default() -> Self {
		Self {
			ty: Default::default(),
			variables: Default::default(),
			arrays: Default::default(),
			constraints: Default::default(),
			objectives: Default::default(),
		}
	}
}

impl<'de, Identifier: From<String>, Var: IntoVar> Deserialize<'de> for Instance<Identifier, Var> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Deserialized content of <variables> element
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: From<String>, Var: IntoVar"))]
		enum V<Identifier, Var> {
			/// Deserialized <var> element
			#[serde(rename = "var")]
			Variable(Variable<Identifier>),
			/// Deserialized <array> element
			#[serde(rename = "array")]
			Array(Array<Identifier, Var>),
		}
		/// Deserialized <variables> element
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: From<String>, Var: IntoVar"))]
		struct Variables<Identifier, Var> {
			/// Deserialized content of <variables> element
			#[serde(rename = "$value")]
			vars: Vec<V<Identifier, Var>>,
		}
		/// Deserialized <constraints> element
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: From<String>, Var: IntoVar"))]
		struct Constraints<Identifier, Var> {
			/// Deserialized content of <constraints> element
			#[serde(rename = "$value")]
			content: Vec<MetaConstraint<Identifier, Var>>,
		}
		/// Deserialized <instance> element
		#[derive(Deserialize)]
		#[serde(bound(deserialize = "Identifier: From<String>, Var: IntoVar"))]
		struct Instance<Identifier, Var> {
			/// Deserialized type attribute
			#[serde(rename = "@type")]
			ty: FrameworkType,
			/// Deserialized <variables> element
			variables: Option<Variables<Identifier, Var>>,
			/// Deserialized <constraints> element
			constraints: Option<Constraints<Identifier, Var>>,
			/// Deserialized <objectives> element
			#[serde(default = "Objectives::default")]
			objectives: Objectives<Identifier, Var>,
		}
		let inst: Instance<Identifier, Var> = Deserialize::deserialize(deserializer)?;
		let mut variables = Vec::new();
		let mut arrays = Vec::new();
		for v in inst.variables.map(|v| v.vars).into_iter().flatten() {
			match v {
				V::Variable(var) => variables.push(var),
				V::Array(arr) => arrays.push(arr),
			}
		}
		Ok(Self {
			ty: inst.ty,
			variables,
			arrays,
			constraints: inst.constraints.map_or_else(Vec::new, |c| c.content),
			objectives: inst.objectives,
		})
	}
}

impl<Identifier: Serialize + Display> Serialize for Instance<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		/// Helper struct to serialize the <variables> element
		#[derive(Serialize)]
		struct Variables<'a, Identifier: Display> {
			/// Values serialized as <var> elements
			var: &'a Vec<Variable<Identifier>>,
			/// Values serialized as <array> elements
			array: &'a Vec<Array<Identifier>>,
		}
		impl<Identifier: Display> Variables<'_, Identifier> {
			/// Check whether there are any variables or arrays to serialize
			fn is_empty(&self) -> bool {
				self.var.is_empty() && self.array.is_empty()
			}
		}
		/// Helper struct to serialize the <constraints> element
		#[derive(Serialize)]
		struct Constraints<'a, Identifier: Display> {
			/// Constraints to be serialized
			#[serde(rename = "$value")]
			content: &'a Vec<MetaConstraint<Identifier>>,
		}
		impl<Identifier: Display> Constraints<'_, Identifier> {
			/// Check whether there are any constraints to serialize
			fn is_empty(&self) -> bool {
				self.content.is_empty()
			}
		}
		/// Helper struct to serialize the <instance> element
		#[derive(Serialize)]
		#[serde(rename = "instance")]
		struct Instance<'a, Identifier: Display> {
			/// Value serialized as the type attribute
			#[serde(rename = "@type")]
			ty: FrameworkType,
			/// Value serialized as the <variables> element
			#[serde(skip_serializing_if = "Variables::is_empty")]
			variables: Variables<'a, Identifier>,
			/// Value serialized as the <constraints> element
			#[serde(skip_serializing_if = "Constraints::is_empty")]
			constraints: Constraints<'a, Identifier>,
			/// Value serialized as the <objectives> element
			#[serde(skip_serializing_if = "Objectives::is_empty")]
			objectives: &'a Objectives<Identifier>,
		}
		let x = Instance {
			ty: self.ty,
			variables: Variables {
				var: &self.variables,
				array: &self.arrays,
			},
			constraints: Constraints {
				content: &self.constraints,
			},
			objectives: &self.objectives,
		};
		Serialize::serialize(&x, serializer)
	}
}

// Note: flatten of MetaInfo does not seem to work here
// (https://github.com/tafia/quick-xml/issues/761)
impl<Identifier: Display, Var: Display> Serialize for Instantiation<Identifier, Var> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		/// Helper struct to serialize the instantiation element
		#[derive(Serialize)]
		#[serde(
			rename = "instantiation",
			bound(serialize = "Identifier: Display, Var: Display")
		)]
		struct Instantiation<'a, Identifier, Var> {
			/// Value serialized as the id attribute
			#[serde(
				rename = "@id",
				skip_serializing_if = "Option::is_none",
				serialize_with = "serialize_ident"
			)]
			identifier: &'a Option<Identifier>,
			/// Value serialized as the note attribute
			#[serde(rename = "@note", skip_serializing_if = "Option::is_none")]
			note: &'a Option<String>,
			/// Value serialized as the type attribute
			#[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
			ty: &'a Option<InstantiationType>,
			/// Value serialized as the cost attribute
			#[serde(rename = "@cost", skip_serializing_if = "Option::is_none")]
			cost: &'a Option<IntVal>,
			/// Variable references serialized as <list>
			#[serde(serialize_with = "serialize_list")]
			list: &'a Vec<Var>,
			/// Values serialized as <values>
			#[serde(serialize_with = "serialize_opt_list")]
			values: &'a Vec<Option<IntVal>>,
		}
		Instantiation {
			identifier: &self.info.identifier,
			note: &self.info.note,
			ty: &self.ty,
			cost: &self.cost,
			list: &self.list,
			values: &self.values,
		}
		.serialize(serializer)
	}
}

impl<Identifier> Objectives<Identifier> {
	/// Check whether there are no objectives.
	pub fn is_empty(&self) -> bool {
		self.objectives.is_empty()
	}
}

impl<Identifier: Clone + Hash + Eq + ToString> ObjExp<Identifier, VarRef<Identifier>> {
	/// Expand the domain definitions of the array domain defintiions into
	/// [`SimpleRef`].
	pub fn unroll(
		&self,
		instance: &Instance<Identifier, VarRef<Identifier>>,
	) -> Result<ObjExp<Identifier, SimpleRef<Identifier>>, UnrollError> {
		let arrays: HashMap<Identifier, &[usize]> = instance
			.arrays
			.iter()
			.map(|arr| (arr.identifier.clone(), &arr.size[..]))
			.collect();

		let list = self
			.list
			.iter()
			.map(|v| v.unroll(&arrays, &[], &[]))
			.collect::<Result<Vec<_>, _>>()?
			.into_iter()
			.flatten()
			.collect();

		Ok(ObjExp {
			info: self.info.clone(),
			ty: self.ty.clone(),
			list,
			coeffs: self.coeffs.clone(),
		})
	}
}

impl<Identifier, Var> Default for Objectives<Identifier, Var> {
	fn default() -> Self {
		Self {
			combination: CombinationType::default(),
			objectives: Vec::new(),
		}
	}
}

impl Display for Placeholder {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Placeholder::Position(i) => write!(f, "%{}", i),
			Placeholder::Remainder => write!(f, "%..."),
		}
	}
}

impl<Identifier: Clone + Hash + Eq + ToString> VarRef<Identifier> {
	/// Expand the reference into the list of expressions it denotes.
	///
	/// Placeholders are resolved using `args` (one entry per `<args>` token, so
	/// a single placeholder can stand for a whole list) and `remainder` (the
	/// flattened tokens matched by `%...`). Array slices are expanded using the
	/// dimensions given in `arrays`.
	pub(crate) fn unroll(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Vec<Exp<SimpleRef<Identifier>>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Vec<Exp<SimpleRef<Identifier>>>, UnrollError> {
		match self {
			&VarRef::Placeholder(Placeholder::Position(i)) if i < args.len() => Ok(args[i].clone()),
			&VarRef::Placeholder(Placeholder::Position(i)) => Err(UnrollError::ArgMissing {
				placeholder: i,
				args_len: args.len(),
			}),
			VarRef::Placeholder(Placeholder::Remainder) => Ok(remainder.to_vec()),
			VarRef::Ident(ident) => Ok(vec![Exp::Var(SimpleRef::Ident(ident.clone()))]),
			VarRef::ArrayAccess(ident, indexings) => {
				let Some(size) = arrays.get(ident) else {
					return Err(UnrollError::UnknownIdentifier(ident.to_string()));
				};
				if indexings.len() != size.len() {
					return Err(UnrollError::UnexpectedIndexes {
						expected_len: size.len(),
						args_len: indexings.len(),
					});
				}
				Ok(indexings
					.iter()
					.enumerate()
					.map(|(i, idx)| match idx {
						&Indexing::Single(i) => i..=i,
						&Indexing::Range(start, end) => start..=end,
						Indexing::Full => 0..=(size[i] - 1),
					})
					.multi_cartesian_product()
					.map(|idxs| Exp::Var(SimpleRef::ArrayAccess(ident.clone(), idxs)))
					.collect())
			}
		}
	}

	/// Expand an array slice (e.g. `x[][]`) into the rows of the matrix it
	/// denotes.
	///
	/// The width of a row is given by the last indexing operation, since
	/// [`Self::unroll`] varies the last index the fastest.
	pub(crate) fn unroll_matrix(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Vec<Exp<SimpleRef<Identifier>>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Vec<Vec<Exp<SimpleRef<Identifier>>>>, UnrollError> {
		let VarRef::ArrayAccess(ident, indexings) = self else {
			return Err(UnrollError::UnexpectedIndexes {
				expected_len: 2,
				args_len: 0,
			});
		};
		let Some(size) = arrays.get(ident) else {
			return Err(UnrollError::UnknownIdentifier(ident.to_string()));
		};
		let flat = self.unroll(arrays, args, remainder)?;
		let row_len = match indexings.last() {
			Some(&Indexing::Single(_)) => 1,
			Some(&Indexing::Range(start, end)) => end - start + 1,
			// `unroll` has already checked that the number of indexing
			// operations matches the number of dimensions of the array.
			Some(Indexing::Full) => size[indexings.len() - 1],
			None => {
				return Err(UnrollError::UnexpectedIndexes {
					expected_len: 2,
					args_len: 0,
				})
			}
		};
		Ok(flat.chunks(row_len).map(<[_]>::to_vec).collect())
	}

	/// Same as [`Self::unroll`], but requires that the reference denotes exactly
	/// one expression.
	pub(crate) fn unroll_single(
		&self,
		arrays: &HashMap<Identifier, &[usize]>,
		args: &[Vec<Exp<SimpleRef<Identifier>>>],
		remainder: &[Exp<SimpleRef<Identifier>>],
	) -> Result<Exp<SimpleRef<Identifier>>, UnrollError> {
		let res = self.unroll(arrays, args, remainder)?;
		match &res[..] {
			[exp] => Ok(exp.clone()),
			_ => Err(UnrollError::UnexpectedLength {
				expected_len: 1,
				args_len: res.len(),
			}),
		}
	}
}

impl VarRef {
	/// Parse a list of variable references.
	fn parse_vec<'de, D: Deserializer<'de>, R: IntoVar>(
		deserializer: D,
	) -> Result<Vec<R>, D::Error> {
		/// Visitor for parsing a list of variable references.
		struct V<X>(PhantomData<X>);
		impl<X: From<String>> Visitor<'_> for V<X> {
			type Value = Vec<VarRef<X>>;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a list of variable references")
			}

			fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
				let v = v.trim();
				let (_, v) = all_consuming(whitespace_seperated(VarRef::parse))
					.parse(v)
					.map_err(|_| E::custom(format!("invalid variable references `{v}'")))?;
				Ok(v)
			}
		}
		let visitor = V::<String>(PhantomData);
		Ok(deserializer
			.deserialize_str(visitor)?
			.into_iter()
			.map(R::into_var)
			.collect())
	}
}

impl<Identifier: From<String>> VarRef<Identifier> {
	/// Parse a variable reference.
	pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
		// First try to see whether the variable is a placeholder
		let placeholder: IResult<&str, Placeholder> = preceded(
			char('%'),
			alt((
				map(digit1, |p: &str| Placeholder::Position(p.parse().unwrap())),
				map(tag("..."), |_| Placeholder::Remainder),
			)),
		)
		.parse(input);
		if let Ok((input, placeholder)) = placeholder {
			return Ok((input, Self::Placeholder(placeholder)));
		}
		// Parse a normal identifier
		let (input, ident) = identifier(input)?;
		// Optionally add an array access tail
		let (input, v) = many0(delimited(char('['), opt(idx_range), char(']'))).parse(input)?;
		// Create VarRef object
		Ok((
			input,
			if v.is_empty() {
				VarRef::Ident(ident)
			} else {
				let v = v
					.into_iter()
					.map(|r| {
						r.map(|r| {
							if r.start() == r.end() {
								Indexing::Single(*r.start())
							} else {
								Indexing::Range(*r.start(), *r.end())
							}
						})
						.unwrap_or(Indexing::Full)
					})
					.collect();
				VarRef::ArrayAccess(ident, v)
			},
		))
	}
}

impl<Identifier: Display> Display for VarRef<Identifier> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			VarRef::Ident(ident) => ident.fmt(f),
			VarRef::ArrayAccess(ident, v) => {
				write!(
					f,
					"{}{}",
					ident,
					v.iter()
						.map(|i| format!(
							"[{}]",
							match i {
								Indexing::Single(v) => v.to_string(),
								Indexing::Range(a, b) => format!("{}..{}", a, b),
								Indexing::Full => String::new(),
							}
						))
						.collect::<Vec<_>>()
						.join("")
				)
			}
			VarRef::Placeholder(placeholder) => placeholder.fmt(f),
		}
	}
}

impl<I: From<String>> IntoVar for VarRef<I> {
	fn into_var(var: VarRef) -> Self {
		match var {
			VarRef::Ident(s) => VarRef::Ident(I::from(s)),
			VarRef::ArrayAccess(s, idxs) => VarRef::ArrayAccess(I::from(s), idxs),
			VarRef::Placeholder(p) => VarRef::Placeholder(p),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{fmt::Debug, fs::File, io::BufReader, path::Path};

	use expect_test::ExpectFile;
	use serde::{de::DeserializeOwned, Serialize};

	use crate::{Instance, Instantiation};

	fn test_successful_serialization<T: Debug + DeserializeOwned + Serialize + PartialEq>(
		file: &Path,
		exp: ExpectFile,
	) {
		let rdr = BufReader::new(File::open(file).unwrap());
		let inst: T = quick_xml::de::from_reader(rdr).unwrap();
		exp.assert_debug_eq(&inst);
		let output = quick_xml::se::to_string(&inst).unwrap();
		let inst2: T = quick_xml::de::from_str(&output).unwrap();
		assert_eq!(inst, inst2)
	}

	/// Round-trip the instance, and additionally check the flat list of
	/// constraints produced by [`Instance::unroll_constraints`].
	fn test_successful_unroll(file: &Path, exp: ExpectFile, unrolled: ExpectFile) {
		test_successful_serialization::<Instance>(file, exp);
		let rdr = BufReader::new(File::open(file).unwrap());
		let inst: Instance = quick_xml::de::from_reader(rdr).unwrap();
		unrolled.assert_debug_eq(&inst.unroll_constraints().unwrap());
	}

	macro_rules! test_file {
		($file:ident) => {
			test_file!($file, Instance);
		};
		($file:ident, $t:ident) => {
			#[test]
			fn $file() {
				test_successful_serialization::<$t>(
					std::path::Path::new(&format!("./corpus/{}.xml", stringify!($file))),
					expect_test::expect_file![&format!(
						"../corpus/{}.debug.txt",
						stringify!($file)
					)],
				)
			}
		};
	}

	macro_rules! test_unroll {
		($file:ident) => {
			#[test]
			fn $file() {
				test_successful_unroll(
					std::path::Path::new(&format!("./corpus/{}.xml", stringify!($file))),
					expect_test::expect_file![&format!(
						"../corpus/{}.debug.txt",
						stringify!($file)
					)],
					expect_test::expect_file![&format!(
						"../corpus/{}.unroll.txt",
						stringify!($file)
					)],
				)
			}
		};
	}

	test_file!(knapsack);
	// A `<args>` token can expand to a whole list, so `%0` must bind to all of
	// `x[0][]` and `%1` to the value that follows it.
	test_unroll!(group_list_arg);

	test_file!(xcsp3_ex_001);
	test_file!(xcsp3_ex_002);
	test_file!(xcsp3_ex_003);
	test_file!(xcsp3_ex_004);
	test_file!(xcsp3_ex_005);
	test_file!(xcsp3_ex_006);
	test_file!(xcsp3_ex_007);
	// test_file!(xcsp3_ex_008);
	// test_file!(xcsp3_ex_009);
	// test_file!(xcsp3_ex_010);
	// test_file!(xcsp3_ex_011);
	// test_file!(xcsp3_ex_012);
	// test_file!(xcsp3_ex_013);
	// test_file!(xcsp3_ex_014);
	// test_file!(xcsp3_ex_015);
	// test_file!(xcsp3_ex_016);
	// test_file!(xcsp3_ex_017);
	test_file!(xcsp3_ex_018);
	test_file!(xcsp3_ex_019);
	// test_file!(xcsp3_ex_020);
	test_file!(xcsp3_ex_021);
	test_file!(xcsp3_ex_022);
	test_file!(xcsp3_ex_023, Instantiation);
	test_file!(xcsp3_ex_024);
	test_file!(xcsp3_ex_025, Instantiation);
	test_file!(xcsp3_ex_026, Instantiation);
	test_file!(xcsp3_ex_027, Instantiation);
	test_file!(xcsp3_ex_028, Instantiation);
	test_file!(xcsp3_ex_029);
	test_file!(xcsp3_ex_030);
	test_file!(xcsp3_ex_031);
	test_file!(xcsp3_ex_032);
	test_file!(xcsp3_ex_033);
	test_file!(xcsp3_ex_034);
	test_file!(xcsp3_ex_035);
	test_file!(xcsp3_ex_036);
	test_file!(xcsp3_ex_037);
	test_file!(xcsp3_ex_038);
	test_file!(xcsp3_ex_039);
	// test_file!(xcsp3_ex_040);
	test_file!(xcsp3_ex_041);
	// test_file!(xcsp3_ex_042);
	test_file!(xcsp3_ex_043);
	test_file!(xcsp3_ex_044);
	test_file!(xcsp3_ex_045);
	test_file!(xcsp3_ex_046);
	test_file!(xcsp3_ex_047);
	// test_file!(xcsp3_ex_048);
	test_file!(xcsp3_ex_049);
	// test_file!(xcsp3_ex_050);
	test_file!(xcsp3_ex_051);
	test_file!(xcsp3_ex_052);
	test_file!(xcsp3_ex_053);
	test_file!(xcsp3_ex_054);
	test_file!(xcsp3_ex_055);
	test_file!(xcsp3_ex_056);
	test_file!(xcsp3_ex_057);
	test_file!(xcsp3_ex_058);
	test_file!(xcsp3_ex_059);
	test_file!(xcsp3_ex_060);
	// test_file!(xcsp3_ex_061);
	// test_file!(xcsp3_ex_062);
	test_file!(xcsp3_ex_063);
	test_file!(xcsp3_ex_064);
	test_file!(xcsp3_ex_065);
	test_file!(xcsp3_ex_066);
	test_file!(xcsp3_ex_067);
	test_file!(xcsp3_ex_068);
	test_file!(xcsp3_ex_069);
	// test_file!(xcsp3_ex_070);
	// test_file!(xcsp3_ex_071);
	test_file!(xcsp3_ex_072);
	test_unroll!(xcsp3_ex_073);
	test_file!(xcsp3_ex_074);
	test_file!(xcsp3_ex_075);
	test_file!(xcsp3_ex_076);
	test_file!(xcsp3_ex_077);
	test_file!(xcsp3_ex_078);
	// test_file!(xcsp3_ex_079);
	// test_file!(xcsp3_ex_080);
	// test_file!(xcsp3_ex_081);
	// test_file!(xcsp3_ex_082);
	// test_file!(xcsp3_ex_083);
	test_unroll!(xcsp3_ex_084);
	test_file!(xcsp3_ex_085);
	test_file!(xcsp3_ex_086);
	// test_file!(xcsp3_ex_087);
	// test_file!(xcsp3_ex_088);
	test_file!(xcsp3_ex_089);
	// test_file!(xcsp3_ex_090);
	test_file!(xcsp3_ex_091);
	// test_file!(xcsp3_ex_092);
	// test_file!(xcsp3_ex_093);
	// test_file!(xcsp3_ex_094);
	// test_file!(xcsp3_ex_095);
	// test_file!(xcsp3_ex_096);
	test_file!(xcsp3_ex_097);
	// test_file!(xcsp3_ex_098);
	// test_file!(xcsp3_ex_099);
	test_file!(xcsp3_ex_100);
	test_file!(xcsp3_ex_101);
	// test_file!(xcsp3_ex_102);
	// test_file!(xcsp3_ex_103);
	// test_file!(xcsp3_ex_104);
	// test_file!(xcsp3_ex_105);
	// test_file!(xcsp3_ex_106);
	// test_file!(xcsp3_ex_107);
	// test_file!(xcsp3_ex_108);
	// test_file!(xcsp3_ex_109);
	// test_file!(xcsp3_ex_110);
	// test_file!(xcsp3_ex_111);
	// test_file!(xcsp3_ex_112);
	test_unroll!(xcsp3_ex_113);
	// test_file!(xcsp3_ex_114);
	test_unroll!(xcsp3_ex_115);
	test_unroll!(xcsp3_ex_116);
	test_unroll!(xcsp3_ex_117);
	test_unroll!(xcsp3_ex_118);
	// test_file!(xcsp3_ex_119);
	// test_file!(xcsp3_ex_120);
	// test_file!(xcsp3_ex_121);
	// test_file!(xcsp3_ex_122);
	// test_file!(xcsp3_ex_123);
	// test_file!(xcsp3_ex_124);
	// test_file!(xcsp3_ex_125);
	// test_file!(xcsp3_ex_126);
	test_unroll!(xcsp3_ex_127);
	test_unroll!(xcsp3_ex_128);
	// test_file!(xcsp3_ex_129);
	test_unroll!(xcsp3_ex_130);
	test_unroll!(xcsp3_ex_131);
	// test_file!(xcsp3_ex_132);
	// test_file!(xcsp3_ex_133);
	// test_file!(xcsp3_ex_134);
	// test_file!(xcsp3_ex_135);
	// test_file!(xcsp3_ex_136);
	// test_file!(xcsp3_ex_137);
	// test_file!(xcsp3_ex_138);
	// test_file!(xcsp3_ex_139);
	// test_file!(xcsp3_ex_140);
	// test_file!(xcsp3_ex_141);
	// test_file!(xcsp3_ex_142);
	// test_file!(xcsp3_ex_143);
	// test_file!(xcsp3_ex_144);
	// test_file!(xcsp3_ex_145);
	// test_file!(xcsp3_ex_146);
	// test_file!(xcsp3_ex_147);
	// test_file!(xcsp3_ex_148);
	// test_file!(xcsp3_ex_149);
	// test_file!(xcsp3_ex_150);
	// test_file!(xcsp3_ex_151);
	test_unroll!(xcsp3_ex_152);
	test_unroll!(xcsp3_ex_153);
	test_unroll!(xcsp3_ex_154);
	test_unroll!(xcsp3_ex_155);
	test_unroll!(xcsp3_ex_156);
	test_unroll!(xcsp3_ex_157);
	test_unroll!(xcsp3_ex_158);
	// test_file!(xcsp3_ex_159);
	// test_file!(xcsp3_ex_160);
	test_unroll!(xcsp3_ex_161);
	// test_file!(xcsp3_ex_162);
	// test_file!(xcsp3_ex_163);
	// test_file!(xcsp3_ex_164);
	// test_file!(xcsp3_ex_165);
	test_unroll!(xcsp3_ex_166);
	test_file!(xcsp3_ex_167);
	// test_file!(xcsp3_ex_168);
	// test_file!(xcsp3_ex_169);
	// test_file!(xcsp3_ex_170);
	// test_file!(xcsp3_ex_171);
	// test_file!(xcsp3_ex_172);
	// test_file!(xcsp3_ex_173);
	// test_file!(xcsp3_ex_174);
	// test_file!(xcsp3_ex_175);
	// test_file!(xcsp3_ex_176);
	// test_file!(xcsp3_ex_177);
	// test_file!(xcsp3_ex_178);
	// test_file!(xcsp3_ex_179);
	// test_file!(xcsp3_ex_180);
	// test_file!(xcsp3_ex_181);
	// test_file!(xcsp3_ex_182);
	// test_file!(xcsp3_ex_183);
	// test_file!(xcsp3_ex_184);
	// test_file!(xcsp3_ex_185);
	// test_file!(xcsp3_ex_186);
	// test_file!(xcsp3_ex_187);
	// test_file!(xcsp3_ex_188);
	// test_file!(xcsp3_ex_189);
	// test_file!(xcsp3_ex_190);
	// test_file!(xcsp3_ex_191);
}
