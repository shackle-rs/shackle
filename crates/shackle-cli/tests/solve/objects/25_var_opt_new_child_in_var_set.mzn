% A `var opt new` child of objects in a var set. The domain check on each
% object's child identity must allow the child to be absent: this used to be
% unsatisfiable, as the objects outside the set have no child.
class B ( var 1..3: y );
class A ( var opt new B: b );
var set(1..2) of new A: as;
constraint card(as) = 1;
output ["\(as) \([a.b | a in as]) \([b.y | b in B])\n"];
solve satisfy;
