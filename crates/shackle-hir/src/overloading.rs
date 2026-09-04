//! Overloading validation and named argument handling

use salsa::SalsaValue;
use shackle_diagnostics::{
	DuplicateFunction, FunctionAlreadyDefined, IllegalOverload, IllegalOverloading,
};
use shackle_ty::{Overload, OverloadedFunction, match_fn, registry::TypeRegistry};
use shackle_utils::{InternedString, hash::Map};

use crate::{
	Db, FunctionItem, GlobalScope, Identifier, Item, PatternTy, diagnostics::Diagnostics,
	ids::PatternRef,
};

/// Info about a function parameter
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::SalsaValue)]
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::SalsaValue)]
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

impl<'db> FunctionEntry<'db> {
	/// Get the names of the parameters
	pub fn param_names(&self) -> Vec<Option<Identifier<'db>>> {
		self.kinds
			.iter()
			.map(|k| match k {
				ParamKind::Unnamed => None,
				ParamKind::Named { name, .. } => Some(*name),
			})
			.collect()
	}

	/// Check if this function can be called using named arguments
	pub fn can_call_using_names(&self) -> bool {
		if self.kinds.is_empty() {
			return false;
		}

		let mut found_named = false;
		for k in self.kinds.iter() {
			if let ParamKind::Named { .. } = k {
				found_named = true;
			} else if found_named {
				// Unnamed parameter after named parameter cannot be called using names
				return false;
			}
		}

		found_named
	}

	/// Pretty print as a signature using the given name
	pub fn pretty_print(&self, db: &'db dyn Db, name: Identifier<'db>) -> String {
		let tys = TypeRegistry::lookup(db);
		let ret = if self.overload.return_type() == tys.par_bool {
			"test".to_owned()
		} else if self.overload.return_type() == tys.var_bool {
			"predicate".to_owned()
		} else {
			format!("function {}:", self.overload.return_type().pretty_print(db))
		};
		let args = self
			.overload
			.params()
			.iter()
			.zip(self.kinds.iter())
			.map(|(ty, kind)| match kind {
				ParamKind::Unnamed => ty.pretty_print(db),
				ParamKind::Named { name, has_default } if *has_default => format!(
					"{}: {} = <default>",
					ty.pretty_print(db),
					name.pretty_print(db)
				),
				ParamKind::Named { name, .. } => {
					format!("{}: {}", ty.pretty_print(db), name.pretty_print(db))
				}
			})
			.collect::<Vec<_>>()
			.join(", ");
		format!("{} {}({})", ret, name.pretty_print(db), args)
	}
}

impl<'db> Overload<'db> for FunctionEntry<'db> {
	fn overload(&self) -> &OverloadedFunction<'db> {
		&self.overload
	}

	fn tie_break(&self, other: &Self) -> Option<bool> {
		match (self.has_body, other.has_body) {
			(true, false) => Some(true),
			(false, true) => Some(false),
			(false, false) => Some(true), // Both don't have bodies, so arbitrarily pick the first one
			(true, true) => None,         // Both have bodies, so ambiguous
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::SalsaValue)]
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

/// Validate all function overloading
#[salsa::tracked(returns(copy))]
pub fn validate_all_overloading(db: &dyn Db) {
	for (name, _) in GlobalScope::functions(db) {
		validate_overloading(db, name);
	}
}

/// Validate that all function overloads for the given name are legal
///
/// Accumulates diagnostics into the database
pub fn validate_overloading<'db>(db: &'db dyn Db, name: Identifier<'db>) {
	validate_overloading_internal(db, name.into())
}

#[salsa::tracked(returns(copy))]
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
					// Cannot tell apart via positional call, so names must be able to disambiguate
					if !a.can_call_using_names() || !b.can_call_using_names() {
						incompat_fns[i + j + 1] = incompat_fns[i].or(Some(i));
						continue;
					}

					let mut name_positions = a
						.param_names()
						.iter()
						.enumerate()
						.filter_map(|(i, n)| n.map(|n| (n, i)))
						.collect::<Map<_, _>>();

					let a_name_count = name_positions.len();
					let mut reordered_params = b.overload.params().to_vec();
					let mut count = 0_usize;
					for (b_param, b_name) in b.overload.params().iter().zip(b.param_names()) {
						let Some(name) = b_name else {
							continue;
						};
						let Some(pos) = name_positions.remove(&name) else {
							break;
						};
						reordered_params[pos] = *b_param;
						count += 1;
					}

					if count != a_name_count {
						continue;
					}

					let mut b_overload = b.overload.clone();
					b_overload.set_params(reordered_params.into_boxed_slice());

					if b_overload
						.instantiate_ty_params(db, a.overload.params())
						.is_ok()
					{
						// Same function with multiple definitions
						same_fns[i + j + 1] = same_fns[i].or(Some(i));
					}
				}
				if a.param_names() == b.param_names()
					&& !b.overload.return_type().is_subtype_of(db, fta.return_type)
				{
					// a should be able to dispatch to b, but cannot due to incompatible return types
					incompat_fns[i + j + 1] = incompat_fns[i].or(Some(i));
				}
			} else if let Ok((_, ftb)) = b.overload.instantiate_ty_params(db, a.overload.params())
				&& a.param_names() == b.param_names()
				&& !a.overload.return_type().is_subtype_of(db, ftb.return_type)
			{
				// b should be able to dispatch to a, but cannot due to incompatible return types
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
				signature: first_fn.pretty_print(db, first_pattern.identifier(db).unwrap()),
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, SalsaValue)]
struct SubsumeMap<'db> {
	map: Map<FunctionItem<'db>, FunctionItem<'db>>,
	reverse: Map<FunctionItem<'db>, Vec<FunctionItem<'db>>>,
}

#[salsa::tracked]
fn overload_subsumes<'db>(db: &'db dyn Db, name: InternedString<'db>) -> SubsumeMap<'db> {
	let ps = GlobalScope::find_function(db, name.into());
	let mut buckets: Map<_, Vec<_>> = Map::default();
	for p in ps.iter() {
		let signature = p.item(db).signature(db);
		if let PatternTy::Function(f) = &signature.patterns[&p.pattern(db)] {
			let Item::Function(pf) = p.item(db) else {
				continue;
			};
			buckets
				.entry(f.kinds.clone())
				.or_default()
				.push((pf, *f.clone()));
		}
	}
	let mut result = SubsumeMap::default();
	for overloads in buckets.into_values() {
		if overloads.len() > 1 {
			for (p, f) in overloads.iter() {
				let Ok(res) = match_fn(db, overloads.iter().cloned(), f.overload.params()) else {
					continue;
				};
				if res.function.0 != *p {
					let _ = result.map.insert(*p, res.function.0);
					result.reverse.entry(res.function.0).or_default().push(*p);
				}
			}
		}
	}
	result
}

impl<'db> FunctionItem<'db> {
	/// Get the function that subsumes this function, if any
	pub fn subsumed(&self, db: &'db dyn Db) -> Option<FunctionItem<'db>> {
		let function = self.function(db);
		let Some(name) = function.data()[function.pattern].identifier() else {
			return None;
		};
		overload_subsumes(db, name.into()).map.get(self).cloned()
	}

	/// Get the functions subsumed by this function
	pub fn subsumes_others(&self, db: &'db dyn Db) -> Option<&'db Vec<FunctionItem<'db>>> {
		let function = self.function(db);
		let Some(name) = function.data()[function.pattern].identifier() else {
			return None;
		};
		overload_subsumes(db, name.into()).reverse.get(self)
	}
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_syntax::InputLang;

	use crate::{
		CompilerDatabase,
		diagnostics::Errors,
		input::{CompilerSettings, InlineModelFile, InputFiles},
		overloading::validate_all_overloading,
	};

	fn test_overloading(model: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model = InlineModelFile::new(&db, model.to_owned(), InputLang::MiniZinc);
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![model.into()]);
		validate_all_overloading(&db);
		let errors = validate_all_overloading::accumulated::<Errors>(&db);
		let result = errors
			.iter()
			.map(|e| e.to_string())
			.collect::<Vec<_>>()
			.join("\n");
		expected.assert_eq(&result);
	}

	#[test]
	fn test_overloading_named_incompat_return() {
		test_overloading(
			r#"
			test foo(int: a, int: b) = true;
			function int: foo(int: b, int: a) = 10;
		"#,
			expect!["Function with the signature 'test foo(int: a, int: b)' already defined"],
		);
	}

	#[test]
	fn test_overloading_unnamed_incompat_return() {
		test_overloading(
			r#"
			function int: foo(int);
			function bool: foo(int);

		"#,
			expect!["Return type conflicts with return type of other overloads"],
		);
	}

	#[test]
	fn test_overload_subsumes() {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let code = r#"
			function int: foo(int: a, int: b) = 1;
			function int: foo(int: a, int: b);
			function int: foo(int: c, int: d) = 2;
		"#;
		let model = InlineModelFile::new(&db, code.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model]);
		let functions = model
			.hir(&db)
			.items(&db)
			.iter()
			.filter_map(|i| i.try_unwrap_function().ok())
			.collect::<Vec<_>>();
		assert!(functions[0].subsumed(&db).is_none());
		assert_eq!(functions[1].subsumed(&db), Some(functions[0]));
		assert!(functions[2].subsumed(&db).is_none());
		let subsumes = functions[0]
			.subsumes_others(&db)
			.expect("should have subsumes");
		assert_eq!(subsumes.len(), 1);
		assert_eq!(subsumes[0], functions[1]);
	}
}
