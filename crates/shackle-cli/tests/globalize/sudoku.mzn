% Rows and columns of a 4x4 Latin square, written with pairwise disequalities
int: n = 4;
array[1..n, 1..n] of var 1..n: x;
constraint forall(r in 1..n, c1, c2 in 1..n where c1 < c2)(x[r, c1] != x[r, c2]);
constraint forall(c in 1..n, r1, r2 in 1..n where r1 < r2)(x[r1, c] != x[r2, c]);
