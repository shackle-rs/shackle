% Two var references to objects of the same class can be compared. A var
% reference is an enum value underneath, so this needs `=` on var enums.
class A ( 1..2: x );
set of new A: as = [(x: 1), (x: 2)];
var A: r;
var A: s;
constraint r = s;
constraint r.x = 2;
solve satisfy;
