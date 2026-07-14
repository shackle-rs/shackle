% Contradictory twin of array_field_var_index_var_reached: the read-back
% constraint must BIND (sum(xs) = 0 contradicts xs[3] = 1). Guards against
% the constraint silently flattening away.
class A ( array[1..3] of var 0..2: xs; );
var set(1..1) of new A: as_;
constraint forall(a in as_)(sum(a.xs) = 0 /\ a.xs[3] = 1);
solve satisfy;
