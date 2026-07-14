class A (
  var 0..2: x1;
  var 0..2: x2;
  var 3..4: z = x1 + x2;
);

var set(0..2) of new A: as;
constraint card(as) = 0;
output ["card=", show(card(as)), "\n"];
solve satisfy;
