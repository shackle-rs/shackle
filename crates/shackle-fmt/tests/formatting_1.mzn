annotation foo; % foo
annotation bar(
	int: x, /* bar */
);
x = 1;
% hello
constraint true;
1..3: x;
var 1..3: y = x + 1;
enum Foo = {A, B, C} ++ D(1..3);
enum Bar = _(1..10);
function var int: hello(var int: x);
function var int: hello(var int: x) = x + 3;
include "foo.mzn";
output ["hello\n"];
output :: "section" ["hello\n"];
predicate world(var bool);
predicate world(var bool: b) = not b;
test world(int: i) = i > 10;
solve minimize
	/* solve comment */
	foo + /* hello */
		bar;
type X = set of int;
