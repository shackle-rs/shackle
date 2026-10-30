% Arrays of `var opt bool`, also inside a tuple. Each element is created
% wrapped in a 1-tuple, which needs an unwrapping function for opt bools too.
array [1..3] of var opt bool: w;
tuple(array [1..2] of var opt bool, var 1..2): t;
constraint occurs(w[1]) /\ deopt(w[1]) /\ absent(w[2]) /\ occurs(w[3]) /\ not deopt(w[3]);
constraint absent(t.1[1]) /\ occurs(t.1[2]) /\ deopt(t.1[2]) /\ t.2 = 2;
solve satisfy;
