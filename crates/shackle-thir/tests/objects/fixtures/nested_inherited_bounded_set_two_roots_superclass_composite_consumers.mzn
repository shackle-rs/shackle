int: n;

class A (
  0..4: x;
);

class B extends A (
  0..4: y;
);

class Root (
  var set(0..n) of new B: bs;
);

var new Root: r1;
var new Root: r2;
var set of B: bs1 = r1.bs;
var set of B: bs2 = r2.bs;
var set of A: as = A;
constraint forall([b.y >= 0 | b in bs1]);
constraint forall([b.y >= 0 | b in bs2]);
constraint forall([a.x >= 0 | a in as]);
solve satisfy;