int: n;

class B (
  0..4: y;
);

class A (
  var set(0..n) of new B: bs;
);

var new A: a1;
var new A: a2;
var set of B: bs = B;
constraint forall([b.y >= 0 | b in bs]);
solve satisfy;