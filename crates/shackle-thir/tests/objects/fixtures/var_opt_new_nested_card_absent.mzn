% A `var opt new` root whose class carries a nested exact-cardinality
% `set(2..2) of new B` field. `absent(a)` must be SAT: no object is realised,
% so the class body is vacuous. The nested-cardinality invariant used to be
% emitted over POTENTIAL storage (`forall(i in A_a_storage)(card(i.children)
% in 2..2)`), which still constrained the unrealised slot — whose `children`
% defaults to `{}` — making absence UNSAT. It is now a realised-set class
% invariant (`forall(this in A)(...)`), like the var-declared nested set case.
class B(2..3: x);
class A(set(2..2) of new B: children);
var opt new A: a;
constraint absent(a);
solve satisfy;
output ["absent ok\n"];
