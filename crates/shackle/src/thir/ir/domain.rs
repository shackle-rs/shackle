//! Representation of variable domains

use std::ops::Deref;

use crate::{
	hir::Identifier,
	thir::db::Thir,
	ty::{Ty, TyData},
};

use super::{ExpressionAllocator, ExpressionId};

pub use crate::hir::{OptType, VarType};

/// Ascribed domain of a variable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Domain {
	ty: Ty,
	data: DomainData,
}

impl Deref for Domain {
	type Target = DomainData;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

/// Ascribed domain of a variable
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainData {
	/// Bounded by an expression
	Bounded(ExpressionId),
	/// Array index sets and element domain
	Array(Box<Domain>, Box<Domain>),
	/// Set domain
	Set(Box<Domain>),
	/// Tuple domain
	Tuple(Vec<Domain>),
	/// Record domain
	Record(Vec<(Identifier, Domain)>),
	/// Unbounded domain
	Unbounded,
}

impl Domain {
	/// The type of the variable this domain is for (not of the domain)
	pub fn ty(&self) -> Ty {
		self.ty
	}

	/// Create a domain for a scalar variable bounded by the given expression
	pub fn bounded(
		db: &dyn Thir,
		owner: &ExpressionAllocator,
		inst: VarType,
		opt: OptType,
		expression: ExpressionId,
	) -> Self {
		let dom_ty = owner[expression].ty();
		let ty = match dom_ty.lookup(db.upcast()) {
			TyData::Set(VarType::Par, OptType::NonOpt, e) => e
				.with_inst(db.upcast(), inst)
				.unwrap()
				.with_opt(db.upcast(), opt),
			_ => unreachable!("Invalid domain type"),
		};
		Self::bounded_unchecked(ty, expression)
	}

	/// Create a domain for a variable of type `ty` bounded by the given `expression`.
	///
	/// Does not check if the domain and type actually make sense.
	pub fn bounded_unchecked(ty: Ty, expression: ExpressionId) -> Self {
		Self {
			ty,
			data: DomainData::Bounded(expression),
		}
	}

	/// Create a domain for an array variable
	pub fn array(db: &dyn Thir, dims: Domain, elem: Domain) -> Self {
		Self {
			ty: Ty::array(db.upcast(), dims.ty(), elem.ty()).expect("Invalid array type"),
			data: DomainData::Array(Box::new(dims), Box::new(elem)),
		}
	}

	/// Create a domain for a set variable bounded by the given expression
	pub fn set(db: &dyn Thir, inst: VarType, opt: OptType, element: Domain) -> Self {
		Self {
			ty: Ty::par_set(db.upcast(), element.ty())
				.expect("Invalid set element type")
				.with_inst(db.upcast(), inst)
				.expect("Cannot make var set domain")
				.with_opt(db.upcast(), opt),
			data: DomainData::Set(Box::new(element)),
		}
	}

	/// Create a domain for a tuple
	pub fn tuple(db: &dyn Thir, fields: impl IntoIterator<Item = Domain>) -> Self {
		let fields: Vec<_> = fields.into_iter().collect();
		Self {
			ty: Ty::tuple(db.upcast(), fields.iter().map(|d| d.ty())),
			data: DomainData::Tuple(fields),
		}
	}

	/// Create a domain for a tuple
	pub fn record(db: &dyn Thir, fields: impl IntoIterator<Item = (Identifier, Domain)>) -> Self {
		let fields: Vec<_> = fields.into_iter().collect();
		Self {
			ty: Ty::record(db.upcast(), fields.iter().map(|(i, d)| (*i, d.ty()))),
			data: DomainData::Record(fields),
		}
	}

	/// Create an unbounded domain for a variable of the given type
	pub fn unbounded(ty: Ty) -> Self {
		Self {
			ty,
			data: DomainData::Unbounded,
		}
	}
}
