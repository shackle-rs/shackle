int: m;
int: n;

class B (
  0..2: y;
);

class A (
  var set(0..n) of new B: bs;
  var set(0..n) of new B: cs;
);

var set(0..m) of new A: as;
constraint forall([
  sum([
    let { var set of B: owned = a.bs union a.cs } in bool2int(b in owned) | a in as
  ]) = 1
  | b in B
]);
solve satisfy;
