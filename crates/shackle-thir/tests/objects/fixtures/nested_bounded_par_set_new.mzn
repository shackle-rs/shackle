int: n = 2;

class A (
	int: a;
	set(0..n) of new B: bs;
);

class B (
	int: b;
);

set of new A: as = [
	(a: 1, bs: [(b: 1), (b: 2)]),
];

solve satisfy;