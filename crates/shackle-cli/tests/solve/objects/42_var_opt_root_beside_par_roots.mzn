% A `var opt new` root of a class that par roots also reach: the optional
% object is absent in one solution and present in the other.
class Leaf ( 1..3: v );
new Leaf: a = (v: 1);
set of new Leaf: as = [(v: 2)];
var opt new Leaf: b;
constraint occurs(b) -> b.v = 3;
output ["\(Leaf) \(b)\n"];
