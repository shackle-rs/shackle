% Pairwise disequality over an array, sized by the data file
int: n;
array[1..n] of var 1..n: x;
constraint forall(i, j in 1..n where i < j)(x[i] != x[j]);
