class A (
  0..4: x;
);

class B extends A (
  0..4: y;
);

var new B: b;
constraint b.x <= b.y;
solve satisfy;
