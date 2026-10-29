% Arrays of optional values are shown like MiniZinc shows them, with absent
% values as `<>`, in the default output.
enum E = {e1, e2};
array [E] of var opt 1..3: y;
array [1..3] of var opt 1..2: z;
constraint absent(y[e2]) /\ y[e1] = 2;
constraint occurs(z[1]) /\ absent(z[2]) /\ absent(z[3]);
solve satisfy;
