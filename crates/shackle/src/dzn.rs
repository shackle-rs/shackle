//! # The DataZinc parser
//!
//! This module contains a parser for the DataZinc format (i.e., `.dzn`) files.
//! These files are often used to provide data for MiniZinc models.

use std::{
	fmt::Display,
	fs::read_to_string,
	path::{Path, PathBuf},
	str::FromStr,
	sync::Arc,
};

use itertools::Itertools;
use miette::SourceSpan;
use nom::{
	branch::alt,
	bytes::complete::{is_not, tag, take_until, take_while_m_n},
	character::complete::{alpha1, alphanumeric1, char, multispace1, one_of},
	combinator::{eof, map, map_opt, map_res, opt, recognize, value as replace, verify},
	error::{ErrorKind, ParseError},
	multi::{fold_many0, many0, many0_count, many1},
	number::complete::recognize_float,
	sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
	Finish, IResult, InputLength, Parser,
};
use nom_locate::LocatedSpan;
use rustc_hash::FxHashMap;

use crate::{
	diagnostics::{FileError, ShackleError, SyntaxError},
	value::{Array, Index, Polarity, Record, Set, Value},
};

pub(crate) type Span<'a> = LocatedSpan<&'a str, (Option<PathBuf>, Arc<String>)>;

/// Value parsed in a data file.
///
/// These values can still contain unmatched enum values or enum constructors,
/// for which the internal value has not yet been determined.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedVal {
	/// Normal MiniZinc value
	Val(Value),
	/// Identifier to be matched to an identifier value
	EnumVal(EnumVal),
	/// Constructor used to define an enumerated type, or create a value of an enumerated type.
	EnumCtor(EnumCtor),
	/// Set of values of an enumerated type, specified as a list of values
	EnumSetList(Vec<EnumVal>),
	/// Set of values of an enumerated type, specified as a range
	EnumRange((EnumVal, EnumVal)),
}

impl Display for ParsedVal {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ParsedVal::Val(v) => write!(f, "{v}"),
			ParsedVal::EnumVal(v) => write!(f, "{v}"),
			ParsedVal::EnumCtor(v) => write!(f, "{v}"),
			ParsedVal::EnumSetList(li) => write!(f, "{{{}}}", li.iter().format(", ")),
			ParsedVal::EnumRange((from, to)) => write!(f, "{from}..{to}"),
		}
	}
}

/// Constructor for an enumerated type
#[derive(Debug, Clone, PartialEq)]
pub enum EnumCtor {
	/// List of identifiers describing an enumerated type
	ValueList(Vec<String>),
	/// Constructor call with a set as an argument
	SetArg((String, Set)),
	/// Constructor call with an identifier argument
	NameArg((String, String)),
	/// The concatenation of multiple other types of constructors
	Concat(Vec<EnumCtor>),
}

impl Display for EnumCtor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			EnumCtor::ValueList(li) => write!(f, "{{{}}}", li.iter().format(", ")),
			EnumCtor::SetArg((ident, range)) => write!(f, "{ident}({range})"),
			EnumCtor::NameArg((ident, arg)) => write!(f, "{ident}({arg})"),
			EnumCtor::Concat(li) => write!(f, "{}", li.iter().format(" ++ ")),
		}
	}
}

/// Value of an enumerated type, not yet matched to its type in the instance
#[derive(Debug, Clone, PartialEq)]
pub enum EnumVal {
	/// Simple enumerated value
	Ident(String),
	/// Constructor call with an integer argument
	IntArg((String, i64)),
	/// Constructor call with an identifier argument
	/// Warning: Could be a type constructor if the argument is an identifier points to enumerated type.
	EnumArg((String, Box<EnumVal>)),
}

impl Display for EnumVal {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			EnumVal::Ident(ident) => write!(f, "{ident}"),
			EnumVal::IntArg((ident, i)) => write!(f, "{ident}({i})"),
			EnumVal::EnumArg((ident, arg)) => write!(f, "{ident}({arg})"),
		}
	}
}

// TODO: Use FileError in all the different combinators

/// Parses a DataZinc file, returning a mapping of the name of the left hand
/// side of the assignment items to the values on the right hand side.
///
/// # Warning
/// Enumerated types and their values are unified during parsing. Values of
/// these types will only have a valid reference to their type, and integer
/// value, after it has been fitted to the model. The member declaration of an
/// enumerated type might be represented as a [`SetList`] of [`EnumValue`]s.
pub fn parse_dzn_file(path: &Path) -> Result<FxHashMap<String, ParsedVal>, ShackleError> {
	let content = read_to_string(path).map_err(|err| FileError {
		file: path.to_path_buf(),
		message: err.to_string(),
		other: Vec::new(),
	})?;
	parse_dzn_string(Arc::new(content), Some(path.to_path_buf()))
}

/// Parses a DataZinc string, returning a mapping of the name of the left hand
/// side of the assignment items to the values on the right hand side.
///
/// An optional filename can be given that will be used to indicate the location
/// if an error occurs
pub fn parse_dzn_string(
	content: Arc<String>,
	filename: Option<PathBuf>,
) -> Result<FxHashMap<String, ParsedVal>, ShackleError> {
	let span = Span::new_extra(&content, (filename, content.clone()));
	let result = dzn(span);
	match result.finish() {
		Ok((_, map)) => Ok(map),
		Err(err) => Err(SyntaxError {
			src: err.input.clone().into(),
			span: SourceSpan::new(err.input.location_offset().into(), 0.into()),
			msg: err.to_string(),
			other: Vec::new(),
		}
		.into()),
	}
}

/// Parse given string as DZN definitions
fn dzn(input: Span) -> IResult<Span, FxHashMap<String, ParsedVal>> {
	let (input, map) = seperated_fold(
		assignment,
		sep(char(';')),
		FxHashMap::default,
		|mut map, (k, v)| {
			map.insert(k.into(), v);
			map
		},
	)(input)?;
	let (input, _) = eof(input)?;
	Ok((input, map))
}

/// Parse DZN assignment item
pub fn assignment(input: Span) -> IResult<Span, (&str, ParsedVal)> {
	let (input, ident) = identifier(input)?;
	let (input, _) = sep(char('='))(input)?;
	let (input, val) = value_or_enum(input)?;
	Ok((input, (ident, val)))
}

/// Read and ignore any whitespace or comment items
fn ws(input: Span) -> IResult<Span, ()> {
	let line_comment = replace((), pair(char('%'), is_not("\n\r")));
	let block_comment = replace((), tuple((tag("/*"), take_until("*/"), tag("*/"))));
	let whitespace = replace((), multispace1);

	replace((), many0(alt((line_comment, block_comment, whitespace))))(input)
}

/// Create combinator that serves as a seperator, allowing whitespace (and comments) before and after
fn sep<'a, O, F>(mut f: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, ()>
where
	F: Parser<Span<'a>, O, nom::error::Error<Span<'a>>>,
{
	move |i: Span| {
		let (i, _) = ws(i)?;
		let (i, _) = f.parse(i)?;
		ws(i)
	}
}

/// Parse an identifier that is on the left hand side of an assignment item, or
/// used to identify an enum element, or a enum constructor
fn identifier(input: Span) -> IResult<Span, &str> {
	// FIXME This is the basic rust identifier, but should match MiniZinc's identifiers
	map(
		recognize(pair(
			alt((alpha1, tag("_"))),
			many0_count(alt((alphanumeric1, tag("_")))),
		)),
		|out: Span| *out.fragment(),
	)(input)
}

/// Parse a [`ParsedVal`], possibly using enums, that can be placed on right
/// hand side of a DataZinc assignment item
fn value_or_enum(input: Span) -> IResult<Span, ParsedVal> {
	alt((
		map(value, ParsedVal::Val),
		// WARNING: Should be after values since other identifiers would be matched
		// (e.g., infinity, array1d, enum constructors)
		enum_expression,
	))(input)
}

/// Parse a non-enum [`Value`] that can be placed on right hand side of a
/// DataZinc assignment item
fn value(input: Span) -> IResult<Span, Value> {
	alt((
		replace(Value::Absent, tag("<>")),
		infinity,
		map(boolean, Value::Boolean),
		map(string, Value::String),
		map(array, Value::Array),
		map(set, Value::Set),
		// WARNING: record should come before tuple
		map(record, Value::Record),
		map(tuple_value, Value::Tuple),
		// WARNING: Float must come before integer
		map(float, Value::Float),
		map(integer, Value::Integer),
	))(input)
}

/// Parse an infinity literal
fn infinity(input: Span) -> IResult<Span, Value> {
	let (input, p) = opt(char('-'))(input)?;
	let negate = matches!(p, Some('-'));
	let (input, _) = ws(input)?;
	let (input, _) = alt((tag("infinity"), tag("∞")))(input)?;
	Ok((
		input,
		Value::Infinity(if negate { Polarity::Neg } else { Polarity::Pos }),
	))
}

/// Parse an Boolean literal
///
/// Only simple `true` and `false` are accepted
fn boolean(input: Span) -> IResult<Span, bool> {
	alt((replace(true, tag("true")), replace(false, tag("false"))))(input)
}

/// Parse an integer literal
///
/// Integer literals in DataZinc are allowed to be given in binary, octal,
/// hexadecimal, and decimal notation
fn integer(input: Span) -> IResult<Span, i64> {
	let (input, p) = opt(char('-'))(input)?;
	let negate = matches!(p, Some('-'));
	let (input, _) = ws(input)?;
	let binary = map_res(
		preceded(
			alt((tag("0b"), tag("0B"))),
			recognize(many1(terminated(one_of("01"), many0(char('_'))))),
		),
		|out: Span| i64::from_str_radix(&str::replace(out.fragment(), "_", ""), 2),
	);
	let octal = map_res(
		preceded(
			alt((tag("0o"), tag("0O"))),
			recognize(many1(terminated(one_of("01234567"), many0(char('_'))))),
		),
		|out: Span| i64::from_str_radix(&str::replace(out.fragment(), "_", ""), 8),
	);
	let hexadecimal = map_res(
		preceded(
			alt((tag("0x"), tag("0X"))),
			recognize(many1(terminated(
				one_of("0123456789abcdefABCDEF"),
				many0(char('_')),
			))),
		),
		|out: Span| i64::from_str_radix(&str::replace(out.fragment(), "_", ""), 16),
	);
	let decimal = map_res(
		recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
		|out: Span| str::replace(out.fragment(), "_", "").parse::<i64>(),
	);
	map(alt((binary, octal, hexadecimal, decimal)), move |i| {
		if negate {
			-i
		} else {
			i
		}
	})(input)
}

fn float(input: Span) -> IResult<Span, f64> {
	let (input, p) = opt(char('-'))(input)?;
	let negate = matches!(p, Some('-'));
	let (input, _) = ws(input)?;
	// FIXME: Should guarantee that it starts and ends with a number
	// TODO: Add hexadecimal floating point numbers
	map_res(
		verify(recognize_float, |s: &Span| {
			for c in s.fragment().chars() {
				if c == '.' || c == 'e' || c == 'E' {
					return true;
				}
			}
			false
		}),
		move |s: Span| {
			let f = s.fragment().parse::<f64>()?;
			Ok::<f64, <f64 as FromStr>::Err>(if negate { -f } else { f })
		},
	)(input)
}

/// Parse a string literal
///
/// String literals in DataZinc can contain escape characters that can be used
/// to add special (non-ascii) characters or as a way to use white-space that is
/// not contained in the final string
///
/// ## Warning
///
/// String literals in DataZinc cannot contain string interpolation. Any
/// interpolation that has to be executed has to be contained in the model, not
/// in the data.
fn string(input: Span) -> IResult<Span, String> {
	// Parse a backslash, followed by any amount of whitespace. This is used
	// later to discard any escaped whitespace.
	let escaped_whitespace = preceded(char('\\'), multispace1);
	// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
	let escaped_char = preceded(
		char('\\'),
		alt((
			unicode,
			replace('\n', char('n')),
			replace('\r', char('r')),
			replace('\t', char('t')),
			replace('\u{08}', char('b')),
			replace('\u{0C}', char('f')),
			replace('\\', char('\\')),
			replace('/', char('/')),
			replace('"', char('"')),
		)),
	);
	// Parse a non-empty block of text that doesn't include \ or "
	let simple_literal = verify(is_not("\"\\"), |s: &Span| !s.fragment().is_empty());

	let build_string = fold_many0(
		alt((
			map(simple_literal, |s: Span| s.fragment().to_string()),
			map(escaped_char, |c| c.to_string()),
			replace("".to_string(), escaped_whitespace),
		)),
		String::new,
		|mut str, frag| {
			str.push_str(&frag);
			str
		},
	);

	delimited(char('"'), build_string, char('"'))(input)
}

/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1 to 6
/// hexadecimal numerals.
fn unicode(input: Span) -> IResult<Span, char> {
	// Parse string fragment
	let delimited_hex = preceded(
		char('u'),
		delimited(
			char('{'),
			take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit()),
			char('}'),
		),
	);
	// Convert to corresponding integer value
	let convert_u32 = map_res(delimited_hex, move |hex: Span| {
		u32::from_str_radix(hex.fragment(), 16)
	});
	// Convert to corresponding character
	map_opt(convert_u32, std::char::from_u32)(input)
}

fn array(input: Span) -> IResult<Span, Array> {
	let list = seperated_fold(value, sep(char(',')), Vec::new, |mut v, e| {
		v.push(e);
		v
	});

	let simple_list = map(delimited(char('['), list, char(']')), |v| {
		if !v.is_empty() {
			Array::new(vec![Index::Integer(1..=v.len() as i64)], v)
		} else {
			Array::default()
		}
	});

	alt((simple_list,))(input)
}

fn set(input: Span) -> IResult<Span, Set> {
	let list = seperated_fold(value, sep(char(',')), Vec::new, |mut v, e| {
		v.push(e);
		v
	});
	let simple_list = map(
		delimited(sep(char('{')), list, sep(char('}'))),
		Set::SetList,
	);
	let empty_utf8 = map(tag("∅"), |_| Set::SetList(Vec::new()));

	let float_range = map(
		separated_pair(float, sep(tag("..")), float),
		|(from, to)| Set::FloatRangeList(vec![from..=to]),
	);
	let int_range = map(
		separated_pair(integer, sep(tag("..")), integer),
		|(from, to)| Set::IntRangeList(vec![from..=to]),
	);

	alt((empty_utf8, simple_list, float_range, int_range))(input)
}

fn enum_value(input: Span) -> IResult<Span, EnumVal> {
	let (input, ident) = identifier(input)?;
	let (input, val) = if let Ok((inner, _)) = sep(char('('))(input.clone()) {
		if let Ok((inner, i)) = integer(inner.clone()) {
			(inner, EnumVal::IntArg((ident.to_owned(), i)))
		} else {
			let (inner, arg) = enum_value(inner)?;
			(inner, EnumVal::EnumArg((ident.to_owned(), Box::new(arg))))
		}
	} else {
		return Ok((input, EnumVal::Ident(ident.to_owned())));
	};
	let (input, _) = sep(char(')'))(input)?;
	Ok((input, val))
}

fn enum_set_list(input: Span) -> IResult<Span, ParsedVal> {
	let list = seperated_fold(enum_value, sep(char(',')), Vec::new, |mut v, c| {
		v.push(c);
		v
	});
	map(
		delimited(sep(char('{')), list, sep(char('}'))),
		ParsedVal::EnumSetList,
	)(input)
}

fn enum_ctor(input: Span) -> IResult<Span, EnumCtor> {
	let ctor_call = |input| -> IResult<_, _> {
		let (input, ident) = identifier(input)?;
		let (input, _) = sep(char('('))(input)?;
		let (input, ctor) = if let Ok((inner, s)) = set(input.clone()) {
			(inner, EnumCtor::SetArg((ident.to_owned(), s)))
		} else {
			let (input, arg) = identifier(input)?;
			(input, EnumCtor::NameArg((ident.to_owned(), arg.to_owned())))
		};
		let (input, _) = sep(char(')'))(input)?;
		Ok((input, ctor))
	};
	alt((
		map(
			delimited(
				sep(char('{')),
				seperated_fold(identifier, sep(char(',')), Vec::new, |mut v, c| {
					v.push(c.to_owned());
					v
				}),
				sep(char('}')),
			),
			EnumCtor::ValueList,
		),
		ctor_call,
	))(input)
}

/// Parser for Enum constructors, enum values, and range literals of made up of enum values
fn enum_expression(input: Span) -> IResult<Span, ParsedVal> {
	let concat_next = |input| -> bool { sep(tag("++"))(input).is_ok() };
	if let Ok((inner, val)) = enum_value(input.clone()) {
		eprintln!("val: {val}");
		if let Ok((inner, _)) = sep(tag(".."))(inner.clone()) {
			let (inner, val2) = enum_value(inner)?;
			return Ok((inner, ParsedVal::EnumRange((val, val2))));
		}
		let mut is_ctor = false;
		if let EnumVal::EnumArg(tup) = &val {
			if let EnumVal::Ident(_) = *tup.1 {
				is_ctor = concat_next(inner.clone());
			}
		}
		if !is_ctor {
			return Ok((inner, ParsedVal::EnumVal(val)));
		}
		// Otherwise, start again from original input
	};
	if let Ok((inner, val)) = enum_set_list(input.clone()) {
		if !concat_next(inner.clone()) {
			return Ok((inner, val));
		}
	}
	let (input, ctors) = seperated_fold(enum_ctor, sep(tag("++")), Vec::new, |mut v, c| {
		v.push(c);
		v
	})(input)?;
	debug_assert!(!ctors.is_empty());
	Ok((
		input,
		ParsedVal::EnumCtor(if ctors.len() == 1 {
			ctors[0].clone()
		} else {
			EnumCtor::Concat(ctors)
		}),
	))
}

fn tuple_value(input: Span) -> IResult<Span, Vec<Value>> {
	let (input, _) = sep(char('('))(input)?;
	let (input, val) = value(input)?;
	let (input, _) = sep(char(','))(input)?;
	let (input, v) = seperated_fold(
		value,
		sep(char(',')),
		|| vec![val.clone()],
		|mut v, e| {
			v.push(e);
			v
		},
	)(input)?;
	let (input, _) = sep(char(')'))(input)?;
	Ok((input, v))
}

fn record(input: Span) -> IResult<Span, Record> {
	let named_value = separated_pair(identifier, sep(char(':')), value);
	let (input, pairs) = delimited(
		sep(char('(')),
		seperated_fold(named_value, sep(char(',')), Vec::new, |mut v, e| {
			v.push(e);
			v
		}),
		sep(char(')')),
	)(input)?;
	// FIXME: Detect duplicates
	// FIXME: Ensure at least 1 element
	let rec = pairs
		.into_iter()
		.map(|(k, v)| (Arc::new(k.to_string()), v))
		.collect();
	Ok((input, rec))
}

/// Helper function that works similar to [`nom::multi::fold_many0`], but instead
/// considers an additional seperator between iterations of the main parser.
///
/// Note that an optional seperator is allowed after the final element.
fn seperated_fold<I, O, O2, E, F, S, G, H, R>(
	mut f: F,
	mut sep: S,
	mut init: H,
	mut g: G,
) -> impl FnMut(I) -> IResult<I, R, E>
where
	I: Clone + InputLength,
	F: Parser<I, O, E>,
	S: Parser<I, O2, E>,
	G: FnMut(R, O) -> R,
	H: FnMut() -> R,
	E: ParseError<I>,
{
	move |i: I| {
		let mut res = init();
		let mut input = i;

		loop {
			let i_ = input.clone();
			let len = input.input_len();
			match f.parse(i_) {
				Ok((i, o)) => {
					// infinite loop check: the parser must always consume
					if i.input_len() == len {
						return Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Many0)));
					}

					res = g(res, o);
					input = i;
				}
				Err(nom::Err::Error(_)) => {
					return Ok((input, res));
				}
				Err(e) => {
					return Err(e);
				}
			}
			let i_ = input.clone();
			match sep.parse(i_) {
				Ok((i, _)) => {
					input = i;
				}
				Err(nom::Err::Error(_)) => {
					return Ok((input, res));
				}
				Err(e) => {
					return Err(e);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use super::{identifier, value, Span};
	use crate::{
		dzn::{value_or_enum, EnumCtor, EnumVal, ParsedVal},
		value::{Array, Enum, Index, Polarity, Record, Set, Value},
	};

	fn span(s: &str) -> Span {
		Span::new_extra(s, (Some("test.dzn".into()), Arc::new(s.to_string())))
	}

	#[test]
	fn test_parse_ident() {
		let (_, out) = identifier(span("Albus")).unwrap();
		assert_eq!(out, "Albus");
		assert!(identifier(span("1")).is_err());
	}

	#[test]
	fn test_parse_absent() {
		let (_, out) = value(span("<>")).unwrap();
		assert_eq!(out, Value::Absent);
	}

	#[test]
	fn test_parse_inf() {
		let (_, out) = value(span("infinity")).unwrap();
		assert_eq!(out, Value::Infinity(Polarity::Pos));
		let (_, out) = value(span("-infinity")).unwrap();
		assert_eq!(out, Value::Infinity(Polarity::Neg));
		let (_, out) = value(span("∞")).unwrap();
		assert_eq!(out, Value::Infinity(Polarity::Pos));
		let (_, out) = value(span("-∞")).unwrap();
		assert_eq!(out, Value::Infinity(Polarity::Neg));
	}

	#[test]
	fn test_parse_boolean() {
		let (_, out) = value(span("true")).unwrap();
		assert_eq!(out, Value::Boolean(true));
		let (_, out) = value(span("false")).unwrap();
		assert_eq!(out, Value::Boolean(false));
	}

	#[test]
	fn test_parse_integer() {
		let (_, out) = value(span("0")).unwrap();
		assert_eq!(out, Value::Integer(0));
		let (_, out) = value(span("1")).unwrap();
		assert_eq!(out, Value::Integer(1));
		let (_, out) = value(span("99")).unwrap();
		assert_eq!(out, Value::Integer(99));
		let (_, out) = value(span("-1")).unwrap();
		assert_eq!(out, Value::Integer(-1));
		let (_, out) = value(span("0b1010")).unwrap();
		assert_eq!(out, Value::Integer(10));
		let (_, out) = value(span("0o70")).unwrap();
		assert_eq!(out, Value::Integer(7 * 8));
		let (_, out) = value(span("0xFF")).unwrap();
		assert_eq!(out, Value::Integer(255));
	}
	#[test]
	fn test_parse_float() {
		let (left, out) = value(span("0.")).unwrap();
		assert_eq!(left.fragment(), &"");
		assert_eq!(out, Value::Float(0.));
		let (_, out) = value(span("3.65")).unwrap();
		assert_eq!(out, Value::Float(3.65));
		let (_, out) = value(span("-3.65")).unwrap();
		assert_eq!(out, Value::Float(-3.65));
		let (_, out) = value(span("4.5e10")).unwrap();
		assert_eq!(out, Value::Float(4.5e10));
		let (_, out) = value(span("5E-10")).unwrap();
		assert_eq!(out, Value::Float(5E-10));
	}

	#[test]
	fn test_parse_string() {
		let (_, out) = value(span("\"\"")).unwrap();
		assert_eq!(out, Value::String("".to_string()));
		let (_, out) = value(span("\"test\"")).unwrap();
		assert_eq!(out, Value::String("test".to_string()));
		let (_, out) = value(span("\"    Another test    \"")).unwrap();
		assert_eq!(out, Value::String("    Another test    ".to_string()));
		let (_, out) = value(span("\"\\t\\n\"")).unwrap();
		assert_eq!(out, Value::String("\t\n".to_string()));
	}

	#[test]
	fn test_parse_enum_val() {
		// Simple identifier representing an enum value
		let (_, out) = value_or_enum(span("A")).unwrap();
		assert_eq!(out, ParsedVal::EnumVal(EnumVal::Ident("A".to_owned())));

		// Enum value with integer argument
		let (_, out) = value_or_enum(span("A(1)")).unwrap();
		assert_eq!(
			out,
			ParsedVal::EnumVal(EnumVal::IntArg(("A".to_owned(), 1)))
		);

		// Enum value with another enum value as argument
		let (_, out) = value_or_enum(span("A(B)")).unwrap();
		assert_eq!(
			out,
			ParsedVal::EnumVal(EnumVal::EnumArg((
				"A".to_owned(),
				Box::new(EnumVal::Ident("B".to_owned()))
			)))
		);

		// Complex chain of enum constructors to make value
		let (_, out) = value_or_enum(span("A(B(C(D(-60))))")).unwrap();
		assert_eq!(out.to_string(), "A(B(C(D(-60))))");
	}

	#[test]
	fn test_parse_enum_members() {
		let (_, out) = value_or_enum(span("{ A }")).unwrap();
		assert_eq!(
			out,
			ParsedVal::EnumSetList(vec![EnumVal::Ident("A".to_owned())])
		);

		let (_, out) = value_or_enum(span("{ A, B, C }")).unwrap();
		assert_eq!(
			out,
			ParsedVal::EnumSetList(vec![
				EnumVal::Ident("A".to_owned()),
				EnumVal::Ident("B".to_owned()),
				EnumVal::Ident("C".to_owned())
			])
		);

		let (_, out) = value_or_enum(span("X(1..6)")).unwrap();
		assert_eq!(
			out,
			ParsedVal::EnumCtor(EnumCtor::SetArg((
				"X".to_owned(),
				Set::IntRangeList(vec![1..=6])
			)))
		);

		let (_, out) = value_or_enum(span("{ A } ++ Z(-1..1) ++ X(Y)")).unwrap();
		assert_eq!(
			out,
			ParsedVal::EnumCtor(EnumCtor::Concat(vec![
				EnumCtor::ValueList(vec!["A".to_owned()]),
				EnumCtor::SetArg(("Z".to_owned(), Set::IntRangeList(vec![-1..=1]))),
				EnumCtor::NameArg(("X".to_owned(), "Y".to_owned()))
			]))
		)
	}

	#[test]
	fn test_parse_tuple() {
		let (_, out) = value(span("(1,)")).unwrap();
		assert_eq!(out, Value::Tuple(vec![Value::Integer(1)]));
		let (_, out) = value(span("(1, \"foo\")")).unwrap();
		assert_eq!(
			out,
			Value::Tuple(vec![Value::Integer(1), Value::String("foo".to_string())])
		);
		let (_, out) = value(span("(2.5, true, <>,)")).unwrap();
		assert_eq!(
			out,
			Value::Tuple(vec![Value::Float(2.5), Value::Boolean(true), Value::Absent])
		);
		let (_, out) = value(span("([1, 2], {3, 4}, 5)")).unwrap();
		assert_eq!(
			out,
			Value::Tuple(vec![
				Value::Array(Array::new(
					vec![Index::Integer(1..=2)],
					vec![Value::Integer(1), Value::Integer(2)]
				)),
				Value::Set(Set::SetList(vec![Value::Integer(3), Value::Integer(4)])),
				Value::Integer(5)
			])
		);
		let (_, out) = value(span("(1, (2, (4, 5)), 6)")).unwrap();
		assert_eq!(
			out,
			Value::Tuple(vec![
				Value::Integer(1),
				Value::Tuple(vec![
					Value::Integer(2),
					Value::Tuple(vec![Value::Integer(4), Value::Integer(5)])
				]),
				Value::Integer(6)
			])
		);
	}

	#[test]
	fn test_parse_set() {
		let (_, out) = value(span("{ }")).unwrap();
		assert_eq!(out, Value::Set(Set::SetList(vec![])));
		let (_, out) = value(span("∅")).unwrap();
		assert_eq!(out, Value::Set(Set::SetList(vec![])));
		let (_, out) = value(span("{1.0}")).unwrap();
		assert_eq!(out, Value::Set(Set::SetList(vec![Value::Float(1.0)])));
		let (_, out) = value(span("{1,2.2}")).unwrap();
		assert_eq!(
			out,
			Value::Set(Set::SetList(vec![Value::Integer(1), Value::Float(2.2)]))
		);
		let (_, out) = value(span("1..3")).unwrap();
		assert_eq!(out, Value::Set(Set::IntRangeList(vec![1..=3])));
		let (_, out) = value(span("1.0..3.3")).unwrap();
		assert_eq!(out, Value::Set(Set::FloatRangeList(vec![1.0..=3.3])));
	}

	#[test]
	fn test_parse_record() {
		let a = Arc::new("a".to_string());
		let b = Arc::new("b".to_string());
		let c = Arc::new("c".to_string());
		let d = Arc::new("d".to_string());
		let e = Arc::new("e".to_string());
		let f = Arc::new("f".to_string());
		let (_, out) = value(span("(a: 1, b: 2.5)")).unwrap();
		assert_eq!(
			out,
			Value::Record(
				vec![
					(a.clone(), Value::Integer(1)),
					(b.clone(), Value::Float(2.5))
				]
				.into_iter()
				.collect::<Record>()
			)
		);
		let (_, out) = value(span("(a: {1, 2}, b: (3.5, true), c: [<>])")).unwrap();
		assert_eq!(
			out,
			Value::Record(
				vec![
					(
						b.clone(),
						Value::Tuple(vec![Value::Float(3.5), Value::Boolean(true)])
					),
					(
						a.clone(),
						Value::Set(Set::SetList(vec![Value::Integer(1), Value::Integer(2)]))
					),
					(
						c.clone(),
						Value::Array(Array::new(vec![Index::Integer(1..=1)], vec![Value::Absent]))
					)
				]
				.into_iter()
				.collect::<Record>()
			)
		);
		let (_, out) = value(span("(a: 1, b: (c: 2, d: (e: 3, f: 4)))")).unwrap();
		assert_eq!(
			out,
			Value::Record(
				vec![
					(
						b,
						Value::Record(
							vec![
								(c, Value::Integer(2)),
								(
									d,
									Value::Record(
										vec![(e, Value::Integer(3),), (f, Value::Integer(4),),]
											.into_iter()
											.collect::<Record>()
									)
								)
							]
							.into_iter()
							.collect::<Record>()
						)
					),
					(a, Value::Integer(1),),
				]
				.into_iter()
				.collect::<Record>()
			)
		);
	}

	#[test]
	fn test_parse_simple_array() {
		let (_, out) = value(span("[]")).unwrap();
		assert_eq!(out, Value::Array(Array::default()));
		let (_, out) = value(span("[1.0]")).unwrap();
		assert_eq!(
			out,
			Value::Array(Array::new(
				vec![Index::Integer(1..=1)],
				vec![Value::Float(1.0)]
			))
		);
		let (_, out) = value(span("[1, 2.2]")).unwrap();
		assert_eq!(
			out,
			Value::Array(Array::new(
				vec![Index::Integer(1..=2)],
				vec![Value::Integer(1), Value::Float(2.2)]
			))
		);
		let (_, out) = value(span("[<>, <>, 1, <>,]")).unwrap();
		assert_eq!(
			out,
			Value::Array(Array::new(
				vec![Index::Integer(1..=4)],
				vec![
					Value::Absent,
					Value::Absent,
					Value::Integer(1),
					Value::Absent
				]
			))
		);
	}
}
