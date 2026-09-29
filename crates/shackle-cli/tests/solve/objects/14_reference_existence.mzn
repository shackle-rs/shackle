% Exactly one `A` exists, so every reference to an `A` refers to it: the
% scalar, set and array references at the top level and the reference
% attribute `b.a`. `var set of A` has a finite bound (it used to be rejected
% as `var set of int`).
class A ( 1..3: x; );
class B ( var A: a; );
var set(0..3) of new A: as;
var new B: b;
var A: one;
var set of A: picked;
array [1..2] of var A: two;
constraint card(as) = 1;
constraint card(picked) = 1;
output ["as=\(as) one=\(one) picked=\(picked) two=\(two) b.a=\(b.a)\n"];
solve satisfy;
