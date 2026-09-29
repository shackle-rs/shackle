% Output of objects and references by string id. The `var new` root `b` is
% rendered as a record `(id: .., <field>: ..)`: its reference field `a` as the
% id of the object it refers to, its var field fixed. The var reference
% `chosen`, the opt reference `maybe` and the array of references `pair` are
% rendered as ids (`maybe` as `<>` when absent). `show` and string
% interpolation of a reference print its id.
class A ( 1..3: x; );
class B ( A: a; var 1..2: y; );
set of new A: as = [(x: 1), (x: 2)];
var new B: b;
var A: chosen;
var opt A: maybe;
array [1..2] of var A: pair;
output ["chosen=", show(chosen), " b.a=\(b.a)\n"];
solve satisfy;
