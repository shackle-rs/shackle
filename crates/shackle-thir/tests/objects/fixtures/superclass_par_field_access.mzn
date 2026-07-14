class A (
  int: x;
);

class B extends A (
  int: y;
);

new B: b = (x: 1, y: 2);
A: a = b;
constraint a.x <= 2;
solve satisfy;