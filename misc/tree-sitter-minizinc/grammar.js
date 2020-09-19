module.exports = grammar({
  name: 'minizinc',

  extras: $ => [/\s/, $.line_comment, $.block_comment],

  word: $ => $.identifier,

  rules: {
    source_file: $ => seq(sepBy(';', $._items), optional(';')),

    _items: $ => choice(
      $.assignment_item,
      // TODO: Other statements types
    ),

    assignment_item: $ => seq(
      field('name', $.identifier),
      '=',
      field('expr', $._expression)
    ),

    _expression: $ => choice(
      $._literal,
      $.identifier,
      // TODO: Other expression types
    ),


    _literal: $ => choice(
      $.absent,
      $.boolean_literal,
      $.float_literal,
      $.integer_literal,
      $.string_literal,
    ),

    absent: $ => '<>',
    boolean_literal: $ => choice('true', 'false'),
    float_literal: $ => token(choice(
      /\d+\.\d+/,
      /\d+(\.\d+)?[Ee][+-]?\d+/,
      // TODO: Hexadecimal floating point numbers
    )),
    integer_literal: $ => token(choice(
      /[0-9]+/,
      /0x[0-9a-fA-F]+/,
      /0b[01]+/,
      /0o[0-7]+/
    )),

    string_literal: $ => seq(
      '"',
      repeat(choice(
        token.immediate(prec(1, /[^"\n\\]+/)),
        $.escape_sequence
      )),
      '"'
    ),
    escape_sequence: $ => token.immediate(seq(
      '\\',
      choice(
        /[^xuU]/,
        /\d{2,3}/,
        /x[0-9a-fA-F]{2,}/,
        /u[0-9a-fA-F]{4}/,
        /U[0-9a-fA-F]{8}/
      )
    )),

    identifier: $ => /[A-Za-z][A-Za-z0-9_]*/,

    line_comment: $ => token(seq('%', /.*/)),
    block_comment: $ => token(seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')),

  }
});

function sepBy1(sep, rule) {
  return seq(rule, repeat(seq(sep, rule)))
}

function sepBy(sep, rule) {
  return optional(sepBy1(sep, rule))
}
