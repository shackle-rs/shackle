//! Integer viewpoints of 0/1 arrays

use shackle_thir::DeclarationId;

use super::{Globalizer, RawFinding, candidates::Kind};

/// A 2D integer array that represents a 3D 0/1 array `array`, which has
/// exactly one 1 along `axis`
#[derive(Clone, Debug)]
pub(super) struct Viewpoint<'db> {
	pub(super) array: DeclarationId<'db>,
	pub(super) axis: usize,
	pub(super) name: String,
	/// Declaration and channelling constraint (as suggested to the user)
	pub(super) text: String,
	/// Definition as a function of the 0/1 array that does not constrain it,
	/// so it does not change the samples of a group
	pub(super) definition: String,
	/// The channelling constraint, with the domain of the declaration (the
	/// definition has none)
	pub(super) channel: String,
}

impl<'db> Globalizer<'_, 'db> {
	/// The viewpoints for the `binaries_represent_int` findings, sorted by
	/// name (the workers find them in any order)
	pub(super) fn viewpoints(&self, found: &[RawFinding]) -> Vec<Viewpoint<'db>> {
		let mut viewpoints = Vec::<Viewpoint<'db>>::new();
		for raw in found {
			let c = &raw.candidate;
			let Kind::Viewpoint(axis) = c.kind else {
				continue;
			};
			let array = self
				.model
				.top_level_declarations()
				.map(|(d, _)| d)
				.find(|d| self.name(*d) == c.args[0]);
			// A model identifier with the viewpoint's name would clash
			let name = Viewpoint::name(&c.args[0], axis);
			if let Some(array) = array
				&& !self
					.model
					.top_level_declarations()
					.any(|(d, _)| self.name(d) == name)
				&& !viewpoints
					.iter()
					.any(|vp| vp.array == array && vp.axis == axis)
			{
				viewpoints.push(Viewpoint::new(array, &c.args[0], axis));
			}
		}
		viewpoints.sort_by(|a, b| a.name.cmp(&b.name));
		viewpoints
	}
}

impl<'db> Viewpoint<'db> {
	fn name(x: &str, axis: usize) -> String {
		format!("{x}_3{}", ["a", "b", "c"][axis])
	}

	pub(super) fn new(array: DeclarationId<'db>, x: &str, axis: usize) -> Self {
		let name = Self::name(x, axis);
		let set = |k: usize| format!("index_set_{}of3({x})", k + 1);
		let others = (0..3).filter(|k| *k != axis).collect::<Vec<_>>();
		let idx = ["i", "j", "k"];
		let declaration = format!(
			"array [{}, {}] of var {}: {name};",
			set(others[0]),
			set(others[1]),
			set(axis)
		);
		let channel = format!(
			"constraint forall (i in {}, j in {}, k in {}) ({x}[i, j, k] = 1 <-> {name}[{}, {}] = {});",
			set(0),
			set(1),
			set(2),
			idx[others[0]],
			idx[others[1]],
			idx[axis],
		);
		let definition = format!(
			"array [{b}, {c}] of var int: {name} = array2d({b}, {c}, [sum ({} in {}) ({} * {x}[i, j, k]) | {} in {b}, {} in {c}]);",
			idx[axis],
			set(axis),
			idx[axis],
			idx[others[0]],
			idx[others[1]],
			b = set(others[0]),
			c = set(others[1]),
		);
		Viewpoint {
			array,
			axis,
			text: format!("{declaration} {channel}"),
			channel: format!(
				"{channel}\nconstraint forall (v in array1d({name})) (v in {});",
				set(axis)
			),
			name,
			definition,
		}
	}
}
