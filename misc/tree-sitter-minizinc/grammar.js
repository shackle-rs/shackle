module.exports = grammar({
  name: 'minizinc',

  rules: {
    source_file: $ => 'constraint',
  }
});
