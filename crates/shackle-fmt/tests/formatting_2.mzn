v = 1;
v = 1.5;
v = (foo,);
v = (foo, bar);
v = (a: 1, b: "foo");
v = {1, 2.5, 5};
v = true;
v = false;
v = "foo";
v = hello;
v = <>;
v = infinity;
v = _;
v = [hello, world, 1, 2];
v = [a: hello, b: world, c: 1, d: 2];
v = [|
	hello, world |
	one, two
|];
v = [|
	a: b: |
	hello, world |
	one, two
|];
v = [|
	a: b: |
	c: hello, world |
	d: one, two
|];
v = [|
	a: hello, world |
	b: one, two
|];
v = foo[1, two];
v = [hello | i in 1..3, j = foo where blah];
v = {hello | i in 1..3, j = foo where blah};
v = if one then two elseif three then four else five endif;
v = foo(1, 2, 3);
v = -hello;
v = hello + world;
v = hello + to + the + world;
v = foo[3..<];
v = foo (i in bar) (baz);
v = "hello: \(hello)";
v =
	case foo of
		Foo(x) => x,
		Bar => 0,
		<> => 1,
		(foo, bar) => 2,
		(x,) => x,
		(one: a, two: b) => a + b,
		true => 1,
		hello => hello,
		"foo" => 2,
		1 => 1,
		-1 => 2,
		_ => 4,
	endcase;
v = let { var 1..3: x = hello } in true;
v =
	let {
		var 1..3: x = hello;
		var int: y = x * 2;
	} in x + y;
v = foo.1;
v = foo.bar;
v = lambda (int: x) => x + 1;
v = lambda int: (int: x) => x + 1;
v = (1 + hello) :: world;
solve :: blah maximize
	a_long_name +
		another_long_name +
		another_long_name +
		another_long_name +
		another_long_name +
		another_long_name;
