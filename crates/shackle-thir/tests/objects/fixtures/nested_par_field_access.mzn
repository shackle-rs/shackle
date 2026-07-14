class B (
  int: y;
);

class A (
  new B: b;
  int: x;
);

new A: a = (b: (y: 1), x: 2);
constraint a.b.y >= 0;
solve satisfy;