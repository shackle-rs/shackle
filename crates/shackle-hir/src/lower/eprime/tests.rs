use expect_test::expect;

use crate::lower::tests::check_lower_item_eprime;

#[test]
fn test_lower_integer_domain() {
	check_lower_item_eprime(
		"find i: int",
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
                            "i",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	);
	check_lower_item_eprime(
		"find i: int(1, 3..10)",
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
                len: 8,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        3,
                    ),
                    <Expression::3>: IntegerLiteral(
                        10,
                    ),
                    <Expression::4>: Identifier(
                        "..",
                    ),
                    <Expression::5>: Call {
                        kind: Operator,
                        function: <Expression::4>,
                        arguments: [
                            <Expression::2>,
                            <Expression::3>,
                        ],
                    },
                    <Expression::6>: SetLiteral {
                        members: [
                            <Expression::1>,
                        ],
                    },
                    <Expression::7>: Identifier(
                        "union",
                    ),
                    <Expression::8>: Call {
                        kind: Synthetic,
                        function: <Expression::7>,
                        arguments: [
                            <Expression::6>,
                            <Expression::5>,
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
                        domain: <Expression::8>,
                    },
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "i",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]),
	)
}

#[test]
fn test_lower_boolean_domain() {
	check_lower_item_eprime(
		r#"
          find x: bool
      "#,
		expect![[r#"
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
                        primitive_type: Bool,
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
"#]],
	);
}

#[test]
fn test_domain_expressions() {
	check_lower_item_eprime(
		"letting x be domain int(1) intersect int(1)",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::2>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::12>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 12,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: SetLiteral {
                        members: [
                            <Expression::1>,
                        ],
                    },
                    <Expression::3>: IntegerLiteral(
                        1,
                    ),
                    <Expression::4>: SetLiteral {
                        members: [
                            <Expression::3>,
                        ],
                    },
                    <Expression::5>: Identifier(
                        "intersect",
                    ),
                    <Expression::6>: Call {
                        kind: Operator,
                        function: <Expression::5>,
                        arguments: [
                            <Expression::2>,
                            <Expression::4>,
                        ],
                    },
                    <Expression::7>: IntegerLiteral(
                        1,
                    ),
                    <Expression::8>: SetLiteral {
                        members: [
                            <Expression::7>,
                        ],
                    },
                    <Expression::9>: IntegerLiteral(
                        1,
                    ),
                    <Expression::10>: SetLiteral {
                        members: [
                            <Expression::9>,
                        ],
                    },
                    <Expression::11>: Identifier(
                        "intersect",
                    ),
                    <Expression::12>: Call {
                        kind: Operator,
                        function: <Expression::11>,
                        arguments: [
                            <Expression::8>,
                            <Expression::10>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 2,
                data: {
                    <Type::1>: Bounded {
                        inst: Some(
                            Par,
                        ),
                        opt: None,
                        domain: <Expression::6>,
                    },
                    <Type::2>: Set {
                        inst: Par,
                        opt: NonOpt,
                        cardinality: None,
                        element: <Type::1>,
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
"#]],
	);
}

#[test]
fn test_lower_matrix_domain() {
	check_lower_item_eprime(
		"given simple: matrix indexed by [int(1..4)] of bool",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::3>,
            pattern: <Pattern::1>,
            definition: None,
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
                        4,
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
                len: 3,
                data: {
                    <Type::1>: Bounded {
                        inst: Some(
                            Par,
                        ),
                        opt: None,
                        domain: <Expression::4>,
                    },
                    <Type::2>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Bool,
                    },
                    <Type::3>: Array {
                        opt: NonOpt,
                        dimensions: <Type::1>,
                        element: <Type::2>,
                    },
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "simple",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
}

#[test]
fn test_lower_call() {
	check_lower_item_eprime(
		"letting simple = toVec(X,Y)",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::4>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 4,
                data: {
                    <Expression::1>: Identifier(
                        "X",
                    ),
                    <Expression::2>: Identifier(
                        "Y",
                    ),
                    <Expression::3>: Identifier(
                        "toVec",
                    ),
                    <Expression::4>: Call {
                        kind: SourceCall,
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
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "simple",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
}

#[test]
fn test_lower_indexed_access() {
	check_lower_item_eprime(
		r#"
      letting multi = M[2..4]
      "#,
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::6>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 6,
                data: {
                    <Expression::1>: IntegerLiteral(
                        2,
                    ),
                    <Expression::2>: IntegerLiteral(
                        4,
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
                    <Expression::5>: Identifier(
                        "M",
                    ),
                    <Expression::6>: ArrayAccess {
                        collection: <Expression::5>,
                        indices: <Expression::4>,
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "multi",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
}

#[test]
fn test_lower_infix_operator() {
	check_lower_item_eprime(
		r#"
      letting and = x /\ y
      "#,
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
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
                        "/\\",
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
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "and",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
}

#[test]
fn test_lower_prefix_operator() {
	check_lower_item_eprime(
		r#"
      letting negated_bool = !true
      "#,
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::3>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 3,
                data: {
                    <Expression::1>: BooleanLiteral(
                        true,
                    ),
                    <Expression::2>: Identifier(
                        "not",
                    ),
                    <Expression::3>: Call {
                        kind: Operator,
                        function: <Expression::2>,
                        arguments: [
                            <Expression::1>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "negated_bool",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
}

#[test]
fn test_lower_quantification() {
	check_lower_item_eprime(
		"letting simple_sum = sum i : int(1..2) . i",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::8>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 8,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        2,
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
                    <Expression::5>: Identifier(
                        "i",
                    ),
                    <Expression::6>: ArrayComprehension {
                        template: <Expression::5>,
                        indices: None,
                        generators: [
                            Iterator {
                                patterns: [
                                    <Pattern::2>,
                                ],
                                collection: <Expression::4>,
                                where_clause: None,
                            },
                        ],
                    },
                    <Expression::7>: Identifier(
                        "sum",
                    ),
                    <Expression::8>: Call {
                        kind: GeneratorCall,
                        function: <Expression::7>,
                        arguments: [
                            <Expression::6>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 2,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "simple_sum",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "i",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
}

#[test]
fn test_lower_matrix_comprehension() {
	check_lower_item_eprime(
		"letting simple = [ num**2 | num : int(1..5) ]",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::10>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 10,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        5,
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
                    <Expression::5>: Identifier(
                        "num",
                    ),
                    <Expression::6>: Identifier(
                        "num",
                    ),
                    <Expression::7>: IntegerLiteral(
                        2,
                    ),
                    <Expression::8>: Identifier(
                        "**",
                    ),
                    <Expression::9>: Call {
                        kind: Operator,
                        function: <Expression::8>,
                        arguments: [
                            <Expression::6>,
                            <Expression::7>,
                        ],
                    },
                    <Expression::10>: ArrayComprehension {
                        template: <Expression::9>,
                        indices: None,
                        generators: [
                            Iterator {
                                patterns: [
                                    <Pattern::2>,
                                ],
                                collection: <Expression::4>,
                                where_clause: None,
                            },
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 2,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "simple",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "num",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
	check_lower_item_eprime(
		"letting multi = [ [i, i+1] | i : int(1..2) ]",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::12>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 12,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        2,
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
                    <Expression::5>: Identifier(
                        "i",
                    ),
                    <Expression::6>: Identifier(
                        "i",
                    ),
                    <Expression::7>: Identifier(
                        "i",
                    ),
                    <Expression::8>: IntegerLiteral(
                        1,
                    ),
                    <Expression::9>: Identifier(
                        "+",
                    ),
                    <Expression::10>: Call {
                        kind: Operator,
                        function: <Expression::9>,
                        arguments: [
                            <Expression::7>,
                            <Expression::8>,
                        ],
                    },
                    <Expression::11>: TupleLiteral {
                        fields: [
                            <Expression::6>,
                            <Expression::10>,
                        ],
                    },
                    <Expression::12>: ArrayComprehension {
                        template: <Expression::11>,
                        indices: None,
                        generators: [
                            Iterator {
                                patterns: [
                                    <Pattern::2>,
                                ],
                                collection: <Expression::4>,
                                where_clause: None,
                            },
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 2,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "multi",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "i",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
	check_lower_item_eprime(
		"letting multi = [ [i+j | j : int(1..2)] | i : int(1..2) ]",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::16>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 16,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        2,
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
                    <Expression::5>: Identifier(
                        "i",
                    ),
                    <Expression::6>: IntegerLiteral(
                        1,
                    ),
                    <Expression::7>: IntegerLiteral(
                        2,
                    ),
                    <Expression::8>: Identifier(
                        "..",
                    ),
                    <Expression::9>: Call {
                        kind: Operator,
                        function: <Expression::8>,
                        arguments: [
                            <Expression::6>,
                            <Expression::7>,
                        ],
                    },
                    <Expression::10>: Identifier(
                        "j",
                    ),
                    <Expression::11>: Identifier(
                        "i",
                    ),
                    <Expression::12>: Identifier(
                        "j",
                    ),
                    <Expression::13>: Identifier(
                        "+",
                    ),
                    <Expression::14>: Call {
                        kind: Operator,
                        function: <Expression::13>,
                        arguments: [
                            <Expression::11>,
                            <Expression::12>,
                        ],
                    },
                    <Expression::15>: TupleLiteral {
                        fields: [
                            <Expression::5>,
                            <Expression::10>,
                        ],
                    },
                    <Expression::16>: ArrayComprehension {
                        template: <Expression::14>,
                        indices: Some(
                            <Expression::15>,
                        ),
                        generators: [
                            Iterator {
                                patterns: [
                                    <Pattern::2>,
                                ],
                                collection: <Expression::4>,
                                where_clause: None,
                            },
                            Iterator {
                                patterns: [
                                    <Pattern::3>,
                                ],
                                collection: <Expression::9>,
                                where_clause: None,
                            },
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 3,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "multi",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "i",
                        ),
                    ),
                    <Pattern::3>: Identifier(
                        Identifier(
                            "j",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
	check_lower_item_eprime(
		"letting where = [ i+j | i: int(1..3), j : int(1..3), i<j]",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::19>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 19,
                data: {
                    <Expression::1>: Identifier(
                        "i",
                    ),
                    <Expression::2>: Identifier(
                        "j",
                    ),
                    <Expression::3>: Identifier(
                        "<",
                    ),
                    <Expression::4>: Call {
                        kind: Operator,
                        function: <Expression::3>,
                        arguments: [
                            <Expression::1>,
                            <Expression::2>,
                        ],
                    },
                    <Expression::5>: IntegerLiteral(
                        1,
                    ),
                    <Expression::6>: IntegerLiteral(
                        3,
                    ),
                    <Expression::7>: Identifier(
                        "..",
                    ),
                    <Expression::8>: Call {
                        kind: Operator,
                        function: <Expression::7>,
                        arguments: [
                            <Expression::5>,
                            <Expression::6>,
                        ],
                    },
                    <Expression::9>: IntegerLiteral(
                        1,
                    ),
                    <Expression::10>: IntegerLiteral(
                        3,
                    ),
                    <Expression::11>: Identifier(
                        "..",
                    ),
                    <Expression::12>: Call {
                        kind: Operator,
                        function: <Expression::11>,
                        arguments: [
                            <Expression::9>,
                            <Expression::10>,
                        ],
                    },
                    <Expression::13>: Identifier(
                        "i",
                    ),
                    <Expression::14>: Identifier(
                        "j",
                    ),
                    <Expression::15>: Identifier(
                        "i",
                    ),
                    <Expression::16>: Identifier(
                        "j",
                    ),
                    <Expression::17>: Identifier(
                        "+",
                    ),
                    <Expression::18>: Call {
                        kind: Operator,
                        function: <Expression::17>,
                        arguments: [
                            <Expression::15>,
                            <Expression::16>,
                        ],
                    },
                    <Expression::19>: ArrayComprehension {
                        template: <Expression::18>,
                        indices: None,
                        generators: [
                            Iterator {
                                patterns: [
                                    <Pattern::2>,
                                ],
                                collection: <Expression::8>,
                                where_clause: Some(
                                    <Expression::4>,
                                ),
                            },
                            Iterator {
                                patterns: [
                                    <Pattern::3>,
                                ],
                                collection: <Expression::12>,
                                where_clause: None,
                            },
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 3,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "where",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "i",
                        ),
                    ),
                    <Pattern::3>: Identifier(
                        Identifier(
                            "j",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
	check_lower_item_eprime(
		"letting indexed = [ i | i : int(1..5) ; int(1..2) ]",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::13>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 13,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        5,
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
                    <Expression::5>: Identifier(
                        "i",
                    ),
                    <Expression::6>: Identifier(
                        "i",
                    ),
                    <Expression::7>: ArrayComprehension {
                        template: <Expression::6>,
                        indices: None,
                        generators: [
                            Iterator {
                                patterns: [
                                    <Pattern::2>,
                                ],
                                collection: <Expression::4>,
                                where_clause: None,
                            },
                        ],
                    },
                    <Expression::8>: IntegerLiteral(
                        1,
                    ),
                    <Expression::9>: IntegerLiteral(
                        2,
                    ),
                    <Expression::10>: Identifier(
                        "..",
                    ),
                    <Expression::11>: Call {
                        kind: Operator,
                        function: <Expression::10>,
                        arguments: [
                            <Expression::8>,
                            <Expression::9>,
                        ],
                    },
                    <Expression::12>: Identifier(
                        "array1d",
                    ),
                    <Expression::13>: Call {
                        kind: Synthetic,
                        function: <Expression::12>,
                        arguments: [
                            <Expression::11>,
                            <Expression::7>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 2,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "indexed",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "i",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
}

#[test]
fn test_lower_absolute() {
	check_lower_item_eprime(
		"letting absolute = | x |",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::3>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 3,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                    <Expression::2>: Identifier(
                        "abs",
                    ),
                    <Expression::3>: Call {
                        kind: Operator,
                        function: <Expression::2>,
                        arguments: [
                            <Expression::1>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "absolute",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
}

#[test]
fn test_lower_const_definition() {
	check_lower_item_eprime(
		"letting one = 1",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::1>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "one",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	)
}

#[test]
fn test_lower_param_declaration() {
	// This will output the last param declaration
	check_lower_item_eprime(
		r#"
      given y, x: int
    "#,
		expect![[r#"
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
                            "x",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
	// This test results in a constraint output due to the where clause
	check_lower_item_eprime(
		r#"
      given y: int
        where y < x
    "#,
		expect![[r#"
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
                        "y",
                    ),
                    <Expression::2>: Identifier(
                        "x",
                    ),
                    <Expression::3>: Identifier(
                        "<",
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
"#]],
	);
}

#[test]
fn test_lower_domain_alias() {
	check_lower_item_eprime(
		"letting x be domain int",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::2>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::6>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 6,
                data: {
                    <Expression::1>: Infinity,
                    <Expression::2>: Identifier(
                        "-",
                    ),
                    <Expression::3>: Call {
                        kind: Synthetic,
                        function: <Expression::2>,
                        arguments: [
                            <Expression::1>,
                        ],
                    },
                    <Expression::4>: Infinity,
                    <Expression::5>: Identifier(
                        "..",
                    ),
                    <Expression::6>: Call {
                        kind: Synthetic,
                        function: <Expression::5>,
                        arguments: [
                            <Expression::3>,
                            <Expression::4>,
                        ],
                    },
                },
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
                        cardinality: None,
                        element: <Type::1>,
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
"#]],
	);
	check_lower_item_eprime(
		"letting x be domain int(1, 2..3)",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::2>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::16>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 16,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        2,
                    ),
                    <Expression::3>: IntegerLiteral(
                        3,
                    ),
                    <Expression::4>: Identifier(
                        "..",
                    ),
                    <Expression::5>: Call {
                        kind: Operator,
                        function: <Expression::4>,
                        arguments: [
                            <Expression::2>,
                            <Expression::3>,
                        ],
                    },
                    <Expression::6>: SetLiteral {
                        members: [
                            <Expression::1>,
                        ],
                    },
                    <Expression::7>: Identifier(
                        "union",
                    ),
                    <Expression::8>: Call {
                        kind: Synthetic,
                        function: <Expression::7>,
                        arguments: [
                            <Expression::6>,
                            <Expression::5>,
                        ],
                    },
                    <Expression::9>: IntegerLiteral(
                        1,
                    ),
                    <Expression::10>: IntegerLiteral(
                        2,
                    ),
                    <Expression::11>: IntegerLiteral(
                        3,
                    ),
                    <Expression::12>: Identifier(
                        "..",
                    ),
                    <Expression::13>: Call {
                        kind: Operator,
                        function: <Expression::12>,
                        arguments: [
                            <Expression::10>,
                            <Expression::11>,
                        ],
                    },
                    <Expression::14>: SetLiteral {
                        members: [
                            <Expression::9>,
                        ],
                    },
                    <Expression::15>: Identifier(
                        "union",
                    ),
                    <Expression::16>: Call {
                        kind: Synthetic,
                        function: <Expression::15>,
                        arguments: [
                            <Expression::14>,
                            <Expression::13>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 2,
                data: {
                    <Type::1>: Bounded {
                        inst: Some(
                            Par,
                        ),
                        opt: None,
                        domain: <Expression::8>,
                    },
                    <Type::2>: Set {
                        inst: Par,
                        opt: NonOpt,
                        cardinality: None,
                        element: <Type::1>,
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
"#]],
	);
}

#[test]
fn test_lower_decision_declaration() {
	check_lower_item_eprime(
		"find x : int",
		expect![[r#"
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
"#]],
	);
}

#[test]
fn test_lower_objective() {
	// Will output a satisfy goal if none specified
	check_lower_item_eprime(
		"",
		expect![[r#"
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
"#]],
	);
	check_lower_item_eprime(
		"minimising x",
		expect![[r#"
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
"#]],
	);
}

#[test]
fn test_lower_branching() {
	check_lower_item_eprime(
		r#"
      minimising x
      branching on [x]
      "#,
		expect![[r#"
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
                        kind: Synthetic,
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
"#]],
	)
}

#[test]
fn test_lower_constraint() {
	check_lower_item_eprime(
		"such that x",
		expect![[r#"
    ItemWithData {
        item: Constraint {
            expression: <Expression::1>,
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
                len: 0,
                data: {},
            },
            annotations: {},
        },
    }
"#]],
	)
}

#[test]
fn test_lower_matrix_literal() {
	check_lower_item_eprime(
		"letting matrix1d = [3,4]",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
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
                        3,
                    ),
                    <Expression::2>: IntegerLiteral(
                        4,
                    ),
                    <Expression::3>: ArrayLiteral {
                        members: [
                            <Expression::1>,
                            <Expression::2>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "matrix1d",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
	check_lower_item_eprime(
		"letting matrix2d = [ [2,8], [3,7] ]",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::5>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 5,
                data: {
                    <Expression::1>: IntegerLiteral(
                        2,
                    ),
                    <Expression::2>: IntegerLiteral(
                        8,
                    ),
                    <Expression::3>: IntegerLiteral(
                        3,
                    ),
                    <Expression::4>: IntegerLiteral(
                        7,
                    ),
                    <Expression::5>: ArrayLiteral2D {
                        rows: NonIndexed(
                            2,
                        ),
                        columns: NonIndexed(
                            2,
                        ),
                        members: [
                            <Expression::1>,
                            <Expression::2>,
                            <Expression::3>,
                            <Expression::4>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "matrix2d",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
	check_lower_item_eprime(
		"letting matrix3d = [ [[1,2],[3,4]], [[5,6],[7,8]] ]",
		expect![[r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::23>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 23,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        2,
                    ),
                    <Expression::3>: IntegerLiteral(
                        3,
                    ),
                    <Expression::4>: IntegerLiteral(
                        4,
                    ),
                    <Expression::5>: IntegerLiteral(
                        5,
                    ),
                    <Expression::6>: IntegerLiteral(
                        6,
                    ),
                    <Expression::7>: IntegerLiteral(
                        7,
                    ),
                    <Expression::8>: IntegerLiteral(
                        8,
                    ),
                    <Expression::9>: IntegerLiteral(
                        1,
                    ),
                    <Expression::10>: IntegerLiteral(
                        2,
                    ),
                    <Expression::11>: Identifier(
                        "..",
                    ),
                    <Expression::12>: Call {
                        kind: Synthetic,
                        function: <Expression::11>,
                        arguments: [
                            <Expression::9>,
                            <Expression::10>,
                        ],
                    },
                    <Expression::13>: IntegerLiteral(
                        1,
                    ),
                    <Expression::14>: IntegerLiteral(
                        2,
                    ),
                    <Expression::15>: Identifier(
                        "..",
                    ),
                    <Expression::16>: Call {
                        kind: Synthetic,
                        function: <Expression::15>,
                        arguments: [
                            <Expression::13>,
                            <Expression::14>,
                        ],
                    },
                    <Expression::17>: IntegerLiteral(
                        1,
                    ),
                    <Expression::18>: IntegerLiteral(
                        2,
                    ),
                    <Expression::19>: Identifier(
                        "..",
                    ),
                    <Expression::20>: Call {
                        kind: Synthetic,
                        function: <Expression::19>,
                        arguments: [
                            <Expression::17>,
                            <Expression::18>,
                        ],
                    },
                    <Expression::21>: ArrayLiteral {
                        members: [
                            <Expression::1>,
                            <Expression::2>,
                            <Expression::3>,
                            <Expression::4>,
                            <Expression::5>,
                            <Expression::6>,
                            <Expression::7>,
                            <Expression::8>,
                        ],
                    },
                    <Expression::22>: Identifier(
                        "array3d",
                    ),
                    <Expression::23>: Call {
                        kind: Synthetic,
                        function: <Expression::22>,
                        arguments: [
                            <Expression::12>,
                            <Expression::16>,
                            <Expression::20>,
                            <Expression::21>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "matrix3d",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
"#]],
	);
}

#[test]
fn test_lower_output() {
	check_lower_item_eprime(
		"showing [show(x)]",
		expect![[r#"
    ItemWithData {
        item: Output {
            section: None,
            expression: <Expression::4>,
        },
        data: ItemData {
            expressions: Arena {
                len: 4,
                data: {
                    <Expression::1>: Identifier(
                        "x",
                    ),
                    <Expression::2>: Identifier(
                        "show",
                    ),
                    <Expression::3>: Call {
                        kind: SourceCall,
                        function: <Expression::2>,
                        arguments: [
                            <Expression::1>,
                        ],
                    },
                    <Expression::4>: ArrayLiteral {
                        members: [
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
"#]],
	)
}
