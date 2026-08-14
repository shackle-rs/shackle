% A class `A` with a single `var 0..2: x` attribute, introduced through the
% bounded-set-of-new path: `var set(0..n) of new A: as`. The canonical
% projection is `(card(as), sum(a in as)(a.x))` — both aggregates are
% symmetry-invariant over which potentials are realised.

int: n = 2;

class A (
  var 0..2: x;
);

var set(0..n) of new A: as;

output [
  "card=" ++ show(card(as)) ++
  " sum=" ++ show(sum(a in as)(a.x))
];
solve satisfy;
