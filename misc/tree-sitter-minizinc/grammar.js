const PREC = {
  call: 15,
  annotation: 14,
  unary: 13,
  exponent: 12,
  multiplicative: 11,
  additive: 10,
  intersect: 9,
  dotdot: 8,
  symdiff: 7,
  diff: 6,
  union: 5,
  comparative: 4,
  and: 3,
  xor: 2,
  or: 1,
  implication: 2,
  equivalence: 1,
};

const primitive_types = ["ann", "bool", "float", "int", "string"];

module.exports = grammar({
  name: "minizinc",

  extras: ($) => [/\s/, $.line_comment, $.block_comment],

  word: ($) => $.identifier,

  conflicts: ($) => [
    [$._expression, $.generator],
    [$._expression, $.assignment],
  ],

  supertypes: ($) => [$._expression, $._item, $._type],

  rules: {
    source_file: ($) => seq(sepBy(";", $._item)),

    _item: ($) =>
      choice(
        $.annotation,
        $.assignment,
        $.constraint,
        $.declaration,
        $.function_item,
        $.goal,
        $.include,
        $.output,
        $.predicate
        // TODO: Other statements types
      ),

    annotation: ($) =>
      seq(
        "annotation",
        field("name", $.identifier),
        optional(field("parameters", $._parameters)),
        optional(seq("=", field("expr", $._expression)))
      ),

    assignment: ($) =>
      seq(field("name", $.identifier), "=", field("expr", $._expression)),

    constraint: ($) => seq("constraint", $._expression),

    declaration: ($) =>
      seq(
        field("type", $._type),
        ":",
        field("name", $.identifier),
        optional(seq("=", field("expr", $._expression)))
      ),

    function_item: ($) =>
      seq(
        "function",
        field("type", $._type),
        ":",
        field("name", $.identifier),
        field("parameters", $._parameters),
        optional(field("annotations", $._annotations)),
        optional(seq("=", field("expr", $._expression)))
      ),

    goal: ($) =>
      seq(
        "solve",
        field(
          "strategy",
          choice(
            "satisfy",
            seq("maximize", $._expression),
            seq("minimize", $._expression)
          )
        )
      ),

    include: ($) => seq("include", $.string_literal),

    output: ($) => seq("output", $._expression),

    predicate: ($) =>
      seq(
        field("type", choice("predicate", "test")),
        field("name", $.identifier),
        field("parameters", $._parameters),
        optional(field("annotations", $._annotations)),
        optional(seq("=", field("expr", $._expression)))
      ),

    _annotations: ($) => repeat1(seq("::", $._expression)),

    _parameters: ($) =>
      seq("(", sepBy(",", seq($._type, optional(seq(":", $.identifier)))), ")"),
    _expression: ($) =>
      choice(
        $.identifier,
        $._literal,

        $.array_comprehension,
        $.call,
        $.generator_call,
        $.if_then_else,
        $.indexed_access,
        $.infix_operator,
        $.let_expression,
        $.prefix_operator,
        $.set_comprehension,
        $.string_interpolation,
        $.parenthesised_expression
      ),

    parenthesised_expression: ($) => seq("(", $._expression, ")"),

    array_comprehension: ($) =>
      seq("[", $._expression, "|", sepBy1(",", $.generator), "]"),

    call: ($) =>
      prec(
        PREC.call,
        seq(
          field("name", $.identifier),
          "(",
          field("arguments", sepBy(",", $._expression)),
          ")"
        )
      ),

    generator_call: ($) =>
      prec(
        PREC.call,
        seq(
          field("name", $.identifier),
          "(",
          field("generators", sepBy1(",", $.generator)),
          ")",
          "(",
          field("template", $._expression),
          ")"
        )
      ),

    generator: ($) =>
      seq(
        $.identifier,
        "in",
        $._expression,
        optional(seq("where", $._expression))
      ),

    if_then_else: ($) =>
      seq(
        "if",
        $._expression,
        "then",
        $._expression,
        repeat(seq("elseif", $._expression, "then", $._expression)),
        optional(seq("else", $._expression)),
        "endif"
      ),

    indexed_access: ($) =>
      prec(
        PREC.call,
        seq(
          field("collection", $._expression),
          "[",
          field("indices", seq($._expression, repeat(seq(",", $._expression)))),
          "]"
        )
      ),

    infix_operator: ($) => {
      const table = [
        [prec.left, PREC.equivalence, "<->"],
        [prec.left, PREC.implication, choice("->", "<-")],
        [prec.left, PREC.or, "\\/"],
        [prec.left, PREC.xor, "xor"],
        [prec.left, PREC.and, "/\\"],
        // TODO: Should really be nonassoc
        // prettier-ignore
        [prec.left, PREC.comparative, choice(
          "=", "==", "!=", "<", "<=", ">", ">=", "in", "subset", "superset"
        )],
        [prec.left, PREC.union, "union"],
        [prec.left, PREC.diff, "diff"],
        [prec.left, PREC.symdiff, "symdiff"],
        [prec.left, PREC.intersect, "intersect"],
        // TODO: Could be nonassoc, will always give type error
        [prec.left, PREC.dotdot, ".."],
        [prec.left, PREC.additive, choice("+", "-", "++")],
        [prec.left, PREC.multiplicative, choice("*", "/", "div", "mod")],
        [prec.left, PREC.exponent, "^"],
        [prec.left, PREC.annotation, "::"],
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

    let_expression: ($) =>
      seq(
        "let",
        "{",
        field(
          "let",
          sepBy(choice(",", ";"), choice($.declaration, $.constraint))
        ),
        "}",
        "in",
        field("in", $._expression)
      ),

    prefix_operator: ($) =>
      prec(
        PREC.unary,
        seq(field("operator", choice("-", "not", "¬")), $._expression)
      ),

    set_comprehension: ($) =>
      seq("{", $._expression, "|", sepBy1(",", $.generator), "}"),

    // TODO: Decide if string_literal and string_interpolation should be combined
    string_interpolation: ($) =>
      seq(
        '"',
        optional($.string_content),
        repeat1(seq("\\(", $._expression, ")", optional($.string_content))),
        '"'
      ),

    _type: ($) => choice($.array_type, $.base_type, $.set_type),
    array_type: ($) =>
      seq("array", "[", sepBy1(",", $.base_type), "]", "of", $.base_type),
    base_type: ($) =>
      seq(
        optional(field("var_par", choice("var", "par"))),
        optional(field("opt", "opt")),
        choice($.primitive_type, $._expression)
      ),
    primitive_type: ($) => choice(...primitive_types),
    set_type: ($) => seq("set", "of", $.base_type),

    _literal: ($) =>
      choice(
        $.absent,
        $.array_literal,
        $.boolean_literal,
        $.float_literal,
        $.integer_literal,
        $.set_literal,
        $.string_literal
      ),

    absent: ($) => "<>",
    array_literal: ($) => seq("[", sepBy(",", $._expression), "]"),
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
    set_literal: ($) => seq("{", sepBy(",", $._expression), "}"),

    string_literal: ($) =>
      seq('"', alias(optional($.string_content), "content"), '"'),
    string_content: ($) =>
      repeat1(choice(token.immediate(prec(1, /[^"\n\\]+/)), $.escape_sequence)),
    escape_sequence: ($) =>
      token.immediate(
        seq(
          "\\",
          choice(
            /[^xuU]/,
            /\d{2,3}/,
            /x[0-9a-fA-F]{2,}/,
            /u[0-9a-fA-F]{4}/,
            /U[0-9a-fA-F]{8}/
          )
        )
      ),

    identifier: ($) => /[A-Za-z][A-Za-z0-9_]*/,

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
