% Arrays indexed by a class are output keyed by the ids of the objects, and
% several dimensions by tuples of an id and the other indices. Objects in the
% elements are output as ids too.
class A ( 0..10: x );
class B ( 1..2: y );
set of new A: as = [(x: 3), (x: 4)];
set of new B: bs = [(y: 1), (y: 2)];
array [A] of var 0..10: scores;
array [A, 1..2] of var 0..1: grid;
array [A] of var B: partner;
constraint forall(a in A) (scores[a] = a.x + 1);
constraint forall(a in A, j in 1..2) (grid[a, j] = bool2int(a.x = 3 /\ j = 2));
constraint forall(a in A) (partner[a].y = a.x - 2);
solve satisfy;
