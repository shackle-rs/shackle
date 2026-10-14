% Class-indexed arrays accessed by a var reference, by a reference attribute,
% and by a generator in a two-dimensional `array [A, 1..2]`.
class A ( 0..10: x );
class B ( A: r; var 1..2: y );
set of new A: as = [(x: 3), (x: 4)];
var new B: b;
var A: chosen;
array [A] of var 0..10: scores;
array [A, 1..2] of var 0..1: grid;
constraint scores[chosen] = 7;
constraint scores[b.r] = b.r.x;
constraint sum(a in A, j in 1..2) (grid[a, j]) = 1;
output [show(scores), " chosen=\(chosen)"];
solve satisfy;
