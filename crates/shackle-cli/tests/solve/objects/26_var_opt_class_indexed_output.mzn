% `A.x` on a var set of objects is an array of optional values, indexed by the
% objects. It can be output, and shows the objects that exist.
class A ( var 1..3: x );
var set(1..2) of new A: as;
array [A] of var opt int: ys :: output = A.x;
constraint card(as) = 1;
constraint forall (a in A) (a.x = 2);
output ["\(ys)\n"];
solve satisfy;
