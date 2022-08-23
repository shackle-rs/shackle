//! Representation of variable domains

use crate::{arena::ArenaIndex, ty::Ty};

use super::{Expression, ExpressionBuilder, Identifier, ItemData};

/// Ascribed domain of a variable
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Domain {
	/// Bounded by an expression
	Bounded(Ty, ArenaIndex<Expression>),
	/// Array index sets and element domain
	Array(Ty, Box<Domain>, Box<Domain>),
	/// Set domain
	Set(Ty, Box<Domain>),
	/// Tuple domain
	Tuple(Ty, Vec<Domain>),
	/// Record domain
	Record(Ty, Vec<(Identifier, Domain)>),
	/// Unbounded domain
	Unbounded(Ty),
}

impl Domain {
	/// Get the type stored in this domain
	pub fn ty(&self) -> Ty {
		match self {
			Domain::Bounded(ty, _)
			| Domain::Array(ty, _, _)
			| Domain::Set(ty, _)
			| Domain::Tuple(ty, _)
			| Domain::Record(ty, _)
			| Domain::Unbounded(ty) => *ty,
		}
	}
}

/// Builder for domains
#[derive(Clone, Debug)]
pub struct DomainBuilder(Ty, DomainBuilderInner);

impl DomainBuilder {
	/// Create a bounded domain
	pub fn bounded(ty: Ty, bounds: Box<dyn ExpressionBuilder>) -> Self {
		Self(ty, DomainBuilderInner::Bounded(bounds))
	}

	/// Create an array domain
	pub fn array(ty: Ty, index_sets: DomainBuilder, domain: DomainBuilder) -> Self {
		Self(
			ty,
			DomainBuilderInner::Array(Box::new(index_sets), Box::new(domain)),
		)
	}

	/// Create a set domain
	pub fn set(ty: Ty, domain: DomainBuilder) -> Self {
		Self(ty, DomainBuilderInner::Set(Box::new(domain)))
	}

	/// Create a tuple domain
	pub fn tuple(ty: Ty, fields: impl IntoIterator<Item = DomainBuilder>) -> Self {
		Self(ty, DomainBuilderInner::Tuple(fields.into_iter().collect()))
	}

	/// Create a record domain
	pub fn record(ty: Ty, fields: impl IntoIterator<Item = (Identifier, DomainBuilder)>) -> Self {
		Self(ty, DomainBuilderInner::Record(fields.into_iter().collect()))
	}

	/// Create an unbounded domain
	pub fn unbounded(ty: Ty) -> Self {
		Self(ty, DomainBuilderInner::Unbounded)
	}

	/// Finish building domain
	pub fn finish(&self, owner: &mut ItemData) -> Domain {
		match self {
			Self(ty, DomainBuilderInner::Bounded(builder)) => {
				Domain::Bounded(*ty, builder.finish(owner))
			}
			Self(ty, DomainBuilderInner::Array(index_sets, domain)) => Domain::Array(
				*ty,
				Box::new(index_sets.finish(owner)),
				Box::new(domain.finish(owner)),
			),
			Self(ty, DomainBuilderInner::Set(element)) => {
				Domain::Set(*ty, Box::new(element.finish(owner)))
			}
			Self(ty, DomainBuilderInner::Tuple(fields)) => {
				Domain::Tuple(*ty, fields.into_iter().map(|f| f.finish(owner)).collect())
			}
			Self(ty, DomainBuilderInner::Record(fields)) => Domain::Record(
				*ty,
				fields.iter().map(|(i, f)| (*i, f.finish(owner))).collect(),
			),
			Self(ty, DomainBuilderInner::Unbounded) => Domain::Unbounded(*ty),
		}
	}
}

#[derive(Clone, Debug)]
enum DomainBuilderInner {
	Bounded(Box<dyn ExpressionBuilder>),
	Array(Box<DomainBuilder>, Box<DomainBuilder>),
	Set(Box<DomainBuilder>),
	Tuple(Vec<DomainBuilder>),
	Record(Vec<(Identifier, DomainBuilder)>),
	Unbounded,
}
