# Parsing

`tree-sitter` is used to generate a concrete syntax tree from a MiniZinc model.

Since this concrete syntax tree is too low level to perform most useful
compilation steps, an abstract syntax tree will be constructed (with the AST
nodes linked to the related CST nodes) during [AST generation](ast-gen.md).
