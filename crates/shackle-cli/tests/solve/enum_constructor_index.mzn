% The same unresolved constructor, but reached through an array's index set
% rather than through a value. This is the path that used to panic, since an
% array shows its indices whenever they are not a one-based integer range.
enum Foo = Bar(1..2);
array[Foo] of var 1..2: a;
constraint a[Bar(1)] < a[Bar(2)];
solve satisfy;
