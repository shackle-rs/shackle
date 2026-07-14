class B (
	0..2: y;
);

int: n;
constraint let {
	var set(0..n) of new B: bs;
} in card(bs) >= 0;
solve satisfy;