% A class without objects has an empty set of objects: `Leaf` has none, and
% `Hold` is only the type of a reference attribute.
class Leaf ( 1..3: v );
class Hold ( Leaf: pick );
class Item ( 1..3: w );
set of new Item: items = [(w: 1), (w: 2)];
var 1..3: x;
constraint x = sum (i in items) (i.w);
output ["\(Leaf) \(Hold) \([i.w | i in Item]) \(x) \(card(Leaf))\n"];
