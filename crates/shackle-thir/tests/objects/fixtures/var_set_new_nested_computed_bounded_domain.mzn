% Binding declared domain + total RHS on a NESTED class:
% B is var-reached via `A.children` under a var-existence root, so its slots
% can be unrealised and `z = x1 + x2` at the pinned frees (0 + 0 = 2 min...
% joint-lb 0) falls outside the declared 3..4. The domain
% relocation applies to nested owners too: the element
% record's `z` is relaxed to unbounded and the declared domain is re-imposed
% as the realised-set invariant `forall(this in B)(this.z in 3..4)` — no
% per-slot value guard. `card(as) = 0` (everything unrealised) must be SAT.
class B(var 0..2: x1; var 0..2: x2; var 3..4: z = x1 + x2);
class A(set(2..2) of new B: children);
var set(0..1) of new A: as;
constraint card(as) = 0;
solve satisfy;
output ["card=", show(card(as)), "\n"];
