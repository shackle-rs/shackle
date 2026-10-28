% `C.f` for an inherited attribute read through the subclass, and for an array
% attribute.
class A ( int: x );
class B extends A ( array [1..2] of int: ys );
new B: b1 = (x: 1, ys: [1, 2]);
new B: b2 = (x: 2, ys: [3, 4]);
array [B] of int: xs = B.x;
array [B] of array [int] of int: yss = B.ys;
output [show(xs), " ", show(yss)];
solve satisfy;
