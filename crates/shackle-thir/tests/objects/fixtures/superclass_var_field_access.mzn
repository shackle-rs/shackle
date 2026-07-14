class A (
  0..2: x;
);

class B extends A (
  0..2: y;
);

var new B: b;
var A: a = b;
constraint a.x <= 2;
solve satisfy;