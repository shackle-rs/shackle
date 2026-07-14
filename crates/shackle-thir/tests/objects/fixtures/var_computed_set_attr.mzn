% A var-reached class (`var new A`) with a computed *set* attribute
% (`var set of int: z = {x, 2*x}`). A free `var set of int` decision is illegal
% (var sets need an upper bound or RHS), so this field can only be realised as a
% defined alias in the storage-reconstruction comprehension — never as a free
% `_storage` decision. `x` is var-forced by `var new`, so the computed set is
% itself var; the reconstruction declares the alias with the varified storage
% type so its column matches how the field is used elsewhere.
class A (
  0..4: x;
  var set of int: z = {x, 2 * x};
);

var new A: a;
constraint a.x = 3;
solve satisfy;
output ["x=", show(a.x), " z=", show(a.z), "\n"];
