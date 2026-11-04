% Values of an enumerated type shown in an output item, par and var, alone and
% in an array.
enum E = {a, b, c};
E: s = c;
var E: e;
var 1..2: x;
constraint e != s /\ e != a;
output ["\(s) \(e) \([s, e]) \(x)\n"];
