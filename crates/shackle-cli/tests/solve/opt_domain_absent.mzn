% The domain of a defined `var opt` declaration is checked on its value, which
% must not rule out an absent value: this used to be unsatisfiable when `x` was
% absent.
var opt 1..3: x :: no_output;
var opt 1..3: y :: no_output = x;
output ["\(y)\n"];
solve satisfy;
