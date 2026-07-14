% Array-typed attribute on a VAR-REACHED class (var set(1..1) of new):
% validation admits the uniform single-dimension shape, storage keeps the
% array-of-var-elements field, and the iteration reads it back through the
% var identity (erase_enum(a) is a var index into A_objects, taking the
% per-position fast path). sum = 1 with xs[3] = 1 forces xs = [0, 0, 1].
class A ( array[1..3] of var 0..2: xs; );
var set(1..1) of new A: as_;
constraint forall(a in as_)(sum(a.xs) = 1 /\ a.xs[3] = 1);
output ["xs=", show([a.xs[j] | a in as_, j in 1..3])];
solve satisfy;
