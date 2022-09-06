# MicroZinc specification

MicroZinc is a simple language used to define transformations to be performed
by the interpreter. It is a simplified subset of MiniZinc. The transformation
are represented in the language through the use of function definitions. A
function of type `var bool`, describing a relation, can be defined as a
native constraint. Otherwise, a function body must be provided for
rewriting. The function body can use a restriction of the MiniZinc expression
language. An important difference between MiniZinc and MicroZinc is that a
well-formed *MicroZinc does not contain partial expressions*. Any partiality in
the MiniZinc model must be explicitly expressed using total functions in
MicroZinc. As such, constraints introduced in MicroZinc let expressions can
be globally enforced. They are guaranteed to only constrain the decision
variables introduced in the same let expression.

### TODO: Remaining questions

- How many bits will we need to steal for efficient value storage?).
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
		\mathsf{predicate}~\mathit{ident}~\mathsf{(}\mathit{typing} [\mathsf{,}~ \mathit{typing}]\*\mathsf{)}~[\mathsf{=}~\mathsf{let}~\mathsf{\\\{}\mathit{item}\*\mathsf{\\\}}~\mathsf{in}~\mathsf{true}]\mathsf{;} \\\\
	\mathit{typing} &::=&
		\mathit{typeinst}~\mathsf{:}~\mathit{ident} \\\\
	\\\\
	\mathit{letExpr} &::=&
		\mathit{val} \\\\&|&
		\mathsf{let}~\mathsf{\\\{}\mathit{item}\*\mathsf{\\\}}~\mathsf{in}~\mathit{val} \\\\
	\mathit{item} &::=&
		\mathit{typing}~\mathsf{;} \\\\&|&
		\mathit{typing}~\mathsf{=}~\mathit{expr}~\mathsf{;} \\\\&|&
		\mathsf{constraint}~\mathit{ident}~\mathsf{(}~\mathit{val} [ \mathsf{,}~ \mathit{val}]\*~\mathsf{)}~\mathsf{;} \\\\
	\mathit{expr} &::=&
		\mathit{letExpr} \\\\&|&
		\mathit{ident}~\mathsf{(}~\mathit{val} [ \mathsf{,}~ \mathit{val}]\*~\mathsf{)} \\\\&|&
		\mathsf{if}~\mathit{val}~\mathsf{then}~\mathit{letExpr}~[\mathsf{elseif}~\mathit{val}~\mathsf{then}~\mathit{letExpr}]\*~\mathsf{else}~\mathit{letExpr}~\mathsf{endif} \\\\&|&
		\mathsf{[}[\mathit{tuple}\mathsf{:}]~\mathit{letExpr}~\mathsf{|}~\mathit{genExpr} [ \mathsf{,}~ \mathit{genExpr}]\*\mathsf{]} \\\\
	\mathit{genExpr} &::=&
		\mathit{ident}~\mathsf{in}~\mathit{letExpr}~[\mathsf{where}~\mathit{letExpr}] \\\\&|&
		\mathit{ident}~\mathsf{=}~\mathit{letExpr} \\\\
	\mathit{val} &::=&
		\mathit{lit}~|~\mathit{range}~|~\mathit{tuple} \\\\&|&
		\mathsf{\\{}\mathit{lit}~\mathsf{,}~ [\mathit{lit}~\mathsf{,}]\*\mathsf{\\}} \\\\&|&
		\mathsf{array}\mathit{i62}\mathsf{d(}\mathit{range}~\mathsf{,}~[\mathit{range}~\mathsf{,}]\*~\mathsf{[}\mathit{lit}~\mathsf{,}~ [\mathit{lit}~\mathsf{,}]\*\mathsf{])} \\\\&|&
		\mathit{ident}~\mathsf{.}~\mathit{i62} \\\\&|&
		\mathit{ident}~\mathsf{[}\mathit{lit}\mathsf{]} \\\\
	\mathit{tuple} &::=& 
		\mathsf{(}\mathit{lit}~\mathsf{,}~ [\mathit{lit}~\mathsf{,}]\*\mathsf{)} \\\\
	\mathit{range} &::=&
		\mathit{lit}\mathsf{..}\mathit{lit} \\\\&|&
		\mathit{ident} \\\\
	\mathit{lit} &::=&
		\mathit{bool} \\\\&|&
		\mathit{i62} \\\\&|&
		\mathit{f62} \\\\&|&
		\mathit{str} \\\\&|&
		\mathit{ident} \\\\
	\\\\
	\mathit{bool} &::=&
		\mathsf{true}~|~\mathsf{false} \\\\
	\mathit{i62} &::=&
		/\texttt{[0-9]+}/ \\\\
	\mathit{f62} &::=&
		/\texttt{[0-9]+.[0-9]+}/ \\\\&|&
		/\texttt{[0-9]+.[0-9]+e[-+]?[0-9]+}/ \\\\
	\mathit{str} &::=&
		/\texttt{"[\^\"]\*"}/ \\\\
	\mathit{ident} &::=&
		/\texttt{[A-Za-z][A-Za-z0-9_]*}/ \\\\
\end{array}
\\]

## Typing

The following syntax describes the types available in MicroZinc. The type syntax are used directly in let-expressions and parameter declarations.

\\[
\begin{array}{lcl}
	\mathit{typeinst} &::=&
		\mathsf{array}~\mathsf{[}\mathit{i62}\mathsf{..}\mathit{i62} [ \mathsf{,}~ \mathit{i62}\mathsf{..}\mathit{i62}]\*\mathsf{]}~\mathsf{of}~\mathit{baseType} \\\\&|&
		\mathit{baseType} \\\\
	\mathit{baseType} &::=&
		\mathsf{tuple}~\mathsf{(}\mathit{typeinst} [ \mathsf{,}~ \mathit{typeinst}]\*\mathsf{)} \\\\&|&
		\mathit{domType} \\\\&|&
		\mathit{primType} \\\\
	\mathit{domType} &::=&
		\mathsf{var}~\mathit{val}\\\\&|&
		\mathsf{var}~\mathsf{set}~\mathsf{of}~\mathit{val}\\\\
	\mathit{primType} &::=&
		\mathsf{par}~\mathsf{bool}~|~\mathsf{var}~\mathsf{bool}\\\\&|&
		\mathsf{par}~\mathsf{int}~|~\mathsf{var}~\mathsf{int}\\\\&|&
		\mathsf{par}~\mathsf{float}~|~\mathsf{var}~\mathsf{float}\\\\&|&
		\mathsf{par}~\mathsf{set}~\mathsf{of}~\mathsf{int}~|~\mathsf{var}~\mathsf{set}~\mathsf{of}~\mathsf{int}\\\\&|&
		\mathsf{par}~\mathsf{set}~\mathsf{of}~\mathsf{float}~|~\mathsf{string}
\end{array}
\\]

Importantly, MicroZinc includes two types of sub-typing. When type \\( T_1 \\) is a sub-type of type \\( T_2 \\) then \\( T_1 \\) can be used anywhere where the type \\( T_2 \\) is required.
 - In MicroZinc, \\( \mathsf{par}~T \\) is a sub-type of \\( \mathsf{var}~T \\).
 - MicroZinc also has numeric subtyping (i.e., \\( \mathsf{par~bool} \\) is a subtype of \\( \mathsf{par~int} \\), which is a sub-type of \\( \mathsf{par~float} \\), and similarly \\( \mathsf{var~bool} \\) is a sub-type of \\( \mathsf{var~int} \\), which is a sub-type of \\( \mathsf{var~float} \\))

The following rules describe the conditions under which a MicroZinc program is correctly typed. In these rules the variable \\( \Gamma \\) will denote the typing context. This context contains known types for identifiers.

### Functions and calls

At the top level of the MicroZinc program we find different functions. The program is well-typed if the type of the body of each function matches the declared type of the function given the declared types for the arguments. The types of all functions are available when typing the body expression of a function, allowing for (mutual) recursive functions.

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} T : T^r$}
	\AxiomC{$ \Gamma, f^{id} : (T'_1 \equiv )\langle T^p_1, \dots, T^p_n \rangle \rightarrow T^{r}\vdash{} funcs : \langle T'_2, \dots, T'_m \rangle $}
	\RightLabel{(T-Builtin)}
	\BinaryInfC{$ \vdash{} \mathsf{function}~T~\mathsf{:}~f^{id}~\mathsf{(} T^p_1 : x_1\mathsf{,}\dots\mathsf{,}T^p_n : x_n\mathsf{)}~\mathsf{;} funcs : \langle T'_1, \dots, T'_m \rangle $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} T : T^r$}
	\AxiomC{$ \Gamma, f^{id}_1 : (T'_1 \equiv) \langle T^p_1, \dots, T^p_n \rangle \rightarrow T^r \vdash{} f_2 \mathsf{;} \dots \mathsf{;} f_m : \langle T'_2, \dots, T'_m \rangle $}
	\noLine{}
	\BinaryInfC{$\Gamma, f^{id}_1 : T'_1, \dots, f^{id}_m : T'_m, x_1 : T^p_1, \dots, x_n : T^p_n \vdash{} E : T^r$}
	\RightLabel{(T-Func)}
	\UnaryInfC{$ \Gamma \vdash{} \mathsf{function}~T~\mathsf{:}~f^{id}_1~\mathsf{(}~x_1: T^p_1, \dots, x_n: T^p_n \mathsf{)}~\mathsf{=}~E~\mathsf{;} f_2 \mathsf{;} \dots \mathsf{;} f_m : \langle T'_1, \dots, T'_m \rangle $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma, f^{id} : (T'_1 \equiv )\langle T^p_1, \dots, T^p_n \rangle \rightarrow \mathsf{pred}\vdash{} funcs : \langle T'_2, \dots, T'_m \rangle $}
	\RightLabel{(T-Slv-Native)}
	\UnaryInfC{$ \vdash{} \mathsf{predicate}~f^{id}~\mathsf{(} T^p_1 : x_1\mathsf{,}\dots\mathsf{,}T^p_n : x_n\mathsf{)}~\mathsf{;} funcs : \langle T'_1, \dots, T'_m \rangle $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma, f^{id}_1 : (T'_1 \equiv) \langle T^p_1, \dots, T^p_n \rangle \rightarrow \mathsf{pred} \vdash{} f_2 \mathsf{;} \dots \mathsf{;} f_m : \langle T'_2, \dots, T'_m \rangle $}
	\noLine{}
	\UnaryInfC{$\Gamma, f^{id}_1 : T'_1, \dots, f^{id}_m : T'_m, x_1 : T^p_1, \dots, x_n : T^p_n \vdash{} E : \mathsf{par~bool}$}
	\RightLabel{(T-Pred)}
	\UnaryInfC{$ \Gamma \vdash{} \mathsf{function}~T~\mathsf{:}~f^{id}_1~\mathsf{(}~x_1: T^p_1, \dots, x_n: T^p_n \mathsf{)}~\mathsf{=}~E~\mathsf{;} f_2 \mathsf{;} \dots \mathsf{;} f_m : \langle T'_1, \dots, T'_m \rangle $}
\end{prooftree}
\\]

Calls are defined in both the context of a constraint and on the right hand side of an assignment of a let-expression. In both cases the typing of call is described by the following rule.

\\[
\begin{prooftree}
	\AxiomC{$ f^{id} : \langle T^p_1, \dots, T^p_n \rangle \rightarrow T^r \in \Gamma$}
	\AxiomC{$ \Gamma \vdash{} x_1 : T^p_1 ~~ \dots ~~ \Gamma \vdash{} x_n : T^p_n$}
	\RightLabel{(T-Call)}
	\BinaryInfC{$ \Gamma \vdash{} f^{id} ~\mathsf{(}~x_1 \mathsf{,} \dots  \mathsf{,} x_n~\mathsf{)} : T^r $}
\end{prooftree}
\\]

### Let expressions and identifiers

Identifiers are typed simply using a lookup in the typing context. The typing of the let expression iteratively adds the types of each declaration item to the typing context. The program is well-typed when all expressions on the right-hand side of a declaration match their declared types, and any constraint items are of \\( \mathsf{var~bool} \\) type.

\\[
\begin{prooftree}
	\AxiomC{$ x : T \in \Gamma $}
	\RightLabel{(T-Ident)}
	\UnaryInfC{$ \Gamma \vdash{} x : T$}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} x : T$}
	\RightLabel{(T-Let-Base)}
	\UnaryInfC{$ \Gamma \vdash{} \mathsf{let}~\mathsf{\\\{}~\mathsf{\\\}}~\mathsf{in}~\mathit{x} : T$}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} T : T'$}
	\AxiomC{$ \Gamma, x : T' \vdash{} \mathsf{let}~\mathsf{\\\{}~items~\mathsf{\\\}}~\mathsf{in}~y : T''$}
	\RightLabel{(T-Let-Decl1)}
	\BinaryInfC{$ \Gamma \vdash{} \mathsf{let}~\mathsf{\\\{}~T~\mathsf{:}~x~\mathsf{;}~items~\mathsf{\\\}}~\mathsf{in}~y : T''$}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} T : T'$}
	\AxiomC{$ \Gamma \vdash{} E : T'$}
	\AxiomC{$ \Gamma, x : T' \vdash{} \mathsf{let}~\mathsf{\\\{}~items~\mathsf{\\\}}~\mathsf{in}~y : T''$}
	\RightLabel{(T-Let-Decl2)}
	\TrinaryInfC{$ \Gamma \vdash{} \mathsf{let}~\mathsf{\\\{}~T~\mathsf{:}~x = E~\mathsf{;}~items~\mathsf{\\\}}~\mathsf{in}~y : T''$}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} E : \mathsf{pred}$}
	\AxiomC{$ \Gamma, x : T \vdash{} \mathsf{let}~\mathsf{\\\{}~items~\mathsf{\\\}}~\mathsf{in}~y : T'$}
	\RightLabel{(T-Let-Con)}
	\BinaryInfC{$ \Gamma \vdash{} \mathsf{let}~\mathsf{\\\{}~\mathsf{constraint}~E~\mathsf{;}~items~\mathsf{\\\}}~\mathsf{in}~y : T'$}
\end{prooftree}
\\]

### Comprehensions and Generators

As shown in the following rules, the \\( \mathit{genExpr} \\) rules will must have either type \\( \mathsf{set~of~int} \\) or \\( \mathsf{array1d~of}~T \\). The \\( \text{T_Comp} \\) rule, to type array comprehensions, will use the \\( elem \\) function which maps the former type to \\( \mathsf{int} \\) and the latter to \\( T \\).

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma, \vdash{} E : T$}
	\RightLabel{(T-Comp-Expr)}
	\UnaryInfC{$ \Gamma \vdash{} \mathsf{[} E~\mathsf{|}~\mathsf{]} : \mathsf{array1d~of~} T $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} I : T$}
	\AxiomC{$ T \in \\{ \mathsf{set~of~int}, \mathsf{array1d~of}~V \\}$}
	\AxiomC{$ \Gamma, x : T \vdash{} \mathsf{[} E~\mathsf{|}~gens~\mathsf{]} : T' $}
	\RightLabel{(T-Comp-In)}
	\TrinaryInfC{$ \Gamma \vdash{} \mathsf{[} E~\mathsf{|}~x~\mathsf{in}~I, gens~\mathsf{]} : T' $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} B : \mathsf{par~bool}$}
	\AxiomC{$ \Gamma \vdash{} \mathsf{[} E~\mathsf{|}~x~\mathsf{in}~I, gens~\mathsf{]} : T $}
	\RightLabel{(T-Comp-Where)}
	\BinaryInfC{$ \Gamma \vdash{} \mathsf{[} E~\mathsf{|}~x~\mathsf{in}~I~\mathsf{where}~B, gens~\mathsf{]} : T $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} E_2 : T$}
	\AxiomC{$ \Gamma, x : T \vdash{} \mathsf{[} E_1~\mathsf{|}~gens~\mathsf{]} : T' $}
	\RightLabel{(T-Comp-Asg)}
	\BinaryInfC{$ \Gamma \vdash{} \mathsf{[} E_1~\mathsf{|}~x~\mathsf{=}~E_2, gens~\mathsf{]} : T' $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma\vdash{} \mathsf{[} I_i~\mathsf{|}~gens~\mathsf{]} : \mathsf{array1d~of~int}, \forall{} 1 \leq{} i \leq{} X$}
	\AxiomC{$ \Gamma\vdash{} \mathsf{[} E~\mathsf{|}~gens~\mathsf{]} : \mathsf{array1d~of}~T $}
	\RightLabel{(T-Comp-Ind)}
	\BinaryInfC{$ \Gamma \vdash{} \mathsf{[(}I_1, \dots, I_X \mathsf{):} E~\mathsf{|}gens~\mathsf{]} : \mathsf{array}X\mathsf{d~of}~T $}
\end{prooftree}
\\]

### Arrays, Sets, and Tuples

MicroZinc has three different container types. Arrays can contain multiple, possibly duplicate, elements of the same type, each associated with a unique index with which the element can be retrieved.

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} r_i : \mathsf{par~set~of~int}, \forall{} 1 \leq{} i \leq{} X$}
	\AxiomC{$ \Gamma \vdash{} x_i : T, \forall{} 1 \leq{} i \leq{} n$}
	\RightLabel{(T-Arr)}
	\BinaryInfC{$ \Gamma \vdash{} \mathsf{array}X\mathsf{d(}~r_1 \mathsf{,} \dots \mathsf{,} r_X \mathsf{,} \mathsf{[} x_1 \mathsf{,} \dots \mathsf{,} x_n \mathsf{])} : \mathsf{array}X\mathsf{d~of}~T $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} v_i : \mathsf{par}~\mathsf{int}, \forall{} 1 \leq{} i \leq{} X$}
	\AxiomC{$ \Gamma \vdash{} x : \mathsf{array}X\mathsf{d~of}~T$}
	\RightLabel{(T-Ind)}
	\BinaryInfC{$ \Gamma \vdash{} x \mathsf{[} v_1 \mathsf{,} \dots \mathsf{,} v_X \mathsf{]} : T $}
\end{prooftree}
\\]

Sets contain a certain number of unique elements of the same type. Ranges of elements are also typed as sets.

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} x_1 : T, \forall{} 1 \leq{} i \leq{} n$}
	\AxiomC{$ T \in {\mathsf{par~int}, \mathsf{par~float}}$}
	\RightLabel{(T-Set)}
	\BinaryInfC{$ \Gamma \vdash{} \mathsf{\\{} x_1 \mathsf{,} \dots \mathsf{,} x_n \mathsf{\\}} : \mathsf{par~set~of}~T $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} x_1 : T $}
	\AxiomC{$ \Gamma \vdash{} x_2 : T $}
	\AxiomC{$ T \in {\mathsf{par~int}, \mathsf{par~float}}$}
	\RightLabel{(T-Range)}
	\TrinaryInfC{$ \Gamma \vdash{} x_1 \mathsf{..} x_2 : \mathsf{par~set~of}~T $}
\end{prooftree}
\\]

Finally, tuples are collections of elements with possibly different types. The number of elements in a tuple is known during type checking

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} x_i : T_i, \forall{} 1 \leq{} i \leq{} n$}
	\RightLabel{(T-Tup)}
	\UnaryInfC{$ \Gamma \vdash{} \mathsf{(} x_1\mathsf{,} \dots\mathsf{,} x_n \mathsf{)} : \mathsf{tuple(} T_1 \mathsf{,} \dots \mathsf{,} T_n \mathsf{)}$}
\end{prooftree}
\\]
\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} x : \mathsf{tuple(} T_1 \mathsf{,} \dots \mathsf{,} T_i \mathsf{,} \dots \mathsf{,} T_n \mathsf{)}$}
	\AxiomC{$ i \in 1 \mathsf{..} n $}
	\RightLabel{(T-Acc)}
	\BinaryInfC{$ \Gamma \vdash{} x \mathsf{.} i : T_i $}
\end{prooftree}
\\]


### If-then-else Expressions

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} x : \mathsf{par}~\mathsf{bool} $}
	\AxiomC{$ \Gamma \vdash{} E_1 : T$}
	\AxiomC{$ \Gamma \vdash{} E_2 : T$}
	\RightLabel{(T-If)}
	\TrinaryInfC{$ \Gamma \vdash{} \mathsf{if}~x~\mathsf{then}~E_1~\mathsf{else}~E_2~\mathsf{endif} : T $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} x_1 : \mathsf{par}~\mathsf{bool} $}
	\AxiomC{$ \Gamma \vdash{} E_1 : T$}
	\AxiomC{$ \Gamma \vdash{} \mathsf{if}~x_2~\mathsf{then}~E_2 \dots \mathsf{else}~E_n~\mathsf{endif} : T$}
	\RightLabel{(T-ElseIf)}
	\TrinaryInfC{$ \Gamma \vdash{} \mathsf{if}~x_1~\mathsf{then}~E_1~\mathsf{elseif}~x_2~\mathsf{then}~E_2 \dots \mathsf{else}~E_n~\mathsf{endif} : T $}
\end{prooftree}
\\]

### Literals

The remaining parts of the MicroZinc laguage are simple literals that have an intrinsic type.

\\[
\begin{prooftree}
	\AxiomC{}
	\RightLabel{(T-True)}
	\UnaryInfC{$\vdash{} \mathsf{true} : \mathsf{par}~\mathsf{bool}$}
\end{prooftree}
\begin{prooftree}
	\AxiomC{}
	\RightLabel{(T-False)}
	\UnaryInfC{$\vdash{} \mathsf{false} : \mathsf{par}~\mathsf{bool}$}
\end{prooftree}
\\]
\\[
\begin{prooftree}
	\AxiomC{}
	\RightLabel{(T-i62)}
	\UnaryInfC{$\vdash{} /\texttt{[0-9]+}/ : \mathsf{par}~\mathsf{int}$}
\end{prooftree}
\begin{prooftree}
	\AxiomC{}
	\RightLabel{(T-Str)}
	\UnaryInfC{$\vdash{} /\texttt{"[\^\"]\*"}/ : \mathsf{string}$}
\end{prooftree}
\\]
\\[
\begin{prooftree}
	\AxiomC{}
	\RightLabel{(T-f62)}
	\UnaryInfC{$\vdash{} /\texttt{[0-9]+.[0-9]+}/ : \mathsf{par}~\mathsf{float}$}
\end{prooftree}
\begin{prooftree}
	\AxiomC{}
	\RightLabel{(T-f62-exp)}
	\UnaryInfC{$\vdash{} /\texttt{[0-9]+.[0-9]+e[-+]?[0-9]+}/: \mathsf{par}~\mathsf{float}$}
\end{prooftree}
\\]

### Type Instances

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} I_i : \mathsf{par}~\mathsf{set}~\mathsf{of}~\mathsf{int}, \forall{} 1 \leq{} i \leq{} X$}
	\AxiomC{$ \Gamma \vdash{} T : T'$}
	\RightLabel{(T-Type-Arr)}
	\BinaryInfC{$ \Gamma \vdash{} \mathsf{array}~\mathsf{[}I_1 \mathsf{,} \dots, I_X\mathsf{]}~\mathsf{of}~\mathit{T} : \mathsf{array}X\mathsf{d~of}~T' $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} T_1 : T'_1 ~~ \dots ~~ \Gamma \vdash{} T_n : T'_n$}
	\RightLabel{(T-Type-Tup)}
	\UnaryInfC{$ \Gamma \vdash{} \mathsf{tuple}\mathsf{(}\mathit{T_1}\mathsf{,} \dots, T_n\mathsf{)} : \mathsf{tuple(}T'_1, \dots, T'_n\mathsf{)} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} S : \mathsf{par}~\mathsf{set}~\mathsf{of}~\mathsf{int}$}
	\RightLabel{(T-Int-Dom)}
	\UnaryInfC{$ \Gamma \vdash{} \mathsf{var}~S : \mathsf{var}~\mathsf{int} $}
\end{prooftree}
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} S : \mathsf{par}~\mathsf{set}~\mathsf{of}~\mathsf{float}$}
	\RightLabel{(T-Flt-Dom)}
	\UnaryInfC{$ \Gamma \vdash{} \mathsf{var}~S : \mathsf{var}~\mathsf{float} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Gamma \vdash{} S : \mathsf{par}~\mathsf{set}~\mathsf{of}~\mathsf{int}$}
	\RightLabel{(T-Set-Dom)}
	\UnaryInfC{$ \Gamma \vdash{} \mathsf{var}~\mathsf{set}~\mathsf{of}~S : \mathsf{var}~\mathsf{set}~\mathsf{of}~\mathsf{int} $}
\end{prooftree}
\\]

The remaining variants (\\( \mathit{primType} \\)) are trivial, and their expression directly describes their type.


## Operational Semantics

Note that only function calls and the items in let expressions change the environment.

\\(
\def\tuple#1{\left\langle #1 \right\rangle}
\def\Sem#1#2{[\\![#1]\\!]\tuple{#2}}
\def\SemLet#1#2{[\\![#1]\\!]_L\tuple{#2}}
\def\Prog\ensuremath{\mathcal{P}}
\def\Env\ensuremath{\sigma}
\\)

### Function calls

\\[
\begin{prooftree}
	\AxiomC{$ \mathsf{function}~T\mathsf{:}~F\mathsf{(} p_1, \dots, p_k \mathsf{)} = E; \in{} \Prog{},~\text{where the}~p_i~\text{are fresh} $}
	\AxiomC{$ \Sem{E_{[p_i \mapsto a_i, \forall{} 1 \leq{} i \leq{} k]}}{\Prog, \Env} \Rightarrow{} \tuple{v, \Env'} $}
	\RightLabel{(E-Call)}
	\BinaryInfC{$ \Sem{F\mathsf{(}a_1, \ldots, a_k\mathsf{)}}{\Prog, \Env} \Rightarrow{} \tuple{v, \Env'} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ F \in \text{Builtins} $}
	\RightLabel{(E-Call-Builtin)}
	\UnaryInfC{$ \Sem{F\mathsf{(}a_1, \ldots, a_k\mathsf{)}}{\Prog, \Env} \Rightarrow{} \tuple{\mathit{eval}(F, \langle a_1, \dots, a_k \rangle), \Env'} $}
\end{prooftree}
\\]

TODO: The following rule comes from my thesis to call a native constraint, but I'm not sure if it works correctly when you have a functionally defined Boolean variable (call on RHS).

\\[
\begin{prooftree}
	\AxiomC{$ \mathsf{function~var~bool:}~F\mathsf{(}p_1, \ldots, p_k\mathsf{)}; \in \Prog $}
	\RightLabel{(E-Call-Native)}
	\UnaryInfC{$ \Sem{F\mathsf{(}a_1, \ldots, a_k\mathsf{)}}{\Prog, \Env} \Rightarrow{} \tuple{\mathsf{constraint}~ F\mathsf{(}a_1, \ldots, a_k\mathsf{)}, \Env} $}
\end{prooftree}
\\]

### Let expressions

Constraint in let-expression are aggregated and bound to the returned variable. As such, let expressions are evaluated with an addition context collection \\( C \\), which contains the constraints enforced in the let-expression. Evaluation rules that use this special context are indicated using \\( L \\).

\\[
\begin{prooftree}
	\AxiomC{$ \SemLet{\mathsf{let}~\mathsf{\\\{}~items~\mathsf{\\\}}~\mathsf{in}~y}{\Prog, \Env, \emptyset} \Rightarrow \tuple{x, \Env'} $}
	\RightLabel{(E-Let)}
	\UnaryInfC{$ \Sem{\mathsf{let}~\mathsf{\\\{}~items~\mathsf{\\\}}~\mathsf{in}~y}{\Prog, \Env} \Rightarrow{} \tuple{x, \Env'} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{y}{\Prog, \Env} \Rightarrow \tuple{x, \Env} $}
	\RightLabel{(E-Let-In)}
	\UnaryInfC{$ \SemLet{\mathsf{let}~\mathsf{\\\{}\mathsf{\\\}}~\mathsf{in}~y}{\Prog, \Env, C} \Rightarrow{} \tuple{x, \Env \cup x~\texttt{↳}~C\} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{E}{\Prog, \Env} \Rightarrow \tuple{c, \Env'} $}
	\AxiomC{$ \SemLet{\mathsf{let}~\mathsf{\\\{}~items~\mathsf{\\\}}~\mathsf{in}~y}{\Prog, \Env', C \cup c} \Rightarrow{} \tuple{x, \Env''\} $}
	\RightLabel{(E-Let-Con)}
	\BinaryInfC{$ \SemLet{\mathsf{let}~\mathsf{\\\{}~\mathsf{constraint}~E~\mathsf{;}~items~\mathsf{\\\}}~\mathsf{in}~y}{\Prog, \Env, C} \Rightarrow{} \tuple{x, \Env''\} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \SemLet{\mathsf{let}~\mathsf{\\\{}~items~\mathsf{\\\}}~\mathsf{in}~y}{\Prog, \Env \cup \{T: x\}, C} \Rightarrow{} \tuple{x, \Env'\} $}
	\RightLabel{(E-Let-Decl1)}
	\UnaryInfC{$ \SemLet{\mathsf{let}~\mathsf{\\\{}~T~\mathsf{:}~x~\mathsf{;}~items~\mathsf{\\\}}~\mathsf{in}~y}{\Prog, \Env, C} \Rightarrow{} \tuple{x, \Env'\} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{E}{\Prog, \Env} \Rightarrow \tuple{v, \Env'} $}
	\AxiomC{$ \SemLet{\mathsf{let}~\mathsf{\\\{}~items_{[x \mapsto v]}~\mathsf{\\\}}~\mathsf{in}~y_{[x \mapsto v]}}{\Prog, \Env', C} \Rightarrow{} \tuple{x, \Env''\} $}
	\RightLabel{(E-Let-Decl2)}
	\BinaryInfC{$ \SemLet{\mathsf{let}~\mathsf{\\\{}~T~\mathsf{:}~x = E~\mathsf{;}~items~\mathsf{\\\}}~\mathsf{in}~y}{\Prog, \Env, C} \Rightarrow{} \tuple{x, \Env''\} $}
\end{prooftree}
\\]

### Comprehensions and Generators

\\[
\begin{prooftree}
	\AxiomC{}
	\RightLabel{(T-Comp-NoGen)}
	\UnaryInfC{$ \Sem{\mathsf{[} E \mathsf{|}~\mathsf{]}}{\Prog,\Env} \Rightarrow{} \tuple{\mathsf{[ ]}, \Env} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{I}{\Prog, \Env} \Rightarrow \tuple{\mathsf{[ ]}, \Env'} $}
	\RightLabel{(T-Comp-Empty)}
	\UnaryInfC{$ \Sem{\mathsf{[} E \mathsf{|}~x~\mathsf{in}~I~\mathsf{where}~B, gens~\mathsf{]}}{\Prog,\Env} \Rightarrow{} \tuple{\mathsf{[ ]}, \Env} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{I}{\Prog, \Env} \Rightarrow \tuple{\mathsf{[}v_1~\mathsf{]}, \Env'} $}
	\RightLabel{(T-Comp-It)}
	\UnaryInfC{$ \Sem{\mathsf{[} E \mathsf{|}~x~\mathsf{in}~I~\mathsf{where}~B, gens~\mathsf{]}}{\Prog,\Env} \Rightarrow{} \tuple{\mathsf{[ ]}, \Env} $}
\end{prooftree}
\\]


### Tuples and Arrays

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{x}{\Prog,\Env} \Rightarrow{} \tuple{\mathsf{(} v_1\mathsf{,} \dots\mathsf{,} v_n \mathsf{)}, \Env} $}
	\AxiomC{$ i \in 1 \mathsf{..} n$}
	\RightLabel{(E-Tup-Acc)}
	\BinaryInfC{$ \Sem{x \mathsf{.} i}{\Prog,\Env} \Rightarrow{} \tuple{v_i, \Env} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{x}{\Prog,\Env} \Rightarrow{} \tuple{\mathsf{[} v_n\mathsf{,} \dots\mathsf{,} v_m \mathsf{]}, \Env} $}
	\AxiomC{$ \Sem{y}{\Prog,\Env} \Rightarrow{} \tuple{i, \Env} $}
	\AxiomC{$ i \in n \mathsf{..} m$}
	\RightLabel{(E-Arr-Ind)}
	\TrinaryInfC{$ \Sem{x \mathsf{[} y \mathsf{]}}{\Prog,\Env} \Rightarrow{} \tuple{v_i, \Env} $}
\end{prooftree}
\\]

### If-then-else expressions

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{x_1}{\Prog,\Env} \Rightarrow{} \tuple{\mathsf{true}, \Env} $}
	\AxiomC{$ \Sem{E_1}{\Prog,\Env} \Rightarrow{} \tuple{v, \Env'} $}
	\RightLabel{(E-If)}
	\BinaryInfC{$ \Sem{\mathsf{if}~x_1~\mathsf{then}~E_1~\mathsf{elseif}~x_2~\mathsf{then}~E_2 \dots \mathsf{else}~E_n~\mathsf{endif}}{\Prog,\Env} \Rightarrow{} \tuple{v, \Env'} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{x_1}{\Prog,\Env} \Rightarrow{} \tuple{\mathsf{false}, \Env} $}
	\AxiomC{$ \Sem{\mathsf{if}~x_2~\mathsf{then}~E_2 \dots \mathsf{else}~E_n~\mathsf{endif}}{\Prog,\Env} \Rightarrow{} \tuple{v, \Env'} $}
	\RightLabel{(E-ElseIf)}
	\BinaryInfC{$ \Sem{\mathsf{if}~x_1~\mathsf{then}~E_1~\mathsf{elseif}~x_2~\mathsf{then}~E_2 \dots \mathsf{else}~E_n~\mathsf{endif}}{\Prog,\Env} \Rightarrow{} \tuple{v, \Env'} $}
\end{prooftree}
\\]

\\[
\begin{prooftree}
	\AxiomC{$ \Sem{x}{\Prog,\Env} \Rightarrow{} \tuple{\mathsf{false}, \Env} $}
	\AxiomC{$ \Sem{E_2}{\Prog,\Env} \Rightarrow{} \tuple{v, \Env'} $}
	\RightLabel{(E-Else)}
	\BinaryInfC{$ \Sem{\mathsf{if}~x~\mathsf{then}~E_1~\mathsf{else}~E_2~\mathsf{endif}}{\Prog,\Env} \Rightarrow{} \tuple{v, \Env'} $}
\end{prooftree}
\\]

### Identifiers and Literals

\\[
	\begin{prooftree}
		\AxiomC{$ x \in \mathit{ident} $}
		\AxiomC{$ \{T: x~\texttt{↳}~C \} \in \Env $}
		\RightLabel{(E-Ident)}
		\BinaryInfC{$ \Sem{x}{\Prog, \Env} \Rightarrow{} \tuple{x, \Env} $}
	\end{prooftree}
	\begin{prooftree}
		\AxiomC{$ v \in \mathit{literal} $}
		\RightLabel{(E-Lit)}
		\UnaryInfC{$ \Sem{v}{\Prog, \Env} \Rightarrow{} \tuple{v, \Env} $}
	\end{prooftree}
\\]