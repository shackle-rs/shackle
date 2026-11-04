% `forall`, `exists` and `clause` over arrays that aren't literals at the call,
% and negated or reified: each case has its own variables, and only `b4`'s case
% has more than one solution.
predicate all_of(array [int] of var bool: bs) = forall(bs);
var 0..2: x1;
var 0..2: y1;
constraint all_of([x1 = 1, y1 = 2]);
var 0..2: x2;
var 0..2: y2;
array [int] of var bool: bs2 = [x2 = 1, y2 = 2];
constraint forall(bs2);
var 0..2: x3;
constraint not exists([x3 = 1, x3 = 2]);
var bool: b4;
var 0..2: x4;
constraint b4 <-> exists([x4 = 1, x4 = 2]);
var bool: b5;
var 0..2: x5;
constraint not clause([x5 = 1, b5], [x5 = 2]);
output ["\(x1) \(y1) \(x2) \(y2) \(x3) \(b4) \(x4) \(b5) \(x5)\n"];
