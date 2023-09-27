//! AST representation of Eprime items

use super::{Domain, Expression, Identifier};
use crate::syntax::ast::{
	ast_enum, ast_node, child_with_field_name, optional_child_with_field_name, AstNode,
};

ast_enum!(
	/// Item
	Item,
	// "param_decl" => ParamDecl,
	"const_def" => ConstDef,
	// "domain_alias" => DomAlias,
	// "decision_decl" => DecisionDecl,
	// "objective" => Objective,
	// "branching" => Branching,
	// "heuristic" => Heuristic,
	// "constraint" => Constraint,
);

// ast_node!(
//     /// Parameter Declaration
//     ParamDecl,
//     names,
//     domain,
//     wheres,
// );

// impl ParamDecl {
//     // Get variable being declared
//     pub fn names(&self) -> Children<'_, Identifier> {
//         child_with_field_name(self, "name")
//     }

//     // Domain of variable
//     pub fn domain(&self) -> &Domain {
//         child_with_field_name(self, "domain")
//     }

//     // Where clauses
//     pub fn wheres(&self) -> Children<'_, Where> {
//         child_with_field_name(self, "where")
//     }
// }

ast_node!(
	/// Constant Definition
	ConstDef,
	name,
	definition,
	domain,
);

impl ConstDef {
	/// Get constant being declared
	pub fn name(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Definition of constant
	pub fn definition(&self) -> Expression {
		child_with_field_name(self, "definition")
	}

	/// Optional domain of constant
	pub fn domain(&self) -> Option<Domain> {
		optional_child_with_field_name(self, "domain")
	}
}

// ast_node!(
//     /// Domain Alias
//     DomAlias,
//     name,
//     definition,
// );

// impl DomAlias {
//     // Get alias being declared
//     pub fn name(&self) -> &Identifier {
//         child_with_field_name(self, "name")
//     }

//     // Definition of alias
//     pub fn definition(&self) -> &Domain {
//         child_with_field_name(self, "definition")
//     }
// }

// ast_node!(
//     /// Decision Declaration
//     DecisionDecl,
//     names,
//     domain,
// );

// impl DecisionDecl {
//     // Get variables being declared
//     pub fn names(&self) -> Children<'_, Identifier> {
//         child_with_field_name(self, "name")
//     }

//     // Domain of decision
//     pub fn domain(&self) -> &Domain {
//         child_with_field_name(self, "domain")
//     }
// }

// ast_node!(
//     /// Objective
//     Objective,
//     strategy,
//     expression,
// );

// impl Objective {
//     // Get objective strategy
//     pub fn strategy(&self) -> &ObjectiveStrategy {
//         child_with_field_name(self, "strategy")
//     }

//     // Get objective expression
//     pub fn expr(&self) -> &Expression {
//         child_with_field_name(self, "expression")
//     }
// }

// ast_node!(
//     /// Branching
//     Branching,
//     expressions,
// );

// impl Branching {
//     // Get branching expressions
//     pub fn exprs(&self) -> Children<'_, Expression> {
//         child_with_field_name(self, "expression")
//     }
// }

// ast_node!(
//     /// Heuristic
//     Heuristic,
//     heuristic,
// );

// impl Heuristic {
//     // Get heuristic expression
//     pub fn heuristic(&self) -> &Expression {
//         child_with_field_name(self, "heuristic")
//     }
// }

// ast_node!(
//     /// Constraint
//     Constraint,
//     expressions,
// );

// impl Constraint {
//     // Get constraint expressions
//     pub fn exprs(&self) -> Children<'_, Expression> {
//         child_with_field_name(self, "expression")
//     }
// }
