% The model's output item must be rendered, rather than the values of its
% variables being dumped: the marker text below cannot appear in a dump.
var 1..3: x;
constraint x = 2;
output ["DISTINCTIVE-MARKER x is \(x)\n"];
solve satisfy;
