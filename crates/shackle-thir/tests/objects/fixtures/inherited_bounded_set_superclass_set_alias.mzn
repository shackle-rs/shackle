class A (
  0..4: x;
);

class B extends A (
  0..4: y;
);

int: n;
var set(0..n) of new B: bs;
var set of A: as = bs;
constraint forall([a.x >= 0 | a in as]);
solve satisfy;