% `A.v` for a var attribute of par objects is an array of decisions, and is
% fixed when it is shown in the output.
class A ( int: x; var 1..3: v );
new A: a1 = (x: 1);
new A: a2 = (x: 2);
array [A] of var int: vs = A.v;
constraint sum(vs) = 4;
output [show(A.v)];
solve satisfy;
