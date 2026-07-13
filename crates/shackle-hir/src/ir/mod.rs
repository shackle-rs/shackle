//! High-level intermediate representation of MiniZinc
pub mod container;
pub mod expression;
pub mod ids;
pub mod item;
pub mod pattern;
pub mod primitive;
pub mod types;

pub use container::*;
pub use expression::*;
pub use item::*;
pub use pattern::*;
pub use primitive::*;
pub use types::*;

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod model {
	use crate::{Item, input::ModelFile};

	/// A model (a single `.mzn` file)
	#[salsa::tracked(debug)]
	pub struct Model<'db> {
		/// The model file this came from
		#[returns(copy)]
		pub file: ModelFile,

		/// Items in original order
		#[tracked]
		pub items: Vec<Item<'db>>,
	}
}

pub use model::Model;
