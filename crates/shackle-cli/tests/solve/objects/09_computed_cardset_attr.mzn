% Card-bounded computed set attribute: `z`'s RHS at the pinned defaults
% (x1 = x2 = 0) is {0}, card 1 — OUTSIDE the declared card bound 2..2. The
% card bound must therefore be relocated out of the storage record domain and
% re-imposed on realised objects only, or the `card=0` solution disappears.
%
% Hand count: as = {} gives one fully-pinned solution; the realised A needs
% card(z) = 2, i.e. x1 != x2. Three solutions.
class A(var 0..1: x1; var 0..1: x2; var set(2..2) of 0..1: z = {x1} union {x2});

var set(0..1) of new A: as;

output ["card=" ++ show(card(as)) ++ " codes=" ++ show([2 * a.x1 + a.x2 | a in as])];
solve satisfy;
