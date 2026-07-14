% Ragged array attribute (per-object length `1..len`) read through a var
% receiver: the HIR typer cannot element-wise-varify it (the result's index
% set would be decision-dependent), so the read is rejected with
% IllegalType. The par class DECLARATION itself stays legal — only the
% var-receiver read is refused.
class A ( 1..3: len; array[1..len] of var 0..2: xs; );
set of new A: as = [(len: 1), (len: 2)];
var A: chosen;
constraint sum(chosen.xs) = 1;
solve satisfy;
