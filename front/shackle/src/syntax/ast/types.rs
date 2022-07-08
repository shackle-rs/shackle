//! AST representation for types

use super::{helpers::*, Identifier};
use super::{AstNode, Children, Expression};

ast_enum!(
	/// Type from a declaration
	Type,
	"array_type" => ArrayType,
	"tuple_type" => TupleType,
	"record_type" => RecordType,
	"operation_type" => OperationType,
	"type_base" => TypeBase
);

ast_node!(
	/// Type of an array
	ArrayType,
	dimensions,
	element_type
);

impl ArrayType {
	/// The ranges of the array.
	pub fn dimensions(&self) -> Children<'_, TypeBase> {
		children_with_field_name(self, "dimension")
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
	set_type,
	any_type,
	domain
);

impl TypeBase {
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
		match node.child_by_field_name("opt") {
			Some(_) => OptType::Opt,
			None => OptType::NonOpt,
		}
	}

	/// Get whether this is a set type
	pub fn set_type(&self) -> SetType {
		let node = self.cst_node().as_ref();
		match node.child_by_field_name("set") {
			Some(_) => SetType::Set,
			None => SetType::NonSet,
		}
	}

	/// Get whether this is an any type
	pub fn any_type(&self) -> AnyType {
		let node = self.cst_node().as_ref();
		match node.child_by_field_name("any") {
			Some(_) => AnyType::Any,
			None => AnyType::NonAny,
		}
	}

	/// Get the domain of this type (can be `None` if type is `any`)
	pub fn domain(&self) -> Option<Domain> {
		optional_child_with_field_name(self, "domain")
	}
}

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

/// Whether a type is a set or not
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SetType {
	/// Non-set variable
	NonSet,
	/// Set variable
	Set,
}

/// Whether a type is any or not
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AnyType {
	/// Non-any variable
	NonAny,
	/// Any variable
	Any,
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
	/// Name of identifier without the leading $
	pub fn name(&self) -> &str {
		let text = self.cst_text();
		&text[1..]
	}
}

ast_node!(
	/// Type-inst enum identifier `$$E`
	TypeInstEnumIdentifier,
	name
);

impl TypeInstEnumIdentifier {
	/// Name of identifier without the leading $
	pub fn name(&self) -> &str {
		let text = self.cst_text();
		&text[2..]
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
				.domain()
				.unwrap()
				.cast::<Expression>()
				.unwrap()
				.cast::<Anonymous>()
				.unwrap();
			assert!(x_t
				.element_type()
				.cast::<TypeBase>()
				.unwrap()
				.domain()
				.unwrap()
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
					.domain()
					.unwrap()
					.cast::<Expression>()
					.unwrap()
					.cast::<Identifier>()
					.unwrap()
					.name(),
				"foo"
			);
			assert_eq!(
				y_dims[1]
					.domain()
					.unwrap()
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
				.unwrap()
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
				.unwrap()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_int());
			assert!(x_fields[1]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.unwrap()
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
				.unwrap()
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
				.unwrap()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
			assert!(y_fields_2[1]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.unwrap()
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
				.unwrap()
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
				.unwrap()
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
				.unwrap()
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
				.unwrap()
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
				.unwrap()
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
				.unwrap()
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
				.unwrap()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
			assert!(x_ps[1]
				.cast_ref::<TypeBase>()
				.unwrap()
				.domain()
				.unwrap()
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
			any $T: h;
			var $$E: i;
		"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 9);
		{
			let a = items[0].cast_ref::<Declaration>().unwrap();
			assert_eq!(a.pattern().cast::<Identifier>().unwrap().name(), "a");
			let t = a.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), VarType::Par);
			assert_eq!(t.opt_type(), OptType::NonOpt);
			assert_eq!(t.set_type(), SetType::NonSet);
			assert_eq!(t.any_type(), AnyType::NonAny);
			assert!(t
				.domain()
				.unwrap()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_int());
		}
		{
			let b = items[1].cast_ref::<Declaration>().unwrap();
			assert_eq!(b.pattern().cast::<Identifier>().unwrap().name(), "b");
			let t = b.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), VarType::Var);
			assert_eq!(t.opt_type(), OptType::NonOpt);
			assert_eq!(t.set_type(), SetType::NonSet);
			assert_eq!(t.any_type(), AnyType::NonAny);
			assert!(t
				.domain()
				.unwrap()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_bool());
		}
		{
			let c = items[2].cast_ref::<Declaration>().unwrap();
			assert_eq!(c.pattern().cast::<Identifier>().unwrap().name(), "c");
			let t = c.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), VarType::Var);
			assert_eq!(t.opt_type(), OptType::Opt);
			assert_eq!(t.set_type(), SetType::NonSet);
			assert_eq!(t.any_type(), AnyType::NonAny);
			assert!(t
				.domain()
				.unwrap()
				.cast::<UnboundedDomain>()
				.unwrap()
				.primitive_type()
				.is_string());
		}
		{
			let d = items[3].cast_ref::<Declaration>().unwrap();
			assert_eq!(d.pattern().cast::<Identifier>().unwrap().name(), "d");
			let t = d.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), VarType::Var);
			assert_eq!(t.opt_type(), OptType::NonOpt);
			assert_eq!(t.set_type(), SetType::Set);
			assert_eq!(t.any_type(), AnyType::NonAny);
			let dom = t
				.domain()
				.unwrap()
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
			let t = e.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), VarType::Par);
			assert_eq!(t.set_type(), SetType::Set);
			assert_eq!(t.opt_type(), OptType::Opt);
			assert_eq!(t.any_type(), AnyType::NonAny);
			assert_eq!(
				t.domain()
					.unwrap()
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
			let t = f.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), VarType::Par);
			assert_eq!(t.opt_type(), OptType::NonOpt);
			assert_eq!(t.set_type(), SetType::NonSet);
			assert_eq!(t.any_type(), AnyType::Any);
			assert!(t.domain().is_none());
		}
		{
			let g = items[6].cast_ref::<Declaration>().unwrap();
			assert_eq!(g.pattern().cast::<Identifier>().unwrap().name(), "g");
			let t = g.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), VarType::Par);
			assert_eq!(t.opt_type(), OptType::NonOpt);
			assert_eq!(t.set_type(), SetType::NonSet);
			assert_eq!(t.any_type(), AnyType::NonAny);
			assert_eq!(
				t.domain()
					.unwrap()
					.cast::<TypeInstIdentifier>()
					.unwrap()
					.name(),
				"T"
			);
		}
		{
			let h = items[7].cast_ref::<Declaration>().unwrap();
			assert_eq!(h.pattern().cast::<Identifier>().unwrap().name(), "h");
			let t = h.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), VarType::Par);
			assert_eq!(t.opt_type(), OptType::NonOpt);
			assert_eq!(t.set_type(), SetType::NonSet);
			assert_eq!(t.any_type(), AnyType::Any);
			assert_eq!(
				t.domain()
					.unwrap()
					.cast::<TypeInstIdentifier>()
					.unwrap()
					.name(),
				"T"
			);
		}
		{
			let i = items[8].cast_ref::<Declaration>().unwrap();
			assert_eq!(i.pattern().cast::<Identifier>().unwrap().name(), "i");
			let t = i.declared_type().cast::<TypeBase>().unwrap();
			assert_eq!(t.var_type(), VarType::Var);
			assert_eq!(t.opt_type(), OptType::NonOpt);
			assert_eq!(t.set_type(), SetType::NonSet);
			assert_eq!(t.any_type(), AnyType::NonAny);
			assert_eq!(
				t.domain()
					.unwrap()
					.cast::<TypeInstEnumIdentifier>()
					.unwrap()
					.name(),
				"E"
			);
		}
	}
}
