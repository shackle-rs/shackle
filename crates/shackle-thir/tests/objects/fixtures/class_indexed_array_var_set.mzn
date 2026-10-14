% A class-indexed array over a class whose actual set is a decision. The array
% covers every potential object; the constraint ranges over the existing ones.
class A ( var 1..3: x );
var set(1..2) of new A: as;
array [A] of var 0..10: scores;
constraint forall(a in A) (scores[a] >= a.x);
output [show(scores), " ", show(as)];
solve satisfy;
