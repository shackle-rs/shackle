% A `var set of new` root whose class carries a nested exact-cardinality
% `set(2..2) of new B` field. The cardinality invariant was previously
% MISSING on this root shape entirely (followups P2): `children` was free to
% be `{}` and the realised object solved with `n = 0`. It is now emitted as a
% realised-set class invariant, so the realised object must have exactly two
% children (`ns=[2]`).
class B(2..3: x);
class A(set(2..2) of new B: children; int: n = card(children));
var set(1..1) of new A: as;
constraint card(as) = 1;
solve satisfy;
output ["ns=", show([fix(a.n) | a in as]), "\n"];
