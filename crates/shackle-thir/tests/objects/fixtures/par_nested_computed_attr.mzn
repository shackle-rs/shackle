% A computed attribute on a PAR-NESTED class (B introduced via `A.children`,
% not at a root). An earlier nested contribution template
% fresh-minted `y` as a valueless par decl (`let { int: y_init; } in y_init`
% — invalid MiniZinc, "let variable must be initialised"). The nested
% contribution now runs the reconstruction engine: `y = x + 1` is
% alias-defined per object against the input-read `x`.
class B(2..3: x; int: y = x + 1);
class A(set(1..2) of new B: children);
new A: a = (children: [(x: 2), (x: 3)]);
solve satisfy;
output ["ys=", show([b.y | b in a.children]), "\n"];
