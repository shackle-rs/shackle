% Three pairwise disequalities form an all_different (needs -M 3)
var 1..3: a;
var 1..3: b;
var 1..3: c;
constraint a != b;
constraint b != c;
constraint a != c;
