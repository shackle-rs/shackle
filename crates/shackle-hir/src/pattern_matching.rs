//! Analysis of pattern matching to check exhaustiveness
//!
//! See <http://moscova.inria.fr/~maranget/papers/warn/warn.pdf> for algorithm details

use derive_more::From;
use shackle_diagnostics::{NonExhaustivePatternMatching, UnreachablePattern};
use shackle_ty::{EnumRef, OptType, Ty, TyData, registry::TypeRegistry};
use shackle_utils::{
	InternedString,
	hash::{Map, Set},
};

use crate::{
	BooleanLiteral, Db, Expression, FloatLiteral, Identifier, IntegerLiteral, Item, ItemData,
	Model, Pattern, PatternId, PatternTy, StringLiteral, TypeResult,
	diagnostics::{Errors, Warnings},
	ids::{ExpressionRef, PatternRef},
	lower::lower_models,
};

/// Compute a mapping from (non-introduced) enum types to the constructors for the enum
#[salsa::tracked]
pub fn enum_constructors<'db>(db: &'db dyn Db) -> Map<EnumRef<'db>, Vec<PatternRef<'db>>> {
	let mut result = Map::default();
	for model in lower_models(db).iter() {
		for it in model.items(db).iter() {
			match it {
				Item::Enumeration(item) => {
					let e = item.enumeration(db);
					if let Some(def) = &e.definition {
						let enum_ref = EnumRef::new(
							PatternRef::new(db, *it, e.pattern).identifier(db).unwrap(),
						);
						let constructors = def
							.iter()
							.map(|c| PatternRef::new(db, *it, c.constructor_pattern()))
							.collect::<Vec<_>>();
						let _ = result.insert(enum_ref, constructors);
					}
				}
				Item::EnumAssignment(item) => {
					let e = item.enum_assignment(db);
					let types = it.types(db);
					if let Some(p) = types.name_resolution(e.assignee) {
						let enum_ref = EnumRef::new(p.identifier(db).unwrap());
						let constructors = e
							.definition
							.iter()
							.map(|c| PatternRef::new(db, *it, c.constructor_pattern()))
							.collect::<Vec<_>>();
						let _ = result.insert(enum_ref, constructors);
					}
				}
				_ => (),
			}
		}
	}
	result
}

#[salsa::tracked]
fn lookup_enum_constructors_internal<'db>(
	db: &'db dyn Db,
	e: InternedString<'db>,
) -> Option<Vec<PatternRef<'db>>> {
	let map = enum_constructors(db);
	map.get(&EnumRef::new(e)).cloned()
}

/// Lookup the enum constructors for the given enum type
pub fn lookup_enum_constructors<'db>(
	db: &'db dyn Db,
	e: EnumRef<'db>,
) -> &'db Option<Vec<PatternRef<'db>>> {
	lookup_enum_constructors_internal(db, e.name())
}

/// Check case exhaustiveness for all models
#[salsa::tracked(returns(copy))]
pub fn check_case_exhaustiveness(db: &dyn Db) {
	for model in lower_models(db).iter() {
		model.check_case_exhaustiveness(db);
	}
}

impl<'db> Model<'db> {
	/// Check case exhaustiveness for all items in this model
	pub fn check_case_exhaustiveness(&self, db: &'db dyn Db) {
		check_model_case_exhaustiveness(db, *self);
	}
}

#[salsa::tracked(returns(copy))]
fn check_model_case_exhaustiveness<'db>(db: &'db dyn Db, model: Model<'db>) {
	log::info!("Checking case exhaustiveness for model: {}", model.file(db));
	for item in model.items(db).iter() {
		item.check_case_exhaustiveness(db);
	}
}

impl<'db> Item<'db> {
	/// Check case exhaustiveness for expressions in this item
	pub fn check_case_exhaustiveness(&self, db: &'db dyn Db) {
		check_item_case_exhaustiveness(db, *self);
	}
}

/// Check that all case statements in this item are exhaustive
#[salsa::tracked(returns(copy))]
fn check_item_case_exhaustiveness<'db>(db: &'db dyn Db, item: Item<'db>) {
	let data = item.data(db);
	let types = item.types(db);
	for e in data.expressions.values() {
		if let Expression::Case(c) = e {
			let checker = ExhaustivenessChecker::new(db, data, &types);
			let mut matrix = Matrix::with_capacity(c.cases.len());
			for arm in c.cases.iter() {
				let pat = checker.lower_pattern(arm.pattern);
				let row = vec![pat];
				if !checker.is_useful(&matrix, &row) {
					// Useless case
					let (src, span) = PatternRef::new(db, item, arm.pattern).source_span(db);
					Warnings::add(db, UnreachablePattern { src, span });
				}
				matrix.add_row(row);
			}
			if let Some(pat) = checker.counter_example(&matrix, types[c.expression]) {
				// Non-exhaustive
				let (src, span) = ExpressionRef::new(db, item, c.expression).source_span(db);
				Errors::add(
					db,
					NonExhaustivePatternMatching {
						src,
						span,
						msg: format!("Case '{}' not covered", pat),
					},
				);
			}
		}
	}
}

/// A pattern distilled into its meaning in the context of pattern matching.
#[derive(Clone, Debug, PartialEq, Eq)]
enum SemanticPattern<'db> {
	/// Constructor
	Constructor(
		Ty<'db>,
		PatternConstructor<'db>,
		Box<[SemanticPattern<'db>]>,
	),
	/// A pattern which matches anything (either `_` or an identifier which is not a constructor).
	Wildcard(Ty<'db>),
}

impl<'db> SemanticPattern<'db> {
	fn ty(&self) -> Ty<'db> {
		match self {
			SemanticPattern::Constructor(ty, _, _) => *ty,
			SemanticPattern::Wildcard(ty) => *ty,
		}
	}

	fn pretty_print(&self, db: &'db dyn Db) -> String {
		match self {
			SemanticPattern::Constructor(_, PatternConstructor::Absent, _) => "<>".to_owned(),
			SemanticPattern::Constructor(ty, PatternConstructor::Structure, ps) => {
				match ty.lookup(db) {
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
				let item = p.item(db);
				let data = item.data(db);
				let types = item.types(db);
				match &types[p.pattern(db)] {
					PatternTy::EnumAtom(_) => {
						data[p.pattern(db)].identifier().unwrap().pretty_print(db)
					}
					PatternTy::EnumConstructor(ec) => {
						let call = data[p.pattern(db)].identifier().unwrap().pretty_print(db);
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
enum PatternConstructor<'db> {
	/// Named constructor (for enum/annotation)
	Named(PatternRef<'db>),
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
	String(StringLiteral<'db>),
}

#[derive(Default, Clone, Debug, PartialEq, Eq, From)]
struct Matrix<'db> {
	patterns: Vec<Vec<SemanticPattern<'db>>>,
}

impl<'db> Matrix<'db> {
	fn with_capacity(row_capacity: usize) -> Self {
		Self {
			patterns: Vec::with_capacity(row_capacity),
		}
	}

	fn add_row(&mut self, ps: Vec<SemanticPattern<'db>>) {
		self.patterns.push(ps);
	}

	fn col(&self, c: usize) -> impl '_ + Iterator<Item = &SemanticPattern<'db>> {
		self.patterns.iter().map(move |ps| &ps[c])
	}

	fn iter_rows(&self) -> impl '_ + Iterator<Item = &[SemanticPattern<'db>]> {
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
struct ExhaustivenessChecker<'db> {
	db: &'db dyn Db,
	data: &'db ItemData<'db>,
	types: &'db TypeResult<'db>,
}

impl<'db> ExhaustivenessChecker<'db> {
	fn new(db: &'db dyn Db, data: &'db ItemData, types: &'db TypeResult) -> Self {
		Self { db, data, types }
	}

	fn is_useful(&self, matrix: &Matrix<'db>, row: &[SemanticPattern<'db>]) -> bool {
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
		constructor: &PatternConstructor<'db>,
		arg_count: usize,
		matrix: &Matrix<'db>,
	) -> Matrix<'db> {
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
						let new_row =
							std::iter::repeat_n(SemanticPattern::Wildcard(*ty), arg_count)
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
		constructor: &PatternConstructor<'db>,
		arg_count: usize,
		row: &[SemanticPattern<'db>],
	) -> Vec<SemanticPattern<'db>> {
		let mut iter = row.iter();
		let first = iter.next().unwrap();
		match first {
			SemanticPattern::Constructor(_, c, ps) => {
				assert_eq!(c, constructor);
				ps.iter().cloned().chain(iter.cloned()).collect::<Vec<_>>()
			}
			SemanticPattern::Wildcard(ty) => {
				std::iter::repeat_n(SemanticPattern::Wildcard(*ty), arg_count)
					.chain(iter.cloned())
					.collect::<Vec<_>>()
			}
		}
	}

	fn default_matrix(&self, matrix: &Matrix<'db>) -> Matrix<'db> {
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

	fn check_constructors<'a>(
		&self,
		constructors: impl Iterator<Item = &'a PatternConstructor<'db>>,
		ty: Ty<'db>,
	) -> Result<(), SemanticPattern<'db>>
	where
		'db: 'a,
	{
		let mut required_ctors = Vec::new();
		match ty.lookup(self.db) {
			TyData::Enum(_, o, e) => {
				if *o == OptType::Opt {
					required_ctors.push(PatternConstructor::Absent);
				}
				if let Some(ctors) = lookup_enum_constructors(self.db, *e) {
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
				if *o == OptType::Opt {
					required_ctors.push(PatternConstructor::Absent);
				}
				required_ctors.push(PatternConstructor::Structure);
			}
			TyData::Error => return Ok(()),
			_ => return Err(SemanticPattern::Wildcard(ty)),
		}

		let used_ctors = Set::from_iter(constructors);
		for c in required_ctors {
			if !used_ctors.contains(&&c) {
				// Give this constructor as one which needs to be added to the case expression
				// (The empty list of parameters will be printed as the correct number of _ later)
				return Err(SemanticPattern::Constructor(ty, c, Box::new([])));
			}
		}
		Ok(())
	}

	fn counter_example(&self, matrix: &Matrix<'db>, ty: Ty<'db>) -> Option<String> {
		let ps = self.generate_counter_example(matrix, &[ty])?;
		assert_eq!(ps.len(), 1);
		Some(ps.first().unwrap().pretty_print(self.db))
	}

	fn generate_counter_example(
		&self,
		matrix: &Matrix<'db>,
		tys: &[Ty<'db>],
	) -> Option<Vec<SemanticPattern<'db>>> {
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

	fn lower_pattern(&self, pattern: PatternId<'db>) -> SemanticPattern<'db> {
		let types = TypeRegistry::lookup(self.db);
		let pat_ty = &self.types[pattern];
		if let PatternTy::Destructuring(ty) = pat_ty
			&& *ty == types.error
		{
			return SemanticPattern::Wildcard(*ty);
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
				let field_pats = Map::from_iter(fields.iter().copied());
				SemanticPattern::Constructor(
					*ty,
					PatternConstructor::Structure,
					match ty.lookup(self.db) {
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
