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

use shackle_ty::{ClassRef, Ty};
use shackle_utils::hash::{Map, Set};

use crate::{
	ClassItem, ClassMember, Db, Expression, ExpressionId, Identifier, Item, ItemData, OptType,
	PatternTy, Type, TypeId, VarType, ids::PatternRef, lower::lower_models, scope::GlobalScope,
	typecheck::TypeResult,
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
pub fn introduces_var_existence<'db>(data: &ItemData<'db>, ty: TypeId<'db>) -> bool {
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

/// The classes introduced by some `new` attribute that closes an ownership
/// cycle.
///
/// A cycle in the class graph over `new`-typed attributes — following
/// inheritance, since a subclass has every attribute of its ancestors — makes
/// the potential-object universe infinite. Object validation rejects such
/// models with a proper diagnostic; this query exists for the typer, whose
/// input record for a class inlines the input records of the classes its
/// attributes `new`-introduce. Without a fence that inlining grows by one
/// level of nesting at every signature fixpoint iteration instead of
/// converging, so the typer drops `new` attributes whose introduced class is
/// in this set from the input record.
///
/// Like the var-flow queries, this reads only the AST and the pre-typecheck
/// global scope, so signature construction can consult it without forming a
/// cycle.
#[salsa::tracked(returns(ref))]
pub fn ownership_cyclic_classes<'db>(db: &'db dyn Db) -> Set<PatternRef<'db>> {
	let nodes = class_nodes(db);

	// Ancestors of a class, including itself; guarded against malformed
	// inheritance cycles (rejected by the typer) so the walk terminates.
	let ancestors_or_self = |class: PatternRef<'db>| {
		let mut chain = Vec::new();
		let mut seen: Set<PatternRef<'db>> = Set::default();
		let mut current = Some(class);
		while let Some(c) = current {
			if !seen.insert(c) {
				break;
			}
			chain.push(c);
			current = nodes.get(&c).and_then(|n| n.superclass);
		}
		chain
	};

	// Effective ownership edge: `c -> d` when `c` declares or inherits a `new`
	// attribute introducing `d`.
	let effective_children = |class: PatternRef<'db>| -> Vec<PatternRef<'db>> {
		ancestors_or_self(class)
			.into_iter()
			.flat_map(|a| {
				nodes
					.get(&a)
					.into_iter()
					.flat_map(|n| n.new_attributes.iter().map(|attr| attr.introduced))
			})
			.collect()
	};

	nodes
		.keys()
		.copied()
		.filter(|class| {
			// On a cycle iff reachable from itself via at least one edge.
			let mut reached: Set<PatternRef<'db>> = Set::default();
			let mut worklist = effective_children(*class);
			while let Some(c) = worklist.pop() {
				if c == *class {
					return true;
				}
				if !reached.insert(c) {
					continue;
				}
				worklist.extend(effective_children(c));
			}
			false
		})
		.collect()
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

/// Stable identifier for one object-introduction occurrence in the lowering plan.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, salsa::Update)]
pub struct OccurrenceId(pub u32);

/// Origin of an object-introduction occurrence.
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub enum OccurrenceSource<'db> {
	/// A top-level declaration introducing objects.
	TopLevelDeclaration(PatternRef<'db>),
	/// A class attribute introducing child objects.
	ClassAttribute {
		/// The class that declares the attribute.
		owner_class: PatternRef<'db>,
		/// Item index of the declaring class item.
		item_index: usize,
		/// Attribute name.
		attribute: Identifier<'db>,
	},
}

/// Shape of the potential identity domain for an occurrence.
#[derive(Copy, Clone, Debug, PartialEq, Eq, salsa::Update)]
pub enum LocalDomainSource {
	/// Exactly one potential object is introduced.
	SingleObject,
	/// A top-level collection introduces one block of potential objects.
	TopLevelCollection,
	/// A nested singular attribute reuses the parent ordinal domain.
	OnePerParent,
	/// A nested set attribute flattens child objects across parent ordinals.
	FlattenedChildCollection,
}

/// Class-level metadata needed by object lowering.
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct ClassDescriptor<'db> {
	/// The class pattern.
	pub class_pattern: PatternRef<'db>,
	/// Optional superclass pattern.
	pub superclass: Option<PatternRef<'db>>,
	/// Input record type used to materialize parameter objects.
	pub input_record_ty: Ty<'db>,
	/// Full storage record type used to represent object fields.
	pub storage_record_ty: Ty<'db>,
}

/// One object-introduction occurrence in the lowering plan.
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct Occurrence<'db> {
	/// Stable occurrence id.
	pub id: OccurrenceId,
	/// The class directly introduced by this occurrence.
	pub introduced_class: PatternRef<'db>,
	/// Syntactic source of the occurrence.
	pub source: OccurrenceSource<'db>,
	/// Parent occurrence when this is nested.
	pub parent: Option<OccurrenceId>,
	/// Attribute path from the nearest top-level input root.
	pub path: Vec<Identifier<'db>>,
	/// Shape of the local ordinal domain for this occurrence.
	pub local_domain_source: LocalDomainSource,
	/// Whether this occurrence is reached via a `var new` path. True if the
	/// introducing declaration has var inst or any ancestor occurrence does.
	/// Used to pick the varified storage record type at class registration.
	pub is_var: bool,
}

/// Contribution of one occurrence to a target class enum.
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct OccurrenceContribution<'db> {
	/// Source occurrence.
	pub occurrence: OccurrenceId,
	/// Target class that receives a constructor block.
	pub target_class: PatternRef<'db>,
	/// Constructor index within the target class enum.
	pub constructor_index: usize,
	/// Projection depth where `0` means the directly introduced class.
	pub projection_depth: usize,
}

/// Result of object-lowering analysis.
#[derive(Clone, Debug, Default, PartialEq, Eq, salsa::Update)]
pub struct ClassAnalysisResult<'db> {
	/// Stable class metadata in declaration order.
	pub class_descriptors: Vec<ClassDescriptor<'db>>,
	/// Object-introduction occurrences in lowering order.
	pub occurrences: Vec<Occurrence<'db>>,
	/// Constructor contributions grouped by target class.
	pub contributions: Vec<OccurrenceContribution<'db>>,
	/// Mapping from superclass to direct subclasses.
	pub map_class_to_subclasses: Map<PatternRef<'db>, Vec<PatternRef<'db>>>,
	/// Classes whose storage records must be varified because some var
	/// introduction contributes to them (directly or via subclass projection).
	pub var_reached_classes: Set<PatternRef<'db>>,
	/// Classes whose lowered actual set is a `var set` because their existence
	/// (set membership or optional presence) is a solver decision. A strict
	/// subset of `var_reached_classes`: a singular `var new C` is var-reached
	/// (var storage) but not var-actual (the object always exists).
	pub var_actual_set_classes: Set<PatternRef<'db>>,
}

#[derive(Clone, Debug)]
struct DeclaredNewAttribute<'db> {
	owner_class: PatternRef<'db>,
	item_index: usize,
	attribute: Identifier<'db>,
	introduced_class: PatternRef<'db>,
	local_domain_source: LocalDomainSource,
	/// Whether the attribute declaration itself has a var inst at any level.
	is_var: bool,
	/// Whether the attribute is a singular optional child (`opt new C` or
	/// `var opt new C`). A *par* such field (see `child_is_var` in
	/// [`expand_nested_occurrences`]) is a data-supplied 0-or-1 child and is
	/// rerouted to `FlattenedChildCollection`.
	is_opt_new: bool,
}

/// Whether a declared type is a singular optional `new` introduction
/// (`opt new C` / `var opt new C`).
fn declared_type_is_opt_new<'db>(data: &ItemData<'db>, ty: TypeId<'db>) -> bool {
	matches!(
		&data[ty],
		Type::New {
			opt: OptType::Opt,
			..
		}
	)
}

#[derive(Clone, Debug)]
struct ClassInfo<'db> {
	descriptor: ClassDescriptor<'db>,
	declared_new_attributes: Vec<DeclaredNewAttribute<'db>>,
}

/// Build the object-lowering analysis snapshot for the current HIR.
///
/// Unlike the scope-only queries above, this reads the typechecked signatures,
/// so it must only be consulted after signature typing (object validation and
/// THIR lowering), never from within it.
#[salsa::tracked(returns(ref))]
pub fn analyse_new_objects<'db>(db: &'db dyn Db) -> ClassAnalysisResult<'db> {
	let mut result = ClassAnalysisResult::default();
	let mut class_infos: Map<PatternRef<'db>, ClassInfo<'db>> = Map::default();

	for model in lower_models(db).iter() {
		for it in model.items(db).iter() {
			let Item::Class(item) = it else { continue };
			let class = item.class(db);
			let data = class.data();
			let types = it.types(db);
			let class_pattern = PatternRef::new(db, *it, class.pattern);
			let (superclass, input_record_ty, storage_record_ty) = match &types[class.pattern] {
				PatternTy::ClassDecl {
					input_record_ty,
					storage_record_ty,
					defining_set_ty,
					..
				} => (
					defining_set_ty
						.class_type(db)
						.and_then(|class_ty| class_ty.superclass())
						.and_then(|super_ty| super_ty.class_type(db))
						.and_then(|class_ty| class_pattern_for(db, class_ty)),
					*input_record_ty,
					*storage_record_ty,
				),
				_ => unreachable!("class pattern must typecheck as a class declaration"),
			};

			let descriptor = ClassDescriptor {
				class_pattern,
				superclass,
				input_record_ty,
				storage_record_ty,
			};
			result.class_descriptors.push(descriptor.clone());

			let mut declared_new_attributes = Vec::new();
			for (class_item_index, member) in class.items.iter().enumerate() {
				let ClassMember::Declaration(decl) = member else {
					continue;
				};
				// The domain of a well-typed `new` attribute always resolves.
				// It can be left unresolved when the class graph is cyclic
				// (e.g. `class A ( new B: b; ); class B extends A;` — the
				// A<->B signature cycle makes the typer bail on the domain),
				// which object validation rejects via its scope-only
				// ownership-cycle check; skip the edge instead of panicking so
				// this analysis stays total until that diagnostic surfaces.
				let Some(introduced_class) = data[decl.declared_type]
					.get_new_class(data)
					.and_then(|domain| types.name_resolution(domain))
				else {
					continue;
				};
				let attribute = data[decl.pattern].identifier().unwrap();
				declared_new_attributes.push(DeclaredNewAttribute {
					owner_class: class_pattern,
					item_index: class_item_index,
					attribute,
					introduced_class,
					local_domain_source: local_domain_source(&data[decl.declared_type], true),
					is_var: declared_type_is_var(data, decl.declared_type),
					is_opt_new: declared_type_is_opt_new(data, decl.declared_type),
				});
			}

			if let Some(superclass) = superclass {
				result
					.map_class_to_subclasses
					.entry(superclass)
					.or_default()
					.push(class_pattern);
			}

			let _ = class_infos.insert(
				class_pattern,
				ClassInfo {
					descriptor,
					declared_new_attributes,
				},
			);
		}
	}

	let mut expansion_path = Set::default();
	for model in lower_models(db).iter() {
		for it in model.items(db).iter() {
			let Item::Declaration(item) = it else {
				continue;
			};
			let decl = item.declaration(db);
			let data = decl.data();
			let types = it.types(db);
			let Some((introduced_class, local_domain_source)) =
				top_level_new_class_info(&types, data, decl.declared_type)
			else {
				continue;
			};

			let source_pattern = PatternRef::new(db, *it, decl.pattern);
			let is_var = declared_type_is_var(data, decl.declared_type);
			let _ = add_occurrence(
				&class_infos,
				&mut result.occurrences,
				&mut result.contributions,
				introduced_class,
				OccurrenceSource::TopLevelDeclaration(source_pattern),
				None,
				Vec::new(),
				local_domain_source,
				is_var,
				&mut expansion_path,
			);
		}
	}

	// Share the var-reached / var-actual class sets with the scope-only
	// queries, so THIR (this analysis) and the HIR signatures agree on exactly
	// which classes are var-reached (var storage) and var-actual (var actual
	// set). The scope-based computations mirror the var-flow over the
	// occurrences/contributions built above.
	result.var_reached_classes = var_reached_classes(db).clone();
	result.var_actual_set_classes = var_actual_set_classes(db).clone();

	result
}

fn top_level_new_class_info<'db>(
	types: &TypeResult<'db>,
	data: &ItemData<'db>,
	declared_type: TypeId<'db>,
) -> Option<(PatternRef<'db>, LocalDomainSource)> {
	match &data[declared_type] {
		Type::New { domain, .. } => Some((
			types.name_resolution(*domain).unwrap(),
			LocalDomainSource::SingleObject,
		)),
		Type::Array { element, .. } => data[*element].get_new_class(data).map(|domain| {
			(
				types.name_resolution(domain).unwrap(),
				LocalDomainSource::TopLevelCollection,
			)
		}),
		Type::Set { element, .. } => match &data[*element] {
			Type::New { domain, .. } => Some((
				types.name_resolution(*domain).unwrap(),
				LocalDomainSource::TopLevelCollection,
			)),
			_ => None,
		},
		_ => None,
	}
}

fn local_domain_source<'db>(ty: &Type<'db>, nested: bool) -> LocalDomainSource {
	match ty {
		// A nested `opt new C` field starts as `OnePerParent` here. When it is a
		// PAR (data-supplied) optional child it is rerouted to
		// `FlattenedChildCollection` in `expand_nested_occurrences` — that is
		// the only place `child_is_var` (transitive var-reachedness) is known,
		// and a `var opt new C` / a par `opt new C` on a var-reached owner must
		// stay `OnePerParent` (a free decision, no input list).
		Type::New { .. } if nested => LocalDomainSource::OnePerParent,
		Type::New { .. } => LocalDomainSource::SingleObject,
		Type::Array { .. } if nested => LocalDomainSource::FlattenedChildCollection,
		Type::Array { .. } => LocalDomainSource::TopLevelCollection,
		Type::Set { .. } if nested => LocalDomainSource::FlattenedChildCollection,
		Type::Set { .. } => LocalDomainSource::TopLevelCollection,
		_ => unreachable!("called local_domain_source on a non-new type"),
	}
}

#[allow(
	clippy::too_many_arguments,
	reason = "recursive occurrence expansion threads its full accumulator state"
)]
fn add_occurrence<'db>(
	class_infos: &Map<PatternRef<'db>, ClassInfo<'db>>,
	occurrences: &mut Vec<Occurrence<'db>>,
	contributions: &mut Vec<OccurrenceContribution<'db>>,
	introduced_class: PatternRef<'db>,
	source: OccurrenceSource<'db>,
	parent: Option<OccurrenceId>,
	path: Vec<Identifier<'db>>,
	local_domain_source: LocalDomainSource,
	is_var: bool,
	expansion_path: &mut Set<PatternRef<'db>>,
) -> OccurrenceId {
	let id = OccurrenceId(occurrences.len() as u32);
	occurrences.push(Occurrence {
		id,
		introduced_class,
		source,
		parent,
		path: path.clone(),
		local_domain_source,
		is_var,
	});

	let mut projection_depth = 0;
	let mut current_class = Some(introduced_class);
	let mut per_class_counts: Map<PatternRef<'db>, usize> =
		contributions
			.iter()
			.fold(Map::default(), |mut counts, contribution| {
				*counts.entry(contribution.target_class).or_insert(0) += 1;
				counts
			});
	while let Some(target_class) = current_class {
		let constructor_index = *per_class_counts.entry(target_class).or_insert(0);
		contributions.push(OccurrenceContribution {
			occurrence: id,
			target_class,
			constructor_index,
			projection_depth,
		});
		*per_class_counts.get_mut(&target_class).unwrap() += 1;
		current_class = class_infos[&target_class].descriptor.superclass;
		projection_depth += 1;
	}

	// Termination fence for recursive `new` ownership. A class already on the
	// current expansion path would re-expand itself forever: the occurrence
	// tree of a `new`-cycle is infinite (each object owns a fresh child).
	// Object validation rejects ownership cycles with a proper diagnostic, so
	// the truncated plan is never lowered — the fence only keeps this analysis
	// from overflowing the stack before that diagnostic surfaces. Expansion
	// depth is thereby bounded by the number of classes.
	if expansion_path.insert(introduced_class) {
		expand_nested_occurrences(
			class_infos,
			occurrences,
			contributions,
			introduced_class,
			id,
			path,
			is_var,
			expansion_path,
		);
		let _ = expansion_path.remove(&introduced_class);
	}
	id
}

#[allow(
	clippy::too_many_arguments,
	reason = "recursive occurrence expansion threads its full accumulator state"
)]
fn expand_nested_occurrences<'db>(
	class_infos: &Map<PatternRef<'db>, ClassInfo<'db>>,
	occurrences: &mut Vec<Occurrence<'db>>,
	contributions: &mut Vec<OccurrenceContribution<'db>>,
	class_pattern: PatternRef<'db>,
	parent_occurrence: OccurrenceId,
	base_path: Vec<Identifier<'db>>,
	parent_is_var: bool,
	expansion_path: &mut Set<PatternRef<'db>>,
) {
	let class_info = &class_infos[&class_pattern];
	if let Some(superclass) = class_info.descriptor.superclass {
		expand_nested_occurrences(
			class_infos,
			occurrences,
			contributions,
			superclass,
			parent_occurrence,
			base_path.clone(),
			parent_is_var,
			expansion_path,
		);
	}

	for attribute in class_info.declared_new_attributes.iter() {
		let mut path = base_path.clone();
		path.push(attribute.attribute);
		// A nested attribute is var-reached if the parent occurrence is var
		// or if the attribute declaration itself has var inst: var-attribute
		// storage flows transitively through nested `new` paths.
		let child_is_var = parent_is_var || attribute.is_var;
		// A PAR (data-supplied) `opt new C` field is a 0-or-1 optional child,
		// lowered like a `set(0..1) of new C` collection through the
		// flattened-collection machinery. A `var opt new C`, or a par
		// `opt new C` on a var-reached owner, is instead a free decision
		// (`child_is_var`) with no input list, so it stays `OnePerParent` and
		// the var-scalar machinery handles it.
		let local_domain_source = if attribute.is_opt_new && !child_is_var {
			LocalDomainSource::FlattenedChildCollection
		} else {
			attribute.local_domain_source
		};
		let _ = add_occurrence(
			class_infos,
			occurrences,
			contributions,
			attribute.introduced_class,
			OccurrenceSource::ClassAttribute {
				owner_class: attribute.owner_class,
				item_index: attribute.item_index,
				attribute: attribute.attribute,
			},
			Some(parent_occurrence),
			path,
			local_domain_source,
			child_is_var,
			expansion_path,
		);
	}
}
