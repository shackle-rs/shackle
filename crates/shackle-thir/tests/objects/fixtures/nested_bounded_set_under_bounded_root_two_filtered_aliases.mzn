int: m;
int: n;

class B (
	0..2: y;
);

class A (
	var set(0..n) of new B: bs;
	var set(0..n) of new B: cs;
);

var set(0..m) of new A: as;
constraint forall([
	let {
		var set of B: owned = a.bs union a.cs;
		var set of B: owned_bs = {x | x in owned where x in a.bs};
		var set of B: owned_cs = {x | x in owned where x in a.cs};
	} in card(owned_bs) + card(owned_cs) = card(a.bs) + card(a.cs)
	| a in as
]);
solve satisfy;