% A reference may only refer to an existing object. No `A` exists here, so both
% the top-level reference `one` and the reference attribute `b.a` are
% unsatisfiable (they used to point at an `A` that does not exist).
class A ( 1..3: x; );
class B ( var A: a; );
var set(0..3) of new A: as;
var new B: b;
var A: one;
constraint card(as) = 0;
solve satisfy;
