//! THIR representation of expressions

use std::{
	fmt::Debug,
	ops::{Deref, Index},
};

use rustc_hash::FxHashMap;

use super::{
	AnnotationId, ConstraintId, DeclarationId, Domain, EnumerationId, FunctionId, Identifier,
	Model, OptType, VarType,
};
pub use crate::hir::{BooleanLiteral, FloatLiteral, IntegerLiteral, StringLiteral};
use crate::{
	arena::{Arena, ArenaIndex, ArenaMap},
	thir::{db::Thir, source::Origin},
	ty::{FunctionType, Ty, TyData},
	utils::DebugPrint,
};

/// Storage for expressions in an item.
///
/// Provides constructors for expressions.
/// Accessed via `Item<T>::expressions()` or can be created directly with
/// `ExpressionAllocator::default()` if required (e.g. for items which require
/// expressions for creation).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExpressionAllocator {
	expressions: Arena<Expression>,
	annotations: FxHashMap<ExpressionId, Vec<ExpressionId>>,
	origins: ArenaMap<Expression, Origin>,
}

impl Index<ExpressionId> for ExpressionAllocator {
	type Output = Expression;
	fn index(&self, index: ExpressionId) -> &Self::Output {
		&self.expressions[index]
	}
}

impl ExpressionAllocator {
	/// Directly create an expression of the given type
	pub fn new_unchecked(
		&mut self,
		origin: impl Into<Origin>,
		ty: Ty,
		expression: ExpressionData,
	) -> ExpressionId {
		let idx = self.expressions.insert(Expression {
			ty,
			data: expression,
		});
		self.origins.insert(idx, origin.into());
		idx
	}

	/// Get the annotations for a given expression
	pub fn expression_annotations(
		&self,
		expression: ExpressionId,
	) -> impl '_ + Iterator<Item = ExpressionId> {
		self.annotations
			.get(&expression)
			.into_iter()
			.flat_map(|annotations| annotations.iter().copied())
	}

	/// Add an annotation to an expression
	pub fn annotate_expression(&mut self, expression: ExpressionId, annotation: ExpressionId) {
		self.annotations
			.entry(expression)
			.or_default()
			.push(annotation)
	}

	/// Remove an annotation from an expression
	pub fn unannotate_expression(&mut self, expression: ExpressionId, annotation: ExpressionId) {
		if let Some(annotations) = self.annotations.get_mut(&expression) {
			if let Some(idx) = annotations.iter().position(|ann| *ann == annotation) {
				annotations.swap_remove(idx);
			}
		}
	}

	/// Get the origin for an expression
	pub fn origin(&self, expression: ExpressionId) -> &Origin {
		&self.origins[expression]
	}
}

/// Helper for building expressions
pub struct ExpressionBuilder<'a> {
	db: &'a dyn Thir,
	model: &'a Model,
}

impl<'a> ExpressionBuilder<'a> {
	/// Create a new expression builder
	pub fn new(db: &'a dyn Thir, model: &'a Model) -> Self {
		Self { db, model }
	}

	/// Create an absent literal `<>`
	pub fn absent(&self) -> ExpressionBuilderWithData {
		ExpressionBuilderWithData::new(self.db.type_registry().opt_bottom, ExpressionData::Absent)
	}

	/// Create a boolean literal `true` or `false`
	pub fn boolean(&self, value: BooleanLiteral) -> ExpressionBuilderWithData {
		ExpressionBuilderWithData::new(
			self.db.type_registry().par_bool,
			ExpressionData::BooleanLiteral(value),
		)
	}

	/// Create a integer literal
	pub fn integer(&self, value: IntegerLiteral) -> ExpressionBuilderWithData {
		ExpressionBuilderWithData::new(
			self.db.type_registry().par_int,
			ExpressionData::IntegerLiteral(value),
		)
	}

	/// Create a float literal
	pub fn float(&self, value: FloatLiteral) -> ExpressionBuilderWithData {
		ExpressionBuilderWithData::new(
			self.db.type_registry().par_float,
			ExpressionData::FloatLiteral(value),
		)
	}

	/// Create a string literal
	pub fn string(&self, value: StringLiteral) -> ExpressionBuilderWithData {
		ExpressionBuilderWithData::new(
			self.db.type_registry().string,
			ExpressionData::StringLiteral(value),
		)
	}

	/// Create an `infinity` literal
	pub fn infinity(&self) -> ExpressionBuilderWithData {
		ExpressionBuilderWithData::new(self.db.type_registry().par_int, ExpressionData::Infinity)
	}

	/// Create an identifier which points to an annotation item (the constructor function or the atom)
	pub fn annotation_constructor(&self, item: AnnotationId) -> ExpressionBuilderWithData {
		let ann = self.db.type_registry().ann;
		let ty = if let Some(params) = &self.model[item].parameters {
			Ty::function(
				self.db.upcast(),
				FunctionType {
					params: params.iter().map(|d| self.model[*d].ty()).collect(),
					return_type: ann,
				},
			)
		} else {
			ann
		};

		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::Identifier(ResolvedIdentifier::Annotation(item)),
		)
	}

	/// Create an identifier which points to an annotation destructuring function
	pub fn annotation_destructure(&self, item: AnnotationId) -> ExpressionBuilderWithData {
		let ann = self.db.type_registry().ann;
		let params = self.model[item]
			.parameters
			.as_ref()
			.expect("Cannot destructure annotation atom");
		let ty = if params.len() == 1 {
			Ty::function(
				self.db.upcast(),
				FunctionType {
					params: Box::new([ann]),
					return_type: self.model[params[0]].ty(),
				},
			)
		} else {
			Ty::function(
				self.db.upcast(),
				FunctionType {
					params: Box::new([ann]),
					return_type: Ty::tuple(
						self.db.upcast(),
						params.iter().map(|d| self.model[*d].ty()),
					),
				},
			)
		};
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::Identifier(ResolvedIdentifier::Annotation(item)),
		)
	}

	/// Create an identifier which points to a declaration
	pub fn identifier(&self, item: DeclarationId) -> ExpressionBuilderWithData {
		ExpressionBuilderWithData::new(
			self.model[item].ty(),
			ExpressionData::Identifier(ResolvedIdentifier::Declaration(item)),
		)
	}

	/// Create an identifier which points to the top-level variable or atom with the given name
	pub fn lookup_identifier(&self, name: Identifier) -> ExpressionBuilderWithData {
		let ident = self
			.model
			.lookup_identifier(self.db, name)
			.expect("Identifier not found");
		match &ident {
			ResolvedIdentifier::Annotation(a) => self.annotation_constructor(*a),
			ResolvedIdentifier::Declaration(d) => self.identifier(*d),
			ResolvedIdentifier::Enumeration(e) => self.enumeration_set(*e),
			ResolvedIdentifier::EnumerationMember(m) => self.enum_atom(*m),
			_ => unreachable!(),
		}
	}

	/// Create an identifier which points to the defining set of an enum
	pub fn enumeration_set(&self, item: EnumerationId) -> ExpressionBuilderWithData {
		let ty = Ty::par_set(
			self.db.upcast(),
			Ty::par_enum(self.db.upcast(), self.model[item].enum_type()),
		)
		.unwrap();
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::Identifier(ResolvedIdentifier::Enumeration(item)),
		)
	}

	/// Create an identifier which points to an enum atom
	pub fn enum_atom(&self, member: EnumMemberId) -> ExpressionBuilderWithData {
		let ctor = &self.model[member];
		assert!(ctor.parameters.is_none(), "Not an enum atom");
		ExpressionBuilderWithData::new(
			Ty::par_enum(
				self.db.upcast(),
				self.model[member.enumeration_id()].enum_type(),
			),
			ExpressionData::Identifier(ResolvedIdentifier::EnumerationMember(member)),
		)
	}

	/// Create an identifier which points to an enum constructor function
	pub fn enum_constructor(
		&self,
		member: EnumMemberId,
		kind: EnumConstructorKind,
	) -> ExpressionBuilderWithData {
		let enum_ty = Ty::par_enum(
			self.db.upcast(),
			self.model[member.enumeration_id()].enum_type(),
		);
		let ctor = &self.model[member];
		let params = ctor
			.parameters
			.as_ref()
			.expect("Not an enum constructor")
			.iter()
			.map(|d| self.model[*d].ty());
		let f = match kind {
			EnumConstructorKind::Par => FunctionType {
				params: params.collect(),
				return_type: enum_ty,
			},
			EnumConstructorKind::Var => FunctionType {
				params: params
					.map(|p| p.with_inst(self.db.upcast(), VarType::Var).unwrap())
					.collect(),
				return_type: enum_ty.with_inst(self.db.upcast(), VarType::Var).unwrap(),
			},
			EnumConstructorKind::Opt => FunctionType {
				params: params
					.map(|p| p.with_opt(self.db.upcast(), OptType::Opt))
					.collect(),
				return_type: enum_ty.with_opt(self.db.upcast(), OptType::Opt),
			},
			EnumConstructorKind::VarOpt => FunctionType {
				params: params
					.map(|p| {
						p.with_inst(self.db.upcast(), VarType::Var)
							.unwrap()
							.with_opt(self.db.upcast(), OptType::Opt)
					})
					.collect(),
				return_type: enum_ty
					.with_inst(self.db.upcast(), VarType::Var)
					.unwrap()
					.with_opt(self.db.upcast(), OptType::Opt),
			},
			EnumConstructorKind::Set => FunctionType {
				params: params
					.map(|p| Ty::par_set(self.db.upcast(), p).unwrap())
					.collect(),
				return_type: Ty::par_set(self.db.upcast(), enum_ty).unwrap(),
			},
			EnumConstructorKind::VarSet => FunctionType {
				params: params
					.map(|p| {
						Ty::par_set(self.db.upcast(), p)
							.unwrap()
							.with_inst(self.db.upcast(), VarType::Var)
							.unwrap()
					})
					.collect(),
				return_type: Ty::par_set(self.db.upcast(), enum_ty)
					.unwrap()
					.with_inst(self.db.upcast(), VarType::Var)
					.unwrap(),
			},
		};
		ExpressionBuilderWithData::new(
			Ty::function(self.db.upcast(), f),
			ExpressionData::Identifier(ResolvedIdentifier::EnumerationMember(member)),
		)
	}

	/// Create an identifier which resolves to an enum destructuring function
	pub fn enum_destructure(
		&self,
		member: EnumMemberId,
		kind: EnumConstructorKind,
	) -> ExpressionBuilderWithData {
		let enum_ty = Ty::par_enum(
			self.db.upcast(),
			self.model[member.enumeration_id()].enum_type(),
		);
		let ctor = &self.model[member];
		let params = ctor.parameters.as_ref().unwrap();
		let return_type = if params.len() == 1 {
			self.model[params[0]].ty()
		} else {
			Ty::tuple(self.db.upcast(), params.iter().map(|d| self.model[*d].ty()))
		};
		let f = match kind {
			EnumConstructorKind::Par => FunctionType {
				params: Box::new([enum_ty]),
				return_type,
			},
			EnumConstructorKind::Var => FunctionType {
				params: Box::new([enum_ty.with_inst(self.db.upcast(), VarType::Var).unwrap()]),
				return_type: return_type
					.with_inst(self.db.upcast(), VarType::Var)
					.unwrap(),
			},
			_ => unreachable!("Unsupported destructuring kind"),
		};
		ExpressionBuilderWithData::new(
			Ty::function(self.db.upcast(), f),
			ExpressionData::Identifier(ResolvedIdentifier::EnumerationDestructure(member)),
		)
	}

	/// Create an identifier which resolves to a function item instantiated with the given argument types
	pub fn function(&self, item: FunctionId, args: &[Ty]) -> ExpressionBuilderWithData {
		let fe = self.model[item].function_entry(self.model);
		let ft = fe
			.overload
			.instantiate(self.db.upcast(), args)
			.expect("Failed to instantiate function");
		ExpressionBuilderWithData::new(
			Ty::function(self.db.upcast(), ft),
			ExpressionData::Identifier(ResolvedIdentifier::Function(item)),
		)
	}

	/// Create an identifier which resolves to the type-inst variable with the given index for a function
	pub fn type_inst_var(&self, ty_var: TyVarId) -> ExpressionBuilderWithData {
		ExpressionBuilderWithData::new(
			Ty::type_inst_var(self.db.upcast(), self.model[ty_var].clone()),
			ExpressionData::Identifier(ResolvedIdentifier::TyVarRef(ty_var)),
		)
	}

	/// Create an array literal
	pub fn array(
		&self,
		members: impl IntoIterator<Item = ExpressionId>,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let items: Vec<_> = members.into_iter().collect();
		let tys = items.iter().map(|idx| allocator[*idx].ty());
		ExpressionBuilderWithData::new(
			if items.is_empty() {
				self.db.type_registry().array_of_bottom
			} else {
				Ty::array(
					self.db.upcast(),
					self.db.type_registry().par_int,
					Ty::most_specific_supertype(self.db.upcast(), tys)
						.expect("Non uniform array literal"),
				)
				.expect("Invalid array type")
			},
			ExpressionData::ArrayLiteral(items),
		)
	}

	/// Create a set literal
	pub fn set(
		&self,
		members: impl IntoIterator<Item = ExpressionId>,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let items: Vec<_> = members.into_iter().collect();
		if items.is_empty() {
			return ExpressionBuilderWithData::new(
				self.db.type_registry().set_of_bottom,
				ExpressionData::SetLiteral(items),
			);
		}
		let elem_ty = Ty::most_specific_supertype(
			self.db.upcast(),
			items.iter().map(|idx| allocator[*idx].ty()),
		)
		.expect("Non uniform set literal");
		let ty = if let VarType::Var = elem_ty
			.inst(self.db.upcast())
			.expect("No inst for set literal")
		{
			Ty::par_set(
				self.db.upcast(),
				elem_ty.with_inst(self.db.upcast(), VarType::Par).unwrap(),
			)
			.unwrap()
			.with_inst(self.db.upcast(), VarType::Var)
			.expect("Cannot make set var")
		} else {
			Ty::par_set(self.db.upcast(), elem_ty).expect("Invalid set type")
		};
		ExpressionBuilderWithData::new(ty, ExpressionData::SetLiteral(items))
	}

	/// Create a tuple literal
	pub fn tuple(
		&self,
		members: impl IntoIterator<Item = ExpressionId>,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let items: Vec<_> = members.into_iter().collect();
		ExpressionBuilderWithData::new(
			Ty::tuple(self.db.upcast(), items.iter().map(|e| allocator[*e].ty())),
			ExpressionData::TupleLiteral(items),
		)
	}

	/// Create a record literal
	pub fn record(
		&self,
		members: impl IntoIterator<Item = (Identifier, ExpressionId)>,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let items: Vec<_> = members.into_iter().collect();
		ExpressionBuilderWithData::new(
			Ty::record(
				self.db.upcast(),
				items.iter().map(|(i, e)| (*i, allocator[*e].ty())),
			),
			ExpressionData::RecordLiteral(items),
		)
	}

	/// Create an array comprehension
	pub fn array_comprehension(
		&self,
		generators: impl IntoIterator<Item = Generator>,
		template: ExpressionId,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let generators = generators.into_iter().collect::<Vec<_>>();
		let lift_to_opt = generators
			.iter()
			.any(|g| g.var_where(self.db, allocator) || g.var_set(self.db, allocator));
		let ty = Ty::array(
			self.db.upcast(),
			self.db.type_registry().par_int,
			if lift_to_opt {
				allocator[template]
					.ty()
					.with_inst(self.db.upcast(), VarType::Var)
					.unwrap()
					.with_opt(self.db.upcast(), OptType::Opt)
			} else {
				allocator[template].ty()
			},
		)
		.expect("Invalid array type");
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::ArrayComprehension {
				template,
				indices: None,
				generators,
			},
		)
	}

	/// Create an indexed array comprehension
	pub fn indexed_array_comprehension(
		&self,
		generators: impl IntoIterator<Item = Generator>,
		indices: ExpressionId,
		template: ExpressionId,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let generators = generators.into_iter().collect::<Vec<_>>();
		let lift_to_opt = generators
			.iter()
			.any(|g| g.var_where(self.db, allocator) || g.var_set(self.db, allocator));
		let ty = Ty::array(
			self.db.upcast(),
			allocator[indices].ty(),
			if lift_to_opt {
				allocator[template]
					.ty()
					.with_inst(self.db.upcast(), VarType::Var)
					.unwrap()
					.with_opt(self.db.upcast(), OptType::Opt)
			} else {
				allocator[template].ty()
			},
		)
		.expect("Invalid array type");
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::ArrayComprehension {
				template,
				indices: Some(indices),
				generators,
			},
		)
	}

	/// Create a set comprehension
	pub fn set_comprehension(
		&self,
		generators: impl IntoIterator<Item = Generator>,
		template: ExpressionId,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let generators = generators.into_iter().collect::<Vec<_>>();
		let is_var = generators
			.iter()
			.any(|g| g.var_where(self.db, allocator) || g.var_set(self.db, allocator));
		let elem_ty = allocator[template].ty();
		let ty = if let VarType::Var = elem_ty
			.inst(self.db.upcast())
			.expect("Invalid template inst for set comprehension")
		{
			Ty::par_set(
				self.db.upcast(),
				elem_ty.with_inst(self.db.upcast(), VarType::Par).unwrap(),
			)
			.expect("Invalid set type")
			.with_inst(self.db.upcast(), VarType::Var)
			.expect("Cannot make set var")
		} else {
			let st = Ty::par_set(self.db.upcast(), elem_ty).expect("Invalid set type");
			if is_var {
				st.with_inst(self.db.upcast(), VarType::Var)
					.expect("Cannot make set var")
			} else {
				st
			}
		};
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::SetComprehension {
				template,
				generators: generators.into_iter().collect(),
			},
		)
	}

	/// Create an array access (not used for slicing)
	pub fn array_access(
		&self,
		collection: ExpressionId,
		indices: ExpressionId,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let (mut make_opt, mut ty) = match allocator[collection].ty().lookup(self.db.upcast()) {
			TyData::Array { opt, element, .. } => (opt == OptType::Opt, element),
			_ => unreachable!("Not an array"),
		};
		let mut make_var = false;
		let idx_ty = allocator[indices].ty();
		if idx_ty.inst(self.db.upcast()).expect("No inst for index") == VarType::Var {
			make_var = true;
		}
		if idx_ty
			.opt(self.db.upcast())
			.expect("No optionality for index")
			== OptType::Opt
		{
			make_opt = true;
		}
		if let TyData::Tuple(_, fields) = idx_ty.lookup(self.db.upcast()) {
			for ty in fields.iter() {
				if ty.inst(self.db.upcast()).expect("No inst for index") == VarType::Var {
					make_var = true;
				}
				if ty.opt(self.db.upcast()).expect("No optionality for index") == OptType::Opt {
					make_opt = true;
				}
			}
		}
		if make_var {
			ty = ty
				.with_inst(self.db.upcast(), VarType::Var)
				.expect("Cannot make var");
		}
		if make_opt {
			ty = ty.with_opt(self.db.upcast(), OptType::Opt);
		}
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::ArrayAccess {
				collection,
				indices,
			},
		)
	}

	/// Create a tuple access
	pub fn tuple_access(
		&self,
		tuple: ExpressionId,
		field: IntegerLiteral,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let ty = match allocator[tuple].ty().lookup(self.db.upcast()) {
			TyData::Tuple(opt, fields) => {
				let field_ty = fields[field.0 as usize - 1];
				if opt == OptType::Opt {
					field_ty.with_opt(self.db.upcast(), OptType::Opt)
				} else {
					field_ty
				}
			}
			_ => unreachable!(
				"Tried to perform tuple access on {}",
				allocator[tuple].ty().pretty_print(self.db.upcast())
			),
		};

		ExpressionBuilderWithData::new(ty, ExpressionData::TupleAccess { tuple, field })
	}

	/// Create a record access
	pub fn record_access(
		&self,
		record: ExpressionId,
		field: Identifier,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let ty = match allocator[record].ty().lookup(self.db.upcast()) {
			TyData::Record(opt, fields) => {
				let field_ty = fields
					.iter()
					.find_map(|(i, f)| if *i == field.0 { Some(*f) } else { None })
					.expect("Record field doesn't exist");
				if opt == OptType::Opt {
					field_ty.with_opt(self.db.upcast(), OptType::Opt)
				} else {
					field_ty
				}
			}
			_ => unreachable!(
				"Tried to perform record access on {}",
				allocator[record].ty().pretty_print(self.db.upcast())
			),
		};
		ExpressionBuilderWithData::new(ty, ExpressionData::RecordAccess { record, field })
	}

	/// Create an if-then-else
	pub fn if_then_else(
		&self,
		branches: impl IntoIterator<Item = Branch>,
		else_result: ExpressionId,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let types = self.db.type_registry();
		let branches: Vec<_> = branches.into_iter().collect();
		let result_ty = Ty::most_specific_supertype(
			self.db.upcast(),
			branches
				.iter()
				.map(|b| allocator[b.result].ty())
				.chain([allocator[else_result].ty()]),
		)
		.expect("Invalid if-then-else type");
		let make_var = branches
			.iter()
			.any(|b| allocator[b.condition].ty() == types.var_bool);
		let ty = if make_var {
			result_ty
				.with_inst(self.db.upcast(), VarType::Var)
				.expect("Cannot make var")
		} else {
			result_ty
		};
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::IfThenElse {
				branches,
				else_result,
			},
		)
	}

	/// Create a case expression
	pub fn case(
		&self,
		scrutinee: ExpressionId,
		branches: impl IntoIterator<Item = CaseBranch>,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let make_var = allocator[scrutinee]
			.ty()
			.inst(self.db.upcast())
			.expect("No inst for case scrutinee")
			== VarType::Var;
		let branches: Vec<_> = branches.into_iter().collect();
		let result_ty = Ty::most_specific_supertype(
			self.db.upcast(),
			branches.iter().map(|b| allocator[b.result].ty()),
		)
		.expect("Invalid case result type");
		let ty = if make_var {
			result_ty
				.with_inst(self.db.upcast(), VarType::Var)
				.expect("Cannot make var")
		} else {
			result_ty
		};
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::Case {
				scrutinee,
				branches,
			},
		)
	}

	/// Create a function call
	pub fn call(
		&self,
		function: ExpressionId,
		arguments: impl IntoIterator<Item = ExpressionId>,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let ty = match allocator[function].ty().lookup(self.db.upcast()) {
			TyData::Function(_, ft) => ft.return_type,
			_ => unreachable!("Invalid function type"),
		};
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::Call {
				function,
				arguments: arguments.into_iter().collect(),
			},
		)
	}

	/// Create a call to a function with the given name
	pub fn lookup_call(
		&self,
		function: Identifier,
		arguments: impl IntoIterator<Item = ExpressionId>,
		allocator: &ExpressionAllocator,
	) -> CallBuilder {
		let args: Vec<_> = arguments.into_iter().collect();
		let arg_tys: Vec<_> = args.iter().map(|arg| allocator[*arg].ty()).collect();
		let lookup = self
			.model
			.lookup_function(self.db, function, &arg_tys)
			.unwrap_or_else(|e| {
				panic!(
					"Function {}({}) not found:\n{}",
					function.pretty_print(self.db.upcast()),
					arg_tys
						.iter()
						.map(|ty| ty.pretty_print(self.db.upcast()))
						.collect::<Vec<_>>()
						.join(", "),
					e.debug_print(self.db.upcast())
				)
			});
		let ret_ty = lookup.fn_type.return_type;
		CallBuilder::new(
			Ty::function(self.db.upcast(), lookup.fn_type),
			ret_ty,
			lookup.function,
			args,
		)
	}

	/// Create a let expression
	pub fn let_expression(
		&self,
		items: impl IntoIterator<Item = LetItem>,
		in_expression: ExpressionId,
		allocator: &ExpressionAllocator,
	) -> ExpressionBuilderWithData {
		let ty = allocator[in_expression].ty();
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::Let {
				items: items.into_iter().collect(),
				in_expression,
			},
		)
	}

	/// Create a lambda function expression
	pub fn lambda(
		&self,
		domain: Domain,
		parameters: impl IntoIterator<Item = DeclarationId>,
		body: ExpressionId,
	) -> ExpressionBuilderWithData {
		let params: Vec<_> = parameters.into_iter().collect();
		let ty = Ty::function(
			self.db.upcast(),
			FunctionType {
				return_type: domain.ty(),
				params: params.iter().map(|d| self.model[*d].ty()).collect(),
			},
		);
		ExpressionBuilderWithData::new(
			ty,
			ExpressionData::Lambda {
				domain,
				parameters: params,
				body,
			},
		)
	}
}

/// Helper for building expressions
pub struct ExpressionBuilderWithData {
	ty: Ty,
	data: ExpressionData,
	annotations: Vec<ExpressionId>,
}

impl ExpressionBuilderWithData {
	fn new(ty: Ty, data: ExpressionData) -> Self {
		Self {
			ty,
			data,
			annotations: Vec::new(),
		}
	}

	/// Add an annotation to the expression
	pub fn with_annotation(mut self, ann: ExpressionId) -> Self {
		self.annotations.push(ann);
		self
	}

	/// Add the given annotations to the expression
	pub fn with_annotations(mut self, annotations: impl IntoIterator<Item = ExpressionId>) -> Self {
		self.annotations.extend(annotations);
		self
	}

	/// Build the expression
	pub fn finish(
		self,
		allocator: &mut ExpressionAllocator,
		origin: impl Into<Origin>,
	) -> ExpressionId {
		let idx = allocator.new_unchecked(origin.into(), self.ty, self.data);
		for ann in self.annotations {
			allocator.annotate_expression(idx, ann);
		}
		idx
	}
}

/// Helper for building calls
pub struct CallBuilder {
	fn_ty: Ty,
	ret_ty: Ty,
	fn_id: FunctionId,
	args: Vec<ExpressionId>,
	annotations: Vec<ExpressionId>,
}

impl CallBuilder {
	fn new(fn_ty: Ty, ret_ty: Ty, fn_id: FunctionId, args: Vec<ExpressionId>) -> Self {
		Self {
			fn_ty,
			ret_ty,
			fn_id,
			args,
			annotations: Vec::new(),
		}
	}

	/// Add an annotation to the call
	pub fn with_annotation(mut self, ann: ExpressionId) -> Self {
		self.annotations.push(ann);
		self
	}

	/// Add the given annotations to the call
	pub fn with_annotations(mut self, annotations: impl IntoIterator<Item = ExpressionId>) -> Self {
		self.annotations.extend(annotations);
		self
	}

	/// Build the call
	pub fn finish(
		self,
		allocator: &mut ExpressionAllocator,
		origin: impl Into<Origin>,
	) -> ExpressionId {
		let origin = origin.into();
		let function = allocator.new_unchecked(
			origin,
			self.fn_ty,
			ExpressionData::Identifier(ResolvedIdentifier::Function(self.fn_id)),
		);
		let idx = allocator.new_unchecked(
			origin,
			self.ret_ty,
			ExpressionData::Call {
				function,
				arguments: self.args,
			},
		);
		for ann in self.annotations {
			allocator.annotate_expression(idx, ann);
		}
		idx
	}
}

/// Type of an expression index in an arena
pub type ExpressionId = ArenaIndex<Expression>;

/// An expression
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
	ty: Ty,
	data: ExpressionData,
}

impl Expression {
	/// Get the type of this expression
	pub fn ty(&self) -> Ty {
		self.ty
	}
}

impl Deref for Expression {
	type Target = ExpressionData;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

/// An expression
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionData {
	/// Absent `<>`
	Absent,
	/// Bool literal
	BooleanLiteral(BooleanLiteral),
	/// Integer literal
	IntegerLiteral(IntegerLiteral),
	/// Float literal
	FloatLiteral(FloatLiteral),
	/// String literal
	StringLiteral(StringLiteral),
	/// Infinity
	Infinity,
	/// Identifier
	Identifier(ResolvedIdentifier),
	/// Array literal
	ArrayLiteral(Vec<ExpressionId>),
	/// Set literal
	SetLiteral(Vec<ExpressionId>),
	/// Tuple literal
	TupleLiteral(Vec<ExpressionId>),
	/// Record literal
	RecordLiteral(Vec<(Identifier, ExpressionId)>),
	/// Array comprehension
	ArrayComprehension {
		/// Value of the comprehension
		template: ExpressionId,
		/// The indices to generate
		indices: Option<ExpressionId>,
		/// Generators of the comprehension
		generators: Vec<Generator>,
	},
	/// Set comprehension
	SetComprehension {
		/// Value of the comprehension
		template: ExpressionId,
		/// Generators of the comprehension
		generators: Vec<Generator>,
	},
	/// Array access
	ArrayAccess {
		/// The array being indexed into
		collection: ExpressionId,
		/// The indices
		indices: ExpressionId,
	},
	/// Tuple access
	TupleAccess {
		/// Tuple being accessed
		tuple: ExpressionId,
		/// Field being accessed
		field: IntegerLiteral,
	},
	/// Record access
	RecordAccess {
		/// Record being accessed
		record: ExpressionId,
		/// Field being accessed
		field: Identifier,
	},
	/// If-then-else
	IfThenElse {
		/// The if-then and elseif-then branches
		branches: Vec<Branch>,
		/// The else result
		else_result: ExpressionId,
	},
	/// Case expression
	Case {
		/// The expression being matched on
		scrutinee: ExpressionId,
		/// The case match arms
		branches: Vec<CaseBranch>,
	},
	/// Function call
	Call {
		/// Function being called
		function: ExpressionId,
		/// Call arguments
		arguments: Vec<ExpressionId>,
	},
	/// Let expression
	Let {
		/// Items in this let expression
		items: Vec<LetItem>,
		/// Value of the let expression
		in_expression: ExpressionId,
	},
	/// Lambda function
	Lambda {
		/// Domain of return type
		domain: Domain,
		/// Function parameters
		parameters: Vec<DeclarationId>,
		/// Function body
		body: ExpressionId,
	},
}

/// An identifier which resolves to a declaration
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedIdentifier {
	/// Identifier resolves to an annotation
	Annotation(AnnotationId),
	/// Identifier resolves to an annotation destructor
	AnnotationDestructure(AnnotationId),
	/// Identifier resolves to a declaration
	Declaration(DeclarationId),
	/// Identifier resolves to an enumeration
	Enumeration(EnumerationId),
	/// Identifier resolves to an enumeration member with the given index
	EnumerationMember(EnumMemberId),
	/// Identifier resolves to the destructor for an enumeration member with the given index
	EnumerationDestructure(EnumMemberId),
	/// Identifier resolves to a function
	Function(FunctionId),
	/// Identifier resolves to a type-inst variable in a function
	TyVarRef(TyVarId),
}

/// Reference to a member of an enum
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EnumMemberId {
	parent: EnumerationId,
	index: u32,
}

impl EnumMemberId {
	/// Create a new reference to a enum member
	pub fn new(enumeration: EnumerationId, index: u32) -> Self {
		Self {
			parent: enumeration,
			index,
		}
	}

	/// Get the enumeration id
	pub fn enumeration_id(&self) -> EnumerationId {
		self.parent
	}

	/// Get the index of the enum member inside the enum
	pub fn member_index(&self) -> u32 {
		self.index
	}
}

/// Kind of enum constructor (or destructor)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EnumConstructorKind {
	/// par enum
	Par,
	/// var enum
	Var,
	/// par opt enum
	Opt,
	/// var opt enum
	VarOpt,
	/// set of enum
	Set,
	/// var set of enum
	VarSet,
}

impl EnumConstructorKind {
	/// Gets the enum constructor kind which was used to create something of this type
	pub fn from_ty(db: &dyn Thir, ty: Ty) -> Self {
		let is_var = ty.inst(db.upcast()).unwrap() == VarType::Var;
		let is_opt = ty.opt(db.upcast()).unwrap() == OptType::Opt;
		let is_set = ty.is_set(db.upcast());
		match (is_var, is_opt, is_set) {
			(false, false, false) => EnumConstructorKind::Par,
			(true, false, false) => EnumConstructorKind::Var,
			(false, true, false) => EnumConstructorKind::Opt,
			(true, true, false) => EnumConstructorKind::VarOpt,
			(false, false, true) => EnumConstructorKind::Set,
			(true, false, true) => EnumConstructorKind::VarSet,
			_ => unreachable!(),
		}
	}
}

/// Reference to a type-inst variable defined by a function
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TyVarId {
	parent: FunctionId,
	index: u32,
}

impl TyVarId {
	/// Create a new reference to a type-inst variable
	pub fn new(function: FunctionId, index: u32) -> Self {
		Self {
			parent: function,
			index,
		}
	}

	/// Get the function id
	pub fn function_id(&self) -> FunctionId {
		self.parent
	}

	/// Get the index of the type-inst variable inside the function
	pub fn ty_var_index(&self) -> u32 {
		self.index
	}
}

/// Comprehension generator
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Generator {
	/// Generator which iterates over a collection
	Iterator {
		/// Generator declaration
		declarations: Vec<DeclarationId>,
		/// Expression being iterated over
		collection: ExpressionId,
		/// Where clause
		where_clause: Option<ExpressionId>,
	},
	/// Generator which is an assignment
	Assignment {
		/// The assignment to generate
		assignment: DeclarationId,
		/// Where clause
		where_clause: Option<ExpressionId>,
	},
}

impl Generator {
	/// Whether this generator has a var where clause
	pub fn var_where(&self, db: &dyn Thir, expressions: &ExpressionAllocator) -> bool {
		match self {
			Generator::Iterator {
				where_clause: Some(w),
				..
			}
			| Generator::Assignment {
				where_clause: Some(w),
				..
			} => expressions[*w].ty().inst(db.upcast()).unwrap() == VarType::Var,
			_ => false,
		}
	}

	/// Whether this generator iterates over a var set
	pub fn var_set(&self, db: &dyn Thir, expressions: &ExpressionAllocator) -> bool {
		match self {
			Generator::Iterator { collection, .. } => matches!(
				expressions[*collection].ty().lookup(db.upcast()),
				TyData::Set(VarType::Var, _, _)
			),
			_ => false,
		}
	}
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
	/// The boolean condition
	pub condition: ExpressionId,
	/// The result if the condition holds
	pub result: ExpressionId,
}

impl Branch {
	/// Create a new branch for an if-then-else
	pub fn new(condition: ExpressionId, result: ExpressionId) -> Self {
		Self { condition, result }
	}
}

/// A branch of a `Case`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseBranch {
	/// The pattern to match
	pub pattern: Pattern,
	/// The value if the pattern matches
	pub result: ExpressionId,
}

impl CaseBranch {
	/// Create a new case branch
	pub fn new(pattern: Pattern, result: ExpressionId) -> Self {
		Self { pattern, result }
	}
}

/// A pattern for a case expression.
///
/// In THIR, patterns are only used for case expressions.
/// Destructuring assignments are represented using multiple declarations.
///
/// Note that patterns at this level do not represent binding to variables.
/// Instead, the anonymous wildcard pattern is used, and destructuring happens
/// via destructor functions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pattern {
	/// Enum constructor call
	EnumConstructor {
		/// The enum item member
		member: EnumMemberId,
		/// Type of constructor
		kind: EnumConstructorKind,
		/// The constructor call arguments
		args: Vec<Pattern>,
	},
	/// Annotation constructor call
	AnnotationConstructor {
		/// The annotation item
		item: AnnotationId,
		/// The constructor call arguments
		args: Vec<Pattern>,
	},
	/// Tuple
	Tuple(Vec<Pattern>),
	/// Record
	Record(Vec<(Identifier, Pattern)>),
	/// Literal expression (e.g. enum atoms, numbers, strings, <>)
	Expression(ExpressionId),
	/// Wildcard pattern _
	Anonymous(Ty),
}

/// An item in a let expression
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LetItem {
	/// A local constraint item
	Constraint(ConstraintId),
	/// A local declaration item
	Declaration(DeclarationId),
}
