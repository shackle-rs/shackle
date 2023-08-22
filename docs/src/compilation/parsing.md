# Parsing

Tree-sitter is used to generate a concrete syntax tree from a MiniZinc model. The grammar is located in the
`parsers/tree-sitter-minizinc/grammar.js` file. There is also a corpus of tests which can be run to test the parser's
output.

Since this concrete syntax tree is too low level to perform most useful compilation steps, an abstract syntax tree will
be constructed (with the AST nodes linked to the related CST nodes) during [AST generation](./ast/ast.md).

## Example

The model

```mzn
{{#include ../examples/simple-model.mzn}}
```

Gives the CST

```
{{#include ../examples/simple-model-cst.txt}}
```
