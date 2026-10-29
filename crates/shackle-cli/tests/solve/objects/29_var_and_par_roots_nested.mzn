% A par set root and a var singular root of one class, each object with a
% nested child: the children of each root are numbered within that root.
class Leaf ( 1..3: v );
class Node ( new Leaf: main );
set of new Node: nodes = [(main: (v: 1)), (main: (v: 2))];
var new Node: x;
constraint x.main.v = 3;
output ["\([n.main.v | n in nodes]) \(x.main.v)\n"];
solve satisfy;
