% n-queens with pairwise constraints. The diagonals are not found: their
% arguments (`q[i] + i`) are expressions, not arrays of the model.
int: n = 8;
array[1..n] of var 1..n: q;
constraint forall(i, j in 1..n where i < j)(
  q[i] != q[j] /\ q[i] + i != q[j] + j /\ q[i] - i != q[j] - j
);
