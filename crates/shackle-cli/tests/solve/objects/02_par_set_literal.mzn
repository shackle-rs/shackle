% Par-path smoke test: a `set of new` introduction from a constructor
% literal list, with a par field aggregate in the output and a var that
% depends on a par field read. Solutions project to sum=14 with y in {0, 1}.

class A (
  int: x;
);

set of new A: as = [(x: 3), (x: 11)];

var 0..1: y;

constraint y <= max(a in as)(a.x);

output ["sum=" ++ show(sum(a in as)(a.x)) ++ " y=" ++ show(y)];
solve satisfy;
