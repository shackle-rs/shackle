# Parsing

Tree-sitter is used to generate a concrete syntax tree from a MiniZinc model.

Since this concrete syntax tree is too low level to perform most useful
compilation steps, an abstract syntax tree will be constructed (with the AST
nodes linked to the related CST nodes) during [AST generation](ast-gen.md).

## Notes

- Tree-sitter uses the idea of anonymous nodes as a substitute for an AST,
  however, this would not solve the problem of e.g. having different concrete
  syntax for function generators and comprehensions. Code transformations also
  seem to be generally outside the scope of Tree-sitter.
- Tree-sitter uses a query system for searching for patterns in the CST. It's
  unclear how useful this functionality would be, since it seems like we will
  be interacting with an AST we produce be walking the CST instead of with the
  CST directly.
