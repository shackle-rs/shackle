% Family (b) of the field-only universe fallback: a field-only SUPERCLASS
% of a var-existence subclass. B's actual set derives correctly from the
% parent field (empty here), but A has no recorded field introduction and
% falls back to `A = A_potential` — claiming a phantom member with no
% underlying B. Correct semantics: A = image of B = {} and nA=0.
class R ( var set(0..1) of new B: bs; );
class A ( var 0..1: y; );
class B extends A ();
constraint forall(r in R)(card(r.bs) = 0);
var new R: r;
output ["nA=", show(card(A)), " nB=", show(card(B))];
solve satisfy;
