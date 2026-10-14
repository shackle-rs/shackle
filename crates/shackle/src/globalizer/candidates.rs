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
}

/// A global constraint call to check against the samples
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Candidate {
	pub(super) name: &'static str,
	pub(super) args: Vec<String>,
	/// Par side condition that makes the candidate non-trivial
	pub(super) extra: Option<String>,
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

const GLOBALS: &[Global] = &[Global {
	name: "all_different",
	signature: &[ArgType::Int(1)],
	check: |a| a[0].var,
	extra: |a| Some(format!("length({}) > 1", a[0].text)),
}];

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
				result.push(Candidate {
					name: global.name,
					args: chosen.iter().map(|a| a.text.clone()).collect(),
					extra: (global.extra)(&chosen),
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
		let found = candidates(&args, &[], &[]);
		assert_eq!(
			found,
			[Candidate {
				name: "all_different",
				args: vec!["x".to_owned()],
				extra: Some("length(x) > 1".to_owned()),
			}]
		);
		assert_eq!(found[0].checked(), "(length(x) > 1) /\\ all_different(x)");
	}

	#[test]
	fn filters_by_name() {
		let args = [arg("x", true, ArgType::Int(1))];
		assert!(candidates(&args, &["different".to_owned()], &[]).len() == 1);
		assert!(candidates(&args, &["circuit".to_owned()], &[]).is_empty());
		assert!(candidates(&args, &[], &["all_diff".to_owned()]).is_empty());
	}
}
