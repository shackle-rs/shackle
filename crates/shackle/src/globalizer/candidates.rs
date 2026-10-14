//! Table of global constraints the globalizer can suggest, with their argument
//! signatures and the filters that reject trivial or ill-typed argument lists.
//!
//! Ported from `GroupOutput.potentialConstraints`, `check` and `extra` in the
//! Haskell globalizer.

use itertools::Itertools;

/// Type of a candidate argument
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum ArgType {
	/// Integer (or integer array with the given number of dimensions)
	Int(usize),
}

/// A possible argument for a candidate global
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Arg {
	/// MiniZinc text of the argument
	pub(super) text: String,
	/// Whether the argument contains decision variables
	pub(super) var: bool,
	pub(super) ty: ArgType,
	/// Leaders (indices into the variant's leaders) the argument uses
	pub(super) leaders: Vec<usize>,
}

/// A global constraint call to check against the samples
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Candidate {
	pub(super) name: &'static str,
	pub(super) args: Vec<String>,
	/// Par side condition that makes the candidate non-trivial
	pub(super) extra: Option<String>,
	/// Leaders the arguments use (sorted)
	pub(super) leaders: Vec<usize>,
}

impl Candidate {
	/// The global call, e.g. `all_different(x)`
	pub(super) fn call(&self) -> String {
		format!("{}({})", self.name, self.args.join(", "))
	}

	/// The call guarded by its side condition
	pub(super) fn checked(&self) -> String {
		match &self.extra {
			Some(extra) => format!("({extra}) /\\ {}", self.call()),
			None => self.call(),
		}
	}
}

struct Global {
	name: &'static str,
	signature: &'static [ArgType],
	/// Rejects argument lists the global is not useful for
	check: fn(&[&Arg]) -> bool,
	extra: fn(&[&Arg]) -> Option<String>,
}

const GLOBALS: &[Global] = &[
	Global {
		name: "all_different",
		signature: &[ArgType::Int(1)],
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
	},
	Global {
		name: "decreasing",
		signature: &[ArgType::Int(1)],
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
	},
	Global {
		name: "increasing",
		signature: &[ArgType::Int(1)],
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
	},
];

/// Whether `name` passes the include/exclude substring filters
pub(super) fn selected(name: &str, include: &[String], exclude: &[String]) -> bool {
	(include.is_empty() || include.iter().any(|f| name.contains(f.as_str())))
		&& !exclude.iter().any(|f| name.contains(f.as_str()))
}

/// All candidate calls that can be built from `args`
pub(super) fn candidates(args: &[Arg], include: &[String], exclude: &[String]) -> Vec<Candidate> {
	let mut result = Vec::new();
	for global in GLOBALS {
		if !selected(global.name, include, exclude) {
			continue;
		}
		let slots = global
			.signature
			.iter()
			.map(|ty| args.iter().filter(|a| a.ty == *ty).collect::<Vec<_>>());
		for chosen in slots.multi_cartesian_product() {
			if (global.check)(&chosen) {
				let mut leaders = chosen
					.iter()
					.flat_map(|a| a.leaders.iter().copied())
					.collect::<Vec<_>>();
				leaders.sort_unstable();
				leaders.dedup();
				result.push(Candidate {
					name: global.name,
					args: chosen.iter().map(|a| a.text.clone()).collect(),
					extra: (global.extra)(&chosen),
					leaders,
				});
			}
		}
	}
	result
}

#[cfg(test)]
mod tests {
	use super::*;

	fn arg(text: &str, var: bool, ty: ArgType) -> Arg {
		Arg {
			text: text.to_owned(),
			var,
			ty,
			leaders: Vec::new(),
		}
	}

	#[test]
	fn all_different_needs_a_var_array() {
		let args = [
			arg("x", true, ArgType::Int(1)),
			arg("p", false, ArgType::Int(1)),
			arg("y", true, ArgType::Int(0)),
			arg("m", true, ArgType::Int(2)),
		];
		let found = candidates(&args, &["all_different".to_owned()], &[]);
		assert_eq!(
			found,
			[Candidate {
				name: "all_different",
				args: vec!["x".to_owned()],
				extra: Some("length(x) > 1".to_owned()),
				leaders: Vec::new(),
			}]
		);
		assert_eq!(found[0].checked(), "(length(x) > 1) /\\ all_different(x)");
	}

	#[test]
	fn increasing_and_decreasing_need_three_elements() {
		let args = [
			arg("x", true, ArgType::Int(1)),
			arg("p", false, ArgType::Int(1)),
		];
		let found = candidates(&args, &["creasing".to_owned()], &[]);
		let checked = found.iter().map(Candidate::checked).collect::<Vec<_>>();
		assert_eq!(
			checked,
			[
				"(length(x) > 2) /\\ decreasing(x)",
				"(length(x) > 2) /\\ increasing(x)"
			]
		);
	}

	#[test]
	fn candidates_collect_leaders_of_their_arguments() {
		let mut row = arg("x[i, ..]", true, ArgType::Int(1));
		row.leaders = vec![1, 0, 1];
		let found = candidates(&[row], &["all_different".to_owned()], &[]);
		assert_eq!(found[0].leaders, [0, 1]);
	}

	#[test]
	fn filters_by_name() {
		let args = [arg("x", true, ArgType::Int(1))];
		assert!(candidates(&args, &["different".to_owned()], &[]).len() == 1);
		assert!(candidates(&args, &["circuit".to_owned()], &[]).is_empty());
		let names = candidates(&args, &[], &["all_diff".to_owned()])
			.into_iter()
			.map(|c| c.name)
			.collect::<Vec<_>>();
		assert_eq!(names, ["decreasing", "increasing"]);
	}
}
