% A par class with a chain of computed attributes, where each computed
% attribute's RHS depends on the previous one (`z = y + 4` depends on
% `y = x + 1`). The storage reconstruction must emit the generator aliases in
% declaration order so `y` is in scope when `z` is defined.
class A (
  int: x;
  int: y = x + 1;
  int: z = y + 4;
);

set of new A: a = [(x: 3), (x: 5)];
solve satisfy;
output [
  "xs=", show([p.x | p in a]), "\n",
  "ys=", show([p.y | p in a]), "\n",
  "zs=", show([p.z | p in a]), "\n",
];
