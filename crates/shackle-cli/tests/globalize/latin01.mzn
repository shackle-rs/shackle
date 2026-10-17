% A Latin square as a 0/1 array: b[i, j, k] = 1 if cell (i, j) has value k.
% The initial pass finds integer viewpoints of b.
int: n = 3;
array[1..n, 1..n, 1..n] of var 0..1: b;
constraint forall(i, j in 1..n)(sum(k in 1..n)(b[i, j, k]) = 1);
constraint forall(i, k in 1..n)(sum(j in 1..n)(b[i, j, k]) = 1);
