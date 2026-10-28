% `A.x` on a class whose actual set is a decision. It is indexed by every
% potential object, and its elements are absent for the objects that do not
% exist.
class A ( var 1..3: x );
var set(1..2) of new A: as;
any: ys = A.x;
constraint card(as) = 2;
constraint sum(y in ys) (if occurs(y) then deopt(y) else 0 endif) = 5;
output [show(as)];
solve satisfy;
