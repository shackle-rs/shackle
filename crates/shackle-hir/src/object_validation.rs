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

	fn db_for(source: &str) -> CompilerDatabase {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc);
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![file.into()]);
		db
	}

	fn db_for_with_stdlib(source: &str) -> CompilerDatabase {
		let mut db = CompilerDatabase::default();
		let file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc);
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![file.into()]);
		db
	}

	fn user_hir_errors(db: &CompilerDatabase) -> Vec<&shackle_diagnostics::Error> {
		run_hir_phase(db).errors
	}

	/// Assert that running the HIR phase on `source` surfaces an
	/// `UnsupportedObjectFeature` diagnostic whose message contains
	/// `expected_substring`.
	fn check_unsupported_object_diagnostic(source: &str, expected_substring: &str) {
		let db = db_for_with_stdlib(source);
		let errors = user_hir_errors(&db);
		assert!(
			!errors.is_empty(),
			"expected validation to surface an UnsupportedObjectFeature diagnostic; \
			 instead the HIR phase succeeded",
		);
		let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
		assert!(
			messages
				.iter()
				.any(|m| m.contains("UnsupportedObjectFeature") && m.contains(expected_substring)),
			"no UnsupportedObjectFeature diagnostic matched '{expected_substring}'\n\
			 actual diagnostics:\n{}",
			messages.join("\n---\n")
		);
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

	#[test]
	fn object_negative_syntax_shaped_lowering_gap_diagnostics() {
		// `array [int] of new A` used to panic in THIR with
		// `Handle array of new with non-bounded dimensions`. Array-of-new roots are
		// now disallowed altogether (a clean diagnostic via
		// `validate_object_lowering`).
		check_unsupported_object_diagnostic(
			r#"
    class A ();
    array [int] of new A: xs;
    solve satisfy;
    "#,
			"not supported",
		);
	}

	/// A `set of new A` class attribute with no cardinality is
	/// unrepresentable once the class is varified by a `var new`
	/// introduction: the potential-object universe would be unbounded, so
	/// the field is not varifiable (the same failure class as a `string` or
	/// `set of float` attribute in a varified class). HIR validation rejects
	/// it with a clean diagnostic instead of letting THIR panic in nested
	/// contribution lowering. Both the directly-introduced class `C` and the
	/// transitively-reached `B` are flagged.
	#[test]
	fn object_negative_var_reached_set_of_new_needs_cardinality() {
		check_unsupported_object_diagnostic(
			r#"
    class A ( var 1..3: x; );
    class B ( set of new A: bas; var 1..5: by; );
    class C ( set of new B: cbs; );
    var new C: c;
    solve satisfy;
    "#,
			"must declare a cardinality",
		);
	}

	/// The cardinality requirement is specific to *var*-reached classes. A
	/// par-reached `set of new A` is instantiated with concrete objects
	/// (whose count supplies the cardinality), so it must NOT be rejected.
	#[test]
	fn object_par_reached_set_of_new_without_cardinality_is_allowed() {
		let db = db_for_with_stdlib(
			r#"
    class A ( int: x; );
    class B ( set of new A: as; int: y; );
    set of new B: bs =
      [(as: [(x: 3), (x: 5)], y: 10), (as: [(x: 23)], y: 4)];
    solve satisfy;
    "#,
		);
		let errors = user_hir_errors(&db);
		if !errors.is_empty() {
			let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
			assert!(
				!messages
					.iter()
					.any(|m| m.contains("must declare a cardinality")),
				"par-reached `set of new` was wrongly rejected:\n{}",
				messages.join("\n---\n")
			);
		}
	}

	/// Recursive `new` OWNERSHIP is rejected: a cycle in the
	/// class graph over `new`-typed attributes means every potential object owns
	/// a fresh child of a class that transitively owns such an object again, so
	/// the potential-object universe would be infinite. Used to crash the
	/// compiler with a stack overflow in `class_analysis` occurrence expansion.
	/// This is the two-line repro from the A3 brief, verbatim.
	#[test]
	fn object_negative_ownership_cycle_direct_self() {
		check_unsupported_object_diagnostic(
			r#"
    class Node ( opt new Node: next; );
    var new Node: root;
    solve satisfy;
    "#,
			"must not form an ownership cycle",
		);
	}

	/// A cyclic class is rejected at its declaration even when nothing
	/// instantiates it: the class is unrealizable per se, and rejecting only
	/// reached cycles would report the error far from its cause.
	#[test]
	fn object_negative_ownership_cycle_uninstantiated() {
		check_unsupported_object_diagnostic(
			r#"
    class Node ( opt new Node: next; );
    solve satisfy;
    "#,
			"must not form an ownership cycle",
		);
	}

	/// Mutual ownership cycle A -> B -> A.
	#[test]
	fn object_negative_ownership_cycle_mutual() {
		check_unsupported_object_diagnostic(
			r#"
    class A ( new B: b; );
    class B ( new A: a; );
    var new A: root;
    solve satisfy;
    "#,
			"must not form an ownership cycle",
		);
	}

	/// Ownership cycle closed through inheritance: `B extends A` inherits A's
	/// `new B` attribute, so every `B` object owns a fresh `B`. The cycle test
	/// must consider inherited `new` attributes — occurrence expansion descends
	/// through them too.
	#[test]
	fn object_negative_ownership_cycle_through_inheritance() {
		check_unsupported_object_diagnostic(
			r#"
    class A ( new B: b; );
    class B extends A ();
    solve satisfy;
    "#,
			"must not form an ownership cycle",
		);
	}

	/// Longer ownership cycle A -> B -> C -> A.
	#[test]
	fn object_negative_ownership_cycle_longer_chain() {
		check_unsupported_object_diagnostic(
			r#"
    class A ( new B: b; );
    class B ( new C: c; );
    class C ( new A: a; );
    solve satisfy;
    "#,
			"must not form an ownership cycle",
		);
	}

	/// Set-shaped back-edge: the cycle detection sees `new` through set (and
	/// array) wrappers, same as occurrence expansion does.
	#[test]
	fn object_negative_ownership_cycle_set_shaped() {
		check_unsupported_object_diagnostic(
			r#"
    class A ( var set(0..2) of new B: bs; );
    class B ( new A: a; );
    var new A: root;
    solve satisfy;
    "#,
			"must not form an ownership cycle",
		);
	}

	/// REFERENCE cycles — class-typed attributes without `new` — are the
	/// supported idiom for recursive/cyclic structure (a bounded pool plus
	/// references) and must NOT be caught by the ownership-cycle rejection.
	/// The wagon's Seat<->Handrail reference cycle passes HIR validation clean.
	#[test]
	fn object_ownership_cycle_guard_ignores_reference_cycles() {
		let db = db_for_with_stdlib(
			r#"
% Reference cycle between two var-reached classes: Seat references Handrail
% (`opt Handrail:`) while Handrail's reference-set field contains Seats
% (`set(1..1) of Seat:`). Classes are predeclared in topological item order,
% but a reference cycle has no valid order, so whichever class predeclares
% first used to keep an unsubstituted `Class<X>`-typed storage field —
% frozen into `<C>_objects` and panicking `lowered_ty_matches` when the
% top-level forall read `deopt(w.handrail).attached`.
% `repair_predeclared_class_objects_domains` rebuilds the storage domains
% once every class is registered.
class Wagon (
  set(1..1) of new Seat: seats;
  opt new Handrail: handrail;
);
class Handrail (
  set(1..1) of Seat: attached;
);
class Seat (
  opt Handrail: handrail;
  var 1..2: comfort;
);

constraint forall(w in Wagon)(occurs(w.handrail));
constraint forall(w in Wagon where occurs(w.handrail))(
  forall(s in deopt(w.handrail).attached)(s in w.seats /\ s.comfort = 2)
);

var new Wagon: w;
output ["comforts=", show([s.comfort | s in w.seats])];
solve satisfy;
"#,
		);
		let errors = user_hir_errors(&db);
		if !errors.is_empty() {
			let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
			panic!(
				"reference-cycle model wrongly rejected at HIR validation:\n{}",
				messages.join("\n---\n")
			);
		}
	}

	/// Mixed par+var reach fence: a par *singular* `new K` root of a class that is
	/// also var-reached and has object-typed attributes is rejected at validation.
	/// The par singular reconstruction stores class-typed fields in the
	/// slice-ordinal representation while the var-reached storage holds
	/// `var set of <Child>_potential` identities, so the two contributions cannot
	/// Cross-introduction: `var opt new C` mixed with other top-level
	/// introductions of `C`'s hierarchy is SUPPORTED. The
	/// reached classes' actual sets are emitted FREE with a subset lower bound
	/// (the definite roots) rather than an `=` union, so the opt occurrence's
	/// membership stays the free decision and a co-occurring definite root is not
	/// clobbered. Pin that HIR validation no longer rejects these shapes (their
	/// solution-equivalence is pinned by the `opt_mixed_*` pairs).
	#[test]
	fn object_opt_root_mixed_with_other_introductions_is_allowed() {
		for source in [
			// Par singular + var opt of the SAME class.
			r#"
    class A ( 0..4: x; );
    new A: a = (x: 3);
    var opt new A: a2;
    solve satisfy;
    "#,
			// Par set root + var opt.
			r#"
    class A ( 0..4: x; );
    set of new A: a1 = [(x: 3)];
    var opt new A: a2;
    solve satisfy;
    "#,
			// Par SUPERCLASS root + var opt SUBCLASS.
			r#"
    class A ( 0..4: x; );
    class ASub extends A ( 0..1: y; );
    new A: a = (x: 3);
    var opt new ASub: s;
    solve satisfy;
    "#,
			// Var opt SUPERCLASS + definite SUBCLASS.
			r#"
    class A ( 0..4: x; );
    class ASub extends A ( 0..1: y; );
    new ASub: s = (x: 3, y: 0);
    var opt new A: a2;
    solve satisfy;
    "#,
			// Opt root of a class introduced only via a NESTED field elsewhere.
			r#"
    class A ( 0..1: x; );
    class P ( new A: kid; );
    new P: p = (kid: (x: 0));
    var opt new A: a2;
    solve satisfy;
    "#,
		] {
			let db = db_for_with_stdlib(source);
			let errors = user_hir_errors(&db);
			if !errors.is_empty() {
				let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
				assert!(
					!messages
						.iter()
						.any(|m| m.contains("UnsupportedObjectFeature")
							|| m.contains("not supported")),
					"supported mixed-opt shape was wrongly fenced:\n{}",
					messages.join("\n---\n")
				);
			}
		}
	}

	/// A fully-PAR object-carrying class introduced two or more `new`-hops
	/// below a par root now lowers — `reconstructed_deep_nested_contribution_expr`
	/// flattens the field owner's par input records and mints
	/// `<GrandChild>_potential` identities from a flat 1-D prefix sum over the flat
	/// position, depth-agnostically. Pin that HIR validation no longer fences these
	/// shapes (they solve correctly — see the `deep_nested_object_*` equivalence
	/// pairs). Covers set/singular/mixed edges, singular and opt grand-fields,
	/// depth-3, and a var LEAF attribute (par existence, var storage).
	#[test]
	fn object_par_root_deep_nested_object_is_supported() {
		for source in [
			// Depth-2, all set edges: A.bs -> B.cs -> C (owns `ds: set of new D`).
			r#"
    class D ( 2..3: v; );
    class C ( set(1..2) of new D: ds; );
    class B ( set(1..2) of new C: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(ds: [(v: 2)])])]);
    solve satisfy;
    "#,
			// Depth-2, both edges singular, set-of-new root.
			r#"
    class D ( 2..3: v; );
    class C ( set(1..2) of new D: ds; );
    class B ( new C: c; );
    class A ( new B: b; );
    set of new A: as = [(b: (c: (ds: [(v: 2)])))];
    solve satisfy;
    "#,
			// Depth-2, mixed set-then-singular edge.
			r#"
    class D ( 2..3: v; );
    class C ( set(1..2) of new D: ds; );
    class B ( new C: c; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(c: (ds: [(v: 2)]))]);
    solve satisfy;
    "#,
			// Depth-2 with a SINGULAR grand-field (`d: new D`) -> `D_occ(ci)`.
			r#"
    class D ( 2..3: v; );
    class C ( new D: d; );
    class B ( set(1..2) of new C: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(d: (v: 2))])]);
    solve satisfy;
    "#,
			// Depth-2 with an OPT grand-field (`d: opt new D`).
			r#"
    class D ( 2..3: v; );
    class C ( opt new D: d; );
    class B ( set(1..2) of new C: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(d: (v: 2))])]);
    solve satisfy;
    "#,
			// Depth-3: C (depth 2) owns `ds: set of new D`, D (depth 3) owns
			// `es: set of new E`. Both deep classes mint.
			r#"
    class E ( 2..3: w; );
    class D ( set(1..2) of new E: es; );
    class C ( set(1..2) of new D: ds; );
    class B ( set(1..2) of new C: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(ds: [(es: [(w: 2)])])])]);
    solve satisfy;
    "#,
			// Depth-2 with a VAR leaf attribute on the deep grand-child (par
			// existence, var storage) — must NOT be fenced (var reach ≠ var actual
			// set).
			r#"
    class D ( var 2..3: v; );
    class C ( set(1..2) of new D: ds; );
    class B ( set(1..2) of new C: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(ds: [()])])]);
    solve satisfy;
    "#,
		] {
			let db = db_for_with_stdlib(source);
			let errors = user_hir_errors(&db);
			if !errors.is_empty() {
				let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
				assert!(
					!messages
						.iter()
						.any(|m| m.contains("UnsupportedObjectFeature")
							|| m.contains("not supported")),
					"supported deep par nested object shape was wrongly fenced:\n{}",
					messages.join("\n---\n")
				);
			}
		}
	}

	/// A class introduced whose superclass owns an object (class-typed) field
	/// now lowers — the superclass PROJECTION contribution reads the subclass's
	/// minted `<GrandChild>_potential` identities (`[(ds: proj.ds) | proj in
	/// <Sub>_objects]`) instead of the inline input record. Previously this
	/// mis-lowered at DEPTH 1 (unfenced — invalid emission) and was blanket-fenced
	/// at depth ≥ 2; the `needs_storage_projection` predicate now forces projection
	/// for object fields (`selected_nested_contribution_expr`). Covers depth-1 and
	/// depth-2 nesting and a multi-level inheritance chain. (These solve correctly
	/// — see the `subclass_inherited_object_field` equivalence pair.)
	#[test]
	fn object_subclass_inherited_object_field_is_supported() {
		for source in [
			// Depth-1: A.cs (set new C2), C2 extends C, C owns `ds: set of new D`.
			r#"
    class D ( 2..3: v; );
    class C ( set(1..2) of new D: ds; );
    class C2 extends C ( 0..1: extra; );
    class A ( set(1..2) of new C2: cs; );
    new A: a = (cs: [(ds: [(v: 2)], extra: 0)]);
    solve satisfy;
    "#,
			// Depth-2: A.bs -> B.cs (set new C2), C2 extends C (owns `ds`).
			r#"
    class D ( 2..3: v; );
    class C ( set(1..2) of new D: ds; );
    class C2 extends C ( 0..1: extra; );
    class B ( set(1..2) of new C2: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(ds: [(v: 2)], extra: 0)])]);
    solve satisfy;
    "#,
			// Multi-level inheritance chain C3 -> C2 -> C (C owns `ds`), depth-2.
			r#"
    class D ( 2..3: v; );
    class C ( set(1..2) of new D: ds; );
    class C2 extends C ( 0..1: e2; );
    class C3 extends C2 ( 0..1: e3; );
    class B ( set(1..2) of new C3: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(ds: [(v: 2)], e2: 0, e3: 1)])]);
    solve satisfy;
    "#,
		] {
			let db = db_for_with_stdlib(source);
			let errors = user_hir_errors(&db);
			if !errors.is_empty() {
				let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
				assert!(
					!messages
						.iter()
						.any(|m| m.contains("UnsupportedObjectFeature")
							|| m.contains("not supported")),
					"supported subclass inherited-object-field shape was wrongly fenced:\n{}",
					messages.join("\n---\n")
				);
			}
		}
	}

	/// `array [d] of new C` roots are DISALLOWED by design: an array of new objects
	/// assigns each object two independent identities (its own `<C>_potential`
	/// identity and its array index) and adds no expressivity over `set of new C`.
	/// Every array-of-new root shape — scalar member, object-carrying member, opt
	/// member, any dim (`1..u`, literal set, `int`, enum, non-1-based) — is rejected
	/// at HIR validation with a clear diagnostic directing to `set of new C`. (Use a
	/// reference array `array [E] of C` or a key attribute for keyed access.)
	#[test]
	fn object_negative_array_of_new_root_disallowed() {
		for source in [
			// Scalar member, 1-based range (formerly the one supported shape).
			r#"
    class A ( 0..9: x; );
    array [1..2] of new A: as;
    solve satisfy;
    "#,
			// Scalar member, 1-based literal set.
			r#"
    class A ( 0..9: x; );
    array [{1, 2}] of new A: as = [ (x: 1), (x: 2) ];
    solve satisfy;
    "#,
			// Object-carrying member (`set of new B`).
			r#"
    class B ( 0..9: v; );
    class A ( set of new B: kids; );
    array [1..2] of new A: as = [ (kids: [(v: 7)]), (kids: []) ];
    solve satisfy;
    "#,
			// Par `opt new B` member.
			r#"
    class B ( 0..9: v; );
    class A ( opt new B: kid; );
    array [1..2] of new A: as = [ (kid: (v: 7)), (kid: <>) ];
    solve satisfy;
    "#,
			// Non-1-based range.
			r#"
    class A ( 0..9: x; );
    array [3..4] of new A: as;
    solve satisfy;
    "#,
		] {
			check_unsupported_object_diagnostic(source, "not supported");
		}
	}

	/// Mixed par+var reach of one class is otherwise IMPLEMENTED: the shapes an
	/// earlier fence rejected — par singular roots of var-reached classes with
	/// object-typed fields — now lower and solve, as do par set roots plus
	/// `var new`, subclass mixes in both directions, and nested par reach
	/// through set-shaped fields. Pin that HIR validation no longer rejects
	/// them.
	#[test]
	fn object_mixed_par_var_reach_supported_shapes_are_allowed() {
		for source in [
			// Par singular + var singular, object-typed field AND
			// computed attribute.
			r#"
    class B ( 2..3: x; );
    class A ( set(1..1) of new B: children; int: n = card(children); );
    new A: a = (children: [(x: 2)]);
    var new A: a2;
    solve satisfy;
    "#,
			// The same shape without the computed attribute.
			r#"
    class B ( 2..3: x; );
    class A ( set(1..1) of new B: children; );
    new A: a = (children: [(x: 2)]);
    var new A: a2;
    solve satisfy;
    "#,
			// Par singular + var singular, scalar class with computed attribute.
			r#"
    class A ( 0..4: x; int: y = x + 1; );
    new A: a = (x: 3);
    var new A: a2;
    solve satisfy;
    "#,
			// Par set root + var set root, class WITH an object-typed field.
			r#"
    class B ( 2..3: x; );
    class A ( set(1..1) of new B: children; );
    set of new A: a1 = [(children: [(x: 2)])];
    var set(1..2) of new A: a2;
    solve satisfy;
    "#,
			// Par singular root + var SET root (the class-set union mix).
			r#"
    class A ( 0..4: x; );
    new A: a = (x: 3);
    var set(1..2) of new A: as;
    solve satisfy;
    "#,
			// Nested par reach through a SET-shaped field (identity-minting
			// path) while the class is var-reached from elsewhere.
			r#"
    class B ( 2..3: x; );
    class A ( set(1..1) of new B: children; );
    class P ( set(1..1) of new A: kids; );
    new P: p = (kids: [(children: [(x: 2)])]);
    var new A: a2;
    solve satisfy;
    "#,
			// Subclass directions: par superclass root + var subclass root, and
			// par subclass root + var superclass root.
			r#"
    class B ( 2..3: x; );
    class A ( set(1..1) of new B: children; );
    class ASub extends A ( 0..1: y; );
    new A: a = (children: [(x: 2)]);
    var new ASub: a2;
    solve satisfy;
    "#,
			r#"
    class B ( 2..3: x; );
    class A ( set(1..1) of new B: children; );
    class ASub extends A ( 0..1: y; );
    new ASub: s = (children: [(x: 2)], y: 1);
    var new A: a2;
    solve satisfy;
    "#,
			// Depth-1: par root owning a SINGULAR `new A: kid`
			// field where A is object-carrying (`children`) and var-reached — the
			// exact shape the old F2 fence rejected. Now mints child identities.
			r#"
    class B ( 2..3: x; );
    class A ( set(1..1) of new B: children; );
    class P ( new A: kid; );
    new P: p = (kid: (children: [(x: 2)]));
    var new A: a2;
    solve satisfy;
    "#,
			// Doubly-singular grand-child: par root -> singular `new A: kid`
			// -> singular `new B: gc`. Both `new`-hops mint identities.
			r#"
    class B ( 2..3: x; );
    class A ( new B: gc; );
    class P ( new A: kid; );
    new P: p = (kid: (gc: (x: 2)));
    var new A: a2;
    solve satisfy;
    "#,
			// Par-only: the same singular object nesting with no var root —
			// the pre-existing par-only silent-invalid the depth-1 fix closes.
			r#"
    class B ( 2..3: x; );
    class A ( set(1..1) of new B: children; );
    class P ( new A: kid; );
    new P: p = (kid: (children: [(x: 2)]));
    solve satisfy;
    "#,
		] {
			let db = db_for_with_stdlib(source);
			let errors = user_hir_errors(&db);
			if !errors.is_empty() {
				let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
				assert!(
					!messages
						.iter()
						.any(|m| m.contains("UnsupportedObjectFeature")
							|| m.contains("not supported")),
					"supported mixed par+var shape was wrongly fenced:\n{}",
					messages.join("\n---\n")
				);
			}
		}
	}

	/// Ragged array attributes stay rejected: a var-receiver read of a
	/// per-object-length array cannot be element-wise varified (its index set
	/// would be decision-dependent), and a var-reached class cannot declare one
	/// at all (the dimension references a sibling attribute, which var-reach
	/// forces to var). Uniform single-dimension array attributes are accepted —
	/// see the array_field_var_index_* fixtures.
	#[test]
	fn object_array_field_ragged_rejections() {
		let db = db_for(
			r#"
% Ragged array attribute (per-object length `1..len`) read through a var
% receiver: the HIR typer cannot element-wise-varify it (the result's index
% set would be decision-dependent), so the read is rejected with
% IllegalType. The par class DECLARATION itself stays legal — only the
% var-receiver read is refused.
class A ( 1..3: len; array[1..len] of var 0..2: xs; );
set of new A: as = [(len: 1), (len: 2)];
var A: chosen;
constraint sum(chosen.xs) = 1;
solve satisfy;
"#,
		);
		let errors = user_hir_errors(&db);
		assert!(
			errors
				.iter()
				.any(|e| e.to_string().contains("Illegal type")),
			"var-receiver read of a ragged array attribute must be an IllegalType error, got:\n{}",
			errors
				.iter()
				.map(|e| e.to_string())
				.collect::<Vec<_>>()
				.join("\n---\n")
		);
		check_unsupported_object_diagnostic(
			r#"
% Ragged array attribute on a var-reached class: rejected at object
% validation (the dimension references a sibling attribute, which the
% var-reach cascade forces to var — and a dimension must be par).
class A ( 1..3: len; array[1..len] of var 0..2: xs; );
var set(0..1) of new A: as_;
solve satisfy;
"#,
			"references a sibling attribute",
		);
	}

	/// Sibling-dependent domains are not varifiable: `var 1..z: s` is only valid
	/// while `z` is par, and the var-reach cascade forces every attribute of a
	/// var-reached class to var — a MiniZinc domain must be par. (Unfenced, the
	/// per-object mint emitted a var-bounded let domain the target MiniZinc
	/// rejects: "type-inst must be par set".)
	#[test]
	fn object_negative_var_reached_attribute_dependent_domain() {
		check_unsupported_object_diagnostic(
			r#"
    class A ( 0..5: x; int: z = x; var 1..z: s; );
    var set(0..1) of new A: as;
    solve satisfy;
    "#,
			"references a sibling attribute",
		);
	}

	/// The same dependent-domain shape stays legal in a par-reached class: the
	/// sibling stays par, so the per-object `let { var 1..z: .. }` mint is valid
	/// (Step-1 dependent-domain support).
	#[test]
	fn object_par_reached_attribute_dependent_domain_is_allowed() {
		let db = db_for_with_stdlib(
			r#"
    class A ( 0..5: x; int: z = x; var 1..z: s; );
    new A: a = (x: 2);
    solve satisfy;
    "#,
		);
		let errors = user_hir_errors(&db);
		if !errors.is_empty() {
			let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
			assert!(
				!messages
					.iter()
					.any(|m| m.contains("references a sibling attribute")),
				"par-reached dependent domain was wrongly rejected:\n{}",
				messages.join("\n---\n")
			);
		}
	}

	/// A non-computed attribute of a var-reached class becomes a FREE `_storage`
	/// decision, so `make_var` succeeding is not enough — the field must be
	/// declarable as a free var decision. An unbounded `var set of int` passes
	/// `make_var` but has no finite element domain: the emitted model compiled
	/// and then aborted in gecode. Computed set attributes stay
	/// exempt (they are aliases, covered by
	/// `object_var_reached_computed_unvarifiable_attribute_is_allowed`).
	#[test]
	fn object_negative_var_reached_unbounded_free_var_set() {
		check_unsupported_object_diagnostic(
			r#"
    class A ( var 0..5: x; var set of int: z; );
    var new A: a;
    solve satisfy;
    "#,
			"a free var set needs a finite element domain",
		);
	}

	/// In a var-reached class the var-reach cascade forces every attribute to
	/// var, so an attribute whose type cannot be made var (`string`, `set of
	/// float`, ...) is an error — the same failure class as the uncardinalitied
	/// `set of new` attribute. HIR validation rejects it instead of silently
	/// keeping the field par at `signature.rs` (`make_var(ty).unwrap_or(ty)`).
	#[test]
	fn object_negative_var_reached_unvarifiable_attribute() {
		check_unsupported_object_diagnostic(
			r#"
    class A ( string: s; );
    var new A: a;
    solve satisfy;
    "#,
			"cannot be made var",
		);
		check_unsupported_object_diagnostic(
			r#"
    class A ( set of float: s; );
    var new A: a;
    solve satisfy;
    "#,
			"cannot be made var",
		);
	}

	/// A *computed* attribute is exempt from the varifiability rule: it is defined
	/// as a reconstruction alias, not a free storage decision, so an unbounded
	/// `var set of int` (which cannot be a free var decision) is fine when computed.
	#[test]
	fn object_var_reached_computed_unvarifiable_attribute_is_allowed() {
		let db = db_for_with_stdlib(
			r#"
    class A ( 0..4: x; var set of int: z = {x, 2 * x}; );
    var new A: a;
    solve satisfy;
    "#,
		);
		let errors = user_hir_errors(&db);
		if !errors.is_empty() {
			let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
			assert!(
				!messages.iter().any(|m| m.contains("cannot be made var")),
				"var-reached *computed* set attribute was wrongly rejected:\n{}",
				messages.join("\n---\n")
			);
		}
	}

	/// Array-typed *computed* attributes aren't yet supported on var-reached
	/// classes: reading `obj.y` reconstructs each object record by projecting field
	/// columns, which can't reshape an array-of-arrays column. Rejected with a
	/// clear message rather than panicking in the THIR lowering.
	#[test]
	fn object_negative_var_reached_computed_array_attribute() {
		check_unsupported_object_diagnostic(
			r#"
    class A ( var 1..3: x; array[int] of var int: y = [x, x, x]; );
    var new A: a;
    solve satisfy;
    "#,
			"computed attribute `y` of array type",
		);
	}

	/// A par-reached class may carry a non-varifiable attribute (e.g. a
	/// `string`): the varifiability rule applies only to var-reached classes.
	#[test]
	fn object_par_reached_unvarifiable_attribute_is_allowed() {
		let db = db_for_with_stdlib(
			r#"
    class A ( string: s; );
    new A: a;
    solve satisfy;
    "#,
		);
		let errors = user_hir_errors(&db);
		if !errors.is_empty() {
			let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
			assert!(
				!messages.iter().any(|m| m.contains("cannot be made var")),
				"par-reached unvarifiable attribute was wrongly rejected:\n{}",
				messages.join("\n---\n")
			);
		}
	}

	#[test]
	fn object_negative_array_of_new_with_enum_dimension_diagnosed() {
		// `array [E] of new A` where `E` is a named set/enum identifier — like every
		// array-of-new root, now disallowed altogether.
		check_unsupported_object_diagnostic(
			r#"
    class A ();
    set of int: SomeSet = {1, 2, 3};
    array [SomeSet] of new A: xs;
    solve satisfy;
    "#,
			"not supported",
		);
	}

	#[test]
	fn object_negative_new_in_tuple_class_attribute_diagnosed() {
		// PLAIN tuple attributes are supported (A6); a tuple CONTAINING a
		// `new` introduction is not — occurrence expansion and contribution
		// recording do not look inside structured types, so letting it
		// through would silently skip introduction bookkeeping.
		check_unsupported_object_diagnostic(
			r#"
    class B (int: y);
    class A (
      tuple(new B, int): t;
    );
    A: a;
    solve satisfy;
    "#,
			"tuple types containing class references or `new` introductions are not supported",
		);
	}

	#[test]
	fn object_negative_class_ref_in_record_class_attribute_diagnosed() {
		// PLAIN record attributes are supported (A6); a record CONTAINING a
		// class reference is not — the engine's identity-or-read rule and
		// input exclusion key on top-level class types only.
		check_unsupported_object_diagnostic(
			r#"
    class B (int: y);
    class A (
      record(B: ref, int: n): r;
    );
    A: a;
    solve satisfy;
    "#,
			"record types containing class references or `new` introductions are not supported",
		);
	}

	#[test]
	fn object_negative_class_attribute_array_of_new_diagnosed() {
		// `array [d] of new C` as a class attribute is not currently
		// supported. The fixture shape `set of new C` works.
		check_unsupported_object_diagnostic(
			r#"
    class B (int: y);
    class A (
      array [1..3] of new B: bs;
    );
    set of new A: as;
    solve satisfy;
    "#,
			"`array of new C` as a class attribute is not supported yet",
		);
	}

	#[test]
	fn object_negative_class_attribute_array_of_class_reference_diagnosed() {
		// Only a SINGLE-dimension `array [d] of B` reference attribute has an
		// object-storage lowering. Every other array-of-class shape erases its
		// per-object dimensions in the shared storage record and is rejected:
		//
		// - a multi-dimension array (`array [_,_] of B`) — on a PAR owner (a var
		//   owner would already trip the varifiability check with a different
		//   message);
		check_unsupported_object_diagnostic(
			r#"
    class B (var 0..1: v);
    class A (
      array [1..2, 1..2] of B: pool;
    );
    new A: a;
    solve satisfy;
    "#,
			"single-dimension array of class references",
		);
		// - an array whose element wraps a class in a set (`array [d] of set of B`).
		check_unsupported_object_diagnostic(
			r#"
    class B (var 0..1: v);
    class A (
      array [1..2] of set of B: pool;
    );
    var new A: a;
    solve satisfy;
    "#,
			"single-dimension array of class references",
		);
	}

	#[test]
	fn object_class_attribute_array_of_class_reference_supported() {
		// A single-dimension `array [d] of B` REFERENCE attribute (B a class,
		// no `new`) is supported — dims-preserving storage plus a column-projected
		// var-identity read-back. The HIR phase must NOT fence it, on a var-reached
		// owner (free identity decisions per slot)...
		let var = db_for_with_stdlib(
			r#"
    class B (var 0..1: v);
    class A (
      array [1..2] of B: pool;
      var 1..2: sel;
      var B: ref = pool[sel];
    );
    var new A: a;
    solve satisfy;
    "#,
		);
		let errors = user_hir_errors(&var);
		if !errors.is_empty() {
			let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
			assert!(
				!messages
					.iter()
					.any(|m| m.contains("array of class references")),
				"single-dimension `array [d] of B` was wrongly fenced on a var owner:\n{}",
				messages.join("\n---\n")
			);
		}
		// ...nor on a par owner whose pool is supplied as data.
		let par = db_for_with_stdlib(
			r#"
    class B (var 0..1: v);
    class A (
      array [1..2] of B: pool;
    );
    new B: b1;
    new B: b2;
    new A: a = (pool: [b1, b2]);
    solve satisfy;
    "#,
		);
		let errors = user_hir_errors(&par);
		if !errors.is_empty() {
			let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
			assert!(
				!messages
					.iter()
					.any(|m| m.contains("array of class references")),
				"single-dimension `array [d] of B` was wrongly fenced on a par owner:\n{}",
				messages.join("\n---\n")
			);
		}
	}

	#[test]
	fn object_par_opt_new_field_supported() {
		// An `opt new C` field is supported for both par and var owners
		// (the end-to-end present/absent read-back is exercised by the
		// `par_opt_new_field` / `par_opt_new_field_nested` equivalence pairs). A
		// par (data-supplied) optional child lowers like `set(0..1) of new C` via
		// a 0/1-length input list, so the old `opt record` fence is gone. Assert
		// the HIR phase surfaces no `opt new C` UnsupportedObjectFeature for a par
		// owner...
		let par = db_for_with_stdlib(
			r#"
    class B (int: v);
    class A (
      opt new B: kid;
    );
    new A: a = (kid: (v: 3));
    solve satisfy;
    "#,
		);
		let errors = user_hir_errors(&par);
		if !errors.is_empty() {
			let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
			assert!(
				!messages.iter().any(|m| m.contains("opt new C")),
				"par `opt new C` field was wrongly fenced:\n{}",
				messages.join("\n---\n")
			);
		}
		// ...nor for a var-reached owner, whose `opt new C` field is a free
		// decision (a different lowering path, kept `OnePerParent`).
		let var = db_for_with_stdlib(
			r#"
    class B (var 0..1: v);
    class A (
      var 0..1: w;
      opt new B: kid;
      constraint occurs(kid) <-> w = 1;
    );
    var set(1..1) of new A: as;
    "#,
		);
		let errors = user_hir_errors(&var);
		if !errors.is_empty() {
			let messages: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
			assert!(
				!messages.iter().any(|m| m.contains("opt new C")),
				"var-reached owner's `opt new C` field was wrongly fenced:\n{}",
				messages.join("\n---\n")
			);
		}
	}
}
