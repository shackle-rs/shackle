//! Candidate arguments of a group

use itertools::Itertools;
use shackle_hir::constants::IdentifierRegistry;
use shackle_thir::{
	Call, Callable, DeclarationId, ExpressionData, Model, ResolvedIdentifier,
	traverse::{Visitor, visit_call, visit_expression},
};
use shackle_ty::{OptType, Ty, VarType};

use super::{
	Globalizer,
	candidates::{Arg, Elem},
	groups::Unrolled,
	parts::is_par_int,
};

/// Finds array accesses `x[i, j]` whose indices are leaders or par integers
struct AccessCollector<'u, 'g, 'a, 'db> {
	globalizer: &'u Globalizer<'a, 'db>,
	unrolled: &'u Unrolled<'g, 'a, 'db>,
	array_access: shackle_hir::Identifier<'db>,
	/// For each array: the allowed indices (declaration, leader) per position
	accesses: Vec<(DeclarationId<'db>, Vec<Vec<IndexChoice<'db>>>)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct IndexChoice<'db> {
	decl: DeclarationId<'db>,
	leader: Option<usize>,
}

impl<'a, 'db> Visitor<'a, 'db> for AccessCollector<'_, '_, '_, 'db> {
	fn visit_call(&mut self, model: &'a Model<'db>, call: &'a Call<'db>) {
		if let Callable::Function(f) = &call.function
			&& model[*f].name() == self.array_access
			&& call.arguments.len() == 2
			&& let ExpressionData::Identifier(ResolvedIdentifier::Declaration(x)) =
				&*call.arguments[0]
			&& model[*x].top_level()
		{
			let indices = match &*call.arguments[1] {
				ExpressionData::TupleLiteral(t) => t.iter().collect::<Vec<_>>(),
				_ => vec![&call.arguments[1]],
			};
			let pos = match self.accesses.iter().position(|(d, _)| d == x) {
				Some(pos) => pos,
				None => {
					self.accesses.push((*x, vec![Vec::new(); indices.len()]));
					self.accesses.len() - 1
				}
			};
			let choices = &mut self.accesses[pos].1;
			for (i, idx) in indices.iter().enumerate() {
				if let ExpressionData::Identifier(ResolvedIdentifier::Declaration(d)) = &***idx {
					let leader = self.unrolled.leader_of(*d);
					let choice = IndexChoice { decl: *d, leader };
					if (leader.is_some() || is_par_int(self.globalizer.db, &model[*d]))
						&& i < choices.len()
						&& !choices[i].contains(&choice)
					{
						choices[i].push(choice);
					}
				}
			}
		}
		visit_call(self, model, call);
	}
}

impl<'a, 'db> Globalizer<'a, 'db> {
	/// For a declaration that can be an argument (an integer, enum, opt
	/// integer, Boolean or integer set, or an array of them): whether it is a
	/// variable, its number of dimensions and its element type
	pub(super) fn arg_decl(&self, d: DeclarationId<'db>) -> Option<(bool, usize, Elem)> {
		let db = self.db;
		let ty = self.model[d].ty();
		let elem = ty.elem_ty(db).filter(|_| ty.is_array(db)).unwrap_or(ty);
		// TODO: candidates that mix enums and integers rely on MiniZinc's
		// implicit enum2int coercion, which is deprecated. Track the enum of
		// each argument and add `enum2int` before that coercion is removed.
		let int = |t: Ty<'db>| t.is_int(db) || t.is_enum(db);
		let opt = elem.opt(db) == Some(OptType::Opt);
		let kind = if int(elem) {
			if opt { Elem::OptInt } else { Elem::Int }
		} else if elem.is_bool(db) && !opt {
			Elem::Bool
		} else if elem.is_set(db)
			&& elem
				.elem_ty(db)
				.is_some_and(|e| int(e) && e.opt(db) == Some(OptType::NonOpt))
		{
			Elem::Set
		} else {
			return None;
		};
		Some((
			elem.inst(db) == Some(VarType::Var),
			ty.dims(db).unwrap_or(0),
			kind,
		))
	}

	/// Candidate arguments: the group's int declarations, their slices and
	/// accesses with leader or par indices, the viewpoints of its 0/1 arrays,
	/// and its scalar variables as one array
	pub(super) fn arguments(&self, u: &Unrolled<'_, 'a, 'db>) -> Vec<Arg> {
		let arg = |text: String, var, dims, elem, leaders| Arg {
			text,
			var,
			dims,
			elem,
			leaders,
		};
		// The literal 0 (e.g. a lower bound)
		let mut args = vec![arg("0".to_owned(), false, 0, Elem::Int, Vec::new())];
		let mut scalars = Vec::<(Elem, String)>::new();
		for &d in u.decls {
			let Some((var, dims, elem)) = self.arg_decl(d) else {
				continue;
			};
			if dims == 0 && var {
				scalars.push((elem, self.name(d)));
			}
			args.push(arg(self.name(d), var, dims, elem, Vec::new()));
		}
		for vp in self
			.viewpoints
			.iter()
			.filter(|vp| u.decls.contains(&vp.array))
		{
			args.push(arg(vp.name.clone(), true, 2, Elem::Int, Vec::new()));
		}
		// Scalar variables of the group of one type as one array (e.g.
		// `[a, b, c]`). Two variables are left out: such a global is just a
		// binary constraint.
		scalars.sort();
		for (elem, names) in &scalars.into_iter().chunk_by(|(e, _)| *e) {
			let names = names.map(|(_, n)| n).collect::<Vec<_>>();
			if names.len() > 2 {
				args.push(arg(
					format!("[{}]", names.join(", ")),
					true,
					1,
					elem,
					Vec::new(),
				));
			}
		}

		let mut collector = AccessCollector {
			globalizer: self,
			unrolled: u,
			array_access: IdentifierRegistry::lookup(self.db).functions.array_access,
			accesses: Vec::new(),
		};
		for part in &u.parts {
			for e in part.loops.iter().flat_map(|l| l.exprs()).chain([part.body]) {
				visit_expression(&mut collector, self.model, e);
			}
		}
		// Accesses of `array` with one index (or a slice `..`, as `None`) per
		// position
		let mut slices =
			|array: String, var: bool, elem: Elem, positions: Vec<&Vec<IndexChoice<'db>>>| {
				let choices = positions.iter().map(|cs| {
					std::iter::once(None)
						.chain(cs.iter().map(Some))
						.collect::<Vec<_>>()
				});
				for chosen in choices.multi_cartesian_product() {
					if chosen.iter().all(Option::is_none) {
						continue;
					}
					let indices = chosen
						.iter()
						.map(|c| match c {
							None => "..".to_owned(),
							Some(c) => match c.leader {
								Some(m) => self.name(u.leader_loop(m).decl),
								None => self.name(c.decl),
							},
						})
						.join(", ");
					let text = format!("{array}[{indices}]");
					if args.iter().any(|a| a.text == text) {
						continue;
					}
					let dims = chosen.iter().filter(|c| c.is_none()).count();
					let leaders = chosen.iter().flatten().filter_map(|c| c.leader).collect();
					args.push(arg(text, var, dims, elem, leaders));
				}
			};
		for (x, positions) in &collector.accesses {
			let Some((var, dims, elem)) = self.arg_decl(*x) else {
				continue;
			};
			if dims != positions.len() {
				continue;
			}
			slices(self.name(*x), var, elem, positions.iter().collect());
			// The viewpoint has the indices of the other two axes
			for vp in self.viewpoints.iter().filter(|vp| vp.array == *x) {
				let other = (0..3)
					.filter(|k| *k != vp.axis)
					.map(|k| &positions[k])
					.collect();
				slices(vp.name.clone(), true, Elem::Int, other);
			}
		}
		args
	}
}
