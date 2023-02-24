//! Representation of variable domains
use std::ops::Deref;

use crate::{
	hir::Identifier,
	thir::{db::Thir, source::Origin},
	ty::{Ty, TyData},
};

use super::Expression;

pub use crate::hir::{OptType, VarType};

/// Ascribed domain of a variable
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Domain {
	ty: Ty,
	data: DomainData,
	origin: Origin,
}

impl Deref for Domain {
	type Target = DomainData;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl Domain {
	/// The type of the variable this domain is for (not of the domain)
	pub fn ty(&self) -> Ty {
		self.ty
	}

	/// Get the origin of this domain
	pub fn origin(&self) -> Origin {
		self.origin
	}

	/// Create a domain bounded by an expression
	///
	/// E.g. `var 1..3`
	pub fn bounded(
		db: &dyn Thir,
		origin: impl Into<Origin>,
		inst: VarType,
		opt: OptType,
		expression: Expression,
	) -> Self {
		let dom_ty = expression.ty();
		let ty = match dom_ty.lookup(db.upcast()) {
			TyData::Set(VarType::Par, OptType::NonOpt, e) => e
				.with_inst(db.upcast(), inst)
				.unwrap()
				.with_opt(db.upcast(), opt),
			_ => unreachable!("Invalid domain type"),
		};
		Self {
			ty,
			data: DomainData::Bounded(Box::new(expression)),
			origin: origin.into(),
		}
	}

	/// Create an array domain
	///
	/// E.g. `array [int] of 1..3`
	pub fn array(
		db: &dyn Thir,
		origin: impl Into<Origin>,
		dimensions: Domain,
		element: Domain,
	) -> Self {
		let ty = Ty::array(db.upcast(), dimensions.ty(), element.ty()).expect("Invalid array type");
		Self {
			ty,
			data: DomainData::Array(Box::new(dimensions), Box::new(element)),
			origin: origin.into(),
		}
	}

	/// Create a set variable domain
	///
	/// E.g. `var set of 1..3`
	pub fn set(
		db: &dyn Thir,
		origin: impl Into<Origin>,
		inst: VarType,
		opt: OptType,
		element: Domain,
	) -> Self {
		let ty = Ty::par_set(db.upcast(), element.ty())
			.expect("Invalid set element type")
			.with_inst(db.upcast(), inst)
			.expect("Cannot make var set domain")
			.with_opt(db.upcast(), opt);
		Self {
			ty,
			data: DomainData::Set(Box::new(element)),
			origin: origin.into(),
		}
	}

	/// Create a tuple variable domain
	///
	/// E.g. `tuple(1..2, string)`
	pub fn tuple(
		db: &dyn Thir,
		origin: impl Into<Origin>,
		fields: impl IntoIterator<Item = Domain>,
	) -> Self {
		let fields = fields.into_iter().collect::<Vec<_>>();
		let ty = Ty::tuple(db.upcast(), fields.iter().map(|d| d.ty()));
		Self {
			ty,
			data: DomainData::Tuple(fields),
			origin: origin.into(),
		}
	}

	/// Create a record variable domain
	///
	/// E.g. `record(1..2: x, string: y)`
	pub fn record(
		db: &dyn Thir,
		origin: impl Into<Origin>,
		fields: impl IntoIterator<Item = (Identifier, Domain)>,
	) -> Self {
		let fields = fields.into_iter().collect::<Vec<_>>();
		let ty = Ty::record(db.upcast(), fields.iter().map(|(i, d)| (*i, d.ty())));
		Self {
			ty,
			data: DomainData::Record(fields),
			origin: origin.into(),
		}
	}

	/// Create an unbounded domain
	pub fn unbounded(origin: impl Into<Origin>, ty: Ty) -> Self {
		Self {
			ty,
			data: DomainData::Unbounded,
			origin: origin.into(),
		}
	}

	/// Get the inner data
	pub fn into_inner(self) -> (Ty, DomainData, Origin) {
		(self.ty, self.data, self.origin)
	}
}

/// Ascribed domain of a variable
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DomainData {
	/// Bounded by an expression
	Bounded(Box<Expression>),
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
