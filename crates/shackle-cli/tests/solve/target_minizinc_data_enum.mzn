% An enum defined in data is given to MiniZinc as its definition
enum E;
E: e;
var E: x;
constraint x = e;
solve satisfy;
