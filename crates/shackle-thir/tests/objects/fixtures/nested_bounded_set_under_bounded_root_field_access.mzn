int: m;
int: n;

class B (
  0..4: y;
);

class A (
  var set(0..n) of new B: bs;
);

var set(0..m) of new A: as;
constraint forall([b.y >= 0 | a in as, b in a.bs]);
solve satisfy;
