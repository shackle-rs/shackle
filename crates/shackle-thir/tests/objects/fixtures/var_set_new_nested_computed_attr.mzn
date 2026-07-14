% A computed attribute on a VAR-REACHED NESTED class: B is introduced via
% `A.children` under a `var set of new A` root, so B's storage is var and its
% slots can be unrealised. An earlier lowering registered uninitialized
% FULL-record nested storage (the computed `y` a free decision) and panicked
% during lowering on the storage-record mismatch. Now the free decisions live
% in `B_children_storage` (free record — no `y`) and `B_children_objects` is
% the engine reconstruction over it: `y = x + 1` alias-defined per slot, with
% the realisation test `p in B` (elided here — total RHS, non-binding domain).
% card(as) = 1 realises one A whose two children solve at x = 2, so ys=[3, 3].
class B(var 2..3: x; var int: y = x + 1);
class A(set(2..2) of new B: children);
var set(0..1) of new A: as;
constraint card(as) = 1;
solve satisfy;
output ["ys=", show([fix(b.y) | a in as, b in a.children]), "\n"];
