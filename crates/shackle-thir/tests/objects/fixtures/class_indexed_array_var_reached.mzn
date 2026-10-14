% A class-indexed array of decisions over a var-reached class with a par actual
% set. `array [A]` is indexed by the potential universe `A_potential`, and
% `scores[a]` indexes it by the par identity a generator over `A` binds.
class A (
  0..10: x;
);

var new A: a1;
var new A: a2;

array[A] of var 0..10: scores;

constraint forall(a in A) (scores[a] <= a.x);
solve satisfy;
