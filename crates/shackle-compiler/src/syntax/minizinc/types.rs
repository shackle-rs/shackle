//! AST representation for types

use super::{Children, Expression, Identifier};
use crate::syntax::ast::{
	ast_enum, ast_node, child_with_field_name, children_with_field_name, AstNode,
};

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
	var_type,
	fields
);

impl TupleType {
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

	/// The types of the tuple fields
	pub fn fields(&self) -> Children<'_, Type> {
		children_with_field_name(self, "field")
	}
}

ast_node!(
	/// Type of a record
	RecordType,
	var_type,
	fields
);

impl RecordType {
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VarType {
	/// Fixed parameter
	Par,
	/// Decision variable
	Var,
}

impl VarType {
	/// `var` if this is var, otherwise `None`
	pub fn pretty_print(&self) -> Option<String> {
		match *self {
			VarType::Var => Some("var".to_owned()),
			_ => None,
		}
	}
}

/// Whether a type is opt or non-opt
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OptType {
	/// Non-optional variable
	NonOpt,
	/// Optional variable
	Opt,
}

impl OptType {
	/// `opt` if this is opt, otherwise `None`
	pub fn pretty_print(&self) -> Option<String> {
		match *self {
			OptType::Opt => Some("opt".to_owned()),
			_ => None,
		}
	}
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
		matches!(*self, PrimitiveType::Ann)
	}

	/// Return whether this is an `ann`.
	pub fn is_bool(&self) -> bool {
		matches!(*self, PrimitiveType::Bool)
	}

	/// Return whether this is an `ann`.
	pub fn is_float(&self) -> bool {
		matches!(*self, PrimitiveType::Float)
	}

	/// Return whether this is an `ann`.
	pub fn is_int(&self) -> bool {
		matches!(*self, PrimitiveType::Int)
	}

	/// Return whether this is an `ann`.
	pub fn is_string(&self) -> bool {
		matches!(*self, PrimitiveType::String)
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
	use expect_test::expect;

	use crate::syntax::ast::test::*;

	#[test]
	fn test_array_type() {
		check_ast(
			r#"
			array [_] of bool: x;
			array [foo, bar] of bool: y;
		"#,
			expect!([r#"
    MznModel(
        Model {
            items: [
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "x",
                                },
                            ),
                        ),
                        declared_type: ArrayType(
                            ArrayType {
                                cst_kind: "array_type",
                                dimensions: [
                                    TypeBase(
                                        TypeBase {
                                            cst_kind: "type_base",
                                            var_type: None,
                                            opt_type: None,
                                            any_type: false,
                                            domain: Bounded(
                                                Anonymous(
                                                    Anonymous {
                                                        cst_kind: "anonymous",
                                                    },
                                                ),
                                            ),
                                        },
                                    ),
                                ],
                                element_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Unbounded(
                                            UnboundedDomain {
                                                cst_kind: "primitive_type",
                                                primitive_type: Bool,
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "y",
                                },
                            ),
                        ),
                        declared_type: ArrayType(
                            ArrayType {
                                cst_kind: "array_type",
                                dimensions: [
                                    TypeBase(
                                        TypeBase {
                                            cst_kind: "type_base",
                                            var_type: None,
                                            opt_type: None,
                                            any_type: false,
                                            domain: Bounded(
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                            name: "foo",
                                                        },
                                                    ),
                                                ),
                                            ),
                                        },
                                    ),
                                    TypeBase(
                                        TypeBase {
                                            cst_kind: "type_base",
                                            var_type: None,
                                            opt_type: None,
                                            any_type: false,
                                            domain: Bounded(
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                            name: "bar",
                                                        },
                                                    ),
                                                ),
                                            ),
                                        },
                                    ),
                                ],
                                element_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Unbounded(
                                            UnboundedDomain {
                                                cst_kind: "primitive_type",
                                                primitive_type: Bool,
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_tuple_type() {
		check_ast(
			r#"
			tuple(int, bool): x;
			tuple(int, tuple(bool, float)): y;
		"#,
			expect!([r#"
    MznModel(
        Model {
            items: [
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "x",
                                },
                            ),
                        ),
                        declared_type: TupleType(
                            TupleType {
                                cst_kind: "tuple_type",
                                fields: [
                                    TypeBase(
                                        TypeBase {
                                            cst_kind: "type_base",
                                            var_type: None,
                                            opt_type: None,
                                            any_type: false,
                                            domain: Unbounded(
                                                UnboundedDomain {
                                                    cst_kind: "primitive_type",
                                                    primitive_type: Int,
                                                },
                                            ),
                                        },
                                    ),
                                    TypeBase(
                                        TypeBase {
                                            cst_kind: "type_base",
                                            var_type: None,
                                            opt_type: None,
                                            any_type: false,
                                            domain: Unbounded(
                                                UnboundedDomain {
                                                    cst_kind: "primitive_type",
                                                    primitive_type: Bool,
                                                },
                                            ),
                                        },
                                    ),
                                ],
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "y",
                                },
                            ),
                        ),
                        declared_type: TupleType(
                            TupleType {
                                cst_kind: "tuple_type",
                                fields: [
                                    TypeBase(
                                        TypeBase {
                                            cst_kind: "type_base",
                                            var_type: None,
                                            opt_type: None,
                                            any_type: false,
                                            domain: Unbounded(
                                                UnboundedDomain {
                                                    cst_kind: "primitive_type",
                                                    primitive_type: Int,
                                                },
                                            ),
                                        },
                                    ),
                                    TupleType(
                                        TupleType {
                                            cst_kind: "tuple_type",
                                            fields: [
                                                TypeBase(
                                                    TypeBase {
                                                        cst_kind: "type_base",
                                                        var_type: None,
                                                        opt_type: None,
                                                        any_type: false,
                                                        domain: Unbounded(
                                                            UnboundedDomain {
                                                                cst_kind: "primitive_type",
                                                                primitive_type: Bool,
                                                            },
                                                        ),
                                                    },
                                                ),
                                                TypeBase(
                                                    TypeBase {
                                                        cst_kind: "type_base",
                                                        var_type: None,
                                                        opt_type: None,
                                                        any_type: false,
                                                        domain: Unbounded(
                                                            UnboundedDomain {
                                                                cst_kind: "primitive_type",
                                                                primitive_type: Float,
                                                            },
                                                        ),
                                                    },
                                                ),
                                            ],
                                        },
                                    ),
                                ],
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_record_type() {
		check_ast(
			r#"
			record(int: a, bool: b): x;
			record(int: a, record(bool: c, float: d): b): y;
		"#,
			expect!([r#"
    MznModel(
        Model {
            items: [
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "x",
                                },
                            ),
                        ),
                        declared_type: RecordType(
                            RecordType {
                                cst_kind: "record_type",
                                fields: [
                                    RecordField {
                                        cst_kind: "record_type_field",
                                        name: UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "a",
                                            },
                                        ),
                                        field_type: TypeBase(
                                            TypeBase {
                                                cst_kind: "type_base",
                                                var_type: None,
                                                opt_type: None,
                                                any_type: false,
                                                domain: Unbounded(
                                                    UnboundedDomain {
                                                        cst_kind: "primitive_type",
                                                        primitive_type: Int,
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                    RecordField {
                                        cst_kind: "record_type_field",
                                        name: UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "b",
                                            },
                                        ),
                                        field_type: TypeBase(
                                            TypeBase {
                                                cst_kind: "type_base",
                                                var_type: None,
                                                opt_type: None,
                                                any_type: false,
                                                domain: Unbounded(
                                                    UnboundedDomain {
                                                        cst_kind: "primitive_type",
                                                        primitive_type: Bool,
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "y",
                                },
                            ),
                        ),
                        declared_type: RecordType(
                            RecordType {
                                cst_kind: "record_type",
                                fields: [
                                    RecordField {
                                        cst_kind: "record_type_field",
                                        name: UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "a",
                                            },
                                        ),
                                        field_type: TypeBase(
                                            TypeBase {
                                                cst_kind: "type_base",
                                                var_type: None,
                                                opt_type: None,
                                                any_type: false,
                                                domain: Unbounded(
                                                    UnboundedDomain {
                                                        cst_kind: "primitive_type",
                                                        primitive_type: Int,
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                    RecordField {
                                        cst_kind: "record_type_field",
                                        name: UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "b",
                                            },
                                        ),
                                        field_type: RecordType(
                                            RecordType {
                                                cst_kind: "record_type",
                                                fields: [
                                                    RecordField {
                                                        cst_kind: "record_type_field",
                                                        name: UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                                name: "c",
                                                            },
                                                        ),
                                                        field_type: TypeBase(
                                                            TypeBase {
                                                                cst_kind: "type_base",
                                                                var_type: None,
                                                                opt_type: None,
                                                                any_type: false,
                                                                domain: Unbounded(
                                                                    UnboundedDomain {
                                                                        cst_kind: "primitive_type",
                                                                        primitive_type: Bool,
                                                                    },
                                                                ),
                                                            },
                                                        ),
                                                    },
                                                    RecordField {
                                                        cst_kind: "record_type_field",
                                                        name: UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                                name: "d",
                                                            },
                                                        ),
                                                        field_type: TypeBase(
                                                            TypeBase {
                                                                cst_kind: "type_base",
                                                                var_type: None,
                                                                opt_type: None,
                                                                any_type: false,
                                                                domain: Unbounded(
                                                                    UnboundedDomain {
                                                                        cst_kind: "primitive_type",
                                                                        primitive_type: Float,
                                                                    },
                                                                ),
                                                            },
                                                        ),
                                                    },
                                                ],
                                            },
                                        ),
                                    },
                                ],
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_operation_type() {
		check_ast(
			r#"
			op(int: (bool, string)): x;
		"#,
			expect!([r#"
    MznModel(
        Model {
            items: [
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "x",
                                },
                            ),
                        ),
                        declared_type: OperationType(
                            OperationType {
                                cst_kind: "operation_type",
                                return_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Unbounded(
                                            UnboundedDomain {
                                                cst_kind: "primitive_type",
                                                primitive_type: Int,
                                            },
                                        ),
                                    },
                                ),
                                parameter_types: [
                                    TypeBase(
                                        TypeBase {
                                            cst_kind: "type_base",
                                            var_type: None,
                                            opt_type: None,
                                            any_type: false,
                                            domain: Unbounded(
                                                UnboundedDomain {
                                                    cst_kind: "primitive_type",
                                                    primitive_type: Bool,
                                                },
                                            ),
                                        },
                                    ),
                                    TypeBase(
                                        TypeBase {
                                            cst_kind: "type_base",
                                            var_type: None,
                                            opt_type: None,
                                            any_type: false,
                                            domain: Unbounded(
                                                UnboundedDomain {
                                                    cst_kind: "primitive_type",
                                                    primitive_type: String,
                                                },
                                            ),
                                        },
                                    ),
                                ],
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_type_base() {
		check_ast(
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
			expect!([r#"
    MznModel(
        Model {
            items: [
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "a",
                                },
                            ),
                        ),
                        declared_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: None,
                                opt_type: None,
                                any_type: false,
                                domain: Unbounded(
                                    UnboundedDomain {
                                        cst_kind: "primitive_type",
                                        primitive_type: Int,
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "b",
                                },
                            ),
                        ),
                        declared_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: Some(
                                    Var,
                                ),
                                opt_type: None,
                                any_type: false,
                                domain: Unbounded(
                                    UnboundedDomain {
                                        cst_kind: "primitive_type",
                                        primitive_type: Bool,
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "c",
                                },
                            ),
                        ),
                        declared_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: Some(
                                    Var,
                                ),
                                opt_type: Some(
                                    Opt,
                                ),
                                any_type: false,
                                domain: Unbounded(
                                    UnboundedDomain {
                                        cst_kind: "primitive_type",
                                        primitive_type: String,
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "d",
                                },
                            ),
                        ),
                        declared_type: SetType(
                            SetType {
                                cst_kind: "set_type",
                                var_type: Var,
                                opt_type: NonOpt,
                                element_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Bounded(
                                            InfixOperator(
                                                InfixOperator {
                                                    cst_kind: "infix_operator",
                                                    left: IntegerLiteral(
                                                        IntegerLiteral {
                                                            cst_kind: "integer_literal",
                                                            value: Ok(
                                                                1,
                                                            ),
                                                        },
                                                    ),
                                                    operator: Operator {
                                                        cst_kind: "..",
                                                        name: "..",
                                                    },
                                                    right: IntegerLiteral(
                                                        IntegerLiteral {
                                                            cst_kind: "integer_literal",
                                                            value: Ok(
                                                                3,
                                                            ),
                                                        },
                                                    ),
                                                },
                                            ),
                                        ),
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "e",
                                },
                            ),
                        ),
                        declared_type: SetType(
                            SetType {
                                cst_kind: "set_type",
                                var_type: Par,
                                opt_type: Opt,
                                element_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Bounded(
                                            Identifier(
                                                UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                        name: "Foo",
                                                    },
                                                ),
                                            ),
                                        ),
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "f",
                                },
                            ),
                        ),
                        declared_type: AnyType(
                            AnyType {
                                cst_kind: "any_type",
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "g",
                                },
                            ),
                        ),
                        declared_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: None,
                                opt_type: None,
                                any_type: false,
                                domain: TypeInstIdentifier(
                                    TypeInstIdentifier {
                                        cst_kind: "type_inst_id",
                                        name: "$T",
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "h",
                                },
                            ),
                        ),
                        declared_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: None,
                                opt_type: Some(
                                    Opt,
                                ),
                                any_type: false,
                                domain: TypeInstIdentifier(
                                    TypeInstIdentifier {
                                        cst_kind: "type_inst_id",
                                        name: "$T",
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "i",
                                },
                            ),
                        ),
                        declared_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: Some(
                                    Var,
                                ),
                                opt_type: None,
                                any_type: false,
                                domain: TypeInstEnumIdentifier(
                                    TypeInstEnumIdentifier {
                                        cst_kind: "type_inst_enum_id",
                                        name: "$$E",
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}
}
