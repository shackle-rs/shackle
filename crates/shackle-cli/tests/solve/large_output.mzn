% A solution far larger than the buffer of the pipe the interpreter writes to,
% which deadlocks a runner that waits for the interpreter to exit before it
% starts reading. Roughly 900KB of output; a pipe holds 64KB.
var 1..3: x;
constraint x = 2;
output [ "line \(i) of a large output, x=\(x)\n" | i in 1..20000 ];
solve satisfy;
