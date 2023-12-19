v = (
	foo,
	bar,
	three +
		two +
		one +
		four +
		five +
		six +
		three +
		two +
		one +
		four +
		five +
		six,
	one,
	two,
	three,
	one,
	two,
);
v = (
	a: 1,
	b: "foo" ++
		"bar" ++
		"foo" ++
		"bar" ++
		"foo" ++
		"bar" ++
		"foo" ++
		"bar" ++
		"foo" ++
		"bar" ++
		"foo" ++
		"bar" ++
		"foo" ++
		"bar" ++
		"foo" ++
		"bar",
	c: bar,
);
v = {
	1,
	2.5,
	5,
	three +
		two +
		one +
		four +
		five +
		six +
		three +
		two +
		one +
		four +
		five +
		six,
	one,
	two,
	three,
	one,
	two,
};
v = [
	hello,
	world,
	1,
	2,
	three +
		two +
		one +
		four +
		five +
		six +
		three +
		two +
		one +
		four +
		five +
		six,
	one,
	two,
	three,
	one,
	two,
];
v = [
	a: hello,
	(
		hello,
		world,
		1,
		2,
		hello,
		world,
		1,
		2,
		hello,
		world,
		1,
		2,
		hello,
		world,
		1,
		2,
	): three +
		two +
		one +
		four +
		five +
		six +
		three +
		two +
		one +
		four +
		five +
		six,
	c: three +
		two +
		one +
		four +
		five +
		six +
		three +
		two +
		one +
		four +
		five +
		six,
	d: three +
		two +
		one +
		four +
		five +
		six +
		three +
		two +
		one +
		four +
		five +
		six,
];
v = [|
	hello, world |
	one, two
|];
v = [|
	a: b: |
	c: three +
		two +
		one +
		four +
		five +
		six +
		three +
		two +
		one +
		four +
		five +
		six,
	one,
	world |
	d: one, two
|];
v = [
	three +
		two +
		one +
		four +
		five +
		six +
		three +
		two +
		one +
		four +
		five +
		six |
		i in bar,
		j in bar,
		j in bar,
		j in bar,
		j in bar where one,
		j in bar,
		j in bar,
		j in bar,
		j in bar,
		j in bar where one,
		j in bar,
		j in bar,
		j in bar,
		j in bar,
];
v = {
	three +
		two +
		one +
		four +
		five +
		six +
		three +
		two +
		one +
		four +
		five +
		six |
		i in bar,
		j in bar,
		j in bar,
		j in bar,
		j in bar where one,
		j in bar,
		j in bar,
		j in bar,
		j in bar,
		j in bar where one,
		j in bar,
		j in bar,
		j in bar,
		j in bar,
};
v =
	if one then
		two
	elseif three then
		[
			one,
			two,
			three,
			one,
			two,
			three,
			one,
			two,
			three,
			one,
			two,
			three,
			one,
			two,
			three,
			one,
			two,
			three,
		]
	else
		five
	endif;
v =
	foo(
		one,
		two,
		three,
		one,
		two,
		three +
			two +
			one +
			four +
			five +
			six +
			three +
			two +
			one +
			four +
			five +
			six +
			three +
			two +
			one +
			four +
			five +
			six,
		one,
		two,
		three,
		one,
		two,
		three,
		one,
		two,
		three,
		one,
		two,
		three,
	);
v =
	foo[
		one,
		two,
		three,
		one,
		two,
		three,
		one,
		two,
		three,
		one,
		two,
		three,
		one,
		two,
		three,
		one,
		two,
		three,
	];
v =
	foo (
		i in bar,
		j in bar,
		j in bar,
		j in bar,
		j in bar where one,
		j in bar,
		j in bar,
		j in bar,
		j in bar,
		j in bar where one,
		j in bar,
		j in bar,
		j in bar,
		j in bar,
	) (baz);
v = "hello: \(hello)";
v =
	(
		1 +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello +
			hello
	) :: world(hello);
solve
	:: hello(
		a_long_name +
			another_long_name +
			another_long_name +
			another_long_name +
			another_long_name +
			another_long_name,
	) minimize
	one;
