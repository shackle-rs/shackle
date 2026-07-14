int: n;

class B (
  0..4: y;
);

class A (
  var set(0..n) of new B: bs;
);

var new A: a;
constraint forall([b.y >= 0 | b in a.bs]);
solve satisfy;