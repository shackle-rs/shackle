//! Destructuring declarations and HIR-to-THIR pattern lowering.

use rustc_hash::FxHashMap;
use shackle_hir::{
	PatternTy,
	ids::{EntityRef, PatternRef},
};

use crate::{
	lower::{
		LoweredIdentifier,
		expression::{ExpressionCollector, alloc_expression},
	},
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Create declarations which perform destructuring according to the given pattern
	pub(in crate::lower) fn collect_destructuring(
		&mut self,
		root_decl: DeclarationId<'db>,
		top_level: bool,
		pattern: shackle_hir::PatternId<'db>,
	) -> Vec<DeclarationId<'db>> {
		let mut destructuring = Vec::new();
		let mut todo = vec![(0, pattern)];
		while let Some((i, p)) = todo.pop() {
			match &self.data[p] {
				shackle_hir::Pattern::Tuple { fields } => {
					for (idx, field) in fields.iter().enumerate() {
						// Destructuring returns the field inside
						destructuring.push(DestructuringEntry::new(
							i,
							Destructuring::TupleAccess(IntegerLiteral(idx as i64 + 1)),
							*field,
						));
						todo.push((destructuring.len(), *field));
					}
				}
				shackle_hir::Pattern::Record { fields } => {
					for (ident, field) in fields.iter() {
						// Destructuring returns the field inside
						destructuring.push(DestructuringEntry::new(
							i,
							Destructuring::RecordAccess(*ident),
							*field,
						));
						todo.push((destructuring.len(), *field));
					}
				}
				shackle_hir::Pattern::Call {
					function,
					arguments,
				} => {
					let destructuring_pattern = if arguments.len() == 1 {
						// If we have a single arg, destructuring will return the inside directly
						arguments[0]
					} else {
						// Destructuring returns a tuple
						p
					};
					let pat = self.types.pattern_resolution(*function).unwrap();
					let res = &self.parent.resolutions[&pat];
					match res {
						LoweredIdentifier::Callable(Callable::Annotation(ann)) => {
							destructuring.push(DestructuringEntry::new(
								i,
								Destructuring::Annotation(*ann),
								destructuring_pattern,
							));
						}
						LoweredIdentifier::Callable(Callable::EnumConstructor(member)) => {
							destructuring.push(DestructuringEntry::new(
								i,
								Destructuring::Enumeration(*member),
								destructuring_pattern,
							));
						}
						_ => unreachable!(),
					};
					let j = destructuring.len();
					if arguments.len() == 1 {
						todo.push((j, arguments[0]));
					} else {
						for (idx, field) in arguments.iter().enumerate() {
							// Destructuring the tuple returns the field inside
							destructuring.push(DestructuringEntry::new(
								j,
								Destructuring::TupleAccess(IntegerLiteral(idx as i64 + 1)),
								*field,
							));
							todo.push((destructuring.len(), *field));
						}
					}
				}
				shackle_hir::Pattern::Identifier(name) => {
					if matches!(
						&self.types[p],
						PatternTy::Variable(_) | PatternTy::Argument(_)
					) {
						if i > 0 {
							destructuring[i - 1].name = Some(*name);
							// Mark used destructurings as to be created
							let mut c = i;
							loop {
								if c == 0 {
									break;
								}
								let item = &mut destructuring[c - 1];
								if item.create {
									break;
								}
								item.create = true;
								c = item.parent;
							}
						} else {
							self.parent.model[root_decl].set_name(*name);
							let _ = self.parent.resolutions.insert(
								PatternRef::new(self.parent.db, self.item, pattern),
								LoweredIdentifier::ResolvedIdentifier(root_decl.into()),
							);
						}
					}
				}
				_ => (),
			}
		}
		let mut decls = Vec::new();
		let mut decl_map = FxHashMap::default();
		for (idx, item) in destructuring
			.into_iter()
			.enumerate()
			.filter(|(_, item)| item.create)
		{
			let origin = EntityRef::new(
				self.parent.db,
				self.item,
				shackle_hir::ids::EntityId::from(item.pattern),
			);
			let decl = self.introduce_declaration(top_level, origin, |collector| {
				let ident = alloc_expression(
					if item.parent == 0 {
						root_decl
					} else {
						decl_map[&item.parent]
					},
					collector,
					origin,
				);
				match item.kind {
					Destructuring::Annotation(a) => alloc_expression(
						Call {
							function: Callable::AnnotationDestructure(a),
							arguments: vec![ident],
						},
						collector,
						origin,
					),
					Destructuring::Enumeration(e) => alloc_expression(
						Call {
							function: Callable::EnumDestructor(e),
							arguments: vec![ident],
						},
						collector,
						origin,
					),
					Destructuring::RecordAccess(f) => alloc_expression(
						RecordAccess {
							record: Box::new(ident),
							field: f,
						},
						collector,
						origin,
					),
					Destructuring::TupleAccess(f) => alloc_expression(
						TupleAccess {
							tuple: Box::new(ident),
							field: f,
						},
						collector,
						origin,
					),
				}
			});
			if let Some(name) = item.name {
				self.parent.model[decl].set_name(name);
				let _ = self.parent.resolutions.insert(
					PatternRef::new(self.parent.db, self.item, item.pattern),
					LoweredIdentifier::ResolvedIdentifier(decl.into()),
				);
			}
			let _ = decl_map.insert(idx + 1, decl);
			decls.push(decl);
		}
		decls
	}

	/// Lower an HIR pattern into a THIR pattern
	pub(super) fn collect_pattern(&mut self, pattern: shackle_hir::PatternId<'db>) -> Pattern<'db> {
		let db = self.parent.db;
		let origin = EntityRef::new(db, self.item, shackle_hir::ids::EntityId::from(pattern));
		let ty = match &self.types[pattern] {
			PatternTy::Destructuring(ty) => *ty,
			PatternTy::Variable(ty) | PatternTy::Argument(ty) => {
				return Pattern::anonymous(*ty, origin);
			}
			_ => unreachable!(),
		};
		match &self.data[pattern] {
			shackle_hir::Pattern::Absent => {
				Pattern::expression(alloc_expression(Absent, self, origin), origin)
			}
			shackle_hir::Pattern::Anonymous => Pattern::anonymous(ty, origin),
			shackle_hir::Pattern::Boolean(b) => {
				Pattern::expression(alloc_expression(*b, self, origin), origin)
			}
			shackle_hir::Pattern::Call {
				function,
				arguments,
			} => {
				let args = arguments
					.iter()
					.map(|a| self.collect_pattern(*a))
					.collect::<Vec<_>>();
				let pat = self.types.pattern_resolution(*function).unwrap();
				let res = &self.parent.resolutions[&pat];
				match res {
					LoweredIdentifier::Callable(Callable::Annotation(ann)) => {
						Pattern::annotation_constructor(db, &self.parent.model, origin, *ann, args)
					}
					LoweredIdentifier::Callable(Callable::EnumConstructor(member)) => {
						Pattern::enum_constructor(db, &self.parent.model, origin, *member, args)
					}
					_ => unreachable!(),
				}
			}
			shackle_hir::Pattern::Float { negated, value } => {
				let v = alloc_expression(*value, self, origin);
				Pattern::expression(
					if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					},
					origin,
				)
			}
			shackle_hir::Pattern::Identifier(_) => {
				let pat = self.types.pattern_resolution(pattern).unwrap();
				let res = &self.parent.resolutions[&pat];
				match res {
					LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Annotation(a)) => {
						Pattern::expression(alloc_expression(*a, self, origin), origin)
					}
					LoweredIdentifier::ResolvedIdentifier(
						ResolvedIdentifier::EnumerationMember(m),
					) => Pattern::expression(alloc_expression(*m, self, origin), origin),
					_ => unreachable!(),
				}
			}
			shackle_hir::Pattern::Infinity { negated } => {
				let v = alloc_expression(Infinity, self, origin);
				Pattern::expression(
					if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					},
					origin,
				)
			}
			shackle_hir::Pattern::Integer { negated, value } => {
				let v = alloc_expression(*value, self, origin);
				Pattern::expression(
					if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					},
					origin,
				)
			}
			shackle_hir::Pattern::Missing => unreachable!(),
			shackle_hir::Pattern::Record { fields } => {
				let fields = fields
					.iter()
					.map(|(i, p)| (*i, self.collect_pattern(*p)))
					.collect::<Vec<_>>();
				Pattern::record(db, &self.parent.model, origin, fields)
			}
			shackle_hir::Pattern::String(s) => {
				Pattern::expression(alloc_expression(s.clone(), self, origin), origin)
			}
			shackle_hir::Pattern::Tuple { fields } => {
				let fields = fields
					.iter()
					.map(|f| self.collect_pattern(*f))
					.collect::<Vec<_>>();
				Pattern::tuple(db, &self.parent.model, origin, fields)
			}
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DestructuringEntry<'db> {
	parent: usize, // 0 means no parent, otherwise = index of parent + 1
	kind: Destructuring<'db>,
	pattern: shackle_hir::PatternId<'db>,
	name: Option<Identifier<'db>>,
	create: bool,
}

impl<'db> DestructuringEntry<'db> {
	fn new(parent: usize, kind: Destructuring<'db>, pattern: shackle_hir::PatternId<'db>) -> Self {
		Self {
			parent,
			kind,
			pattern,
			name: None,
			create: false,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Destructuring<'db> {
	TupleAccess(IntegerLiteral),
	RecordAccess(Identifier<'db>),
	Enumeration(EnumMemberId<'db>),
	Annotation(AnnotationId<'db>),
}
