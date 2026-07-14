class A (
  int: x;
  constraint this.x >= 0;
);

class B extends A (
  int: y;
  constraint this.x <= this.y;
);

new B: b = (x: 1, y: 2);
solve satisfy;
