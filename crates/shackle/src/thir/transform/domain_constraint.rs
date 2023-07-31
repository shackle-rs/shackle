//! Turn domains into constraints/assertions
//! - Unpack struct variables
//! - Rewrite variable domains into constraints
//! - Rewrite par domains into assertions
//!
//! This has to be done before types are erased, since otherwise error messages may refer to
//! integers instead of enums, or tuples instead of records
//!

// use std::sync::Arc;

// use rustc_hash::FxHashMap;

// use crate::{
// 	constants::IdentifierRegistry,
// 	hir::{Identifier, IntegerLiteral},
// 	thir::{
// 		db::Thir,
// 		source::Origin,
// 		traverse::{add_function, fold_domain, Folder, ReplacementMap},
// 		ArrayComprehension, ArrayLiteral, Branch, Call, Callable, Declaration, DeclarationId,
// 		Domain, DomainData, Expression, ExpressionData, FunctionId, IfThenElse, Item, Let,
// 		LookupCall, Marker, Model,
// 	},
// };

// enum DomainItem {
// 	Array,
// 	Tuple(IntegerLiteral),
// 	Record(Identifier),
// }

// struct DomainRewriter<Dst, Src = ()> {
// 	model: Model<Dst>,
// 	replacement_map: ReplacementMap<Dst, Src>,
// 	ids: Arc<IdentifierRegistry>,
// 	current: Option<FunctionId<Src>>,
// 	constraints: Vec<Expression<Dst>>,
// 	domain_constraints: FxHashMap<FunctionId<Src>, Vec<Expression<Dst>>>,
// }

// impl<Dst: Marker, Src: Marker> Folder<Dst, Src> for DomainRewriter<Dst, Src> {
// 	fn model(&mut self) -> &mut Model<Dst> {
// 		&mut self.model
// 	}

// 	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src> {
// 		&mut self.replacement_map
// 	}

// 	fn add_function(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
// 		self.current = Some(f);
// 		add_function(self, db, model, f);
// 		let constraints = std::mem::replace(&mut self.constraints, Vec::new());
// 		if !self.constraints.is_empty() {
// 			self.domain_constraints.insert(f, constraints);
// 		}
// 		self.current = None;
// 	}

// 	fn fold_domain(
// 		&mut self,
// 		db: &dyn Thir,
// 		model: &Model<Src>,
// 		domain: &Domain<Src>,
// 	) -> Domain<Dst> {
// 		if let DomainData::Bounded(e) = &**domain {
// 			let origin = domain.origin();
// 			let folded = self.fold_expression(db, model, e);
// 			let ty = folded.ty();
// 			let expression = if matches!(&*folded, ExpressionData::Identifier(_)) {
// 				folded
// 			} else {
// 				let mut declaration = Declaration::new(true, Domain::unbounded(db, origin, ty));
// 				declaration.set_definition(folded);
// 				let idx = self.model.add_declaration(Item::new(declaration, origin));
// 				Expression::new(db, &self.model, origin, idx)
// 			};
// 			todo!()
// 			// return Domain::unbounded(
// 			// 	db,
// 			// 	origin,
// 			// 	ty.inst(db.upcast()).unwrap(),
// 			// 	ty.opt(db.upcast()).unwrap(),
// 			// 	expression,
// 			// );
// 		}
// 		fold_domain(self, db, model, domain)
// 	}

// 	fn fold_function_body(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
// 		let dst = self.fold_function_id(db, model, f);
// 		let folded = self.fold_expression(db, model, model[f].body().unwrap());
// 		self.model[dst].set_body(folded);
// 	}
// }

// impl<Dst: Marker, Src: Marker> DomainRewriter<Dst, Src> {
// 	fn collect_domain_constraints(&mut self, domain: &Domain<Dst>, stack: &mut Vec<DomainItem>) {
// 		match &**domain {
// 			DomainData::Array(_, d) => {
// 				stack.push(DomainItem::Array);
// 				self.collect_domain_constraints(d, stack);
// 				stack.pop().unwrap();
// 			}
// 			DomainData::Set(d) => self.collect_domain_constraints(d, stack),
// 			DomainData::Tuple(ds) => {
// 				for (i, d) in ds.iter().enumerate() {
// 					stack.push(DomainItem::Tuple(IntegerLiteral(i as i64 + 1)));
// 					self.collect_domain_constraints(d, stack);
// 					stack.pop();
// 				}
// 			}
// 			DomainData::Record(ds) => {
// 				for (i, d) in ds.iter() {
// 					stack.push(DomainItem::Record(*i));
// 					self.collect_domain_constraints(d, stack);
// 					stack.pop();
// 				}
// 			}
// 			DomainData::Bounded(e) => {
// 				todo!()
// 				// self.add_domain_constraint(e);
// 			}
// 			_ => (),
// 		}
// 	}
// }

// /// Convert function domains to constraints/assertions
// pub fn domain_constraint(db: &dyn Thir, model: &Model) -> Model {
// 	let mut c = DomainRewriter {
// 		model: Model::default(),
// 		replacement_map: ReplacementMap::default(),
// 		ids: db.identifier_registry(),
// 		current: None,
// 		constraints: Vec::new(),
// 		domain_constraints: FxHashMap::default(),
// 	};
// 	c.add_model(db, model);
// 	c.model
// }

// #[cfg(test)]
// mod test {
// 	use expect_test::expect;

// 	use crate::thir::transform::test::check_no_stdlib;

// 	use super::domain_constraint;

// 	#[test]
// 	fn test_domain_constraint() {
// 		check_no_stdlib(
// 			domain_constraint,
// 			r#"
//             predicate foo(var int: x) = true;
//             test foo(int: x) = false;
//             test is_fixed(var int: x);
//             function int: fix(var int: x);
//             "#,
// 			expect!([r#"
//     function var bool: foo(var int: x) = if is_fixed(x) then foo(fix(x)) else true endif;
//     function bool: foo(int: x) = false;
//     function bool: is_fixed(var int: x);
//     function int: fix(var int: x);
//     solve satisfy;
// "#]),
// 		);
// 	}
// }
