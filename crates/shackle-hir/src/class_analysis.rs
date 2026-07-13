//! Analysis of class declarations and object introductions.
//!
//! The queries here answer two kinds of question about a model's classes:
//!
//! - which HIR item declares a given class type ([`class_items`]), and
//! - how var-ness flows through object introductions
//!   ([`var_reached_classes`], [`var_actual_set_classes`]).
//!
//! The var-flow queries are deliberately computed from the AST and the
//! pre-typecheck global scope alone. Signature construction consults
//! [`var_actual_set_classes`] to decide whether a class's defining set is a
//! `var set`, so if these queries were to type any item they would pull
//! signature typing into its own fixpoint iteration.

use shackle_ty::ClassRef;
use shackle_utils::hash::{Map, Set};

use crate::{
	ClassItem, ClassMember, Db, Expression, ExpressionId, Identifier, Item, ItemData, OptType,
	Type, TypeId, VarType, ids::PatternRef, lower::lower_models, scope::GlobalScope,
};

/// Map each class name to the item which declares it.
///
/// Class types identify their class by name, so this is how a [`ClassRef`] is
/// resolved back to the HIR. Reads only the lowered models, which keeps it
/// usable from signature construction.
#[salsa::tracked(returns(ref))]
pub fn class_items<'db>(db: &'db dyn Db) -> Map<Identifier<'db>, ClassItem<'db>> {
	let mut result = Map::default();
	for model in lower_models(db).iter() {
		for it in model.items(db).iter() {
			if let Item::Class(item) = it {
				let c = item.class(db);
				let name = PatternRef::new(db, *it, c.pattern)
					.identifier(db)
					.expect("Class declaration must have identifier pattern");
				let _ = result.insert(name, *item);
			}
		}
	}
	result
}

/// Get the item declaring the class this class type refers to
pub fn class_item_for<'db>(db: &'db dyn Db, class: ClassRef<'db>) -> Option<ClassItem<'db>> {
	class_items(db).get(&Identifier(class.name())).copied()
}

/// Get the pattern declaring the class this class type refers to
pub fn class_pattern_for<'db>(db: &'db dyn Db, class: ClassRef<'db>) -> Option<PatternRef<'db>> {
	let item = class_item_for(db, class)?;
	let item_ref: Item<'db> = item.into();
	Some(PatternRef::new(db, item_ref, item.class(db).pattern))
}

/// Map each class to its direct subclasses.
///
/// Derived from the `extends` edges alone, so like the var-flow queries this is
/// free of any dependency on type inference.
#[salsa::tracked(returns(ref))]
pub fn class_subclasses<'db>(db: &'db dyn Db) -> Map<PatternRef<'db>, Vec<PatternRef<'db>>> {
	let mut result: Map<PatternRef<'db>, Vec<PatternRef<'db>>> = Map::default();
	for (class, node) in class_nodes(db) {
		if let Some(superclass) = node.superclass {
			result.entry(superclass).or_default().push(class);
		}
	}
	result
}

/// Whether a declared type introduces var storage through any
/// `new`/`set of new` reached under array and set wrappers.
fn declared_type_is_var<'db>(data: &ItemData<'db>, ty: TypeId<'db>) -> bool {
	match &data[ty] {
		Type::New {
			inst: VarType::Var, ..
		}
		| Type::Set {
			inst: VarType::Var, ..
		} => true,
		Type::Array { element, .. } | Type::Set { element, .. } => {
			declared_type_is_var(data, *element)
		}
		_ => false,
	}
}

/// Whether a declared type makes the existence of the objects it introduces a
/// solver decision.
///
/// This is not the same as [`declared_type_is_var`], which detects var
/// *storage*. The two diverge on a singular `var new C`: its attributes are
/// decision variables, but the object itself always exists. Var existence
/// requires either a var set of `new` (the solver picks the members) or a
/// `var opt new` (presence is a decision), reached through any number of array
/// or set wrappers. A par set wrapper keeps membership fixed.
pub(crate) fn introduces_var_existence<'db>(data: &ItemData<'db>, ty: TypeId<'db>) -> bool {
	match &data[ty] {
		Type::Set {
			inst: VarType::Var,
			element,
			..
		} if data[*element].is_new(data) => true,
		Type::New {
			inst: VarType::Var,
			opt: OptType::Opt,
			..
		} => true,
		Type::Array { element, .. } | Type::Set { element, .. } => {
			introduces_var_existence(data, *element)
		}
		_ => false,
	}
}

/// Whether a nested attribute introduces a collection of children rather than a
/// single child per parent.
fn attribute_is_set_shaped<'db>(data: &ItemData<'db>, ty: TypeId<'db>) -> bool {
	matches!(&data[ty], Type::Array { .. } | Type::Set { .. })
}

/// Resolve a class-name expression to the class it names, via the global scope.
///
/// This is the same mechanism scope collection uses for `extends`, and it
/// avoids depending on type inference.
fn resolve_class<'db>(
	db: &'db dyn Db,
	data: &ItemData<'db>,
	expr: ExpressionId<'db>,
) -> Option<PatternRef<'db>> {
	let Expression::Identifier(identifier) = &data[expr] else {
		return None;
	};
	let pattern = GlobalScope::find_variable(db, *identifier)?;
	matches!(pattern.item(db), Item::Class(_)).then_some(pattern)
}

/// A `new` attribute declared by a class, as seen from the AST alone.
struct NewAttribute<'db> {
	/// The class whose objects the attribute introduces.
	introduced: PatternRef<'db>,
	/// Whether the attribute is a set/array of `new` rather than a singular `new`.
	set_shaped: bool,
	/// Whether the attribute is itself a var set of `new`.
	var_set: bool,
	/// Whether the attribute is a singular `opt new`, so presence guards it.
	singular_opt: bool,
	/// Whether the attribute's declared type is var storage.
	storage_var: bool,
}

/// The AST-and-scope view of one class.
struct ClassNode<'db> {
	superclass: Option<PatternRef<'db>>,
	new_attributes: Vec<NewAttribute<'db>>,
}

/// Build the AST-and-scope view of every class in the model.
fn class_nodes<'db>(db: &'db dyn Db) -> Map<PatternRef<'db>, ClassNode<'db>> {
	let mut nodes = Map::default();
	for model in lower_models(db).iter() {
		for it in model.items(db).iter() {
			let Item::Class(item) = it else { continue };
			let class = item.class(db);
			let data = class.data();
			let class_pattern = PatternRef::new(db, *it, class.pattern);
			let superclass = class.extends.and_then(|base| resolve_class(db, data, base));
			let mut new_attributes = Vec::new();
			for member in class.items.iter() {
				let ClassMember::Declaration(decl) = member else {
					continue;
				};
				let Some(domain) = data[decl.declared_type].get_new_class(data) else {
					continue;
				};
				let Some(introduced) = resolve_class(db, data, domain) else {
					continue;
				};
				new_attributes.push(NewAttribute {
					introduced,
					set_shaped: attribute_is_set_shaped(data, decl.declared_type),
					var_set: introduces_var_existence(data, decl.declared_type),
					singular_opt: matches!(
						&data[decl.declared_type],
						Type::New {
							opt: OptType::Opt,
							..
						}
					),
					storage_var: declared_type_is_var(data, decl.declared_type),
				});
			}
			let _ = nodes.insert(
				class_pattern,
				ClassNode {
					superclass,
					new_attributes,
				},
			);
		}
	}
	nodes
}

/// Every top-level declaration which introduces objects, as (class, declared type).
fn top_level_introductions<'db>(
	db: &'db dyn Db,
) -> Vec<(PatternRef<'db>, &'db ItemData<'db>, TypeId<'db>)> {
	let mut result = Vec::new();
	for model in lower_models(db).iter() {
		for it in model.items(db).iter() {
			let Item::Declaration(item) = it else {
				continue;
			};
			let decl = item.declaration(db);
			let data = decl.data();
			let Some(domain) = data[decl.declared_type].get_new_class(data) else {
				continue;
			};
			let Some(introduced) = resolve_class(db, data, domain) else {
				continue;
			};
			result.push((introduced, data, decl.declared_type));
		}
	}
	result
}

/// The classes whose storage records must be varified, because some var object
/// introduction reaches them directly or through a subclass projection.
///
/// Computed from the AST and the pre-typecheck global scope only, so that
/// signature construction can consult it without forming a salsa cycle.
#[salsa::tracked(returns(ref))]
pub fn var_reached_classes<'db>(db: &'db dyn Db) -> Set<PatternRef<'db>> {
	let nodes = class_nodes(db);

	// Seed the worklist with the top-level object introductions.
	let mut worklist = top_level_introductions(db)
		.into_iter()
		.map(|(introduced, data, declared_type)| {
			(introduced, declared_type_is_var(data, declared_type))
		})
		.collect::<Vec<_>>();

	// Propagate var-reach through inheritance and nested `new` attributes. An
	// introduction reaches its class and every ancestor, and expansion descends
	// through inherited attributes too. Keying the visited set on class *and*
	// var-ness guarantees termination even when the nesting graph has a cycle.
	let mut result = Set::default();
	let mut visited = Set::default();
	while let Some((class, is_var)) = worklist.pop() {
		if !visited.insert((class, is_var)) {
			continue;
		}
		let mut current = Some(class);
		while let Some(c) = current {
			let Some(node) = nodes.get(&c) else { break };
			if is_var {
				let _ = result.insert(c);
			}
			for attr in node.new_attributes.iter() {
				worklist.push((attr.introduced, is_var || attr.storage_var));
			}
			current = node.superclass;
		}
	}
	result
}

/// How a top-level object introduction shapes the actual set of its class.
#[derive(Copy, Clone, PartialEq, Eq)]
enum TopLevelIntroKind {
	/// `var opt new C`: presence is a decision, and the opt lowering defines the
	/// actual set of `C` and of every superclass via an occurs test.
	Optional,
	/// `var set(...) of new C`: membership is a decision, but only for `C` —
	/// superclasses fall back to the par potential universe.
	VarCollection,
	/// The actual set is par: the objects always exist and membership is fixed.
	Par,
}

fn classify_top_level_intro<'db>(data: &ItemData<'db>, ty: TypeId<'db>) -> TopLevelIntroKind {
	match &data[ty] {
		Type::New {
			inst: VarType::Var,
			opt: OptType::Opt,
			..
		} => TopLevelIntroKind::Optional,
		_ if introduces_var_existence(data, ty) => TopLevelIntroKind::VarCollection,
		_ => TopLevelIntroKind::Par,
	}
}

/// The classes whose lowered actual set is a `var set`, because their
/// membership or presence is a solver decision rather than fixed.
///
/// This is a strict subset of [`var_reached_classes`]: a singular `var new C`
/// makes storage var but leaves the actual set par, since the object always
/// exists and its lowered identity uses the actual set as a par domain.
///
/// Like [`var_reached_classes`], this reads only the AST and the pre-typecheck
/// global scope, so signature construction can consult it without forming a
/// cycle.
#[salsa::tracked(returns(ref))]
pub fn var_actual_set_classes<'db>(db: &'db dyn Db) -> Set<PatternRef<'db>> {
	let nodes = class_nodes(db);
	let var_reached = var_reached_classes(db);
	let mut result = Set::default();

	// Seed var-actual-ness from the top-level introductions, and remember which
	// classes have a top-level introduction at all: those get their actual set
	// from the top-level lowering rather than from a superclass projection.
	let mut reach_worklist = Vec::new();
	let mut top_level_introduced = Set::default();
	for (introduced, data, declared_type) in top_level_introductions(db) {
		match classify_top_level_intro(data, declared_type) {
			TopLevelIntroKind::Optional => {
				// The opt lowering defines the actual set of every projection target.
				let mut current = Some(introduced);
				while let Some(c) = current {
					let _ = result.insert(c);
					current = nodes.get(&c).and_then(|node| node.superclass);
				}
			}
			TopLevelIntroKind::VarCollection => {
				let _ = result.insert(introduced);
			}
			TopLevelIntroKind::Par => {}
		}
		let _ = top_level_introduced.insert(introduced);
		reach_worklist.push(introduced);
	}

	/// How var-actual-ness reaches a class from the class that owns the edge.
	enum EdgeKind {
		/// The owner declares a `new` attribute introducing the child.
		Introduction {
			set_shaped: bool,
			var_set: bool,
			singular_opt: bool,
			storage_var: bool,
		},
		/// The child is a proper ancestor of the owner, and derives its actual
		/// set as identity images gated by the owner's set.
		Projection,
	}
	struct Edge<'db> {
		owner: PatternRef<'db>,
		child: PatternRef<'db>,
		kind: EdgeKind,
	}

	// Collect the propagation edges over the reachable classes. A reached class
	// expands the nested `new` attributes of itself and of every ancestor, since
	// an inherited attribute applies to a subclass object too.
	let mut edges = Vec::new();
	let mut reached = Set::default();
	while let Some(class) = reach_worklist.pop() {
		if !reached.insert(class) {
			continue;
		}
		let mut current = Some(class);
		while let Some(owner) = current {
			let Some(node) = nodes.get(&owner) else { break };
			for attr in node.new_attributes.iter() {
				edges.push(Edge {
					owner,
					child: attr.introduced,
					kind: EdgeKind::Introduction {
						set_shaped: attr.set_shaped,
						var_set: attr.var_set,
						singular_opt: attr.singular_opt,
						storage_var: attr.storage_var,
					},
				});
				reach_worklist.push(attr.introduced);
			}
			if owner != class && !top_level_introduced.contains(&owner) {
				edges.push(Edge {
					owner: class,
					child: owner,
					kind: EdgeKind::Projection,
				});
			}
			current = node.superclass;
		}
	}

	// Propagate to a fixpoint. Set-shaped and singular-opt attributes have
	// statically known var-ness, because var storage makes the membership or
	// presence test a decision. Singular non-opt attributes and superclass
	// projections inherit the owner's var-actual-ness, since their derived sets
	// are guarded by the owner's own realisation. Var-actual-ness only grows and
	// the edges are finite, so this terminates.
	let mut changed = true;
	while changed {
		changed = false;
		for edge in edges.iter() {
			let child_is_var_actual = match &edge.kind {
				EdgeKind::Introduction {
					set_shaped,
					var_set,
					singular_opt,
					storage_var,
				} => {
					let owner_var_reached = var_reached.contains(&edge.owner);
					if *set_shaped {
						owner_var_reached || *var_set
					} else if *singular_opt {
						owner_var_reached || *storage_var
					} else {
						result.contains(&edge.owner)
					}
				}
				EdgeKind::Projection => result.contains(&edge.owner),
			};
			if child_is_var_actual && result.insert(edge.child) {
				changed = true;
			}
		}
	}
	result
}
