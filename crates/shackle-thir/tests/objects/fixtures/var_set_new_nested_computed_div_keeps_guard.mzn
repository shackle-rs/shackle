% The 2.5(a) elision analysis on a NESTED slot: a non-total RHS (`div`) on a
% var-reached nested class must KEEP its realisation guard. The per-field
% predicates are path-independent, so the nested engine contribution guards
% `y` exactly like a root would: `y = if realised then 6 div (x + 1) else
% <witness> endif` with `realised = p in B`. With `card(as) = 0` every A and
% B slot is unrealised and the model must stay SAT via the witness default.
class B(var 0..5: x; var 1..6: y = 6 div (x + 1));
class A(set(2..2) of new B: children);
var set(0..1) of new A: as;
constraint card(as) = 0;
solve satisfy;
output ["card=", show(card(as)), "\n"];
