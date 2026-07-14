% A computed attribute whose RHS is NOT on the totality whitelist (`div`) on
% a `var set of new` root. The 2.5(a) elision analysis must NOT fire: the
% value guard (witness + pin + if-then-else + per-slot `realised` alias)
% REMAINS, because a partial RHS evaluated at an unrealised slot's pinned
% frees turns its definedness side-condition into a hard constraint
% (assessment channel B). Here `6 div (x + 1)` happens to be defined
% everywhere, but the conservative whitelist cannot prove it — the guard
% stays and `card(as) = 0` stays SAT via the in-domain witness default.
class A(var 0..5: x; var 1..6: y = 6 div (x + 1));
var set(0..2) of new A: as;
constraint card(as) = 0;
solve satisfy;
output ["card=", show(card(as)), "\n"];
