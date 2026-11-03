% A `var opt new` root leaves its class's set of objects free, bounded by the
% potential objects. `Base` also holds the par root `bs`.
class Base ( 1..3: v );
class Leaf extends Base ();
set of new Base: bs = [(v: 3)];
var opt new Leaf: one;
constraint occurs(one) -> one.v < 3;
output ["\(Base) \([b.v | b in Base]) \(Leaf)\n"];
