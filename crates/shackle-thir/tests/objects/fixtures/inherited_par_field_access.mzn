class A (
  int: x;
);

class B extends A (
  int: y;
);

new B: b = (x: 1, y: 2);
constraint b.x <= b.y;
solve satisfy;