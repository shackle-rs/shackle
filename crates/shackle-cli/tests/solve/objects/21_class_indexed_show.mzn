% `show`, `showJSON` and string interpolation of an array indexed by a class
% key it by the ids of the objects. Only the objects that exist are shown, not
% every potential object of the var set.
class A ( var 1..3: x );
var set(1..2) of new A: as;
array [A] of var 0..10: scores;
array [A, 1..2] of var 0..1: grid;
constraint card(as) = 1;
constraint forall(a in A) (a.x = 2 /\ scores[a] = a.x + 4);
constraint forall(a in A, j in 1..2) (grid[a, j] = bool2int(j = 1));
output [show(scores), " ", showJSON(scores), " grid=\(grid)\n"];
solve satisfy;
