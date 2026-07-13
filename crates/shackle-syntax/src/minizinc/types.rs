//! AST representation for types

use super::{Children, Expression, Identifier};
use crate::ast::{
	AstNode, ast_enum, ast_node, child_with_field_name, children_with_field_name,
	optional_child_with_field_name,
};

ast_enum!(
	/// Type from a declaration
	Type,
	"array_type" => ArrayType,
	"list_type" => ListType,
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

impl<'tree> ArrayType<'tree> {
	/// The ranges of the array.
	pub fn dimensions(&self) -> Children<'tree, Type<'tree>> {
		children_with_field_name(self, "dimension")
	}

	/// The type contained in the array
	pub fn element_type(&self) -> Type<'tree> {
		child_with_field_name(self, "type")
	}
}

ast_node!(
	/// Type of a list
	ListType,
	element_type
);

impl<'tree> ListType<'tree> {
	/// The type contained in the array
	pub fn element_type(&self) -> Type<'tree> {
		child_with_field_name(self, "type")
	}
}

ast_node!(
	/// Type of a set
	SetType,
	var_type,
	opt_type,
	cardinality,
	element_type
);

impl<'tree> SetType<'tree> {
	/// Get whether this type is var or par
	pub fn var_type(&self) -> VarType {
		self.cst_node()
			.optional_child_with_field_name("var_par")
			.map(|c| match c.kind() {
				"var" => VarType::Var,
				"par" => VarType::Par,
				_ => unreachable!(),
			})
			.unwrap_or(VarType::Par)
	}

	/// Get optionality of type
	pub fn opt_type(&self) -> OptType {
		self.cst_node()
			.optional_child_with_field_name("opt")
			.map(|c| match c.kind() {
				"opt" => OptType::Opt,
				"nonopt" => OptType::NonOpt,
				_ => unreachable!(),
			})
			.unwrap_or(OptType::NonOpt)
	}

	/// Get the declared cardinality of the set
	pub fn cardinality(&self) -> Option<Expression<'tree>> {
		optional_child_with_field_name(self, "cardinality")
	}

	/// The type contained in the array
	pub fn element_type(&self) -> Type<'tree> {
		child_with_field_name(self, "type")
	}
}

ast_node!(
	/// Type of a tuple
	TupleType,
	var_type,
	fields
);

impl<'tree> TupleType<'tree> {
	/// Get whether this type is var or par
	pub fn var_type(&self) -> VarType {
		self.cst_node()
			.optional_child_with_field_name("var_par")
			.map(|c| match c.kind() {
				"var" => VarType::Var,
				"par" => VarType::Par,
				_ => unreachable!(),
			})
			.unwrap_or(VarType::Par)
	}

	/// The types of the tuple fields
	pub fn fields(&self) -> Children<'tree, Type<'tree>> {
		children_with_field_name(self, "field")
	}
}

ast_node!(
	/// Type of a record
	RecordType,
	var_type,
	fields
);

impl<'tree> RecordType<'tree> {
	/// Get whether this type is var or par
	pub fn var_type(&self) -> VarType {
		self.cst_node()
			.optional_child_with_field_name("var_par")
			.map(|c| match c.kind() {
				"var" => VarType::Var,
				"par" => VarType::Par,
				_ => unreachable!(),
			})
			.unwrap_or(VarType::Par)
	}

	/// The types of the tuple fields
	pub fn fields(&self) -> Children<'tree, RecordField<'tree>> {
		children_with_field_name(self, "field")
	}
}

ast_node!(
	/// Field in a record type
	RecordField,
	name,
	field_type
);

impl<'tree> RecordField<'tree> {
	/// The name of the field
	pub fn name(&self) -> Identifier<'tree> {
		child_with_field_name(self, "name")
	}

	/// The type of the field
	pub fn field_type(&self) -> Type<'tree> {
		child_with_field_name(self, "type")
	}
}

ast_node!(
	/// Type of an operation (function)
	OperationType,
	return_type,
	parameter_types,
);

impl<'tree> OperationType<'tree> {
	/// Function return type
	pub fn return_type(&self) -> Type<'tree> {
		child_with_field_name(self, "return_type")
	}

	/// Function parameter types
	pub fn parameter_types(&self) -> Children<'tree, Type<'tree>> {
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

impl<'tree> TypeBase<'tree> {
	/// Get whether this type is var or par.
	///
	/// Gives `None` when omitted rather than `Par` since omitting the inst
	/// when referring to a type-inst alias does not make it par.
	pub fn var_type(&self) -> Option<VarType> {
		self.cst_node()
			.optional_child_with_field_name("var_par")
			.map(|c| match c.kind() {
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
		self.cst_node()
			.optional_child_with_field_name("opt")
			.map(|c| match c.kind() {
				"opt" => OptType::Opt,
				"nonopt" => OptType::NonOpt,
				_ => unreachable!(),
			})
	}

	/// Get whether this type is any $T
	pub fn any_type(&self) -> bool {
		self.cst_node()
			.optional_child_with_field_name("any")
			.is_some()
	}

	/// Get the domain of this type (can be `None` if type is `any`)
	pub fn domain(&self) -> Domain<'tree> {
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
	"new_type" => NewType,
	_ => Bounded(Expression)
);

ast_node!(
	/// Domain which introduces new objects of a class
	NewType,
	name
);

impl<'tree> NewType<'tree> {
	/// Name of the class whose objects are introduced
	pub fn name(&self) -> Identifier<'tree> {
		child_with_field_name(self, "type")
	}
}

ast_node!(
	/// Unbounded primitive type domain
	UnboundedDomain,
	primitive_type
);

impl<'tree> UnboundedDomain<'tree> {
	/// Get the primitive type of this domain
	pub fn primitive_type(&self) -> PrimitiveType {
		match self.cst_node().child(0).unwrap().kind() {
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
	TypeInstIdentifier
);

impl<'tree> TypeInstIdentifier<'tree> {
	/// Name of identifier
	pub fn name<'a>(&self, source: &'a str) -> &'a str {
		self.cst_text(source)
	}
}

ast_node!(
	/// Type-inst enum identifier `$$E`
	TypeInstEnumIdentifier
);

impl<'tree> TypeInstEnumIdentifier<'tree> {
	/// Name of identifier
	pub fn name<'a>(&self, source: &'a str) -> &'a str {
		self.cst_text(source)
	}
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::ast::tests::*;

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
                                },
                            ),
                        ),
                        declared_type: TupleType(
                            TupleType {
                                cst_kind: "tuple_type",
                                var_type: Par,
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
                                },
                            ),
                        ),
                        declared_type: TupleType(
                            TupleType {
                                cst_kind: "tuple_type",
                                var_type: Par,
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
                                            var_type: Par,
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
                                },
                            ),
                        ),
                        declared_type: RecordType(
                            RecordType {
                                cst_kind: "record_type",
                                var_type: Par,
                                fields: [
                                    RecordField {
                                        cst_kind: "record_type_field",
                                        name: UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
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
                                },
                            ),
                        ),
                        declared_type: RecordType(
                            RecordType {
                                cst_kind: "record_type",
                                var_type: Par,
                                fields: [
                                    RecordField {
                                        cst_kind: "record_type_field",
                                        name: UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
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
                                            },
                                        ),
                                        field_type: RecordType(
                                            RecordType {
                                                cst_kind: "record_type",
                                                var_type: Par,
                                                fields: [
                                                    RecordField {
                                                        cst_kind: "record_type_field",
                                                        name: UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
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
                                },
                            ),
                        ),
                        declared_type: SetType(
                            SetType {
                                cst_kind: "set_type",
                                var_type: Var,
                                opt_type: NonOpt,
                                cardinality: None,
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
                                                        },
                                                    ),
                                                    operator: Operator {
                                                        cst_kind: "..",
                                                        name: "..",
                                                    },
                                                    right: IntegerLiteral(
                                                        IntegerLiteral {
                                                            cst_kind: "integer_literal",
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
                                },
                            ),
                        ),
                        declared_type: SetType(
                            SetType {
                                cst_kind: "set_type",
                                var_type: Par,
                                opt_type: Opt,
                                cardinality: None,
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
