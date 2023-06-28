# MicroZinc specification

MicroZinc is a simple language used to define transformations to be performed
by the interpreter. It is a simplified subset of MiniZinc. The transformation
are represented in the language through the use of function definitions. A
function of type `var bool`, describing a relation, can be defined as a
native constraint. Otherwise, a function body must be provided for
rewriting. The function body can use a restriction of the MiniZinc expression
language. An important difference between MiniZinc and MicroZinc is that a
well-formed _MicroZinc does not contain partial expressions_. Any partiality in
the MiniZinc model must be explicitly expressed using total functions in
MicroZinc. As such, constraints introduced in MicroZinc let expressions can
be globally enforced. They are guaranteed to only constrain the decision
variables introduced in the same let expression.

### TODO: Remaining questions

- What annotation operations should be allowed in MicroZinc?
- The operational semantics for array access definition only works for 1-dimensional arrays. How can we generalise them for multiple dimensions?
- Add list of built-in functions

## Syntax

MicroZinc is defined using the following syntax,

\\[
\begin{array}{lcl}
\mathit{program} &::=&
\mathit{func}\* \\\\
\mathit{func} &::=&
\mathsf{function}~\mathit{typeinst}~\mathsf{:}~\mathit{ident}~\mathsf{(}\mathit{typing} [\mathsf{,}~ \mathit{typing}]\*\mathsf{)}~[\mathsf{=}~\mathit{letExpr}]\mathsf{;}\\\\&|&
\mathsf{predicate}~\mathit{ident}~\mathsf{(}\mathit{typing} [\mathsf{,}~ \mathit{typing}]\*\mathsf{)}~[\mathsf{=}~\mathit{letRoot}]\mathsf{;} \\\\
\mathit{typing} &::=&
\mathit{typeinst}~\mathsf{:}~\mathit{ident} \\\\
\\\\
\mathit{expr} &::=&
\mathit{letExpr} \\\\&|&
\mathit{ident}~\mathsf{(}\mathit{val} [ \mathsf{,}~ \mathit{val}]\*\mathsf{)} \\\\&|&
\mathsf{if}~\mathit{val}~\mathsf{then}~\mathit{letExpr}~[\mathsf{elseif}~\mathit{val}~\mathsf{then}~\mathit{letExpr}]\*~\mathsf{else}~\mathit{letExpr}~\mathsf{endif} \\\\&|&
\mathsf{[}[\mathit{tuple}\mathsf{:}]~\mathit{letExpr}~\mathsf{|}~\mathit{genExpr} [ \mathsf{,}~ \mathit{genExpr}]\*\mathsf{]} \\\\
\mathit{rootExpr} &::=&
\mathit{letRoot} \\\\&|&
\mathit{ident}~\mathsf{(}\mathit{val} [ \mathsf{,}~ \mathit{val}]\*\mathsf{)} \\\\&|&
\mathsf{if}~\mathit{val}~\mathsf{then}~\mathit{letRoot}~[\mathsf{elseif}~\mathit{val}~\mathsf{then}~\mathit{letRoot}]\*~\mathsf{else}~\mathit{letRoot}~\mathsf{endif} \\\\&|&
\mathsf{forall*root}\mathsf{(}\mathsf{[}\mathit{letRoot}~\mathsf{|}~\mathit{genExpr} [ \mathsf{,}~ \mathit{genExpr}]\*\mathsf{]}\mathsf{)} \\\\
\mathit{letExpr} &::=&
\mathit{val} \\\\&|&
\mathsf{let}~\mathsf{\\\{}\mathit{item}\*\mathsf{\\\}}~\mathsf{in}~\mathit{val} \\\\
\mathit{letRoot} &::=&
\mathsf{let}~\mathsf{\\\{}\mathit{item}\*\mathsf{\\\}}~\mathsf{in}~\mathsf{root} \\\\
\mathit{item} &::=&
\mathit{typing}~\mathsf{;} \\\\&|&
\mathit{typing}~\mathsf{=}~\mathit{expr}~\mathsf{;} \\\\&|&
\mathsf{constraint}~\mathit{rootExpr}~\mathsf{;} \\\\
\mathit{genExpr} &::=&
\mathit{ident}~\mathsf{in}~\mathit{letExpr}~[\mathsf{where}~\mathit{letExpr}] \\\\&|&
\mathit{ident}~\mathsf{=}~\mathit{letExpr} \\\\
\mathit{val} &::=&
\mathit{lit}~|~\mathit{range}~|~\mathit{tuple} \\\\&|&
\mathsf{\\{}\mathit{lit}~\mathsf{,}~ [\mathit{lit}~\mathsf{,}]\*\mathsf{\\}} \\\\&|&
\mathsf{array}\mathit{int}\mathsf{d(}\mathit{range}~\mathsf{,}~[\mathit{range}~\mathsf{,}]\*~\mathsf{[}\mathit{lit}~\mathsf{,}~ [\mathit{lit}~\mathsf{,}]\*\mathsf{])} \\\\&|&
\mathit{ident}~\mathsf{.}~\mathit{int} \\\\&|&
\mathit{ident}~\mathsf{[}\mathit{lit}\mathsf{]} \\\\
\mathit{tuple} &::=&
\mathsf{(}\mathit{lit}~\mathsf{,}~ [\mathit{lit}~\mathsf{,}]\*\mathsf{)} \\\\
\mathit{range} &::=&
\mathit{lit}\mathsf{..}\mathit{lit} \\\\&|&
\mathit{ident} \\\\
\mathit{lit} &::=&
\mathit{bool} \\\\&|&
\mathit{int} \\\\&|&
\mathit{float} \\\\&|&
\mathit{str} \\\\&|&
\mathit{ident} \\\\
\\\\
\mathit{bool} &::=&
\mathsf{true}~|~\mathsf{false} \\\\
\mathit{int} &::=&
/\texttt{[0-9]+}/ \\\\
\mathit{float} &::=&
/\texttt{0[xX]\([0-9a-fA-F]\*\\.[0-9a-fA-F]+\)|\([0-9a-fA-F]+\\.?\)\([pP][+-]?[0-9]+\)?}/ \\\\
\mathit{str} &::=&
/\texttt{"[\^\"]\*"}/ \\\\
\mathit{ident} &::=&
/\texttt{[A-Za-z]A-Za-z0-9*]\*}/ \\\\
\end{array}
\\]
