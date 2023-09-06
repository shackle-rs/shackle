//! Storage and access for annotations
//!

use std::ops::{Deref, DerefMut};

use super::{
	Call, Callable, Expression, ExpressionData, Identifier, Marker, Model, ResolvedIdentifier,
};

/// Collection of annotations
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Annotations<T: Marker = ()> {
	annotations: Vec<Expression<T>>,
}

impl<T: Marker> Default for Annotations<T> {
	fn default() -> Self {
		Self {
			annotations: Vec::new(),
		}
	}
}

impl<T: Marker> Deref for Annotations<T> {
	type Target = Vec<Expression<T>>;
	fn deref(&self) -> &Self::Target {
		&self.annotations
	}
}

impl<T: Marker> DerefMut for Annotations<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.annotations
	}
}

impl<T: Marker> Annotations<T> {
	/// Whether or not there is is an annotation atom with the given name
	pub fn has(&self, model: &Model<T>, name: Identifier) -> bool {
		self.annotations.iter().any(|ann| match &**ann {
			ExpressionData::Identifier(ResolvedIdentifier::Annotation(item)) => {
				model[*item].name == Some(name)
			}
			_ => false,
		})
	}

	/// Find an annotation which is a call with the given name
	pub fn get_call(&self, model: &Model<T>, name: Identifier) -> Option<&Expression<T>> {
		self.annotations.iter().find(|ann| match &***ann {
			ExpressionData::Call(Call {
				function: Callable::Annotation(item),
				..
			}) => model[*item].name == Some(name),
			_ => false,
		})
	}
}
