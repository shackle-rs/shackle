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
pub mod expression;

use std::{
	borrow::Cow,
	fmt::{self, Display},
	marker::PhantomData,
	ops::RangeInclusive,
	str::FromStr,
};

use nom::{
	character::complete::{char, digit1},
	combinator::{all_consuming, map_res, opt, recognize},
	multi::many0,
	sequence::delimited,
	IResult,
};
pub use rangelist::RangeList;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
	constraint::Constraint,
	expression::{identifier, int, range, sequence, whitespace_seperated, IntExp},
};

/// Definition of a k-dimensional arrays of variables
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Array<Identifier = String> {
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
	pub domains: Vec<(Vec<VarRef<Identifier>>, RangeList<IntVal>)>,
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
	Single(IntVal),
	/// Accessing a slice of a dimension in an array
	Range(IntVal, IntVal),
	/// Accessing the full range of an array
	Full,
}

/// XCSP3 problem instance
#[derive(Clone, PartialEq, Debug, Hash)]
pub struct Instance<Identifier = String> {
	/// The type of the framework used to express the instance.
	pub ty: FrameworkType,
	/// Definitions of the single decision variables
	pub variables: Vec<Variable<Identifier>>,
	/// Definitions of the arrays of decision variables
	pub arrays: Vec<Array<Identifier>>,
	/// Constraints that must be satisfied for a solution to be valid
	pub constraints: Vec<Constraint<Identifier>>,
	/// The objectives to be optimized
	pub objectives: Objectives<Identifier>,
}

/// An assignment from a list of variables to a list of values
///
/// This structure is used both to represent an elementary constraint in an
/// instance, and to represent the solution to an instance.
#[derive(Clone, Debug, PartialEq, Hash, Deserialize)]
#[serde(bound(deserialize = "Identifier: FromStr"))]
pub struct Instantiation<Identifier = String> {
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
	pub list: Vec<VarRef<Identifier>>,
	/// List of values assigned to the variables
	#[serde(
		deserialize_with = "deserialize_int_vals",
		serialize_with = "serialize_list"
	)]
	pub values: Vec<IntVal>,
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

/// Type used to represent integer values
pub type IntVal = i64;

/// Type used to capture optional metadata that can be attached to most XCSP3
/// elements
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
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
	bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display")
)]
pub enum Objective<Identifier = String> {
	/// An objective function where the goal is to find the smallest possible
	/// value.
	#[serde(rename = "minimize")]
	Minimize(ObjExp<Identifier>),
	/// An objective function where the goal is to find the largest possible
	/// value.
	#[serde(rename = "maximize")]
	Maximize(ObjExp<Identifier>),
}

/// Collection of objective functions
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Objectives<Identifier = String> {
	/// Combinator to aggregate multiple objectives
	#[serde(default, rename = "@combination")]
	pub combination: CombinationType,
	/// List of objectives functions
	#[serde(rename = "$value")]
	pub objectives: Vec<Objective<Identifier>>,
}

/// Expression used to represent an objective function
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct ObjExp<Identifier = String> {
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
	pub list: Vec<IntExp<Identifier>>,
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

/// Definition of a variable
#[derive(Clone, Debug, PartialEq, Hash, Deserialize, Serialize)]
#[serde(bound(deserialize = "Identifier: FromStr", serialize = "Identifier: Display"))]
pub struct Variable<Identifier = String> {
	/// Name of the variable
	#[serde(
		rename = "@id",
		deserialize_with = "from_str",
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

/// Reference to a variable, array element, or array slice
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum VarRef<Identifier> {
	/// Reference to a variable
	Ident(Identifier),
	/// Reference to an array element or slice
	ArrayAccess(Identifier, Vec<Indexing>),
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
			cur = *cur.start()..=*next.end();
		} else {
			ranges.push(cur);
			cur = next;
		}
	}
	ranges.push(cur);
	ranges.into_iter().collect()
}

/// Deserialize a string as an identifier
fn deserialize_ident<'de, D: Deserializer<'de>, Identifier: FromStr>(
	deserializer: D,
) -> Result<Option<Identifier>, D::Error> {
	/// Visitor to deserialize a string as an identifier
	struct V<X>(PhantomData<X>);
	impl<'de, X: FromStr> Visitor<'de> for V<X> {
		type Value = Option<X>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("an identfier")
		}

		fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
			Ok(Some(FromStr::from_str(v).map_err(|_| {
				E::custom("unable to create identifier from string")
			})?))
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
	impl<'de> Visitor<'de> for V {
		type Value = Vec<IntVal>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a list of integers")
		}

		fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let (_, v) = all_consuming(whitespace_seperated(int))(v)
				.map_err(|e| E::custom(format!("invalid list of integers {e:?}")))?;
			Ok(v)
		}
	}
	deserializer.deserialize_str(V)
}

/// Deserialize a string as a range list
fn deserialize_range_list<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<RangeList<IntVal>, D::Error> {
	/// Visitor for deserializing a range list
	struct V;
	impl<'de> Visitor<'de> for V {
		type Value = RangeList<IntVal>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a list of ranges")
		}

		fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let (_, r) = all_consuming(whitespace_seperated(range))(v)
				.map_err(|e| E::custom(format!("invalid list of ranges {e:?}")))?;
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
	impl<'de> Visitor<'de> for V {
		type Value = Vec<usize>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("an array size expression")
		}

		fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let (_, r) = all_consuming(sequence(delimited(
				char('['),
				map_res(recognize(digit1), str::parse),
				char(']'),
			)))(v)
			.map_err(|e| E::custom(format!("invalid array size expression {e:?}")))?;
			Ok(r)
		}
	}
	let visitor = V;
	deserializer.deserialize_str(visitor)
}

/// Deserialize a string and call the `FromStr` implementation
fn from_str<'de, D: Deserializer<'de>, I: FromStr>(deserializer: D) -> Result<I, D::Error> {
	let s: Cow<'_, str> = Deserialize::deserialize(deserializer)?;
	match s.parse() {
		Ok(t) => Ok(t),
		Err(_) => Err(serde::de::Error::custom("unable to parse from string")),
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

impl<'de, Identifier: FromStr> Deserialize<'de> for Array<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Helper struct to deserialize the content of the <domain> element
		#[derive(Deserialize)]
		#[serde(bound = "Identifier: FromStr")]
		struct DomainStruct<Identifier: FromStr> {
			/// for attribute
			#[serde(rename = "@for", deserialize_with = "VarRef::parse_vec")]
			vars: Vec<VarRef<Identifier>>,
			/// content of element
			#[serde(rename = "$text", deserialize_with = "deserialize_range_list")]
			domain: RangeList<IntVal>,
		}
		/// Helper enum to deserialize the content of the <array> element
		#[derive(Deserialize)]
		#[serde(bound = "Identifier: FromStr")]
		enum Domain<'a, Identifier: FromStr> {
			/// multiple <domain> elements
			#[serde(rename = "domain")]
			Domain(Vec<DomainStruct<Identifier>>),
			/// single string content
			#[serde(rename = "$text")]
			Direct(Cow<'a, str>),
		}
		/// Helper struct to deserialize an <array> element
		#[derive(Deserialize)]
		#[serde(bound = "Identifier: FromStr")]
		struct Array<'a, Identifier: FromStr> {
			/// id attribute
			#[serde(rename = "@id", deserialize_with = "from_str")]
			identifier: Identifier,
			/// optional note attribute
			#[serde(rename = "@note", default, skip_serializing_if = "Option::is_none")]
			note: Option<String>,
			/// size attribute
			#[serde(rename = "@size", deserialize_with = "deserialize_size")]
			size: Vec<usize>,
			/// content of the element
			#[serde(rename = "$value")]
			domain: Domain<'a, Identifier>,
		}
		let x = Array::deserialize(deserializer)?;
		let domains = match x.domain {
			Domain::Domain(v) => v.into_iter().map(|d| (d.vars, d.domain)).collect(),
			Domain::Direct(s) => {
				let s = all_consuming(whitespace_seperated(range))(s.as_ref())
					.map_err(serde::de::Error::custom)?;
				vec![(
					vec![VarRef::Ident(
						Identifier::from_str("others").unwrap_or_else(|_| {
							panic!("unable to create identifier from `\"others\"`")
						}),
					)],
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

impl<'de, Identifier: Deserialize<'de> + FromStr> Deserialize<'de> for Instance<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		/// Deserialized content of <variables> element
		#[derive(Deserialize)]
		enum V<Identifier: FromStr> {
			/// Deserialized <var> element
			#[serde(rename = "var")]
			Variable(Variable<Identifier>),
			/// Deserialized <array> element
			#[serde(rename = "array")]
			Array(Array<Identifier>),
		}
		/// Deserialized <variables> element
		#[derive(Deserialize)]
		struct Variables<Identifier: FromStr = String> {
			/// Deserialized content of <variables> element
			#[serde(rename = "$value")]
			vars: Vec<V<Identifier>>,
		}
		/// Deserialized <constraints> element
		#[derive(Deserialize)]
		struct Constraints<Identifier: FromStr = String> {
			/// Deserialized content of <constraints> element
			#[serde(rename = "$value")]
			content: Vec<Constraint<Identifier>>,
		}
		/// Deserialized <instance> element
		#[derive(Deserialize)]
		struct Instance<Identifier: FromStr = String> {
			/// Deserialized type attribute
			#[serde(rename = "@type")]
			ty: FrameworkType,
			/// Deserialized <variables> element
			variables: Option<Variables<Identifier>>,
			/// Deserialized <constraints> element
			constraints: Option<Constraints<Identifier>>,
			/// Deserialized <objectives> element
			#[serde(default = "Objectives::default")]
			objectives: Objectives<Identifier>,
		}
		let inst: Instance<Identifier> = Deserialize::deserialize(deserializer)?;
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
		impl<'a, Identifier: Display> Variables<'a, Identifier> {
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
			content: &'a Vec<Constraint<Identifier>>,
		}
		impl<'a, Identifier: Display> Constraints<'a, Identifier> {
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
impl<Identifier: Display> Serialize for Instantiation<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		/// Helper struct to serialize the instantiation element
		#[derive(Serialize)]
		#[serde(rename = "instantiation", bound(serialize = "Identifier: Display"))]
		struct Instantiation<'a, Identifier = String> {
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
			list: &'a Vec<VarRef<Identifier>>,
			/// Values serialized as <values>
			#[serde(serialize_with = "serialize_list")]
			values: &'a Vec<IntVal>,
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

impl<Identifier> Default for Objectives<Identifier> {
	fn default() -> Self {
		Self {
			combination: CombinationType::default(),
			objectives: Vec::new(),
		}
	}
}

impl<Identifier: FromStr> VarRef<Identifier> {
	/// Parse a variable reference.
	pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
		let (input, ident) = identifier(input)?;
		let (input, v) = many0(delimited(char('['), opt(range), char(']')))(input)?;
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

	/// Parse a list of variable references.
	fn parse_vec<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Self>, D::Error> {
		/// Visitor for parsing a list of variable references.
		struct V<X>(PhantomData<X>);
		impl<'de, X: FromStr> Visitor<'de> for V<X> {
			type Value = Vec<VarRef<X>>;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a list of variable references")
			}

			fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
				let (_, v) = all_consuming(whitespace_seperated(VarRef::parse))(v)
					.map_err(|e| E::custom(format!("invalid variable references {e:?}")))?;
				Ok(v)
			}
		}
		let visitor = V::<Identifier>(PhantomData);
		deserializer.deserialize_str(visitor)
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
		assert_eq!(inst, inst2);
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
	pub(crate) use test_file;

	test_file!(knapsack);

	test_file!(xcsp3_ex_001);
	test_file!(xcsp3_ex_002);
	// test_file!(xcsp3_ex_003);
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
	// test_file!(xcsp3_ex_026, Instantiation);
	test_file!(xcsp3_ex_027, Instantiation);
	// test_file!(xcsp3_ex_028, Instantiation);
	test_file!(xcsp3_ex_029);
	test_file!(xcsp3_ex_030);
	test_file!(xcsp3_ex_031);
	test_file!(xcsp3_ex_032);
	test_file!(xcsp3_ex_033);
	test_file!(xcsp3_ex_034);
	test_file!(xcsp3_ex_035);
	test_file!(xcsp3_ex_036);
	test_file!(xcsp3_ex_037);
	// test_file!(xcsp3_ex_038);
	// test_file!(xcsp3_ex_039);
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
	// test_file!(xcsp3_ex_054);
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
	// test_file!(xcsp3_ex_073);
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
	// test_file!(xcsp3_ex_084);
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
	// test_file!(xcsp3_ex_113);
	// test_file!(xcsp3_ex_114);
	// test_file!(xcsp3_ex_115);
	// test_file!(xcsp3_ex_116);
	// test_file!(xcsp3_ex_117);
	// test_file!(xcsp3_ex_118);
	// test_file!(xcsp3_ex_119);
	// test_file!(xcsp3_ex_120);
	// test_file!(xcsp3_ex_121);
	// test_file!(xcsp3_ex_122);
	// test_file!(xcsp3_ex_123);
	// test_file!(xcsp3_ex_124);
	// test_file!(xcsp3_ex_125);
	// test_file!(xcsp3_ex_126);
	// test_file!(xcsp3_ex_127);
	// test_file!(xcsp3_ex_128);
	// test_file!(xcsp3_ex_129);
	// test_file!(xcsp3_ex_130);
	// test_file!(xcsp3_ex_131);
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
	// test_file!(xcsp3_ex_152);
	// test_file!(xcsp3_ex_153);
	// test_file!(xcsp3_ex_154);
	// test_file!(xcsp3_ex_155);
	// test_file!(xcsp3_ex_156);
	// test_file!(xcsp3_ex_157);
	// test_file!(xcsp3_ex_158);
	// test_file!(xcsp3_ex_159);
	// test_file!(xcsp3_ex_160);
	// test_file!(xcsp3_ex_161);
	// test_file!(xcsp3_ex_162);
	// test_file!(xcsp3_ex_163);
	// test_file!(xcsp3_ex_164);
	// test_file!(xcsp3_ex_165);
	// test_file!(xcsp3_ex_166);
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
