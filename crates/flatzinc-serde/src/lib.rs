use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Helper function to help skip in serialisation
fn is_false(b: &bool) -> bool {
	!(*b)
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Annotation {
	Call(Call),
	Atom(String),
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "array")]
pub struct Array {
	#[serde(rename = "a")]
	pub contents: Vec<Literal>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub defined: bool,
	#[serde(default, skip_serializing_if = "is_false")]
	pub introduced: bool,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
	Array(Vec<Literal>),
	Literal(Literal),
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "call")]
pub struct Call {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
	pub args: Vec<Argument>,
	pub id: Identifier,
}

pub type Domain = Vec<Vec<i64>>;

pub type Identifier = String;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Literal {
	Int(i64),
	Float(f64),
	Identifier(Identifier),
	Bool(bool),
	Set(SetLiteral),
	String(StringLiteral),
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "method")]
pub enum Method {
	#[serde(rename = "satisfy")]
	Satisfy,
	#[serde(rename = "minimize")]
	Minimize,
	#[serde(rename = "maximize")]
	Maximize,
}

// TODO: Specialise for IntSet
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "setLiteral")]
pub struct SetLiteral {
	pub set: Vec<Vec<f64>>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "stringLiteral")]
pub struct StringLiteral {
	pub string: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "variable")]
pub struct Variable {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub defined: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub domain: Option<Domain>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub introduced: bool,
	#[serde(rename = "rhs", skip_serializing_if = "Option::is_none")]
	pub value: Option<Literal>,
	#[serde(rename = "type")]
	pub ty: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct SolveObjective {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
	pub method: Method,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub objective: Option<Literal>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct FlatZinc {
	#[serde(default)]
	pub arrays: BTreeMap<String, Array>,
	#[serde(default)]
	pub constraints: Vec<Call>,
	#[serde(default)]
	pub output: Vec<Identifier>,
	pub solve: SolveObjective,
	#[serde(default)]
	pub variables: BTreeMap<String, Variable>,
	#[serde(default, skip_serializing_if = "String::is_empty")]
	pub version: String,
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::FlatZinc;

	#[test]
	fn documentation_example() {
		const FROM_DOCS: &str = r#"
		{
			"variables": {
				"b" : { "type" : "int", "domain" : [[0, 3]] },
				"c" : { "type" : "int", "domain" : [[0, 6]] },
				"X_INTRODUCED_0_" : { "type" : "int", "domain" : [[0, 85000]], "defined" : true }
			},
			"arrays": {
				"X_INTRODUCED_2_" : { "a": [250, 200] },
				"X_INTRODUCED_6_" : { "a": [75, 150] },
				"X_INTRODUCED_8_" : { "a": [100, 150] }
			},
			"constraints": [
				{ "id" : "int_lin_le", "args" : ["X_INTRODUCED_2_", ["b", "c"], 4000]},
				{ "id" : "int_lin_le", "args" : ["X_INTRODUCED_6_", ["b", "c"], 2000]},
				{ "id" : "int_lin_le", "args" : ["X_INTRODUCED_8_", ["b", "c"], 500]},
				{ "id" : "int_lin_eq", "args" : [[400, 450, -1], ["b", "c", "X_INTRODUCED_0_"], 0],
					"ann" : ["ctx_pos"], "defines" : "X_INTRODUCED_0_"}
			],
			"output": ["b", "c"],
			"solve": { "method" : "maximize", "objective" : "X_INTRODUCED_0_" },
			"verson": "1.0"
		}"#;
		let fzn: FlatZinc = serde_json::from_str(FROM_DOCS).unwrap();
		expect![[r#"
    FlatZinc {
        arrays: {
            "X_INTRODUCED_2_": Array {
                contents: [
                    Int(
                        250,
                    ),
                    Int(
                        200,
                    ),
                ],
                ann: [],
                defined: false,
                introduced: false,
            },
            "X_INTRODUCED_6_": Array {
                contents: [
                    Int(
                        75,
                    ),
                    Int(
                        150,
                    ),
                ],
                ann: [],
                defined: false,
                introduced: false,
            },
            "X_INTRODUCED_8_": Array {
                contents: [
                    Int(
                        100,
                    ),
                    Int(
                        150,
                    ),
                ],
                ann: [],
                defined: false,
                introduced: false,
            },
        },
        constraints: [
            Call {
                ann: [],
                args: [
                    Literal(
                        Identifier(
                            "X_INTRODUCED_2_",
                        ),
                    ),
                    Array(
                        [
                            Identifier(
                                "b",
                            ),
                            Identifier(
                                "c",
                            ),
                        ],
                    ),
                    Literal(
                        Int(
                            4000,
                        ),
                    ),
                ],
                id: "int_lin_le",
            },
            Call {
                ann: [],
                args: [
                    Literal(
                        Identifier(
                            "X_INTRODUCED_6_",
                        ),
                    ),
                    Array(
                        [
                            Identifier(
                                "b",
                            ),
                            Identifier(
                                "c",
                            ),
                        ],
                    ),
                    Literal(
                        Int(
                            2000,
                        ),
                    ),
                ],
                id: "int_lin_le",
            },
            Call {
                ann: [],
                args: [
                    Literal(
                        Identifier(
                            "X_INTRODUCED_8_",
                        ),
                    ),
                    Array(
                        [
                            Identifier(
                                "b",
                            ),
                            Identifier(
                                "c",
                            ),
                        ],
                    ),
                    Literal(
                        Int(
                            500,
                        ),
                    ),
                ],
                id: "int_lin_le",
            },
            Call {
                ann: [
                    Atom(
                        "ctx_pos",
                    ),
                ],
                args: [
                    Array(
                        [
                            Int(
                                400,
                            ),
                            Int(
                                450,
                            ),
                            Int(
                                -1,
                            ),
                        ],
                    ),
                    Array(
                        [
                            Identifier(
                                "b",
                            ),
                            Identifier(
                                "c",
                            ),
                            Identifier(
                                "X_INTRODUCED_0_",
                            ),
                        ],
                    ),
                    Literal(
                        Int(
                            0,
                        ),
                    ),
                ],
                id: "int_lin_eq",
            },
        ],
        output: [
            "b",
            "c",
        ],
        solve: SolveObjective {
            ann: [],
            method: Maximize,
            objective: Some(
                Identifier(
                    "X_INTRODUCED_0_",
                ),
            ),
        },
        variables: {
            "X_INTRODUCED_0_": Variable {
                ann: [],
                defined: true,
                domain: Some(
                    [
                        [
                            0,
                            85000,
                        ],
                    ],
                ),
                introduced: false,
                value: None,
                ty: "int",
            },
            "b": Variable {
                ann: [],
                defined: false,
                domain: Some(
                    [
                        [
                            0,
                            3,
                        ],
                    ],
                ),
                introduced: false,
                value: None,
                ty: "int",
            },
            "c": Variable {
                ann: [],
                defined: false,
                domain: Some(
                    [
                        [
                            0,
                            6,
                        ],
                    ],
                ),
                introduced: false,
                value: None,
                ty: "int",
            },
        },
        version: "",
    }
"#]]
		.assert_debug_eq(&fzn);
		let fzn2: FlatZinc = serde_json::from_str(&serde_json::to_string(&fzn).unwrap()).unwrap();
		assert_eq!(fzn, fzn2)
	}
}
