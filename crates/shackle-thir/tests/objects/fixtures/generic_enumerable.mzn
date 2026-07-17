class A ();
A: a;
A: b;
opt A: oa;
set of A: span = a..b;
constraint a in enum_of(a);
constraint b in span;
constraint absent(oa) \/ deopt(oa) in enum_of(oa);
solve satisfy;
