% An objective over an object-set field aggregate: `minimize sum(a in as)(a.x)`
% projects a class field and aggregates it over a var class set, a path no
% `solve satisfy` case reaches.
%
% `card(as) >= 1` is the binding constraint: without it the minimiser would pick
% the empty set for a trivial 0. With it, the optimum is a single realised member
% at the class invariant's floor, giving objective 1. The output prints ONLY the
% objective value, which is invariant across all optimal solutions.

int: n = 2;

class A (
  var 0..2: x;
  constraint this.x >= 1;
);

var set(0..n) of new A: as;
constraint card(as) >= 1;

solve minimize sum(a in as)(a.x);

output ["obj=" ++ show(sum(a in as)(a.x))];
