//! Groups of parts, and the ways to unroll their loops (variants)

use itertools::Itertools;
use rustc_hash::FxHashMap;
use shackle_thir::DeclarationId;

use super::{
	Globalizer,
	parts::{Loop, LoopKind, Part},
};

/// Enumerate the groups of up to `max` parts (as sorted indices) whose
/// variables connect them.
pub(super) fn groups<T: PartialEq>(vars: &[Vec<T>], max: usize) -> Vec<Vec<usize>> {
	let shares = |a: usize, b: usize| vars[a].iter().any(|v| vars[b].contains(v));
	let mut result = Vec::new();
	for size in 1..=max.min(vars.len()) {
		for group in (0..vars.len()).combinations(size) {
			if group.iter().any(|i| vars[*i].is_empty()) {
				continue;
			}
			// Flood fill from the first part
			let reached = super::closure([group[0]], |p| {
				group
					.iter()
					.copied()
					.filter(|other| shares(*p, *other))
					.collect::<Vec<_>>()
			});
			if reached.len() == group.len() {
				result.push(group);
			}
		}
	}
	result
}

/// How a group is unrolled. Leader `m` is the loop `fixed[0][m]` of the
/// first part, and `fixed[k][m]` is the matching loop of the `k`-th part.
/// Each leader is set to the first or the last value of its set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Variant {
	pub(super) fixed: Vec<Vec<usize>>,
}

/// One unrolled group: the parts of a group with a variant
pub(super) struct Unrolled<'g, 'a, 'db> {
	pub(super) parts: Vec<&'g Part<'a, 'db>>,
	pub(super) variant: &'g Variant,
	/// Top-level declarations the parts and the context use
	pub(super) decls: &'g [DeclarationId<'db>],
}

impl<'a, 'db> Unrolled<'_, 'a, 'db> {
	pub(super) fn leaders(&self) -> usize {
		self.variant.fixed[0].len()
	}

	/// The leader that a loop declaration of the group is fixed to
	pub(super) fn leader_of(&self, d: DeclarationId<'db>) -> Option<usize> {
		self.parts
			.iter()
			.zip(&self.variant.fixed)
			.find_map(|(p, f)| f.iter().position(|l| p.loops[*l].decl == d))
	}

	pub(super) fn leader_loop(&self, m: usize) -> &Loop<'a, 'db> {
		&self.parts[0].loops[self.variant.fixed[0][m]]
	}

	/// All instantiations of the leaders (`last[m]`: leader `m` takes the last
	/// value of its set instead of the first)
	pub(super) fn instantiations(&self) -> Vec<Vec<bool>> {
		(0..1usize << self.leaders())
			.map(|mask| (0..self.leaders()).map(|m| mask & (1 << m) != 0).collect())
			.collect()
	}
}

impl<'a, 'db> Globalizer<'a, 'db> {
	pub(super) fn loop_set(&self, l: &Loop<'a, 'db>) -> String {
		match l.kind {
			LoopKind::Iter(c) => self.print(c),
			LoopKind::Assign(_) => String::new(),
		}
	}

	/// The ways to unroll a group (always including no unrolling)
	pub(super) fn variants(&self, group: &[usize]) -> Vec<Variant> {
		let first = &self.parts[group[0]];
		let mut result = Vec::new();
		for subset in (0..first.loops.len()).powerset() {
			if !first.can_fix(self.db, self.model, &subset) {
				continue;
			}
			let leaders = subset.iter().map(|l| &first.loops[*l]).collect::<Vec<_>>();
			let names = leaders
				.iter()
				.map(|l| self.name(l.decl))
				.collect::<Vec<_>>();
			// Every other part must unroll loops over the same sets, with the
			// same `where` clauses (up to the names of the loops): the
			// replacement only shows the first part's
			let mut options = vec![vec![subset.clone()]];
			for p in &group[1..] {
				let part = &self.parts[*p];
				let matches = (0..part.loops.len())
					.permutations(subset.len())
					.filter(|ls| {
						let renaming = ls
							.iter()
							.map(|l| self.name(part.loops[*l].decl))
							.zip(names.iter().cloned())
							.collect::<Vec<_>>();
						ls.iter().zip(&leaders).all(|(l, leader)| {
							let l = &part.loops[*l];
							rename(&self.loop_set(l), &renaming) == self.loop_set(leader)
								&& l.where_clause.map(|w| rename(&self.print(w), &renaming))
									== leader.where_clause.map(|w| self.print(w))
						}) && part.can_fix(self.db, self.model, ls)
					})
					.collect::<Vec<_>>();
				options.push(matches);
			}
			for fixed in options.into_iter().multi_cartesian_product() {
				let variant = Variant { fixed };
				if self.consistent_names(group, &variant) {
					result.push(variant);
				}
			}
		}
		result
	}

	/// Whether no loop name is fixed to two different leaders: each fixed loop
	/// is declared at the top level of the submodel
	fn consistent_names(&self, group: &[usize], variant: &Variant) -> bool {
		let mut leader_of = FxHashMap::default();
		group.iter().zip(&variant.fixed).all(|(p, fixed)| {
			fixed.iter().enumerate().all(|(m, l)| {
				let name = self.name(self.parts[*p].loops[*l].decl);
				*leader_of.entry(name).or_insert(m) == m
			})
		})
	}
}

/// Replace the identifiers `from` by `to` in printed MiniZinc
fn rename(text: &str, renaming: &[(String, String)]) -> String {
	let mut result = String::with_capacity(text.len());
	let mut chars = text.char_indices().peekable();
	while let Some((start, c)) = chars.next() {
		if c.is_alphabetic() || c == '_' {
			let mut end = start + c.len_utf8();
			while let Some((i, c)) = chars.next_if(|(_, c)| c.is_alphanumeric() || *c == '_') {
				end = i + c.len_utf8();
			}
			let word = &text[start..end];
			match renaming.iter().find(|(from, _)| from == word) {
				Some((_, to)) => result.push_str(to),
				None => result.push_str(word),
			}
		} else {
			result.push(c);
		}
	}
	result
}
