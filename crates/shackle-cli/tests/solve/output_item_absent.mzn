% Without an output item there is nothing to render, so the solution falls
% back to a dump of the values of the model's variables.
var 1..3: x;
constraint x = 2;
solve satisfy;
