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
- When the CST changes, ideally we'd avoid recomputing results about portions of
  the AST which haven't changed (which could potentially be the whole AST in the
  case of editing whitespace/comments).
- Doing this would mean that linking the AST to the CST needs to be considered
  carefully.
