% Ragged array attribute on a var-reached class: rejected at object
% validation (the dimension references a sibling attribute, which the
% var-reach cascade forces to var — and a dimension must be par).
class A ( 1..3: len; array[1..len] of var 0..2: xs; );
var set(0..1) of new A: as_;
solve satisfy;
