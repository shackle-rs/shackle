% Nothing here is an all_different
array[1..4] of var 1..4: x;
constraint x[1] < x[2];
constraint x[3] != x[4];
