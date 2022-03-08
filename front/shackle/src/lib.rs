pub mod error;

use error::{NamedSource, ShackleError, SyntaxError};
use tree_sitter::{Parser, TreeCursor};

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

// Parses a list of MiniZinc files given located using the Paths in the vector
pub fn parse_files(paths: Vec<&Path>) -> Result<(), ShackleError> {
	let mut parser = Parser::new();
	parser
		.set_language(tree_sitter_minizinc::language())
		.unwrap();

	for i in paths {
		let file = File::open(Path::new(i)).map_err(|err| ShackleError::FileError {
			file: i.to_path_buf(),
			source: err,
		})?;
		let mut reader = BufReader::new(file);
		let mut buffer = String::new();
		reader
			.read_to_string(&mut buffer)
			.map_err(|err| ShackleError::FileError {
				file: i.to_path_buf(),
				source: err,
			})?;

		let tree = parser
			.parse(&buffer, None)
			.expect("MiniZinc Tree Sitter parser did not return tree object");

		// Find and return syntax errors
		if tree.root_node().has_error() {
			// This would be ideal, but (MISSING) is currently not allowed.
			// let q =
			// 	Query::new(tree_sitter_minizinc::language(), "[(ERROR) (MISSING)] @err").unwrap();
			let mut errors = Vec::new();
			let mut cursor = tree.walk();
			let next_node = |c: &mut TreeCursor| {
				c.goto_next_sibling() || (c.goto_parent() && c.goto_next_sibling())
			};
			loop {
				let node = cursor.node();
				if node.is_error() || node.is_missing() {
					errors.push(SyntaxError {
						src: NamedSource::new(
							i.to_str().expect("Unable to convert path to str"),
							buffer.clone(),
						),
						span: node.byte_range().into(),
						msg: if node.is_missing() {
							format!("Missing {}", node.kind())
						} else {
							format!(
								"Unexpected {}",
								node.child(0)
									.expect("ERROR node must always have a child")
									.kind()
							)
						},
						other: Vec::new(),
					});
					if !next_node(&mut cursor) {
						break;
					}
				} else if node.has_error() && cursor.goto_first_child() {
					continue;
				} else if !next_node(&mut cursor) {
					break;
				}
			}
			let mut first = errors.remove(0);
			first.other = errors;
			return Err(ShackleError::SyntaxError(first));
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		let result = 2 + 2;
		assert_eq!(result, 4);
	}
}
