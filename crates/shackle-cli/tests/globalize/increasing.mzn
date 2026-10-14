% A sorted chain
int: n = 6;
array[1..n] of var 0..9: x;
constraint forall(i in 1..n - 1)(x[i] <= x[i + 1]);
