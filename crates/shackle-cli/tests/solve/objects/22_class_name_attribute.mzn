% `A.x` on a class name is the attribute of every object, indexed by the
% object. For a var set of objects the absent ones have no value, and the
% output shows only the objects that exist.
class A ( 1..5: x );
class B ( var 1..3: y );
set of new A: as = [(x: 1), (x: 3)];
var set(1..3) of new B: bs;
constraint card(bs) = 2;
constraint forall(b in B) (b.y = 2);
constraint sum(A.x) = 4;
output [show(A.x), " ", show(B.y), "\n"];
solve satisfy;
