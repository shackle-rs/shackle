//! Validation of object-syntax constructs.
//!
//! THIR object lowering supports a specific set of introduction and attribute
//! shapes. This pass walks the HIR after type checking and emits
//! `UnsupportedObjectFeature` diagnostics for each unsupported shape, so users
//! see a clean error instead of a THIR panic.
//!
//! Rejected shapes:
//!
//! - `new` attributes forming a recursive ownership cycle (the
//!   potential-object universe would be infinite);
//! - `array [d] of new C` at top level (disallowed by design — the array
//!   index would duplicate the object identity);
//! - top-level declarations whose type contains `new` through a tuple,
//!   record, or other non-direct path;
//! - class attributes whose declared type contains `new` through a tuple,
//!   record, array, or other unsupported shape;
//! - attributes of var-reached classes that cannot be varified or declared
//!   as free decisions.

use shackle_diagnostics::UnsupportedObjectFeature;
use shackle_utils::hash::{Map, Set};

use crate::{
	ClassMember, Db, Expression, ExpressionId, Identifier, Item, ItemData, Pattern, PatternTy,
	PrimitiveType, Type, TypeId,
	class_analysis::var_reached_classes,
	diagnostics::Errors,
	ids::{PatternRef, TypeRef},
	lower::lower_models,
	scope::GlobalScope,
	typecheck::TypeResult,
};

/// Validate that all object-syntax constructs in the model use shapes that
/// THIR lowering can handle. Emits `UnsupportedObjectFeature` diagnostics for
/// any unsupported shape.
pub fn validate_object_lowering(db: &dyn Db) {
	log::info!("Validating object lowering shapes");

	validate_ownership_cycles(db);

	for model in lower_models(db).iter() {
		for item in model.items(db).iter() {
			match item {
				Item::Declaration(d) => {
					validate_root_decl(db, *item, *d);
				}
				Item::Class(c) => {
					validate_class(db, *item, *c);
				}
				_ => {}
			}
		}
	}
}

/// Reject recursive `new` ownership.
///
/// A cycle in the class graph over `new`-typed attributes — including edges
/// through inheritance, since a subclass inherits its ancestors' `new`
/// attributes — means every potential object owns a fresh child whose class
/// (transitively) owns such an object again: the potential-object universe is
/// infinite, so no finite storage/identity lowering exists. This is a
/// language-level rejection, not a missing feature: recursive and cyclic
/// structure is modelled with *reference* attributes (class-typed, without
/// `new`) selecting from a bounded pool of separately-introduced objects.
/// Reference attributes never enter this graph, so supported reference cycles
/// are untouched by construction.
///
/// Cyclic classes are rejected at their declaration whether or not anything
/// instantiates them: such a class is unrealizable per se, and rejecting only
/// reached cycles would report the error far from its cause, at whichever
/// declaration happens to reach it.
///
/// [`super::class_analysis`]'s occurrence expansion carries a matching
/// termination fence (its `expansion_path`), because it can run before the
/// diagnostic emitted here surfaces.
///
/// Class names are resolved via the pre-typecheck global scope (the
/// [`super::class_analysis::var_reached_classes`] technique), NOT via the
/// typechecked item signatures: on a cyclic class graph the typer itself can
/// fail to resolve the very attribute domains that form the cycle (e.g. the
/// A<->B signature cycle of `class A ( new B: b; ); class B extends A;`), and
/// the rejection must not depend on the typer surviving the shape it rejects.
fn validate_ownership_cycles<'db>(db: &'db dyn Db) {
	struct NewAttrEdge<'db> {
		item: Item<'db>,
		declared_type: TypeId<'db>,
		attribute: Identifier<'db>,
		introduced: PatternRef<'db>,
	}

	let resolve_class =
		|data: &ItemData<'db>, expr: ExpressionId<'db>| -> Option<PatternRef<'db>> {
			let Expression::Identifier(identifier) = data[expr] else {
				return None;
			};
			let pattern = GlobalScope::find_variable(db, identifier)?;
			matches!(pattern.item(db), Item::Class(_)).then_some(pattern)
		};

	// The class graph: superclass links and the `new`-typed attributes
	// declared directly on each class.
	let mut superclasses: Map<PatternRef<'db>, Option<PatternRef<'db>>> = Map::default();
	let mut direct_new_attrs: Map<PatternRef<'db>, Vec<NewAttrEdge<'db>>> = Map::default();
	let mut class_order: Vec<PatternRef<'db>> = Vec::new();

	for model in lower_models(db).iter() {
		for item in model.items(db).iter() {
			let Item::Class(c) = item else {
				continue;
			};
			let class = c.class(db);
			let data = class.data();
			let class_pattern = PatternRef::new(db, *item, class.pattern);
			let superclass = class.extends.and_then(|base| resolve_class(data, base));
			let mut edges = Vec::new();
			for ci in class.items.iter() {
				let ClassMember::Declaration(decl) = ci else {
					continue;
				};
				let Some(introduced) = data[decl.declared_type]
					.get_new_class(data)
					.and_then(|domain| resolve_class(data, domain))
				else {
					continue;
				};
				let Some(attribute) = data[decl.pattern].identifier() else {
					continue;
				};
				edges.push(NewAttrEdge {
					item: *item,
					declared_type: decl.declared_type,
					attribute,
					introduced,
				});
			}
			let _ = superclasses.insert(class_pattern, superclass);
			let _ = direct_new_attrs.insert(class_pattern, edges);
			class_order.push(class_pattern);
		}
	}

	// Ancestors of a class, including itself. Guarded against malformed
	// inheritance cycles (rejected by the typer) so the walk always
	// terminates.
	let ancestors_or_self = |class: PatternRef<'db>| -> Vec<PatternRef<'db>> {
		let mut chain = Vec::new();
		let mut seen = Set::default();
		let mut current = Some(class);
		while let Some(c) = current {
			if !seen.insert(c) {
				break;
			}
			chain.push(c);
			current = superclasses.get(&c).copied().flatten();
		}
		chain
	};

	// Effective ownership edge: `c -> d` when `c` declares *or inherits* a
	// `new` attribute introducing `d`. Occurrence expansion descends through
	// inherited attributes too, so the cycle test must as well.
	let effective_children = |class: PatternRef<'db>| -> Vec<PatternRef<'db>> {
		ancestors_or_self(class)
			.into_iter()
			.flat_map(|a| {
				direct_new_attrs
					.get(&a)
					.into_iter()
					.flatten()
					.map(|edge| edge.introduced)
			})
			.collect()
	};

	// Classes reachable from `start` over effective edges, including `start`
	// itself: a zero-length path back to a class that has the attribute makes
	// the attribute's own edge the cycle.
	let reachable_from = |start: PatternRef<'db>| -> Set<PatternRef<'db>> {
		let mut reached = Set::default();
		let mut worklist = vec![start];
		while let Some(c) = worklist.pop() {
			if !reached.insert(c) {
				continue;
			}
			worklist.extend(effective_children(c));
		}
		reached
	};

	// An attribute (declared on `owner`, introducing `introduced`) closes an
	// ownership cycle iff some class that has the attribute — `owner` itself
	// or a transitive subclass — is reachable from `introduced` again.
	let name = |p: PatternRef<'db>| p.identifier(db).map(|i| i.lookup(db)).unwrap_or_default();
	for owner in class_order.iter() {
		for edge in direct_new_attrs[owner].iter() {
			let reach = reachable_from(edge.introduced);
			let Some(witness) = class_order
				.iter()
				.copied()
				.find(|c| reach.contains(c) && ancestors_or_self(*c).contains(owner))
			else {
				continue;
			};
			let attr_name = edge.attribute.lookup(db);
			let witness_name = name(witness);
			let introduced_name = name(edge.introduced);
			let cycle = if witness == edge.introduced {
				format!(
					"attribute `{attr_name}` gives every `{witness_name}` object \
					 its own fresh `{witness_name}`"
				)
			} else {
				format!(
					"attribute `{attr_name}` gives every `{witness_name}` object a \
					 fresh `{introduced_name}`, and `{introduced_name}` transitively \
					 `new`-introduces `{witness_name}` again"
				)
			};
			push_unsupported(
				db,
				edge.item,
				edge.declared_type,
				&format!(
					"`new` attributes must not form an ownership cycle: {cycle}, \
					 so the potential-object universe would be infinite. Model \
					 recursive or cyclic structure with a reference attribute \
					 instead: a class-typed attribute without `new` (e.g. `var opt \
					 {introduced_name}: {attr_name}`) selecting from a bounded pool \
					 of objects introduced elsewhere"
				),
			);
		}
	}
}

/// Validate a top-level (non-class) declaration whose declared type
/// might contain `new`. Supported shapes:
///
/// - `Type::New { .. }`
/// - `Type::Set { element: Type::New { .. }, .. }`
///
/// Anything else that contains `new` is unsupported.
fn validate_root_decl<'db>(db: &'db dyn Db, item: Item<'db>, d: crate::DeclarationItem<'db>) {
	let decl = d.declaration(db);
	let data = decl.data();
	if !data[decl.declared_type].is_new(data) {
		return;
	}

	match &data[decl.declared_type] {
		// Singular `new` roots (par and var) are handled by the general
		// introduction machinery.
		Type::New { .. } => {}
		Type::Set { element, .. } => {
			let element = *element;
			if !matches!(data[element], Type::New { .. }) {
				push_unsupported(
					db,
					item,
					decl.declared_type,
					"only `set of new C` with a direct class element is supported",
				);
			}
		}
		// `array [d] of new C` roots are DISALLOWED by design. An array of new
		// objects assigns each object two independent identities — the
		// object's own `<C>_potential` identity and its array index — and adds
		// no expressivity over `set of new C` (which already lets you refer to
		// the constructed objects directly). The redundant array index would
		// force the dense 1-based identity universe to stay in sync with an
		// arbitrary user index set. Use `set of new C` for a pool, and an
		// explicit reference array (`array [E] of C`) or a key attribute if
		// the objects need to be keyed.
		Type::Array { .. } => {
			push_unsupported(
				db,
				item,
				decl.declared_type,
				"`array [d] of new C` is not supported: an array of new objects \
				 conflates object identity with the array index and adds nothing \
				 over `set of new C`. Use `set of new C` for a pool of objects \
				 (refer to them directly from the set), plus a reference array \
				 `array [E] of C` or a key attribute if you need to index them",
			);
		}
		_ => {
			push_unsupported(
				db,
				item,
				decl.declared_type,
				"this declaration shape with `new` is not supported by object \
				 lowering yet — only `new C`, `set of new C`, and \
				 `var set(...) of new C` are supported at top level",
			);
		}
	}
}

/// Validate each class declaration's attributes for unsupported shapes.
///
/// Supported attribute shapes containing `new`:
///
/// - `new C` (singular, possibly opt, possibly var)
/// - `var set(...) of new C`
/// - `set of new C`
///
/// Unsupported: tuples or records containing `new C`, arrays of `new`, and
/// (on var-reached classes) attributes that cannot be varified or declared
/// as free decisions.
fn validate_class<'db>(db: &'db dyn Db, item: Item<'db>, c: crate::ClassItem<'db>) {
	let class = c.class(db);
	let data = class.data();
	for ci in class.items.iter() {
		if let ClassMember::Declaration(decl) = ci {
			if !data[decl.declared_type].is_new(data) {
				continue;
			}
			match &data[decl.declared_type] {
				Type::New { .. } => {
					// An `opt new C` field (par or var) is supported. A par
					// (data-supplied) optional child lowers like a
					// `set(0..1) of new C` — the child data rides in a
					// 0/1-length input list, stored as an identity-or-absent
					// `opt <C>_potential`. A var / var-reached optional child
					// stays a free decision. No fence needed.
				}
				Type::Set {
					element,
					cardinality,
					..
				} => {
					let element = *element;
					if !matches!(data[element], Type::New { .. }) {
						push_unsupported(
							db,
							item,
							decl.declared_type,
							"only `set of new C` with a direct class element \
							 is supported as a class attribute",
						);
					} else if cardinality.is_none()
						&& var_reached_classes(db).contains(&PatternRef::new(
							db,
							item,
							class.pattern,
						)) {
						// A `set of new C` attribute with no cardinality declares a
						// *variable* number of fresh objects once the class is
						// reached through a `var new` introduction — which forces
						// every attribute to var. Without a cardinality bound the
						// potential-object universe is unbounded, so the field is
						// not varifiable. This is the same failure as a `string` or
						// `set of float` attribute in a varified class; require an
						// explicit cardinality so the universe stays finite. (A
						// purely par-reached `set of new C` is fine — its
						// instantiation supplies the concrete objects, hence the
						// cardinality.)
						let class_name = PatternRef::new(db, item, class.pattern)
							.identifier(db)
							.map(|i| i.lookup(db))
							.unwrap_or_default();
						push_unsupported(
							db,
							item,
							decl.declared_type,
							&format!(
								"a `set of new` attribute of a varified class must \
								 declare a cardinality (e.g. `set(e) of new <Class>`): \
								 class `{class_name}` is reachable from a `var new` \
								 introduction, which forces every attribute to be var, \
								 and an unbounded var set of fresh objects has no finite \
								 storage universe"
							),
						);
					}
				}
				Type::Array { .. } => {
					push_unsupported(
						db,
						item,
						decl.declared_type,
						"`array of new C` as a class attribute is not \
						 supported yet — use `set of new C` or `var set(...) \
						 of new C`",
					);
				}
				Type::Tuple { .. } | Type::Record { .. } => {
					push_unsupported(
						db,
						item,
						decl.declared_type,
						"`new C` cannot appear inside a tuple or record \
						 attribute type",
					);
				}
				_ => {
					push_unsupported(
						db,
						item,
						decl.declared_type,
						"this class attribute shape with `new` is not \
						 supported by object lowering yet",
					);
				}
			}
		}
	}

	// A single-dimension `array [d] of <ClassRef>` attribute (a scalar
	// class-reference element, no `new`) IS supported — dims-preserving
	// storage (`class_storage_field_domain` carries the declared `[d]` and
	// gives each slot the `<C>_potential` identity) plus a column-projected
	// var-identity read-back. Every OTHER array-of-class shape has no storage
	// lowering: a multi-dimension array (`array [_,_] of B`), or an array
	// whose element is itself a set / array / record / tuple containing a
	// class reference. The shared storage record can only give those an
	// erased element type, so the read-back (`arrayXd(<obj>.f, ..)`) fails
	// MiniZinc flattening with "array dimensions unknown". (`array of new C`
	// is rejected in the loop above; sets of class references lower fine.)
	// Reject the unsupported shapes cleanly.
	{
		let types = item.types(db);
		for ci in class.items.iter() {
			let ClassMember::Declaration(decl) = ci else {
				continue;
			};
			if data[decl.declared_type].is_new(data) {
				continue;
			}
			let field_ty = match &types[decl.pattern] {
				PatternTy::Variable(t) | PatternTy::Destructuring(t) => *t,
				_ => continue,
			};
			// An array node whose element (walked) contains a class reference,
			// EXCEPT the supported single-dimension scalar-class-element shape
			// (`array [d] of <Class>`, `d` not a tuple of dims).
			let unsupported_array_of_class = field_ty.walk(db).any(|t| {
				matches!(
					t.lookup(db),
					shackle_ty::TyData::Array { dim, element, .. }
						if element
							.walk(db)
							.any(|e| e.class_type(db).is_some())
							&& (dim.is_tuple(db)
								|| !matches!(
									element.lookup(db),
									shackle_ty::TyData::Class(_, _, _)
								))
				)
			});
			if unsupported_array_of_class {
				let field_name = PatternRef::new(db, item, decl.pattern)
					.identifier(db)
					.map(|i| i.lookup(db))
					.unwrap_or_default();
				push_unsupported(
					db,
					item,
					decl.declared_type,
					&format!(
						"attribute `{}` of type `{}` is not supported: only a \
						 single-dimension array of class references \
						 (`array [d] of <Class>`) has an object storage lowering. \
						 Multi-dimension arrays and arrays whose element wraps a \
						 class (a set/array/record of `<Class>`) cannot carry their \
						 per-object dimensions in the shared storage record. Flatten \
						 to a single-dimension `array [d] of <Class>`, or use \
						 `set of new <Class>` / `set(<n>) of new <Class>`",
						field_name,
						field_ty.pretty_print(db),
					),
				);
			}
		}
	}

	// Varifiability check. When a class is reached through a `var new`
	// introduction the var-reach cascade forces *every* attribute to var.
	// An attribute whose type cannot be made var — a `string`, a `set of
	// float`, an annotation, etc. — is therefore an error, the same as the
	// `set of new` without cardinality case handled above (which `make_var`
	// cannot see, because the missing cardinality lives in the AST, not the
	// resolved `Ty`). Reaching for `make_var` here catches the rest. Only
	// attributes *declared* on this class are checked; inherited ones are
	// caught when their declaring (also var-reached) superclass is
	// validated.
	let class_pattern = PatternRef::new(db, item, class.pattern);
	if var_reached_classes(db).contains(&class_pattern) {
		let types = item.types(db);
		let attribute_patterns = {
			let mut patterns = Vec::new();
			collect_class_attribute_patterns(db, item, &mut patterns);
			patterns
		};
		if let PatternTy::ClassDecl {
			storage_record_ty, ..
		} = &types[class.pattern]
		{
			let storage_fields = storage_record_ty.record_fields(db).unwrap_or_default();
			for ci in class.items.iter() {
				let ClassMember::Declaration(decl) = ci else {
					continue;
				};
				// `new`-shaped attributes are validated above; `make_var`
				// would wrongly accept e.g. an uncardinalitied `set of new C`.
				if data[decl.declared_type].is_new(data) {
					continue;
				}
				let Some(field_name) = PatternRef::new(db, item, decl.pattern).identifier(db)
				else {
					continue;
				};
				let Some((_, field_ty)) = storage_fields.iter().find(|(n, _)| *n == field_name.0)
				else {
					continue;
				};
				// A domain that references a sibling attribute (`var 1..z: s`)
				// is not varifiable: the var-reach cascade forces `z` to var,
				// and a MiniZinc domain must be par. (The lowering's per-object
				// `let { var 1..z: .. }` mint would emit a var-bounded let
				// domain the target MiniZinc rejects outright: "type-inst must
				// be par set".) This holds whether or not the attribute is
				// computed — the domain is illegal either way. A par class may
				// keep such domains: its sibling stays par.
				if type_references_attribute(data, &types, &attribute_patterns, decl.declared_type)
				{
					push_unsupported(
						db,
						item,
						decl.declared_type,
						&format!(
							"the domain of attribute `{}` references a sibling \
							 attribute, but class `{}` is reachable from a `var new` \
							 introduction, which forces every attribute to be var — \
							 and a domain must be par. Declare the attribute with a \
							 par domain and add a class constraint instead (e.g. \
							 `var 1..<ub>: {};` with `constraint {} <= <sibling>;`)",
							field_name.lookup(db),
							class_pattern
								.identifier(db)
								.map(|i| i.lookup(db))
								.unwrap_or_default(),
							field_name.lookup(db),
							field_name.lookup(db),
						),
					);
					continue;
				}
				// Computed attributes (`var set of int: z = g(x)`) are never free
				// storage decisions — they are *defined* as generator aliases in
				// the storage-reconstruction comprehension, inheriting their
				// var-ness from the RHS. So the varifiability check doesn't apply:
				// an unbounded `var set of int` is fine as an alias even though it
				// can't be a free var decision.
				//
				// The one exception is an *array-typed* field: reading such a
				// field back out of storage (`obj.y`) reconstructs each object
				// record by projecting field columns, which can't yet reshape an
				// array-of-arrays column. Rather than let that panic downstream,
				// reject array-typed computed attributes with a clear message.
				if decl.definition.is_some() {
					let has_array = field_ty.walk(db).any(|t| t.dim_ty(db).is_some());
					if has_array {
						push_unsupported(
							db,
							item,
							decl.declared_type,
							&format!(
								"computed attribute `{}` of array type `{}` is not yet \
								 supported on class `{}`, which is reachable from a \
								 `var new` introduction",
								field_name.lookup(db),
								field_ty.pretty_print(db),
								class_pattern
									.identifier(db)
									.map(|i| i.lookup(db))
									.unwrap_or_default(),
							),
						);
					}
					continue;
				}
				if field_ty.make_var(db).is_none() {
					// An array has no var form as a whole, but a
					// single-dimension array attribute with object-independent
					// dimensions and a varifiable element is stored as an
					// array OF var elements, and the column-projected
					// read-back covers it. Ragged dimensions (`array [1..x]`
					// with `x` an attribute) were already rejected by the
					// `type_references_attribute` check above; multi-dimension
					// arrays keep this diagnostic.
					let uniform_array = matches!(
						field_ty.lookup(db),
						shackle_ty::TyData::Array { dim, element, .. }
							if !dim.is_tuple(db) && element.make_var(db).is_some()
					);
					if !uniform_array {
						push_unsupported(
							db,
							item,
							decl.declared_type,
							&format!(
								"attribute `{}` of type `{}` cannot be made var, but class \
								 `{}` is reachable from a `var new` introduction, which \
								 forces every attribute to be var",
								field_name.lookup(db),
								field_ty.pretty_print(db),
								class_pattern
									.identifier(db)
									.map(|i| i.lookup(db))
									.unwrap_or_default(),
							),
						);
						continue;
					}
				}
				// Free-declarability. A non-computed attribute of a var-reached
				// class becomes a FREE decision in the `_storage` array —
				// `make_var` succeeding (a type-level check) is not enough; the
				// field must be declarable as a free var decision. A set with
				// an unbounded element type (`var set of int`) passes
				// `make_var` but a free var set needs a finite element domain:
				// the emitted model compiles and then aborts in the solver.
				// Arrays need no arm here — `make_var` already rejects every
				// array shape above. Computed set attributes stay exempt (they
				// are aliases, not free decisions).
				if let Type::Set { element, .. } = &data[decl.declared_type]
					&& matches!(
						data[*element],
						Type::Primitive {
							primitive_type: PrimitiveType::Int,
							..
						}
					) {
					push_unsupported(
						db,
						item,
						decl.declared_type,
						&format!(
							"attribute `{}` of type `{}` cannot be a free decision \
							 variable, but class `{}` is reachable from a `var new` \
							 introduction: a free var set needs a finite element \
							 domain (e.g. `var set of 0..9`)",
							field_name.lookup(db),
							field_ty.pretty_print(db),
							class_pattern
								.identifier(db)
								.map(|i| i.lookup(db))
								.unwrap_or_default(),
						),
					);
				}
			}
		}
	}
}

/// Collect the attribute patterns of `class_item` and its superclasses
/// (superclass attributes first). Used to detect attribute-referencing
/// domains.
fn collect_class_attribute_patterns<'db>(
	db: &'db dyn Db,
	class_item: Item<'db>,
	out: &mut Vec<PatternRef<'db>>,
) {
	let Item::Class(c) = class_item else {
		return;
	};
	let class = c.class(db);
	let types = class_item.types(db);
	if let Some(base) = class.extends.and_then(|base| types.name_resolution(base)) {
		collect_class_attribute_patterns(db, base.item(db), out);
	}
	for ci in class.items.iter() {
		if let ClassMember::Declaration(d) = ci {
			for pattern in Pattern::identifiers(d.pattern, class.data()) {
				out.push(PatternRef::new(db, class_item, pattern));
			}
		}
	}
}

/// Whether a declared type carries a `Bounded` domain expression that
/// references one of `attribute_patterns` — e.g. `var 1..z: s` or
/// `set of 1..z` where `z` is a sibling (possibly inherited) attribute.
/// Recurses through set elements and array dimensions/elements; `new`-shaped
/// types are validated separately and never reach this check.
fn type_references_attribute<'db>(
	data: &ItemData<'db>,
	types: &TypeResult<'db>,
	attribute_patterns: &[PatternRef<'db>],
	ty: TypeId<'db>,
) -> bool {
	let expr_references = |domain: ExpressionId<'db>| {
		Expression::walk(domain, data).any(|sub| {
			matches!(&data[sub], Expression::Identifier(_))
				&& types
					.name_resolution(sub)
					.is_some_and(|res| attribute_patterns.contains(&res))
		})
	};
	match &data[ty] {
		Type::Bounded { domain, .. } => expr_references(*domain),
		Type::Set { element, .. } => {
			type_references_attribute(data, types, attribute_patterns, *element)
		}
		Type::Array {
			dimensions,
			element,
			..
		} => {
			type_references_attribute(data, types, attribute_patterns, *dimensions)
				|| type_references_attribute(data, types, attribute_patterns, *element)
		}
		Type::Tuple { fields, .. } => fields
			.iter()
			.any(|f| type_references_attribute(data, types, attribute_patterns, *f)),
		Type::Record { fields, .. } => fields
			.iter()
			.any(|(_, f)| type_references_attribute(data, types, attribute_patterns, *f)),
		_ => false,
	}
}

fn push_unsupported<'db>(db: &'db dyn Db, item: Item<'db>, ty: TypeId<'db>, msg: &str) {
	let (src, span) = TypeRef::new(db, item, ty).source_span(db);
	Errors::add(
		db,
		UnsupportedObjectFeature {
			src,
			span,
			msg: msg.to_owned(),
		},
	);
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_syntax::InputLang;

	use crate::{
		db::CompilerDatabase,
		input::{CompilerSettings, InlineModelFile, InputFiles},
		run_hir_phase,
	};

	fn check_object_errors(model: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model = InlineModelFile::new(&db, model.to_owned(), InputLang::MiniZinc);
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![model.into()]);
		let errors = run_hir_phase(&db)
			.errors
			.iter()
			.map(|e| e.to_string())
			.collect::<Vec<_>>()
			.join("\n");
		expected.assert_eq(&errors);
	}

	#[test]
	fn test_supported_object_shapes_pass_validation() {
		check_object_errors(
			r#"
			class A (var bool: x);
			new A: a;
			set of new A: pool;
			"#,
			expect![""],
		);
	}

	#[test]
	fn test_array_of_new_root_is_rejected() {
		check_object_errors(
			r#"
			class A (var bool: x);
			array [int] of new A: pool;
			"#,
			expect!["Unsupported object feature"],
		);
	}

	#[test]
	fn test_new_inside_tuple_attribute_is_rejected() {
		check_object_errors(
			r#"
			class B (var bool: y);
			class A (tuple(int, new B): t);
			"#,
			expect!["Unsupported object feature"],
		);
	}

	#[test]
	fn test_ownership_cycle_is_rejected() {
		check_object_errors(
			r#"
			class A (new A: child);
			"#,
			expect!["Unsupported object feature"],
		);
	}
}
