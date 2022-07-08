//! AST representation for types

use super::{helpers::*, Identifier};
use super::{AstNode, Children, Expression};

ast_enum!(
	/// Type from a declaration
	Type,
	"array_type" => ArrayType,
	"set_type" => SetType,
	"tuple_type" => TupleType,
	"record_type" => RecordType,
	"operation_type" => OperationType,
	"type_base" => TypeBase,
	"any_type" => AnyType,
);

ast_node!(
	/// Type of an array
	ArrayType,
	dimensions,
	element_type
);

impl ArrayType {
	/// The ranges of the array.
	pub fn dimensions(&self) -> Children<'_, Type> {
		children_with_field_name(self, "dimension")
	}

	/// The type contained in the array
	pub fn element_type(&self) -> Type {
		child_with_field_name(self, "type")
	}
}

ast_node!(
	/// Type of a set
	SetType,
	var_type,
	opt_type,
	element_type
);

impl SetType {
	/// Get whether this type is var or par
	pub fn var_type(&self) -> VarType {
		let node = self.cst_node().as_ref();
		node.child_by_field_name("var_par")
			.map(|c| match c.kind() {
				"var" => VarType::Var,
				"par" => VarType::Par,
				_ => unreachable!(),
			})
			.unwrap_or(VarType::Par)
	}

	/// Get optionality of type
	pub fn opt_type(&self) -> OptType {
		let node = self.cst_node().as_ref();
		node.child_by_field_name("opt")
			.map(|c| match c.kind() {
				"opt" => OptType::Opt,
				"nonopt" => OptType::NonOpt,
				_ => unreachable!(),
			})
			.unwrap_or(OptType::NonOpt)
	}

	/// The type contained in the array
	pub fn element_type(&self) -> Type {
		child_with_field_name(self, "type")
	}
}

ast_node!(
	/// Type of a tuple
	TupleType,
	fields
);

impl TupleType {
	/// The types of the tuple fields
	pub fn fields(&self) -> Children<'_, Type> {
		children_with_field_name(self, "field")
	}
}

ast_node!(
	/// Type of a record
	RecordType,
	fields
);

impl RecordType {
	/// The types of the tuple fields
	pub fn fields(&self) -> Children<'_, RecordField> {
		children_with_field_name(self, "field")
	}
}

ast_node!(
	/// Field in a record type
	RecordField,
	name,
	field_type
);

impl RecordField {
	/// The name of the field
	pub fn name(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// The type of the field
	pub fn field_type(&self) -> Type {
		child_with_field_name(self, "type")
	}
}

ast_node!(
	/// Type of an operation (function)
	OperationType,
	return_type,
	parameter_types,
);

impl OperationType {
	/// Function return type
	pub fn return_type(&self) -> Type {
		child_with_field_name(self, "return_type")
	}

	/// Function parameter types
	pub fn parameter_types(&self) -> Children<'_, Type> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// Type from a declaration
	TypeBase,
	var_type,
	opt_type,
	any_type,
	domain
);

impl TypeBase {
	/// Get whether this type is var or par.
	///
	/// Gives `None` when omitted rather than `Par` since omitting the inst
	/// when referring to a type-inst alias does not make it par.
	pub fn var_type(&self) -> Option<VarType> {
		let node = self.cst_node().as_ref();
		node.child_by_field_name("var_par").map(|c| match c.kind() {
			"var" => VarType::Var,
			"par" => VarType::Par,
			_ => unreachable!(),
		})
	}

	/// Get optionality of type.
	///
	/// Gives `None` when omitted rather than `NonOpt` since omitting optionality
	/// when referring to a type-inst alias does not make it non-optional.
	pub fn opt_type(&self) -> Option<OptType> {
		let node = self.cst_node().as_ref();
		node.child_by_field_name("opt").map(|c| match c.kind() {
			"opt" => OptType::Opt,
			"nonopt" => OptType::NonOpt,
			_ => unreachable!(),
		})
	}

	/// Get whether this type is any $T
	pub fn any_type(&self) -> bool {
		self.cst_node()
			.as_ref()
			.child_by_field_name("any")
			.is_some()
	}

	/// Get the domain of this type (can be `None` if type is `any`)
	pub fn domain(&self) -> Domain {
		child_with_field_name(self, "domain")
	}
}

ast_node!(
	/// Type is inferred for RHS
	AnyType
);

/// Whether a type is var or par
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VarType {
	/// Fixed parameter
	Par,
	/// Decision variable
	Var,
}

/// Whether a type is opt or non-opt
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OptType {
	/// Non-optional variable
	NonOpt,
	/// Optional variable
	Opt,
}

ast_enum!(
	/// Domain for a declaration
	Domain,
	"primitive_type" => Unbounded(UnboundedDomain),
	"type_inst_id" => TypeInstIdentifier,
	"type_inst_enum_id" => TypeInstEnumIdentifier,
	_ => Bounded(Expression)
);

ast_node!(
	/// Unbounded primitive type domain
	UnboundedDomain,
	primitive_type
);

impl UnboundedDomain {
	/// Get the primitive type of this domain
	pub fn primitive_type(&self) -> PrimitiveType {
		match self.cst_text() {
			"ann" => PrimitiveType::Ann,
			"bool" => PrimitiveType::Bool,
			"float" => PrimitiveType::Float,
			"int" => PrimitiveType::Int,
			"string" => PrimitiveType::String,
			_ => unreachable!(),
		}
	}
}

/// Primitive base type
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
	/// `ann` type
	Ann,
	/// `bool` type
	Bool,
	/// `float` type
	Float,
	/// `int` type
	Int,
	/// `string` type
	String,
}

impl PrimitiveType {
	/// Return whether this is an `ann`.
	pub fn is_ann(&self) -> bool {
		if let PrimitiveType::Ann = *self {
			true
		} else {
			false
		}
	}

	/// Return whether this is an `ann`.
	pub fn is_bool(&self) -> bool {
		if let PrimitiveType::Bool = *self {
			true
		} else {
			false
		}
	}

	/// Return whether this is an `ann`.
	pub fn is_float(&self) -> bool {
		if let PrimitiveType::Float = *self {
			true
		} else {
			false
		}
	}

	/// Return whether this is an `ann`.
	pub fn is_int(&self) -> bool {
		if let PrimitiveType::Int = *self {
			true
		} else {
			false
		}
	}

	/// Return whether this is an `ann`.
	pub fn is_string(&self) -> bool {
		if let PrimitiveType::String = *self {
			true
		} else {
			false
		}
	}
}

ast_node!(
	/// Type-inst identifier `$T`
	TypeInstIdentifier,
	name
);

impl TypeInstIdentifier {
	/// Name of identifier
	pub fn name(&self) -> &str {
		self.cst_text()
	}
}

ast_node!(
	/// Type-inst enum identifier `$$E`
	TypeInstEnumIdentifier,
	name
);

impl TypeInstEnumIdentifier {
	/// Name of identifier
	pub fn name(&self) -> &str {
		self.cst_text()
	}
}

#[cfg(test)]
mod test {
	use crate::syntax::ast::helpers::test::*;
	use crate::syntax::ast::*;

	#[test]
	fn test_array_type() {
		let model = parse_model(
			r#"
			array [_] of bool: x;
			array [foo, bar] of bool: y;
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 2);
		{
			let x = items[0].cast_ref::<Declaration>().unwrap();
			assert_eq!(x.pattern().cast::<Identifier>().unwrap().name(), "x");
			let x_t = x.declared_type().cast::<ArrayType>().unwrap();
			let x_dims: Vec<_> = x_t.dimensions().collect();
			assert_eq!(x_dims.len(), 1);
			x_dims[0]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<Expression>()
				.unwrap()
				.cast::<Anonymous>()
				.unwrap();
			assert!(x_t
				.element_type()
				.cast::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
		}
		{
			let y = items[1].cast_ref::<Declaration>().unwrap();
			assert_eq!(y.pattern().cast::<Identifier>().unwrap().name(), "y");
			let y_t = y.declared_type().cast::<ArrayType>().unwrap();
			let y_dims: Vec<_> = y_t.dimensions().collect();
			assert_eq!(y_dims.len(), 2);
			assert_eq!(
				y_dims[0]
					.cast_ref::<TypeBase>()
					.unwrap()
					.domain()
					.cast::<Expression>()
					.unwrap()
					.cast::<Identifier>()
					.unwrap()
					.name(),
				"foo"
			);
			assert_eq!(
				y_dims[1]
					.cast_ref::<TypeBase>()
					.unwrap()
					.domain()
					.cast::<Expression>()
					.unwrap()
					.cast::<Identifier>()
					.unwrap()
					.name(),
				"bar"
			);
			assert!(y_t
				.element_type()
				.cast::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
		}
	}

	#[test]
	fn test_tuple_type() {
		let model = parse_model(
			r#"
			tuple(int, bool): x;
			tuple(int, tuple(bool, float)): y;
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 2);
		{
			let x = items[0].cast_ref::<Declaration>().unwrap();
			assert_eq!(x.pattern().cast::<Identifier>().unwrap().name(), "x");
			let x_t = x.declared_type().cast::<TupleType>().unwrap();
			let x_fields: Vec<_> = x_t.fields().collect();
			assert_eq!(x_fields.len(), 2);
			assert!(x_fields[0]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_int());
			assert!(x_fields[1]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
		}
		{
			let y = items[1].cast_ref::<Declaration>().unwrap();
			assert_eq!(y.pattern().cast::<Identifier>().unwrap().name(), "y");
			let y_t = y.declared_type().cast::<TupleType>().unwrap();
			let y_fields: Vec<_> = y_t.fields().collect();
			assert_eq!(y_fields.len(), 2);
			assert!(y_fields[0]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_int());
			let y_fields_2: Vec<_> = y_fields[1]
				.cast_ref::<TupleType>()
				.unwrap()
				.fields()
				.collect();
			assert_eq!(y_fields_2.len(), 2);
			assert!(y_fields_2[0]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
			assert!(y_fields_2[1]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_float());
		}
	}

	#[test]
	fn test_record_type() {
		let model = parse_model(
			r#"
			record(int: a, bool: b): x;
			record(int: a, record(bool: c, float: d): b): y;
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 2);
		{
			let x = items[0].cast_ref::<Declaration>().unwrap();
			assert_eq!(x.pattern().cast::<Identifier>().unwrap().name(), "x");
			let x_t = x.declared_type().cast::<RecordType>().unwrap();
			let x_fields: Vec<_> = x_t.fields().collect();
			assert_eq!(x_fields.len(), 2);
			assert_eq!(x_fields[0].name().name(), "a");
			assert!(x_fields[0]
				.field_type()
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_int());
			assert_eq!(x_fields[1].name().name(), "b");
			assert!(x_fields[1]
				.field_type()
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
		}
		{
			let y = items[1].cast_ref::<Declaration>().unwrap();
			assert_eq!(y.pattern().cast::<Identifier>().unwrap().name(), "y");
			let y_t = y.declared_type().cast::<RecordType>().unwrap();
			let y_fields: Vec<_> = y_t.fields().collect();
			assert_eq!(y_fields.len(), 2);
			assert_eq!(y_fields[0].name().name(), "a");
			assert!(y_fields[0]
				.field_type()
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_int());
			assert_eq!(y_fields[1].name().name(), "b");
			let y_fields_2: Vec<_> = y_fields[1]
				.field_type()
				.cast_ref::<RecordType>()
				.unwrap()
				.fields()
				.collect();
			assert_eq!(y_fields_2.len(), 2);
			assert_eq!(y_fields_2[0].name().name(), "c");
			assert!(y_fields_2[0]
				.field_type()
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
			assert_eq!(y_fields_2[1].name().name(), "d");
			assert!(y_fields_2[1]
				.field_type()
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_float());
		}
	}

	#[test]
	fn test_operation_type() {
		let model = parse_model(
			r#"
			op(int: (bool, string)): x;
		"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let x = items[0].cast_ref::<Declaration>().unwrap();
			assert_eq!(x.pattern().cast::<Identifier>().unwrap().name(), "x");
			let x_t = x.declared_type().cast::<OperationType>().unwrap();
			assert!(x_t
				.return_type()
				.cast::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_int());
			let x_ps: Vec<_> = x_t.parameter_types().collect();
			assert_eq!(x_ps.len(), 2);
			assert!(x_ps[0]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
			assert!(x_ps[1]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_string());
		}
	}

	#[test]
	fn test_type_base() {
		let model = parse_model(
			r#"
			int: a;
			var bool: b;
			var opt string: c;
			var set of 1..3: d;
			par opt set of Foo: e;
			any: f;
			$T: g;
			opt $T: h;
			var $$E: i;
		"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 9);
		{
			let a = items[0].cast_ref::<Declaration>().unwrap();
			assert_eq!(a.pattern().cast::<Identifier>().unwrap().name(), "a");
			let t = a.declared_type().cast::<TypeBase>().unwrap();
			assert!(t.var_type().is_none());
			assert!(t.opt_type().is_none());
			assert!(t
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_int());
		}
		{
			let b = items[1].cast_ref::<Declaration>().unwrap();
			assert_eq!(b.pattern().cast::<Identifier>().unwrap().name(), "b");
			let t = b.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), Some(VarType::Var));
			assert!(t.opt_type().is_none());
			assert!(t
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
		}
		{
			let c = items[2].cast_ref::<Declaration>().unwrap();
			assert_eq!(c.pattern().cast::<Identifier>().unwrap().name(), "c");
			let t = c.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), Some(VarType::Var));
			assert_eq!(t.opt_type(), Some(OptType::Opt));
			assert!(t
				.domain()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_string());
		}
		{
			let d = items[3].cast_ref::<Declaration>().unwrap();
			assert_eq!(d.pattern().cast::<Identifier>().unwrap().name(), "d");
			let t = d.declared_type().cast::<SetType>().unwrap();
			assert_eq!(t.var_type(), VarType::Var);
			assert_eq!(t.opt_type(), OptType::NonOpt);
			let e = t.element_type().cast::<TypeBase>().unwrap();
			assert!(e.var_type().is_none());
			assert!(e.opt_type().is_none());
			let dom = e
				.domain()
				.cast::<Expression>()
				.unwrap()
				.cast::<InfixOperator>()
				.unwrap();
			assert_eq!(dom.left().cast::<IntegerLiteral>().unwrap().value(), 1);
			assert_eq!(dom.operator(), "..");
			assert_eq!(dom.right().cast::<IntegerLiteral>().unwrap().value(), 3);
		}
		{
			let e = items[4].cast_ref::<Declaration>().unwrap();
			assert_eq!(e.pattern().cast::<Identifier>().unwrap().name(), "e");
			let t = e.declared_type().cast::<SetType>().unwrap();
			assert_eq!(t.var_type(), VarType::Par);
			assert_eq!(t.opt_type(), OptType::Opt);
			let u = t.element_type().cast::<TypeBase>().unwrap();
			assert!(u.var_type().is_none());
			assert!(u.opt_type().is_none());
			assert_eq!(
				u.domain()
					.cast::<Expression>()
					.unwrap()
					.cast::<Identifier>()
					.unwrap()
					.name(),
				"Foo"
			);
		}
		{
			let f = items[5].cast_ref::<Declaration>().unwrap();
			assert_eq!(f.pattern().cast::<Identifier>().unwrap().name(), "f");
			assert!(f.declared_type().cast::<AnyType>().is_some());
		}
		{
			let g = items[6].cast_ref::<Declaration>().unwrap();
			assert_eq!(g.pattern().cast::<Identifier>().unwrap().name(), "g");
			let t = g.declared_type().cast::<TypeBase>().unwrap();
			assert!(t.var_type().is_none());
			assert!(t.opt_type().is_none());
			assert_eq!(
				t.domain().cast::<TypeInstIdentifier>().unwrap().name(),
				"$T"
			);
		}
		{
			let h = items[7].cast_ref::<Declaration>().unwrap();
			assert_eq!(h.pattern().cast::<Identifier>().unwrap().name(), "h");
			let t = h.declared_type().cast::<TypeBase>().unwrap();
			assert!(t.var_type().is_none());
			assert_eq!(t.opt_type(), Some(OptType::Opt));
			assert_eq!(
				t.domain().cast::<TypeInstIdentifier>().unwrap().name(),
				"$T"
			);
		}
		{
			let i = items[8].cast_ref::<Declaration>().unwrap();
			assert_eq!(i.pattern().cast::<Identifier>().unwrap().name(), "i");
			let t = i.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), Some(VarType::Var));
			assert!(t.opt_type().is_none());
			assert_eq!(
				t.domain().cast::<TypeInstEnumIdentifier>().unwrap().name(),
				"$$E"
			);
		}
	}
}
