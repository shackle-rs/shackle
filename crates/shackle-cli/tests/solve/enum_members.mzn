% The members of an enumerated type defined in the model have to be resolved
% from the model, so that values of that type can be shown by name. This output
% is byte-identical to MiniZinc's own.
enum Colour = {Red, Green, Blue};
var Colour: c;
solve satisfy;
