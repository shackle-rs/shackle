int: n;

class B (
  0..4: y;
);

class A (
  var set(0..n) of new B: bs;
);

var new A: a1;
var new A: a2;
var set of B: bs1 = a1.bs;
var set of B: bs2 = a2.bs;
var set of B: all_bs = B;
constraint forall([b.y >= 0 | b in bs1]);
constraint forall([b.y >= 0 | b in bs2]);
constraint forall([b.y >= 0 | b in all_bs]);
solve satisfy;