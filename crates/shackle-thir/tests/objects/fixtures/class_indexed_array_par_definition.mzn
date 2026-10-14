% Class-indexed arrays defined by an indexed comprehension over a par class,
% with a declared `array [A]` type and with an inferred (`any`) one. Both are
% indexed by `A_potential`.
class A ( int: x );
new A: a1 = (x: 1);
new A: a2 = (x: 2);
array [A] of int: ys = [a: a.x | a in A];
any: zs = [a: a.x + 1 | a in A];
output [show(ys), " ", show(zs), " ", show(ys[a2])];
solve satisfy;
