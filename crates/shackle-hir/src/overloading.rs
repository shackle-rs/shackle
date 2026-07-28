//! Overloading validation and named argument handling

use shackle_diagnostics::{
	DuplicateFunction, FunctionAlreadyDefined, IllegalOverload, IllegalOverloading,
};
use shackle_ty::{Overload, OverloadedFunction};
use shackle_utils::{InternedString, hash::Map};

use crate::{Db, GlobalScope, Identifier, PatternTy, diagnostics::Diagnostics, ids::PatternRef};

/// Info about a function parameter
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub enum ParamKind<'db> {
	/// An unnamed parameter
	Unnamed,
	/// A parameter with an identifier name
	Named {
		/// Parameter name
		name: Identifier<'db>,
		/// Whether the parameter has a default value
		has_default: bool,
	},
}
/// An overloaded function entry
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub struct FunctionEntry<'db> {
	/// Whether this function has a body
	pub has_body: bool,
	/// Whether the first parameter is the annotated expression
	pub has_annotated_expression: bool,
	/// Kind of each parameter
	pub kinds: Box<[ParamKind<'db>]>,
	/// The overloaded function
	pub overload: OverloadedFunction<'db>,
}

impl<'db> Overload<'db> for FunctionEntry<'db> {
	fn overload(&self) -> &OverloadedFunction<'db> {
		&self.overload
	}

	fn tie_break(&self, other: &Self) -> Option<bool> {
		if self.has_body && !other.has_body {
			Some(true)
		} else if !self.has_body && other.has_body {
			Some(false)
		} else {
			// Just choose the first one, since the ambiguous overload will be reported during overload validation
			Some(true)
		}
	}
}

/// Overload eliminated due to named argument mismatch
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OverloadEliminationReason<'db> {
	/// Name appears in positional argument and named argument
	PositionalNameConflict {
		/// The argument index
		position: usize,
		/// The argument name
		name: Identifier<'db>,
	},
	/// Name missing from call
	MissingParameter {
		/// The missing parameter name
		name: Identifier<'db>,
	},
	/// Mismatch in number of arguments
	ArgumentCountMismatch {
		/// Expected minimum number of arguments
		expected_min: usize,
		/// Expected maximum number of arguments
		expected_max: usize,
		/// Actual number of arguments
		actual: usize,
	},
}

/// Overload eliminaed due to named argument mismatch
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EliminatedOverload<'db> {
	/// The overload
	pub overload: (PatternRef<'db>, FunctionEntry<'db>),
	/// The reason the overload was eliminated
	pub reason: OverloadEliminationReason<'db>,
}

/// An overloaded function entry
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub struct FunctionEntryData<'db> {
	/// The function pattern
	pub pattern: PatternRef<'db>,
	/// Whether this function has a body
	pub has_body: bool,
	/// Kind of each parameter
	pub kinds: Box<[ParamKind<'db>]>,
}

/// Resolves named and default argument calls
///
/// The shackle_ty crate only works with positional, required arguments, so here
/// we convert named into positional parameters, and drop unused default parameters.
/// Eliminated overloads are returned so that diagnostics can be generated.
#[derive(Clone, Debug)]
pub struct NamedArgumentResoler<'db> {
	n_positional: usize,
	name_order: Map<Identifier<'db>, usize>,
	eliminated: Vec<EliminatedOverload<'db>>,
	canidates: Vec<(PatternRef<'db>, FunctionEntry<'db>)>,
}

/// Result of resolving named and default argument calls
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedArgumentResolutionResult<'db> {
	/// The overloads that were eliminated
	pub eliminated: Vec<EliminatedOverload<'db>>,
	/// The overloads that were not eliminated
	pub canidates: Vec<(PatternRef<'db>, FunctionEntry<'db>)>,
}

impl<'db> NamedArgumentResoler<'db> {
	/// Create a new named argument resolver
	pub fn new(n_positional: usize, arg_names: &[Identifier<'db>]) -> Self {
		let mut name_order = Map::new();
		for (i, name) in arg_names.iter().enumerate() {
			let _ = name_order.insert(*name, i + n_positional);
		}
		Self {
			n_positional,
			name_order,
			eliminated: Vec::new(),
			canidates: Vec::new(),
		}
	}

	/// Add an overload to the resolver
	pub fn add_overload(&mut self, pattern: PatternRef<'db>, mut overload: FunctionEntry<'db>) {
		let required_max = overload.kinds.len();
		let required_min = required_max
			- overload
				.kinds
				.iter()
				.filter(|k| {
					matches!(
						k,
						ParamKind::Named {
							has_default: true,
							..
						}
					)
				})
				.count();

		let arg_count = self.n_positional + self.name_order.len();
		if arg_count < required_min || arg_count > required_max {
			self.eliminated.push(EliminatedOverload {
				overload: (pattern, overload),
				reason: OverloadEliminationReason::ArgumentCountMismatch {
					expected_min: required_min,
					expected_max: required_max,
					actual: arg_count,
				},
			});
			return;
		}

		let mut new_params = overload.overload.params()[0..arg_count].to_vec();
		for (i, kind) in overload.kinds.iter().enumerate() {
			if let ParamKind::Named { name, has_default } = kind {
				if i < self.n_positional {
					if self.name_order.contains_key(name) {
						self.eliminated.push(EliminatedOverload {
							reason: OverloadEliminationReason::PositionalNameConflict {
								position: i,
								name: *name,
							},
							overload: (pattern, overload),
						});
						return;
					}
				} else {
					if let Some(&pos) = self.name_order.get(name) {
						new_params[pos] = overload.overload.params()[i];
					} else if !has_default {
						let reason = OverloadEliminationReason::MissingParameter { name: *name };
						self.eliminated.push(EliminatedOverload {
							overload: (pattern, overload),
							reason,
						});
						return;
					};
				}
			}
		}
		overload.overload.set_params(new_params.into_boxed_slice());
		self.canidates.push((pattern, overload));
	}

	/// Finish the resolution and return the result
	pub fn finish(self) -> NamedArgumentResolutionResult<'db> {
		NamedArgumentResolutionResult {
			eliminated: self.eliminated,
			canidates: self.canidates,
		}
	}
}

/// Validate that all function overloads for the given name are legal
///
/// Accumulates diagnostics into the database
pub fn validate_overloading<'db>(db: &'db dyn Db, name: Identifier<'db>) {
	validate_overloading_internal(db, name.into())
}

#[salsa::tracked]
fn validate_overloading_internal<'db>(db: &'db dyn Db, name: InternedString<'db>) {
	let overloads = GlobalScope::find_function(db, name.into());
	check_overloading(db, overloads).accumulate(db);
}

/// Validate that the given overloads are legal, returning the diagnostics
pub fn check_overloading<'db>(db: &'db dyn Db, overloads: &[PatternRef<'db>]) -> Diagnostics {
	let mut diagnostics = Diagnostics::default();

	let mut functions = Vec::with_capacity(overloads.len());
	for p in overloads.iter() {
		match &p.item(db).signature(db).patterns[&p.pattern(db)] {
			PatternTy::Function(f) | PatternTy::AnnotationDestructure(f) => {
				functions.push((*p, *f.clone()));
			}
			PatternTy::AnnotationConstructor(f) => {
				functions.push((*p, *f.clone()));
			}
			PatternTy::EnumConstructor(ecs) => {
				functions.extend(ecs.iter().map(|f| (*p, f.constructor.clone())));
			}
			PatternTy::EnumDestructure(fs) => {
				functions.extend(fs.iter().map(|f| (*p, f.clone())));
			}
			_ => unreachable!(),
		}
	}
	let mut same_fns = functions.iter().map(|_| None).collect::<Vec<_>>();
	let mut incompat_fns = functions.iter().map(|_| None).collect::<Vec<_>>();
	// TODO: Make less horrible
	for (i, (_, a)) in functions.iter().enumerate() {
		for (j, (_, b)) in functions[i + 1..].iter().enumerate() {
			if let Ok((_, fta)) = a.overload.instantiate_ty_params(db, b.overload.params()) {
				if b.overload
					.instantiate_ty_params(db, a.overload.params())
					.is_ok() && (a.has_body && b.has_body
					|| fta.return_type != b.overload.return_type())
				{
					// Same function with multiple definitions
					same_fns[i + j + 1] = same_fns[i].or(Some(i));
				}
				if !b.overload.return_type().is_subtype_of(db, fta.return_type) {
					// Functions have incompatible return types
					incompat_fns[i + j + 1] = incompat_fns[i].or(Some(i));
				}
			} else if let Ok((_, ftb)) = b.overload.instantiate_ty_params(db, a.overload.params())
				&& !a.overload.return_type().is_subtype_of(db, ftb.return_type)
			{
				// Functions have incompatible return types
				incompat_fns[i + j + 1] = incompat_fns[i].or(Some(i));
			}
		}
	}
	let mut drain = functions.iter().cloned().map(Some).collect::<Vec<_>>();
	for i in 0..same_fns.len() {
		let others = same_fns
			.iter()
			.enumerate()
			.filter_map(|(j, dup)| {
				if let Some(x) = dup
					&& *x == i
				{
					let (dup, _) = drain[j].take().unwrap();
					let (src, span) = dup.source_span(db);
					return Some(DuplicateFunction { src, span });
				}
				None
			})
			.collect::<Vec<_>>();
		if !others.is_empty() {
			let (first_pattern, first_fn) = drain[i].take().unwrap();
			let (src, span) = first_pattern.source_span(db);
			diagnostics.add_error(FunctionAlreadyDefined {
				src,
				span,
				signature: first_fn
					.overload
					.pretty_print_item(db, first_pattern.identifier(db).unwrap()),
				others,
			});
		}
	}

	let mut drain = functions.iter().cloned().map(Some).collect::<Vec<_>>();
	for i in 0..incompat_fns.len() {
		let others = incompat_fns
			.iter()
			.enumerate()
			.filter_map(|(j, dup)| {
				if let Some(x) = dup
					&& *x == i
				{
					let (overload, _) = drain[j].take().unwrap();
					let (src, span) = overload.source_span(db);
					return Some(IllegalOverload { src, span });
				}
				None
			})
			.collect::<Vec<_>>();
		if !others.is_empty() {
			let (first, _) = drain[i].take().unwrap();
			let (src, span) = first.source_span(db);
			diagnostics.add_error(IllegalOverloading { src, span, others });
		}
	}

	diagnostics
}
