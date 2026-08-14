% Smallest var-path smoke test: a single `var new` introduction with one
% bounded var attribute and a constraint on a field projection.
% Solutions project to x in {1, 2}.

class A (
  var 0..2: x;
);

var new A: a;

constraint a.x >= 1;

output ["x=" ++ show(a.x)];
solve satisfy;
