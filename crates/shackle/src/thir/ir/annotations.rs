//! Storage and access for annotations
//!

use std::ops::{Deref, DerefMut};

use super::{Call, Expression, ExpressionData, Identifier, Model, ResolvedIdentifier};

/// Collection of annotations
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Annotations {
	annotations: Vec<Expression>,
}

impl Deref for Annotations {
	type Target = Vec<Expression>;
	fn deref(&self) -> &Self::Target {
		&self.annotations
	}
}

impl DerefMut for Annotations {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.annotations
	}
}

impl Annotations {
	/// Whether or not there is is an annotation atom with the given name
	pub fn has(&self, model: &Model, name: Identifier) -> bool {
		self.annotations.iter().any(|ann| match &**ann {
			ExpressionData::Identifier(ResolvedIdentifier::Annotation(item)) => {
				model[*item].name == Some(name)
			}
			_ => false,
		})
	}

	/// Find an annotation which is a call with the given name
	pub fn get_call(&self, model: &Model, name: Identifier) -> Option<&Expression> {
		self.annotations.iter().find(|ann| match &***ann {
			ExpressionData::Call(Call { function, .. }) => match &***function {
				ExpressionData::Identifier(ResolvedIdentifier::Annotation(item)) => {
					model[*item].name == Some(name)
				}
				_ => false,
			},
			_ => false,
		})
	}
}
