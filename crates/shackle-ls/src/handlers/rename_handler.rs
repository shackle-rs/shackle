use lsp_server::{ErrorCode, ResponseError};
use lsp_types::{request::Rename, Position, RenameParams, TextEdit, Url, WorkspaceEdit};
use shackle::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::source::Point,
	syntax::{cst::Cst, db::SourceParser},
};
use std::collections::HashMap;
use tree_sitter::Node;

use crate::{dispatch::RequestHandler, LanguageServerDatabase};

#[derive(Debug)]
pub struct RenameHandler;

fn get_node_text(node: Node, file_text: &[u8]) -> String {
	return node.utf8_text(file_text).unwrap().to_owned();
}

// DFS to find whether 'node' contains a child node which represents
// the definition of a variable 'name'
fn node_contains_definition_of(node: Node, name: &String, cst: &Cst, file_text: &[u8]) -> bool {
	// If the current node is a declaration,
	// and it's child identifier a) exists and b) has the value 'name',
	// then it does contain the definition

	// Whether a node can define new variables
	let is_definition = |node: Node| {
		return node.kind() == "declaration"
			|| node.kind() == "parameter"
			|| node.kind() == "function_item"
			|| node.kind() == "generator"
			|| node.kind() == "enumeration_members";
	};

	if is_definition(node) {
		// Walk over the children
		// If something is an identifier with the same name, then return true
		let mut children_cursor = node.walk();

		for child_node in node.children(&mut children_cursor) {
			if child_node.kind() == "identifier" && &get_node_text(child_node, file_text) == name {
				return true;
			}
		}
	}

	// Otherwise, DFS over the children in an attempt to find it
	let mut children_cursor = node.walk();
	for child_node in node.children(&mut children_cursor) {
		if node_contains_definition_of(child_node, name, cst, file_text) {
			return true;
		}
	}

	false
}

// Find the node which represents the scope of a given identifier
// in theory this should always return a value, but option just incase (imported functions maybe?)
fn find_scope_of_identifier<'a>(
	identifier: &'a Node,
	cst: &Cst,
	var_name: &String,
	file_text: &[u8],
) -> Option<Node<'a>> {
	// Should be checked beforehand, but no harm in checking again
	if identifier.kind() != "identifier" {
		return None;
	}

	let mut current_ancestor: Option<Node> = identifier.parent();

	// Iterate up the tree until you find something that contains
	// the definition of the given node
	while let Some(current_root) = current_ancestor {
		if node_contains_definition_of(current_root, var_name, cst, file_text) {
			break;
		}
		current_ancestor = current_root.parent();
	}

	// check whether a node defines a new scope
	let is_scope = |node: Node| -> bool {
		match node.kind() {
			"source_file" | "predicate" | "function" | "let_expression" | "constraint" => true,
			_ => false,
		}
	};

	// walk up the AST until the lowest scope node ancestor is found
	while current_ancestor.map(is_scope) == Some(false) {
		current_ancestor = current_ancestor.and_then(|x| x.parent());
	}

	return current_ancestor;
}

// Find all the children of 'current_node' that are identifiers with value 'var_name'
fn find_all_identifier_children<'a>(
	current_node: &'a Node,
	var_name: &String,
	file_text: &[u8],
) -> Vec<Node<'a>> {
	let mut to_return = vec![];

	let mut cursor = current_node.walk();

	// This is basically a DFS
	'outer_loop: loop {
		let node = cursor.node();

		if node.kind() == "identifier" {
			let id_name = get_node_text(node, file_text);

			if &id_name == var_name {
				to_return.push(node);
			}
		}

		// If we have reached a leaf, go up
		if !cursor.goto_first_child() {
			while !cursor.goto_next_sibling() {
				if !cursor.goto_parent() {
					break 'outer_loop;
				}
			}
		}
	}

	to_return
}

impl RequestHandler<Rename, (Point, String, ModelRef)> for RenameHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
		params: RenameParams,
	) -> Result<(Point, String, ModelRef), ResponseError> {
		// Verify if the new name is valid
		let mut possible_new_name = params.new_name.chars();

		// The new name must be nonempty
		if let Some(ch) = possible_new_name.nth(0) {
			// The new name must begin with a letter
			if !ch.is_alphabetic() {
				return Err(ResponseError {
					code: ErrorCode::InvalidParams as i32,
					message: "Identifier must begin with a letter".into(),
					data: None,
				});
			}

			// The other characters must be letters, digits or an underscore
			if !possible_new_name.all(|ch| ch.is_alphanumeric() || ch == '_') {
				return Err(ResponseError {
					code: ErrorCode::InvalidParams as i32,
					message: "Identifier can only contain letters, digits or underscores".into(),
					data: None,
				});
			}

			let cursor_pos = Point {
				row: params.text_document_position.position.line as usize,
				column: params.text_document_position.position.character as usize,
			};

			// Need to find the text document
			let text_document =
				db.set_active_file_from_document(&params.text_document_position.text_document)?;

			Ok((cursor_pos, params.new_name, text_document))
		} else {
			return Err(ResponseError {
				code: ErrorCode::InvalidParams as i32,
				message: "Identifier must not be empty".into(),
				data: None,
			});
		}
	}

	fn execute(
		db: &CompilerDatabase,
		(start, new_name, doc): (Point, String, ModelRef),
	) -> Result<Option<WorkspaceEdit>, ResponseError> {
		// If this panics something is very wrong anyway
		let cst = db.cst(*doc).unwrap();

		// Try and find the symbol containing the given node
		// Same method used in find definition
		let line = Point {
			row: start.row,
			column: 0,
		};

		let root = cst.root_node();
		let containing: Node<'_> = match root
			.named_descendant_for_point_range(start, start)
			.or(root.named_descendant_for_point_range(line, line))
		{
			Some(n) => n,
			None => {
				return Err(ResponseError {
					code: ErrorCode::InvalidRequest as i32,
					message: format!("Could not find symbol to rename"),
					data: None,
				})
			}
		};

		// The text of the full file
		// Needed to reconstruct the variable names
		let file_text = cst.text().as_bytes();

		// The variable name that is being renamed
		let var_name = get_node_text(containing, file_text);

		// Find the scope of this variable, if it exists
		let scope_node = find_scope_of_identifier(&containing, &cst, &var_name, file_text)
			.expect("Every node should have some scope above it");

		// Find all identifier children of this node with the correct name
		let all_identifiers = find_all_identifier_children(&scope_node, &var_name, file_text);

		// The url to the file we are in
		// Again, if this panics something is very wrong
		let url = Url::from_file_path(&doc.path(db).unwrap()).unwrap();

		// translate Treesitter Point' to LSP Position
		let to_position = |p: Point| return Position::new(p.row as u32, p.column as u32);

		// Create a vector of all the edits to the file
		let all_edits = all_identifiers
			.iter()
			.map(|identifier_node| {
				let node_range = lsp_types::Range {
					start: to_position(identifier_node.range().start_point),
					end: to_position(identifier_node.range().end_point),
				};
				return TextEdit::new(node_range, new_name.clone());
			})
			.collect();

		// The changes to return
		let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
		changes.insert(url, all_edits);

		return Ok(Some(WorkspaceEdit {
			changes: Some(changes),
			document_changes: None,
			change_annotations: None,
		}));
	}
}
