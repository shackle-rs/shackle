# MiniZinc AST generation

After parsing, a MiniZinc AST will be generated from the CST.

- Whitespace and comments are ignored in the AST
- Generators in calls and comprehensions can generate the same AST nodes

The AST nodes will need to maintain a link to the CST nodes they originated from
(akin to `Location`s in the old compiler).

## Notes

- Using Salsa means that we do not need to include storage for results in the
  AST. Instead, these will be stored in the Salsa database and accessed using
  queries
