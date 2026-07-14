class B (
  0..4: y;
);

class A (
  new B: b;
);

class C extends A (
  0..4: z;
);

var new C: c;
constraint c.b.y <= c.z;
solve satisfy;
