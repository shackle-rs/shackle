class B (
  0..4: y;
);

class A (
  new B: b;
  0..4: x;
);

var new A: a;
constraint a.b.y >= 0;
solve satisfy;
