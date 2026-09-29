% References to a class whose existence is a decision (`A`, introduced by a
% `var set(..) of new` root) are constrained to its actual set, so they cannot
% point at objects that don't exist: the scalar, opt, set and array references
% at the top level, and the reference attributes of `B`'s existing objects.
% References to `C`, whose objects all exist, need no constraint. A class in
% element position (`set of A`) is bounded by the potential enum, which keeps
% it finite once enums are erased.
class A ( 1..3: x; );
class C ( 1..3: y; );
class B ( var A: a; var set of A: as_; C: c; );
var set(0..3) of new A: as;
set of new C: cs = [(y: 1)];
var new B: b :: no_output;
var A: one :: no_output;
var opt A: maybe :: no_output;
var set of A: picked :: no_output;
array [1..2] of var A: two :: no_output;
var C: other :: no_output;
var set of C: others :: no_output;
solve satisfy;
