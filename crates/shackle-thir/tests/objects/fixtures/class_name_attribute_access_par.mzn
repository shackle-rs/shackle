% `A.x` on a par class is the attribute of every object, indexed by the object.
% It lowers to the same indexed comprehension as `[a: a.x | a in A]`.
class A ( int: x );
new A: a1 = (x: 1);
new A: a2 = (x: 2);
array [A] of int: ys = A.x;
any: zs = A.x;
output [show(ys), " ", show(zs), " ", show(ys[a2])];
solve satisfy;
