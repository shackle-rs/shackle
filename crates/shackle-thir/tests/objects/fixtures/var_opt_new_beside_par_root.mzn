% A `var opt new` root of a class that a par root also reaches. Its identity
% is defined like the root's alone, `b = if Leaf_occ_1(1) in Leaf then
% Leaf_occ_1(1) else <> endif`; the class set is free above the par root's
% object.
class Leaf(1..3: v);
new Leaf: a = (v: 1);
var opt new Leaf: b;
constraint occurs(b) -> b.v = 3;
