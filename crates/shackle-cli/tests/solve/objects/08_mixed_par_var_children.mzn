% Mixed par+var reach: a class with an object-typed stored field reached BOTH
% by a par singular top-level instantiation and by a `var new` introduction.
% The par root's child joins B's derived actual set through the par
% contribution block image; the var root's child comes through the recorded
% field introduction.
class B ( 2..3: x; );
class A ( set(1..1) of new B: children; );

new A: a = (children: [(x: 2)]);
var new A: a2;

solve satisfy;

output ["bx=", show([b.x | b in B]), " cardB=", show(card(B)), "\n"];
