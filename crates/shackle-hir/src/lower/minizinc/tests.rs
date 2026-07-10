use expect_test::expect;

use crate::lower::tests::check_lower_item;

#[test]
fn test_lower_assignment() {
	check_lower_item(
		"x = 1;",
		expect!([r#"
    ItemWithData {
        item: Assignment {
            assignee: <Expression::1>,
            definition: <Expression::2>,
        },
        data: ItemData {
            expressions: Arena {
                len: 2,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                    <Expression::2>: IntegerLiteral(
                        1,
                    ),
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 0,
                data: {},
            },
            annotations: {},
        },
    }
"#]),
	);
}

#[test]
fn test_lower_constraint() {
	check_lower_item(
		"constraint x = 1;",
		expect!([r#"
    ItemWithData {
        item: Constraint {
            expression: <Expression::4>,
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 4,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                    <Expression::2>: IntegerLiteral(
                        1,
                    ),
                    <Expression::3>: Identifier(
                        "=",
                    ),
                    <Expression::4>: Call {
                        kind: Operator,
                        function: <Expression::3>,
                        arguments: [
                            <Expression::1>,
                            <Expression::2>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 0,
                data: {},
            },
            annotations: {},
        },
    }
"#]),
	);
}

#[test]
fn test_lower_declaration() {
	check_lower_item(
		"var int: x;",
		expect!([r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: None,
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 0,
                data: {},
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Primitive {
                        inst: Var,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "x",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"tuple(int, int): (x, y) = (1, 2);",
		expect!([r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::3>,
            pattern: <Pattern::3>,
            definition: Some(
                <Expression::3>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 3,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        2,
                    ),
                    <Expression::3>: TupleLiteral {
                        fields: [
                            <Expression::1>,
                            <Expression::2>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 3,
                data: {
                    <Type::1>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                    <Type::2>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                    <Type::3>: Tuple {
                        opt: NonOpt,
                        fields: [
                            <Type::1>,
                            <Type::2>,
                        ],
                    },
                },
            },
            patterns: Arena {
                len: 3,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "x",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "y",
                        ),
                    ),
                    <Pattern::3>: Tuple {
                        fields: [
                            <Pattern::1>,
                            <Pattern::2>,
                        ],
                    },
                },
            },
            annotations: {},
        },
    }
"#]),
	);
}

#[test]
fn test_lower_annotation() {
	check_lower_item(
		"annotation foo;",
		expect!([r#"
    ItemWithData {
        item: Annotation {
            constructor: Atom {
                pattern: <Pattern::1>,
            },
        },
        data: ItemData {
            expressions: Arena {
                len: 0,
                data: {},
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "foo",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"annotation foo(int, float);",
		expect!([r#"
    ItemWithData {
        item: Annotation {
            constructor: Function {
                constructor: <Pattern::1>,
                destructor: <Pattern::2>,
                parameters: [
                    ConstructorParameter {
                        declared_type: <Type::1>,
                        pattern: None,
                    },
                    ConstructorParameter {
                        declared_type: <Type::2>,
                        pattern: None,
                    },
                ],
            },
        },
        data: ItemData {
            expressions: Arena {
                len: 0,
                data: {},
            },
            types: Arena {
                len: 2,
                data: {
                    <Type::1>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                    <Type::2>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Float,
                    },
                },
            },
            patterns: Arena {
                len: 2,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "foo",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "foo⁻¹",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
}

#[test]
fn test_lower_enumeration() {
	check_lower_item(
		"enum Foo;",
		expect!([r#"
    ItemWithData {
        item: Enumeration {
            pattern: <Pattern::1>,
            definition: None,
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 0,
                data: {},
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "Foo",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"enum Foo = {A, B, C};",
		expect!([r#"
    ItemWithData {
        item: Enumeration {
            pattern: <Pattern::1>,
            definition: Some(
                [
                    Named(
                        Atom {
                            pattern: <Pattern::2>,
                        },
                    ),
                    Named(
                        Atom {
                            pattern: <Pattern::3>,
                        },
                    ),
                    Named(
                        Atom {
                            pattern: <Pattern::4>,
                        },
                    ),
                ],
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 0,
                data: {},
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 4,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "Foo",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "A",
                        ),
                    ),
                    <Pattern::3>: Identifier(
                        Identifier(
                            "B",
                        ),
                    ),
                    <Pattern::4>: Identifier(
                        Identifier(
                            "C",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"enum Foo = A(B) ++ {C};",
		expect!([r#"
    ItemWithData {
        item: Enumeration {
            pattern: <Pattern::1>,
            definition: Some(
                [
                    Named(
                        Function {
                            constructor: <Pattern::2>,
                            destructor: <Pattern::3>,
                            parameters: [
                                ConstructorParameter {
                                    declared_type: <Type::1>,
                                    pattern: None,
                                },
                            ],
                        },
                    ),
                    Named(
                        Atom {
                            pattern: <Pattern::4>,
                        },
                    ),
                ],
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: Identifier(
                        "B",
                    ),
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Bounded {
                        inst: None,
                        opt: None,
                        domain: <Expression::1>,
                    },
                },
            },
            patterns: Arena {
                len: 4,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "Foo",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "A",
                        ),
                    ),
                    <Pattern::3>: Identifier(
                        Identifier(
                            "A⁻¹",
                        ),
                    ),
                    <Pattern::4>: Identifier(
                        Identifier(
                            "C",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		r#"
            enum Foo;
            Foo = {A, B, C};
        "#,
		expect!([r#"
    ItemWithData {
        item: EnumAssignment {
            assignee: <Expression::1>,
            definition: [
                Named(
                    Atom {
                        pattern: <Pattern::3>,
                    },
                ),
                Named(
                    Atom {
                        pattern: <Pattern::2>,
                    },
                ),
                Named(
                    Atom {
                        pattern: <Pattern::1>,
                    },
                ),
            ],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: Identifier(
                        "Foo",
                    ),
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 3,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "C",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "B",
                        ),
                    ),
                    <Pattern::3>: Identifier(
                        Identifier(
                            "A",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		r#"
            enum Foo;
            Foo = A(B) ++ {C};
        "#,
		expect!([r#"
    ItemWithData {
        item: EnumAssignment {
            assignee: <Expression::1>,
            definition: [
                Named(
                    Function {
                        constructor: <Pattern::2>,
                        destructor: <Pattern::3>,
                        parameters: [
                            ConstructorParameter {
                                declared_type: <Type::1>,
                                pattern: None,
                            },
                        ],
                    },
                ),
                Named(
                    Atom {
                        pattern: <Pattern::1>,
                    },
                ),
            ],
        },
        data: ItemData {
            expressions: Arena {
                len: 2,
                data: {
                    <Expression::1>: Identifier(
                        "Foo",
                    ),
                    <Expression::2>: Identifier(
                        "B",
                    ),
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Bounded {
                        inst: None,
                        opt: None,
                        domain: <Expression::2>,
                    },
                },
            },
            patterns: Arena {
                len: 3,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "C",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "A",
                        ),
                    ),
                    <Pattern::3>: Identifier(
                        Identifier(
                            "A⁻¹",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
}

#[test]
fn test_lower_function() {
	check_lower_item(
		"function var int: foo(int: x, var bool: y) = if y then x else 0 endif;",
		expect!([r#"
    ItemWithData {
        item: Function {
            return_type: <Type::1>,
            pattern: <Pattern::1>,
            type_inst_vars: [],
            parameters: [
                Parameter {
                    declared_type: <Type::2>,
                    pattern: Some(
                        <Pattern::2>,
                    ),
                    annotations: [],
                },
                Parameter {
                    declared_type: <Type::3>,
                    pattern: Some(
                        <Pattern::3>,
                    ),
                    annotations: [],
                },
            ],
            body: Some(
                <Expression::4>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 4,
                data: {
                    <Expression::1>: Identifier(
                        "y",
                    ),
                    <Expression::2>: Identifier(
                        "x",
                    ),
                    <Expression::3>: IntegerLiteral(
                        0,
                    ),
                    <Expression::4>: IfThenElse {
                        branches: [
                            Branch {
                                condition: <Expression::1>,
                                result: <Expression::2>,
                            },
                        ],
                        else_result: Some(
                            <Expression::3>,
                        ),
                    },
                },
            },
            types: Arena {
                len: 3,
                data: {
                    <Type::1>: Primitive {
                        inst: Var,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                    <Type::2>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                    <Type::3>: Primitive {
                        inst: Var,
                        opt: NonOpt,
                        primitive_type: Bool,
                    },
                },
            },
            patterns: Arena {
                len: 3,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "foo",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "x",
                        ),
                    ),
                    <Pattern::3>: Identifier(
                        Identifier(
                            "y",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"function int: foo(tuple(int, int): (x, y)) = x + y;",
		expect!([r#"
    ItemWithData {
        item: Function {
            return_type: <Type::1>,
            pattern: <Pattern::1>,
            type_inst_vars: [],
            parameters: [
                Parameter {
                    declared_type: <Type::4>,
                    pattern: Some(
                        <Pattern::4>,
                    ),
                    annotations: [],
                },
            ],
            body: Some(
                <Expression::4>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 4,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                    <Expression::2>: Identifier(
                        "y",
                    ),
                    <Expression::3>: Identifier(
                        "+",
                    ),
                    <Expression::4>: Call {
                        kind: Operator,
                        function: <Expression::3>,
                        arguments: [
                            <Expression::1>,
                            <Expression::2>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 4,
                data: {
                    <Type::1>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                    <Type::2>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                    <Type::3>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                    <Type::4>: Tuple {
                        opt: NonOpt,
                        fields: [
                            <Type::2>,
                            <Type::3>,
                        ],
                    },
                },
            },
            patterns: Arena {
                len: 4,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "foo",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "x",
                        ),
                    ),
                    <Pattern::3>: Identifier(
                        Identifier(
                            "y",
                        ),
                    ),
                    <Pattern::4>: Tuple {
                        fields: [
                            <Pattern::2>,
                            <Pattern::3>,
                        ],
                    },
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"predicate foo(int) = true;",
		expect!([r#"
    ItemWithData {
        item: Function {
            return_type: <Type::1>,
            pattern: <Pattern::1>,
            type_inst_vars: [],
            parameters: [
                Parameter {
                    declared_type: <Type::2>,
                    pattern: None,
                    annotations: [],
                },
            ],
            body: Some(
                <Expression::1>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: BooleanLiteral(
                        true,
                    ),
                },
            },
            types: Arena {
                len: 2,
                data: {
                    <Type::1>: Primitive {
                        inst: Var,
                        opt: NonOpt,
                        primitive_type: Bool,
                    },
                    <Type::2>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "foo",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"test foo(int) = false;",
		expect!([r#"
    ItemWithData {
        item: Function {
            return_type: <Type::1>,
            pattern: <Pattern::1>,
            type_inst_vars: [],
            parameters: [
                Parameter {
                    declared_type: <Type::2>,
                    pattern: None,
                    annotations: [],
                },
            ],
            body: Some(
                <Expression::1>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: BooleanLiteral(
                        false,
                    ),
                },
            },
            types: Arena {
                len: 2,
                data: {
                    <Type::1>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Bool,
                    },
                    <Type::2>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "foo",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"function var $$E: foo($T: x, $$E: y);",
		expect!([r#"
    ItemWithData {
        item: Function {
            return_type: <Type::1>,
            pattern: <Pattern::1>,
            type_inst_vars: [
                TypeInstIdentifierDeclaration {
                    name: <Pattern::2>,
                    anonymous: false,
                    is_enum: true,
                    is_varifiable: true,
                    is_indexable: false,
                },
                TypeInstIdentifierDeclaration {
                    name: <Pattern::3>,
                    anonymous: false,
                    is_enum: false,
                    is_varifiable: false,
                    is_indexable: false,
                },
            ],
            parameters: [
                Parameter {
                    declared_type: <Type::2>,
                    pattern: Some(
                        <Pattern::4>,
                    ),
                    annotations: [],
                },
                Parameter {
                    declared_type: <Type::3>,
                    pattern: Some(
                        <Pattern::6>,
                    ),
                    annotations: [],
                },
            ],
            body: None,
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 3,
                data: {
                    <Expression::1>: Identifier(
                        "$$E",
                    ),
                    <Expression::2>: Identifier(
                        "$T",
                    ),
                    <Expression::3>: Identifier(
                        "$$E",
                    ),
                },
            },
            types: Arena {
                len: 3,
                data: {
                    <Type::1>: Bounded {
                        inst: Some(
                            Var,
                        ),
                        opt: Some(
                            NonOpt,
                        ),
                        domain: <Expression::1>,
                    },
                    <Type::2>: Bounded {
                        inst: Some(
                            Par,
                        ),
                        opt: Some(
                            NonOpt,
                        ),
                        domain: <Expression::2>,
                    },
                    <Type::3>: Bounded {
                        inst: Some(
                            Par,
                        ),
                        opt: Some(
                            NonOpt,
                        ),
                        domain: <Expression::3>,
                    },
                },
            },
            patterns: Arena {
                len: 6,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "foo",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "$$E",
                        ),
                    ),
                    <Pattern::3>: Identifier(
                        Identifier(
                            "$T",
                        ),
                    ),
                    <Pattern::4>: Identifier(
                        Identifier(
                            "x",
                        ),
                    ),
                    <Pattern::5>: Identifier(
                        Identifier(
                            "$$E",
                        ),
                    ),
                    <Pattern::6>: Identifier(
                        Identifier(
                            "y",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
}

#[test]
fn test_lower_output() {
	check_lower_item(
		r#"
        output ["foo"];
    "#,
		expect!([r#"
    ItemWithData {
        item: Output {
            section: None,
            expression: <Expression::2>,
        },
        data: ItemData {
            expressions: Arena {
                len: 2,
                data: {
                    <Expression::1>: StringLiteral(
                        InternedString {
                            value: "foo",
                        },
                    ),
                    <Expression::2>: ArrayLiteral {
                        members: [
                            <Expression::1>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 0,
                data: {},
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		r#"
        output :: "foo" [x, y];
    "#,
		expect!([r#"
    ItemWithData {
        item: Output {
            section: Some(
                <Expression::1>,
            ),
            expression: <Expression::4>,
        },
        data: ItemData {
            expressions: Arena {
                len: 4,
                data: {
                    <Expression::1>: StringLiteral(
                        InternedString {
                            value: "foo",
                        },
                    ),
                    <Expression::2>: Identifier(
                        "x",
                    ),
                    <Expression::3>: Identifier(
                        "y",
                    ),
                    <Expression::4>: ArrayLiteral {
                        members: [
                            <Expression::2>,
                            <Expression::3>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 0,
                data: {},
            },
            annotations: {},
        },
    }
"#]),
	);
}

#[test]
fn test_lower_solve() {
	check_lower_item(
		"solve satisfy;",
		expect!([r#"
    ItemWithData {
        item: Solve {
            goal: Satisfy,
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 0,
                data: {},
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 0,
                data: {},
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"solve :: int_search([x], input_order, indomain_min) satisfy;",
		expect!([r#"
    ItemWithData {
        item: Solve {
            goal: Satisfy,
            annotations: [
                <Expression::6>,
            ],
        },
        data: ItemData {
            expressions: Arena {
                len: 6,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                    <Expression::2>: ArrayLiteral {
                        members: [
                            <Expression::1>,
                        ],
                    },
                    <Expression::3>: Identifier(
                        "input_order",
                    ),
                    <Expression::4>: Identifier(
                        "indomain_min",
                    ),
                    <Expression::5>: Identifier(
                        "int_search",
                    ),
                    <Expression::6>: Call {
                        kind: SourceCall,
                        function: <Expression::5>,
                        arguments: [
                            <Expression::2>,
                            <Expression::3>,
                            <Expression::4>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 0,
                data: {},
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"solve minimize x;",
		expect!([r#"
    ItemWithData {
        item: Solve {
            goal: Minimize {
                pattern: <Pattern::1>,
                objective: <Expression::1>,
            },
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "_objective",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"solve :: int_search([x], input_order, indomain_min) minimize x;",
		expect!([r#"
    ItemWithData {
        item: Solve {
            goal: Minimize {
                pattern: <Pattern::1>,
                objective: <Expression::7>,
            },
            annotations: [
                <Expression::6>,
            ],
        },
        data: ItemData {
            expressions: Arena {
                len: 7,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                    <Expression::2>: ArrayLiteral {
                        members: [
                            <Expression::1>,
                        ],
                    },
                    <Expression::3>: Identifier(
                        "input_order",
                    ),
                    <Expression::4>: Identifier(
                        "indomain_min",
                    ),
                    <Expression::5>: Identifier(
                        "int_search",
                    ),
                    <Expression::6>: Call {
                        kind: SourceCall,
                        function: <Expression::5>,
                        arguments: [
                            <Expression::2>,
                            <Expression::3>,
                            <Expression::4>,
                        ],
                    },
                    <Expression::7>: Identifier(
                        "x",
                    ),
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "_objective",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"solve maximize x;",
		expect!([r#"
    ItemWithData {
        item: Solve {
            goal: Maximize {
                pattern: <Pattern::1>,
                objective: <Expression::1>,
            },
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "_objective",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"solve :: int_search([x], input_order, indomain_min) maximize x;",
		expect!([r#"
    ItemWithData {
        item: Solve {
            goal: Maximize {
                pattern: <Pattern::1>,
                objective: <Expression::7>,
            },
            annotations: [
                <Expression::6>,
            ],
        },
        data: ItemData {
            expressions: Arena {
                len: 7,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                    <Expression::2>: ArrayLiteral {
                        members: [
                            <Expression::1>,
                        ],
                    },
                    <Expression::3>: Identifier(
                        "input_order",
                    ),
                    <Expression::4>: Identifier(
                        "indomain_min",
                    ),
                    <Expression::5>: Identifier(
                        "int_search",
                    ),
                    <Expression::6>: Call {
                        kind: SourceCall,
                        function: <Expression::5>,
                        arguments: [
                            <Expression::2>,
                            <Expression::3>,
                            <Expression::4>,
                        ],
                    },
                    <Expression::7>: Identifier(
                        "x",
                    ),
                },
            },
            types: Arena {
                len: 0,
                data: {},
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "_objective",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
}

#[test]
fn test_lower_type_alias() {
	check_lower_item(
		"type Foo = set of int;",
		expect!([r#"
    ItemWithData {
        item: TypeAlias {
            name: <Pattern::1>,
            aliased_type: <Type::2>,
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 0,
                data: {},
            },
            types: Arena {
                len: 2,
                data: {
                    <Type::1>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                    <Type::2>: Set {
                        inst: Par,
                        opt: NonOpt,
                        element: <Type::1>,
                    },
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "Foo",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item(
		"type Foo = var 1..3;",
		expect!([r#"
    ItemWithData {
        item: TypeAlias {
            name: <Pattern::1>,
            aliased_type: <Type::1>,
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 4,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        3,
                    ),
                    <Expression::3>: Identifier(
                        "..",
                    ),
                    <Expression::4>: Call {
                        kind: Operator,
                        function: <Expression::3>,
                        arguments: [
                            <Expression::1>,
                            <Expression::2>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Bounded {
                        inst: Some(
                            Var,
                        ),
                        opt: None,
                        domain: <Expression::4>,
                    },
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "Foo",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
}
