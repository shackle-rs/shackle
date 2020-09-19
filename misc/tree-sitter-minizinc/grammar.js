module.exports = grammar({
  name: 'minizinc',

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
      // TODO: Other expression types
    ),


    _literal: $ => choice(
      // TODO: absent,
      $.boolean_literal,
      // TODO: float_literal,
      // TODO: integer_literal,
      // TODO: string_literal,
    ),

    boolean_literal: $ => choice('true', 'false'),


    identifier: $ => /[A-Za-z][A-Za-z0-9_]*/,

  }
});

function sepBy1(sep, rule) {
  return seq(rule, repeat(seq(sep, rule)))
}

function sepBy(sep, rule) {
  return optional(sepBy1(sep, rule))
}
