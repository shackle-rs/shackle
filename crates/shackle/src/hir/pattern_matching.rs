//! Analysis of pattern matching to check exhaustiveness
//!
//! See http://moscova.inria.fr/~maranget/papers/warn/warn.pdf for algorithm details

use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
	arena::ArenaIndex,
	error::NonExhaustivePatternMatching,
	ty::{EnumRef, Ty, TyData, TypeRegistry},
	warning::{UnreachablePattern, Warning},
	Error,
};

use super::{
	db::Hir,
	ids::{EntityRef, ItemRef, NodeRef, PatternRef},
	BooleanLiteral, Expression, FloatLiteral, Identifier, IntegerLiteral, ItemData, OptType,
	Pattern, PatternTy, StringLiteral, TypeResult,
};

/// Compute a mapping from (non-introduced) enum types to the constructors for the enum
pub fn enum_constructors(db: &dyn Hir) -> Arc<FxHashMap<EnumRef, Arc<Vec<PatternRef>>>> {
	let mut result = FxHashMap::default();
	for m in db.resolve_includes().unwrap().iter() {
		let model = db.lookup_model(*m);
		for (i, e) in model.enumerations.iter() {
			if let Some(def) = &e.definition {
				let item = ItemRef::new(db, *m, i);
				let enum_ref = EnumRef::new(db, PatternRef::new(item, e.pattern));
				let constructors = def
					.iter()
					.map(|c| PatternRef::new(item, c.constructor_pattern()))
					.collect::<Vec<_>>();
				result.insert(enum_ref, Arc::new(constructors));
			}
		}
		for (i, e) in model.enum_assignments.iter() {
			let item = ItemRef::new(db, *m, i);
			let types = db.lookup_item_types(item);
			if let Some(p) = types.name_resolution(e.assignee) {
				let enum_ref = EnumRef::new(db, p);
				let constructors = e
					.definition
					.iter()
					.map(|c| PatternRef::new(item, c.constructor_pattern()))
					.collect::<Vec<_>>();
				result.insert(enum_ref, Arc::new(constructors));
			}
		}
	}
	Arc::new(result)
}

/// Lookup the enum constructors for the given enum type
pub fn lookup_enum_constructors(db: &dyn Hir, e: EnumRef) -> Option<Arc<Vec<PatternRef>>> {
	let map = db.enum_constructors();
	map.get(&e).cloned()
}

/// Check that all case statements in this item are exhaustive
pub fn check_case_exhaustiveness(
	db: &dyn Hir,
	item: ItemRef,
) -> (Arc<Vec<Error>>, Arc<Vec<Warning>>) {
	let mut warnings = Vec::new();
	let mut errors = Vec::new();
	let model = item.model(db);
	let local = item.local_item_ref(db);
	let data = local.data(&model);
	let types = db.lookup_item_types(item);
	for e in data.expressions.values() {
		if let Expression::Case(c) = e {
			let checker = ExhaustivenessChecker::new(db, data, &types);
			let mut matrix = Matrix::with_capacity(c.cases.len());
			for arm in c.cases.iter() {
				let pat = checker.lower_pattern(arm.pattern);
				let row = vec![pat];
				if !checker.is_useful(&matrix, &row) {
					// Useless case
					let (src, span) =
						NodeRef::from(PatternRef::new(item, arm.pattern).into_entity(db))
							.source_span(db);
					warnings.push(UnreachablePattern { src, span }.into());
				}
				matrix.add_row(row);
			}
			if let Some(pat) = checker.counter_example(&matrix, types[c.expression]) {
				// Non-exhaustive
				let (src, span) =
					NodeRef::from(EntityRef::new(db, item, c.expression)).source_span(db);
				errors.push(
					NonExhaustivePatternMatching {
						src,
						span,
						msg: format!("Case '{}' not covered", pat),
					}
					.into(),
				);
			}
		}
	}
	(Arc::new(errors), Arc::new(warnings))
}

/// A pattern distilled into its meaning in the context of pattern matching.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(variant_size_differences)]
enum SemanticPattern {
	/// Constructor
	Constructor(Ty, PatternConstructor, Box<[SemanticPattern]>),
	/// A pattern which matches anything (either `_` or an identifier which is not a constructor).
	Wildcard(Ty),
}

impl SemanticPattern {
	fn ty(&self) -> Ty {
		match self {
			SemanticPattern::Constructor(ty, _, _) => *ty,
			SemanticPattern::Wildcard(ty) => *ty,
		}
	}

	fn pretty_print(&self, db: &dyn Hir) -> String {
		match self {
			SemanticPattern::Constructor(_, PatternConstructor::Absent, _) => "<>".to_owned(),
			SemanticPattern::Constructor(ty, PatternConstructor::Structure, ps) => {
				match ty.lookup(db.upcast()) {
					TyData::Tuple(_, fs) => {
						let args = if fs.len() == ps.len() {
							ps.iter()
								.map(|p| p.pretty_print(db))
								.collect::<Vec<_>>()
								.join(", ")
						} else {
							fs.iter()
								.map(|_| "_".to_owned())
								.collect::<Vec<_>>()
								.join(", ")
						};
						format!("({})", args)
					}
					TyData::Record(_, fs) => {
						let args = if fs.len() == ps.len() {
							fs.iter()
								.zip(ps.iter())
								.map(|((i, _), p)| {
									format!(
										"{}: {}",
										Identifier(*i).pretty_print(db),
										p.pretty_print(db)
									)
								})
								.collect::<Vec<_>>()
								.join(", ")
						} else {
							fs.iter()
								.map(|(i, _)| format!("{}: _", Identifier(*i).pretty_print(db)))
								.collect::<Vec<_>>()
								.join(", ")
						};
						format!("({})", args,)
					}
					_ => unreachable!(),
				}
			}
			SemanticPattern::Constructor(_, PatternConstructor::Named(p), ps) => {
				let item = p.item();
				let model = item.model(db);
				let local = item.local_item_ref(db);
				let data = local.data(&model);
				let types = db.lookup_item_types(item);
				match &types[p.pattern()] {
					PatternTy::EnumAtom(_) => {
						data[p.pattern()].identifier().unwrap().pretty_print(db)
					}
					PatternTy::EnumConstructor(ec) => {
						let call = data[p.pattern()].identifier().unwrap().pretty_print(db);
						let args = ec
							.first()
							.map(|f| {
								if f.overload.params().len() == ps.len() {
									ps.iter()
										.map(|p| p.pretty_print(db))
										.collect::<Vec<_>>()
										.join(", ")
								} else {
									f.overload
										.params()
										.iter()
										.map(|_| "_".to_owned())
										.collect::<Vec<_>>()
										.join(", ")
								}
							})
							.unwrap_or_else(|| "..".to_owned());
						format!("{}({})", call, args)
					}
					_ => "_".to_owned(),
				}
			}
			_ => "_".to_owned(),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PatternConstructor {
	/// Named constructor (for enum/annotation)
	Named(PatternRef),
	/// Tuple/record constructor
	Structure,
	/// Absent literal
	Absent,
	/// Boolean literal
	Boolean(BooleanLiteral),
	/// Float literal
	Float {
		/// Whether this has been negated
		negated: bool,
		/// The literal value
		value: FloatLiteral,
	},
	/// Integer literal
	Integer {
		/// Whether this has been negated
		negated: bool,
		/// The literal value
		value: IntegerLiteral,
	},
	/// Infinity
	Infinity {
		/// Whether this has been negated
		negated: bool,
	},
	/// String literal
	String(StringLiteral),
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct Matrix {
	patterns: Vec<Vec<SemanticPattern>>,
}

impl From<Vec<Vec<SemanticPattern>>> for Matrix {
	fn from(patterns: Vec<Vec<SemanticPattern>>) -> Self {
		Self { patterns }
	}
}

impl Matrix {
	fn with_capacity(row_capacity: usize) -> Self {
		Self {
			patterns: Vec::with_capacity(row_capacity),
		}
	}

	fn add_row(&mut self, ps: Vec<SemanticPattern>) {
		self.patterns.push(ps);
	}

	fn col(&self, c: usize) -> impl '_ + Iterator<Item = &SemanticPattern> {
		self.patterns.iter().map(move |ps| &ps[c])
	}

	fn iter_rows(&self) -> impl '_ + Iterator<Item = &[SemanticPattern]> {
		self.patterns.iter().map(|p| &p[..])
	}

	fn rows(&self) -> usize {
		self.patterns.len()
	}

	fn cols(&self) -> usize {
		self.patterns.first().map(|ps| ps.len()).unwrap_or(0)
	}
}

/// Checks exhaustiveness of case expressions
struct ExhaustivenessChecker<'a> {
	db: &'a dyn Hir,
	data: &'a ItemData,
	types: &'a TypeResult,
	type_registry: Arc<TypeRegistry>,
}

impl<'a> ExhaustivenessChecker<'a> {
	fn new(db: &'a dyn Hir, data: &'a ItemData, types: &'a TypeResult) -> Self {
		Self {
			db,
			data,
			types,
			type_registry: db.type_registry(),
		}
	}

	fn is_useful(&self, matrix: &Matrix, row: &[SemanticPattern]) -> bool {
		assert!(matrix.rows() == 0 || matrix.cols() == row.len());
		if row.is_empty() {
			return matrix.rows() == 0;
		}
		match &row[0] {
			SemanticPattern::Constructor(_, c, ps) => {
				let sm = self.specialise_matrix(c, ps.len(), matrix);
				let sr = self.specialise_row(c, ps.len(), row);
				self.is_useful(&sm, &sr)
			}
			SemanticPattern::Wildcard(ty) => {
				let ctors = matrix
					.col(0)
					.filter_map(|p| match p {
						SemanticPattern::Constructor(_, c, a) => Some((c, a.len())),
						_ => None,
					})
					.collect::<Vec<_>>();
				if self
					.check_constructors(ctors.iter().map(|(c, _)| *c), *ty)
					.is_ok()
				{
					ctors.iter().any(|(c, a)| {
						let sm = self.specialise_matrix(c, *a, matrix);
						let sr = self.specialise_row(c, *a, row);
						self.is_useful(&sm, &sr)
					})
				} else {
					let dm = self.default_matrix(matrix);
					self.is_useful(&dm, &row[1..])
				}
			}
		}
	}

	fn specialise_matrix(
		&self,
		constructor: &PatternConstructor,
		arg_count: usize,
		matrix: &Matrix,
	) -> Matrix {
		matrix
			.iter_rows()
			.filter_map(|row| {
				let mut iter = row.iter();
				let first = iter.next().unwrap();
				match first {
					SemanticPattern::Constructor(_, c, ps) => {
						if c == constructor {
							let new_row =
								ps.iter().cloned().chain(iter.cloned()).collect::<Vec<_>>();
							Some(new_row)
						} else {
							None
						}
					}
					SemanticPattern::Wildcard(ty) => {
						let new_row = std::iter::repeat(SemanticPattern::Wildcard(*ty))
							.take(arg_count)
							.chain(iter.cloned())
							.collect::<Vec<_>>();
						Some(new_row)
					}
				}
			})
			.collect::<Vec<_>>()
			.into()
	}

	fn specialise_row(
		&self,
		constructor: &PatternConstructor,
		arg_count: usize,
		row: &[SemanticPattern],
	) -> Vec<SemanticPattern> {
		let mut iter = row.iter();
		let first = iter.next().unwrap();
		match first {
			SemanticPattern::Constructor(_, c, ps) => {
				assert_eq!(c, constructor);
				ps.iter().cloned().chain(iter.cloned()).collect::<Vec<_>>()
			}
			SemanticPattern::Wildcard(ty) => std::iter::repeat(SemanticPattern::Wildcard(*ty))
				.take(arg_count)
				.chain(iter.cloned())
				.collect::<Vec<_>>(),
		}
	}

	fn default_matrix(&self, matrix: &Matrix) -> Matrix {
		matrix
			.iter_rows()
			.filter_map(|row| {
				let mut iter = row.iter();
				let first = iter.next().unwrap();
				if let SemanticPattern::Wildcard(_) = first {
					Some(iter.cloned().collect::<Vec<_>>())
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
			.into()
	}

	fn check_constructors<'b>(
		&self,
		constructors: impl Iterator<Item = &'b PatternConstructor>,
		ty: Ty,
	) -> Result<(), SemanticPattern> {
		let mut required_ctors = Vec::new();
		match ty.lookup(self.db.upcast()) {
			TyData::Enum(_, o, e) => {
				if o == OptType::Opt {
					required_ctors.push(PatternConstructor::Absent);
				}
				if let Some(ctors) = self.db.lookup_enum_constructors(e) {
					if ctors.iter().any(|ctor| ctor.identifier(self.db).is_none()) {
						// Cannot be fully constructed
						return Err(SemanticPattern::Wildcard(ty));
					}
					required_ctors.extend(ctors.iter().copied().map(PatternConstructor::Named));
				} else {
					return Err(SemanticPattern::Wildcard(ty));
				}
			}
			TyData::Tuple(o, _) | TyData::Record(o, _) => {
				if o == OptType::Opt {
					required_ctors.push(PatternConstructor::Absent);
				}
				required_ctors.push(PatternConstructor::Structure);
			}
			TyData::Error => return Ok(()),
			_ => return Err(SemanticPattern::Wildcard(ty)),
		}

		let used_ctors = FxHashSet::from_iter(constructors);
		for c in required_ctors {
			if !used_ctors.contains(&&c) {
				// Give this constructor as one which needs to be added to the case expression
				// (The empty list of parameters will be printed as the correct number of _ later)
				return Err(SemanticPattern::Constructor(ty, c, Box::new([])));
			}
		}
		Ok(())
	}

	fn counter_example(&self, matrix: &Matrix, ty: Ty) -> Option<String> {
		let ps = self.generate_counter_example(matrix, &[ty])?;
		assert_eq!(ps.len(), 1);
		Some(ps.first().unwrap().pretty_print(self.db))
	}

	fn generate_counter_example(
		&self,
		matrix: &Matrix,
		tys: &[Ty],
	) -> Option<Vec<SemanticPattern>> {
		if matrix.rows() == 0 {
			return Some(
				tys.iter()
					.map(|ty| SemanticPattern::Wildcard(*ty))
					.collect(),
			);
		}
		assert_eq!(matrix.cols(), tys.len());
		if tys.is_empty() {
			return None;
		}
		let ctors = matrix
			.col(0)
			.filter_map(|p| match p {
				SemanticPattern::Constructor(ty, c, ps) => Some((*ty, c, ps)),
				_ => None,
			})
			.collect::<Vec<_>>();
		match self.check_constructors(ctors.iter().map(|(_, c, _)| *c), tys[0]) {
			Ok(()) => ctors.iter().find_map(|(ty, c, ps)| {
				let mut new_tys = Vec::with_capacity(tys.len() + ps.len() - 1);
				new_tys.extend(ps.iter().map(|p| p.ty()));
				new_tys.extend(tys[1..].iter().copied());
				let s = self.specialise_matrix(c, ps.len(), matrix);
				let pats = self.generate_counter_example(&s, &new_tys)?;
				let (pre, post) = pats.split_at(ps.len());
				let ctor = SemanticPattern::Constructor(
					*ty,
					(**c).clone(),
					pre.to_vec().into_boxed_slice(),
				);
				let mut result = Vec::with_capacity(post.len() + 1);
				result.push(ctor);
				result.extend(post.iter().cloned());
				Some(result)
			}),
			Err(p) => {
				let mut ps =
					self.generate_counter_example(&self.default_matrix(matrix), &tys[1..])?;
				ps.insert(0, p);
				Some(ps)
			}
		}
	}

	fn lower_pattern(&self, pattern: ArenaIndex<Pattern>) -> SemanticPattern {
		let pat_ty = &self.types[pattern];
		if let PatternTy::Destructuring(ty) = pat_ty {
			if *ty == self.type_registry.error {
				return SemanticPattern::Wildcard(*ty);
			}
		}
		match (&self.data[pattern], pat_ty) {
			(Pattern::Absent, PatternTy::Destructuring(ty)) => {
				SemanticPattern::Constructor(*ty, PatternConstructor::Absent, Box::new([]))
			}
			(Pattern::Anonymous, PatternTy::Destructuring(ty)) => SemanticPattern::Wildcard(*ty),
			(Pattern::Boolean(b), PatternTy::Destructuring(ty)) => {
				SemanticPattern::Constructor(*ty, PatternConstructor::Boolean(*b), Box::new([]))
			}
			(
				Pattern::Call {
					function,
					arguments,
				},
				PatternTy::Destructuring(ty),
			) => SemanticPattern::Constructor(
				*ty,
				PatternConstructor::Named(self.types.pattern_resolution(*function).unwrap()),
				arguments
					.iter()
					.map(|arg| self.lower_pattern(*arg))
					.collect(),
			),
			(Pattern::Identifier(_), PatternTy::Destructuring(ty)) => SemanticPattern::Constructor(
				*ty,
				PatternConstructor::Named(self.types.pattern_resolution(pattern).unwrap()),
				Box::new([]),
			),
			(Pattern::Identifier(_), PatternTy::Variable(ty)) => SemanticPattern::Wildcard(*ty),
			(Pattern::Float { negated, value }, PatternTy::Destructuring(ty)) => {
				SemanticPattern::Constructor(
					*ty,
					PatternConstructor::Float {
						negated: *negated,
						value: *value,
					},
					Box::new([]),
				)
			}
			(Pattern::Infinity { negated }, PatternTy::Destructuring(ty)) => {
				SemanticPattern::Constructor(
					*ty,
					PatternConstructor::Infinity { negated: *negated },
					Box::new([]),
				)
			}
			(Pattern::Integer { negated, value }, PatternTy::Destructuring(ty)) => {
				SemanticPattern::Constructor(
					*ty,
					PatternConstructor::Integer {
						negated: *negated,
						value: *value,
					},
					Box::new([]),
				)
			}
			(Pattern::Record { fields }, PatternTy::Destructuring(ty)) => {
				let field_pats = FxHashMap::from_iter(fields.iter().copied());
				SemanticPattern::Constructor(
					*ty,
					PatternConstructor::Structure,
					match ty.lookup(self.db.upcast()) {
						TyData::Record(_, fs) => fs
							.iter()
							.map(|(i, _)| self.lower_pattern(field_pats[&Identifier(*i)]))
							.collect(),
						_ => unreachable!(),
					},
				)
			}
			(Pattern::String(s), PatternTy::Destructuring(ty)) => SemanticPattern::Constructor(
				*ty,
				PatternConstructor::String(s.clone()),
				Box::new([]),
			),
			(Pattern::Tuple { fields }, PatternTy::Destructuring(ty)) => {
				SemanticPattern::Constructor(
					*ty,
					PatternConstructor::Structure,
					fields.iter().map(|p| self.lower_pattern(*p)).collect(),
				)
			}
			(Pattern::Missing, PatternTy::Destructuring(ty)) => SemanticPattern::Wildcard(*ty),
			_ => unreachable!(),
		}
	}
}
