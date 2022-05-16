const PREC = {
  call: 18,
  default: 17,
  annotation: 16,
  unary: 15,
  exponent: 14,
  multiplicative: 13,
  additive: 12,
  intersect: 11,
  range: 10,
  symdiff: 9,
  set_diff: 8,
  union: 7,
  comparative: 6,
  conjunction: 5,
  exclusive_disjunction: 4,
  disjunction: 3,
  implication: 2,
  equivalence: 1,
};

const primitive_types = ["ann", "bool", "float", "int", "string"];

const EQUIVALENCE_OPERATORS = ["<->", "⟷", "⇔"];
const IMPLICATION_OPERATORS = ["->", "→", "⇒", "<-", "←", "⇐"];
const DISJUNCTION_OPERATORS = ["\\/", "∨"];
const EXCLUSIVE_DISJUNCTION_OPERATORS = ["xor", "⊻"];
const CONJUNCTION_OPERATORS = ["/\\", "∧"];
// prettier-ignore
const COMPARISON_OPERATORS = [
  "=", "==", "!=", "≠", "<", "<=", "≤", ">", ">=",
  "≥", "in", "∈", "subset", "⊆", "superset", "⊇",
];
const UNION_OPERATORS = ["union", "∪"];
const SET_DIFF_OPERATORS = ["diff", "∖"];
const INTERSECTION_OPERATORS = ["intersect", "∩"];
const RANGE_OPERATORS = ["..", "<..", "..<", "<..<"];
const ADDITIVE_OPERATORS = ["+", "-", "++"];
const MULTIPLICATIVE_OPERATORS = ["*", "/", "div", "mod"];

const OPERATOR_CHARACTERS = `,;:(){}&|$.∞%`.concat(
  getOpChars(EQUIVALENCE_OPERATORS),
  getOpChars(IMPLICATION_OPERATORS),
  getOpChars(DISJUNCTION_OPERATORS),
  getOpChars(EXCLUSIVE_DISJUNCTION_OPERATORS),
  getOpChars(CONJUNCTION_OPERATORS),
  getOpChars(COMPARISON_OPERATORS),
  getOpChars(UNION_OPERATORS),
  getOpChars(SET_DIFF_OPERATORS),
  getOpChars(INTERSECTION_OPERATORS),
  getOpChars(ADDITIVE_OPERATORS),
  getOpChars(MULTIPLICATIVE_OPERATORS)
);

module.exports = grammar({
  name: "minizinc",

  extras: ($) => [/\s/, $.line_comment, $.block_comment],

  word: ($) => $.identifier,

  conflicts: ($) => [
    [$._unannotated_expression, $.generator],
    [$._unannotated_expression, $.assignment],
    [$.array_literal_2d, $.array_literal_2d_row],
  ],

  supertypes: ($) => [$._expression, $._item, $._type],

  rules: {
    source_file: ($) => seq(sepBy(";", field("item", $._item))),

    _item: ($) =>
      choice(
        $.annotation,
        $.assignment,
        $.constraint,
        $.declaration,
        $.enumeration,
        $.function_item,
        $.goal,
        $.include,
        $.output,
        $.predicate
      ),

    annotation: ($) =>
      seq(
        "annotation",
        field("name", $._identifier),
        optional($._parameters),
        optional(seq("=", field("body", $._expression)))
      ),

    assignment: ($) =>
      seq(
        field("name", $._identifier),
        "=",
        field("definition", $._expression)
      ),

    constraint: ($) =>
      seq(
        "constraint",
        optional($._annotation_list),
        field("expression", $._expression)
      ),

    declaration: ($) =>
      seq(
        field("type", $._type),
        ":",
        field("name", $._identifier),
        optional($._annotation_list),
        optional(seq("=", field("definition", $._expression)))
      ),

    enumeration: ($) =>
      seq(
        "enum",
        field("name", $._identifier),
        optional($._annotation_list),
        optional(seq("=", sepBy1("++", field("case", $._enumeration_case))))
      ),

    function_item: ($) =>
      seq(
        "function",
        field("type", $._type),
        ":",
        field("name", $._identifier),
        $._parameters,
        optional($._annotation_list),
        optional(seq("=", field("body", $._expression)))
      ),

    goal: ($) =>
      seq(
        "solve",
        optional($._annotation_list),
        choice(
          field("strategy", "satisfy"),
          seq(
            field("strategy", choice("maximize", "minimize")),
            field("objective", $._expression)
          )
        )
      ),

    include: ($) => seq("include", field("file", $.string_literal)),

    output: ($) =>
      seq(
        "output",
        optional($._annotation_list),
        field("expression", $._expression)
      ),

    predicate: ($) =>
      seq(
        field("type", choice("predicate", "test")),
        field("name", $._identifier),
        $._parameters,
        optional($._annotation_list),
        optional(seq("=", field("body", $._expression)))
      ),

    _annotation_list: ($) =>
      seq(
        repeat1(
          prec.left(
            PREC.annotation,
            seq("::", field("annotation", $._unannotated_expression))
          )
        )
      ),

    _parameters: ($) =>
      seq("(", sepBy(",", field("parameter", $.parameter)), ")"),
    parameter: ($) =>
      seq(
        field("type", $._type),
        optional(seq(":", field("name", $._identifier))),
        optional($._annotation_list)
      ),

    _enumeration_case: ($) =>
      choice(
        $.enumeration_members,
        $.anonymous_enumeration,
        $.enumeration_constructor
      ),

    enumeration_members: ($) =>
      seq("{", sepBy(",", field("member", $._identifier)), "}"),
    anonymous_enumeration: ($) =>
      seq("_", "(", field("argument", $._expression), ")"),
    enumeration_constructor: ($) =>
      seq(
        field("name", $._identifier),
        "(",
        field("argument", $._expression),
        ")"
      ),

    _expression: ($) =>
      choice($._unannotated_expression, $.annotated_expression),
    _unannotated_expression: ($) =>
      choice(
        $._identifier,
        $._literal,

        $.array_comprehension,
        $.call,
        $.generator_call,
        $.if_then_else,
        $.indexed_access,
        $.infix_operator,
        $.let_expression,
        $.prefix_operator,
        $.postfix_operator,
        $.set_comprehension,
        $.string_interpolation,
        $.parenthesised_expression
      ),

    parenthesised_expression: ($) =>
      seq("(", field("expression", $._expression), ")"),

    array_comprehension: ($) =>
      seq(
        "[",
        optional(seq(field("index", $._index_tuple), ":")),
        field("template", $._expression),
        "|",
        sepBy1(",", field("generator", $.generator)),
        "]"
      ),

    call: ($) =>
      prec(
        PREC.call,
        seq(
          field("name", $._identifier),
          "(",
          sepBy(",", field("argument", $._expression)),
          ")"
        )
      ),

    generator_call: ($) =>
      prec(
        PREC.call,
        seq(
          field("name", $._identifier),
          "(",
          sepBy1(",", field("generator", $.generator)),
          ")",
          "(",
          field("template", $._expression),
          ")"
        )
      ),

    generator: ($) =>
      seq(
        sepBy1(",", field("name", $._identifier)),
        "in",
        field("collection", $._expression),
        optional(seq("where", field("where", $._expression)))
      ),

    if_then_else: ($) =>
      seq(
        "if",
        field("condition", $._expression),
        "then",
        field("result", $._expression),
        repeat(
          seq(
            "elseif",
            field("condition", $._expression),
            "then",
            field("result", $._expression)
          )
        ),
        optional(seq("else", field("else", $._expression))),
        "endif"
      ),

    indexed_access: ($) =>
      prec(
        PREC.call,
        seq(
          field("collection", $._expression),
          "[",
          sepBy1(
            ",",
            field("index", choice(...RANGE_OPERATORS, $._expression))
          ),
          "]"
        )
      ),

    infix_operator: ($) => {
      // WARNING: All non-word operators must be included in the OPERATOR_CHARACTERS string
      const table = [
        [prec.left, PREC.equivalence, choice(...EQUIVALENCE_OPERATORS)],
        [prec.left, PREC.implication, choice(...IMPLICATION_OPERATORS)],
        [prec.left, PREC.disjunction, choice(...DISJUNCTION_OPERATORS)],
        [
          prec.left,
          PREC.exclusive_disjunction,
          choice(...EXCLUSIVE_DISJUNCTION_OPERATORS),
        ],
        [prec.left, PREC.conjunction, choice(...CONJUNCTION_OPERATORS)],
        // TODO: Should really be nonassoc
        [prec.left, PREC.comparative, choice(...COMPARISON_OPERATORS)],
        [prec.left, PREC.union, choice(...UNION_OPERATORS)],
        [prec.left, PREC.set_diff, choice(...SET_DIFF_OPERATORS)],
        [prec.left, PREC.symdiff, "symdiff"],
        [prec.left, PREC.intersect, choice(...INTERSECTION_OPERATORS)],
        // TODO: Could be nonassoc, will always give type error
        [prec.left, PREC.range, choice(...RANGE_OPERATORS)],
        [prec.left, PREC.additive, choice(...ADDITIVE_OPERATORS)],
        [prec.left, PREC.multiplicative, choice(...MULTIPLICATIVE_OPERATORS)],
        [prec.left, PREC.exponent, "^"],
        [prec.left, PREC.default, "default"],
      ];

      return choice(
        ...table.map(([assoc, precedence, operator]) =>
          assoc(
            precedence,
            seq(
              field("left", $._expression),
              field("operator", operator),
              field("right", $._expression)
            )
          )
        )
      );
    },

    annotated_expression: ($) =>
      prec(
        PREC.annotation,
        seq(
          field("expression", $._unannotated_expression),
          repeat1(
            prec.left(
              PREC.annotation,
              seq("::", field("annotation", $._unannotated_expression))
            )
          )
        )
      ),

    let_expression: ($) =>
      seq(
        "let",
        "{",
        field(
          "let",
          sepBy(
            choice(",", ";"),
            field("item", choice($.declaration, $.constraint))
          )
        ),
        "}",
        "in",
        field("in", $._expression)
      ),

    prefix_operator: ($) =>
      choice(
        prec(
          PREC.unary,
          seq(
            field("operator", choice("-", "not", "¬")),
            field("operand", $._expression)
          )
        ),
        // TODO: Could be nonassoc, will always give type error
        prec.left(
          PREC.range,
          seq(
            field("operator", choice(...RANGE_OPERATORS)),
            field("operand", $._expression)
          )
        )
      ),

    postfix_operator: ($) =>
      // TODO: Could be nonassoc, will always give type error
      prec.left(
        PREC.range,
        seq(
          field("operand", $._expression),
          field("operator", choice(...RANGE_OPERATORS))
        )
      ),

    set_comprehension: ($) =>
      seq(
        "{",
        field("template", $._expression),
        "|",
        sepBy1(",", field("generator", $.generator)),
        "}"
      ),

    // TODO: Decide if string_literal and string_interpolation should be combined
    string_interpolation: ($) =>
      seq(
        '"',
        optional(field("item", alias($._string_content, "string"))),
        repeat1(
          seq(
            "\\(",
            field("item", alias($._expression, "expression")),
            ")",
            optional(field("item", alias($._string_content, "string")))
          )
        ),
        '"'
      ),

    _type: ($) => choice($.array_type, $.type_base),
    array_type: ($) =>
      seq(
        "array",
        "[",
        sepBy1(",", field("dimension", $.type_base)),
        "]",
        "of",
        field("type", $.type_base)
      ),
    type_base: ($) =>
      choice(
        seq(
          optional(field("var_par", choice("var", "par"))),
          optional(field("opt", "opt")),
          optional(field("set", seq("set", "of"))),
          field(
            "domain",
            choice(
              $.primitive_type,
              $.type_inst_id,
              $.type_inst_enum_id,
              $._expression
            )
          )
        ),
        seq(field("any", "any"), optional(field("domain", $.type_inst_id)))
      ),
    primitive_type: ($) => choice(...primitive_types),
    type_inst_id: ($) => /\$[A-Za-z][A-Za-z0-9_]*/,
    type_inst_enum_id: ($) => /\$\$[A-Za-z][A-Za-z0-9_]*/,

    _literal: ($) =>
      choice(
        $.absent,
        $.anonymous,
        $.array_literal_2d,
        $.array_literal,
        $.boolean_literal,
        $.float_literal,
        $.infinity,
        $.integer_literal,
        $.set_literal,
        $.string_literal
      ),

    absent: ($) => "<>",
    anonymous: ($) => "_",
    array_literal: ($) =>
      seq("[", sepBy(",", field("member", $.array_literal_member)), "]"),
    array_literal_member: ($) =>
      seq(optional(seq($._index_tuple, ":")), field("value", $._expression)),
    _index_tuple: ($) =>
      choice(
        seq(
          "(",
          field("index", $._expression),
          ",",
          sepBy1(",", field("index", $._expression)),
          ")"
        ),
        field("index", $._expression)
      ),
    array_literal_2d: ($) =>
      seq(
        "[|",
        optional(
          seq(
            choice(
              repeat1(seq(field("column_index", $._expression), ":")),
              field("row", $.array_literal_2d_row)
            ),
            repeat(seq("|", field("row", $.array_literal_2d_row))),
            optional("|")
          )
        ),
        "|]"
      ),
    array_literal_2d_row: ($) =>
      seq(
        optional(seq(field("index", $._expression), ":")),
        sepBy1(",", field("member", $._expression))
      ),
    boolean_literal: ($) => choice("true", "false"),
    float_literal: ($) =>
      token(
        choice(
          /\d+\.\d+/,
          /\d+(\.\d+)?[Ee][+-]?\d+/
          // TODO: Hexadecimal floating point numbers
        )
      ),
    integer_literal: ($) =>
      token(choice(/[0-9]+/, /0x[0-9a-fA-F]+/, /0b[01]+/, /0o[0-7]+/)),
    infinity: ($) => choice("infinity", "∞"),
    set_literal: ($) =>
      choice("∅", seq("{", sepBy(",", field("member", $._expression)), "}")),

    string_literal: ($) => seq('"', optional($._string_content), '"'),
    _string_content: ($) =>
      repeat1(field("content", choice($.string_characters, $.escape_sequence))),
    string_characters: ($) => token.immediate(prec(1, /[^"\n\\]+/)),
    escape_sequence: ($) => {
      const simpleEscape = [
        ["\\'", "'"],
        ['\\"', '"'],
        ["\\\\", "\\"],
        ["\\r", "\r"],
        ["\\n", "\n"],
        ["\\t", "\t"],
      ];
      return choice(
        field("escape", choice(...simpleEscape.map(([e, v]) => alias(e, v)))),
        seq("\\", field("escape", alias(/[0-7]{1,3}/, "octal"))),
        seq("\\x", field("escape", alias(/[0-9a-fA-F]{2}/, "hexadecimal"))),
        seq("\\u", field("escape", alias(/[0-9a-fA-F]{4}/, "hexadecimal"))),
        seq("\\U", field("escape", alias(/[0-9a-fA-F]{8}/, "hexadecimal")))
      );
    },

    identifier: ($) => {
      return new RegExp(`[^"'\\s\\.\\-\\[\\]\\^${OPERATOR_CHARACTERS}]+`);
    },
    quoted_identifier: ($) => /'[^']*'/,
    _identifier: ($) => choice($.identifier, $.quoted_identifier),

    line_comment: ($) => token(seq("%", /.*/)),
    block_comment: ($) => token(seq("/*", /([^*]|\*[^\/]|\n)*?\*?/, "*/")),
  },
});

function sepBy(sep, rule) {
  return seq(repeat(seq(rule, sep)), optional(rule));
}

function sepBy1(sep, rule) {
  return seq(rule, repeat(seq(sep, rule)), optional(sep));
}

function getOpChars(list) {
  return list.filter((str) => /^[a-fA-F]*$/.test(str)).join("");
}
