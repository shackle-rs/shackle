//! Representation of variable domains
use std::ops::Deref;

use crate::{
	hir::Identifier,
	thir::{db::Thir, source::Origin},
	ty::{Ty, TyData},
};

use super::{Expression, Marker};

pub use crate::hir::{OptType, VarType};

/// Ascribed domain of a variable
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Domain<T = ()> {
	ty: Ty,
	data: DomainData<T>,
	origin: Origin,
}

impl<T: Marker> Deref for Domain<T> {
	type Target = DomainData<T>;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<T: Marker> Domain<T> {
	/// The type of the variable this domain is for (not of the domain)
	pub fn ty(&self) -> Ty {
		self.ty
	}

	/// Get the origin of this domain
	pub fn origin(&self) -> Origin {
		self.origin
	}

	/// Set the type of this domain without checking if it is valid
	pub fn set_ty_unchecked(&mut self, ty: Ty) {
		self.ty = ty;
	}

	/// Create a domain bounded by an expression
	///
	/// E.g. `var 1..3`
	pub fn bounded(
		db: &dyn Thir,
		origin: impl Into<Origin>,
		inst: VarType,
		opt: OptType,
		expression: Expression<T>,
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
		opt: OptType,
		dimensions: Domain<T>,
		element: Domain<T>,
	) -> Self {
		let ty = Ty::array(db.upcast(), dimensions.ty(), element.ty())
			.expect("Invalid array type")
			.with_opt(db.upcast(), opt);
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
		element: Domain<T>,
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
		opt: OptType,
		fields: impl IntoIterator<Item = Domain<T>>,
	) -> Self {
		let fields = fields.into_iter().collect::<Vec<_>>();
		let ty = Ty::tuple(db.upcast(), fields.iter().map(|d| d.ty())).with_opt(db.upcast(), opt);
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
		opt: OptType,
		fields: impl IntoIterator<Item = (Identifier, Domain<T>)>,
	) -> Self {
		let fields = fields.into_iter().collect::<Vec<_>>();
		let ty = Ty::record(db.upcast(), fields.iter().map(|(i, d)| (*i, d.ty())))
			.with_opt(db.upcast(), opt);
		Self {
			ty,
			data: DomainData::Record(fields),
			origin: origin.into(),
		}
	}

	/// Create an unbounded domain
	///
	/// Normalises structured types, so e.g. providing an array type
	/// will create an domain with `DomainData::Array`.
	pub fn unbounded(db: &dyn Thir, origin: impl Into<Origin>, ty: Ty) -> Self {
		let origin = origin.into();
		match ty.lookup(db.upcast()) {
			TyData::Array { opt, dim, element } => Domain::array(
				db,
				origin,
				opt,
				Domain::unbounded(db, origin, dim),
				Domain::unbounded(db, origin, element),
			),
			TyData::Set(inst, opt, elem) => {
				Domain::set(db, origin, inst, opt, Domain::unbounded(db, origin, elem))
			}
			TyData::Tuple(opt, fields) => Domain::tuple(
				db,
				origin,
				opt,
				fields.iter().map(|f| Domain::unbounded(db, origin, *f)),
			),
			TyData::Record(opt, fields) => Domain::record(
				db,
				origin,
				opt,
				fields
					.iter()
					.map(|(i, f)| (Identifier(*i), Domain::unbounded(db, origin, *f))),
			),
			_ => Domain {
				ty,
				data: DomainData::Unbounded,
				origin,
			},
		}
	}

	/// Get the inner data
	pub fn into_inner(self) -> (Ty, DomainData<T>, Origin) {
		(self.ty, self.data, self.origin)
	}

	/// Walk the contents of this domain
	pub fn walk(&self) -> impl Iterator<Item = &Domain<T>> {
		let mut todo = vec![self];
		std::iter::from_fn(move || {
			let next = todo.pop()?;
			match &**next {
				DomainData::Array(dim, el) => {
					todo.push(el);
					todo.push(dim);
				}
				DomainData::Set(el) => {
					todo.push(el);
				}
				DomainData::Tuple(fields) => {
					todo.extend(fields.iter().rev());
				}
				DomainData::Record(fields) => {
					todo.extend(fields.iter().rev().map(|(_, f)| f));
				}
				_ => (),
			}
			Some(next)
		})
	}
}

/// Ascribed domain of a variable
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DomainData<T = ()> {
	/// Bounded by an expression
	Bounded(Box<Expression<T>>),
	/// Array index sets and element domain
	Array(Box<Domain<T>>, Box<Domain<T>>),
	/// Set domain
	Set(Box<Domain<T>>),
	/// Tuple domain
	Tuple(Vec<Domain<T>>),
	/// Record domain
	Record(Vec<(Identifier, Domain<T>)>),
	/// Unbounded domain
	Unbounded,
}
