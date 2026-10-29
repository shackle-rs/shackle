% The lowering reads the objects in a comprehension whose generators are named
% `p` and `input`. A computed attribute referring to the user's own `p` and
% `input` must still mean those: z is 111 and 112, not computed from the
% generators.
class A ( 1..5: x; int: z = x + p + input );
int: p = 10;
int: input = 100;
set of new A: as = [(x: 1), (x: 2)];
output ["\([a.z | a in as])\n"];
solve satisfy;
