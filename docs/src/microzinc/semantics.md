# The Operational Semantics of MicroZinc

Note that only function calls and the items in let expressions change the environment.

\\(
\def\tuple#1{\left\langle #1 \right\rangle}
\def\Sem#1#2{[\\![#1]\\!]\tuple{#2}}
\def\SemLet#1#2{[\\![#1]\\!]\_L\tuple{#2}}
\def\Prog\ensuremath{\mathcal{P}}
\def\Env\ensuremath{\sigma}
\\)

### Function calls

\\[
\begin{prooftree}
\AxiomC{$ \mathsf{function}~T\mathsf{:}~F\mathsf{(} p*1, \dots, p_k \mathsf{)} = E; \in{} \Prog{},~\text{where the}~p_i~\text{are fresh} $}
	\AxiomC{$ \Sem{E*{[p_i \mapsto a_i, \forall{} 1 \leq{} i \leq{} k]}}{\Prog, \Env} \Rightarrow{} \tuple{v, \Env'} $}
	\RightLabel{(E-Call)}
	\BinaryInfC{$ \Sem{F\mathsf{(}a_1, \ldots, a_k\mathsf{)}}{\Prog, \Env, C} \Rightarrow{} \tuple{v, \Env'} $}
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
	\AxiomC{$ \SemLet{\mathsf{let}~\mathsf{\\\{}~items*{[x \mapsto v]}~\mathsf{\\\}}~\mathsf{in}~y*{[x \mapsto v]}}{\Prog, \Env', C} \Rightarrow{} \tuple{x, \Env''\} $}
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
