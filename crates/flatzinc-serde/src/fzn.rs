//! Parse the original `.fzn` file format.

use winnow::{
	ascii::{digit1, hex_digit1, oct_digit1},
	combinator::{alt, opt},
	Parser, Result,
};

use crate::Literal;

fn literal<'s>(input: &mut &'s str) -> Result<Literal> {
	alt((int.map(Literal::Int),)).parse_next(input)
}

/// Parses an integer literal from the input.
///
/// ```bnf
/// <int-literal> ::= [-]?[0-9]+
///                 | [-]?0x[0-9A-Fa-f]+
///                 | [-]?0o[0-7]+
/// ```
fn int(input: &mut &str) -> Result<i64> {
	let is_negative = opt('-').parse_next(input)?.is_some();

	let unsigned_integer = alt((
		("0x", hex_digit1).try_map(|(_, hex)| i64::from_str_radix(hex, 16)),
		("0o", oct_digit1).try_map(|(_, octal)| i64::from_str_radix(octal, 8)),
		digit1.try_map(|base_ten: &str| base_ten.parse::<i64>()),
	))
	.parse_next(input)?;

	if is_negative {
		Ok(-unsigned_integer)
	} else {
		Ok(unsigned_integer)
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn int_literal() {
		assert_eq!(Ok(Literal::Int(0)), literal(&mut "0"));
		assert_eq!(Ok(Literal::Int(420)), literal(&mut "420"));
		assert_eq!(Ok(Literal::Int(-38)), literal(&mut "-38"));
		assert_eq!(Ok(Literal::Int(0xff32a)), literal(&mut "0xff32a"));
		assert_eq!(Ok(Literal::Int(-0xadc20)), literal(&mut "-0xadc20"));
		assert_eq!(Ok(Literal::Int(0o12356)), literal(&mut "0o12356"));
		assert_eq!(Ok(Literal::Int(-0o230)), literal(&mut "-0o230"));
	}
}
