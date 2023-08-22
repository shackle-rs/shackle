# AST generation

After parsing, a MiniZinc AST is generated from the CST. This provides type-safe accessors to the nodes in the syntax
tree. No desugaring takes place at this stage, and all semantic nodes are are made available other than parentheses
(which made explicit in the tree structure). So nodes like whitespace, comments and semicolons from the CST are removed
in the AST.

This is still too low level for most analysis, and there several constructs which are semantically the same, but with
different syntactic representations. Therefore, the next step is to [resolve the `include` items](includes.md), and then
lower each model into [HIR](../hir/hir.md).

## Example

The model

```mzn
{{#include ../../examples/simple-model.mzn}}
```

Gives the AST

```
{{#include ../../examples/simple-model-ast.txt}}
```
