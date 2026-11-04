% Par `opt new` roots read from data: a record (with ids, or without one for a
% generated id) or `<>`, as in the model.
class Leaf ( 1..3: v );
class Tree ( set of new Leaf: leaves; new Leaf: main; opt new Leaf: spare );
opt new Tree: t :: output;
opt new Tree: u :: output;
