% Inheritance: `B extends A` adds a second bounded var attribute, and both
% the inherited and the derived field are projected off the same handle.

class A (
  var 0..3: x;
);

class B extends A (
  var 0..3: y;
);

var new B: b;
constraint b.x <= b.y;
output [show(b.x) ++ " " ++ show(b.y)];
solve satisfy;
