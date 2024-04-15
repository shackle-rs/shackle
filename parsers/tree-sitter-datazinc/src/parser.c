#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 136
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 124
#define ALIAS_COUNT 0
#define TOKEN_COUNT 97
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 15
#define MAX_ALIAS_SEQUENCE_LENGTH 6
#define PRODUCTION_ID_COUNT 40

enum ts_symbol_identifiers {
  sym_identifier = 1,
  anon_sym_SEMI = 2,
  anon_sym_annotation = 3,
  anon_sym_EQ = 4,
  anon_sym_constraint = 5,
  anon_sym_COLON = 6,
  anon_sym_enum = 7,
  anon_sym_PLUS_PLUS = 8,
  anon_sym_function = 9,
  anon_sym_solve = 10,
  anon_sym_satisfy = 11,
  anon_sym_maximize = 12,
  anon_sym_minimize = 13,
  anon_sym_include = 14,
  anon_sym_output = 15,
  anon_sym_COLON_COLON = 16,
  anon_sym_predicate = 17,
  anon_sym_test = 18,
  anon_sym_LPAREN = 19,
  anon_sym_COMMA = 20,
  anon_sym_RPAREN = 21,
  anon_sym_LBRACE = 22,
  anon_sym_RBRACE = 23,
  anon_sym_type = 24,
  anon_sym_LBRACK = 25,
  anon_sym_PIPE = 26,
  anon_sym_RBRACK = 27,
  anon_sym_in = 28,
  anon_sym_where = 29,
  anon_sym_if = 30,
  anon_sym_then = 31,
  anon_sym_elseif = 32,
  anon_sym_else = 33,
  anon_sym_endif = 34,
  anon_sym_DOT_DOT = 35,
  anon_sym_LT_DOT_DOT = 36,
  anon_sym_DOT = 37,
  aux_sym_tuple_access_token1 = 38,
  anon_sym_union = 39,
  anon_sym_u222a = 40,
  anon_sym_case = 41,
  anon_sym_of = 42,
  anon_sym_endcase = 43,
  anon_sym_EQ_GT = 44,
  anon_sym_lambda = 45,
  anon_sym_let = 46,
  anon_sym_DASH = 47,
  anon_sym_not = 48,
  anon_sym_u00ac = 49,
  anon_sym_DQUOTE = 50,
  anon_sym_BSLASH_LPAREN = 51,
  anon_sym_array = 52,
  anon_sym_var = 53,
  anon_sym_par = 54,
  anon_sym_opt = 55,
  anon_sym_set = 56,
  anon_sym_tuple = 57,
  anon_sym_record = 58,
  anon_sym_op = 59,
  anon_sym_any = 60,
  anon_sym_ann = 61,
  anon_sym_bool = 62,
  anon_sym_float = 63,
  anon_sym_int = 64,
  anon_sym_string = 65,
  sym_type_inst_id = 66,
  sym_type_inst_enum_id = 67,
  sym_absent = 68,
  sym_anonymous = 69,
  anon_sym_LBRACK_PIPE = 70,
  anon_sym_PIPE_RBRACK = 71,
  anon_sym_true = 72,
  anon_sym_false = 73,
  sym_float_literal = 74,
  sym_integer_literal = 75,
  sym_infinity = 76,
  anon_sym_u2205 = 77,
  sym_string_characters = 78,
  anon_sym_BSLASH_SQUOTE = 79,
  anon_sym_BSLASH_DQUOTE = 80,
  anon_sym_BSLASH_BSLASH = 81,
  anon_sym_BSLASHr = 82,
  anon_sym_BSLASHn = 83,
  anon_sym_BSLASHt = 84,
  anon_sym_BSLASH = 85,
  aux_sym_escape_sequence_token1 = 86,
  anon_sym_BSLASHx = 87,
  aux_sym_escape_sequence_token2 = 88,
  anon_sym_BSLASHu = 89,
  aux_sym_escape_sequence_token3 = 90,
  anon_sym_BSLASHU = 91,
  aux_sym_escape_sequence_token4 = 92,
  sym_quoted_identifier = 93,
  anon_sym_CARET_DASH1 = 94,
  sym_line_comment = 95,
  sym_block_comment = 96,
  sym_source_file = 97,
  sym_assignment = 98,
  sym__expression = 99,
  sym_call = 100,
  sym_infix_operator = 101,
  sym_array_literal = 102,
  sym_array_literal_member = 103,
  sym_array_literal_2d = 104,
  sym_array_literal_2d_row = 105,
  sym_boolean_literal = 106,
  sym_set_literal = 107,
  sym_string_literal = 108,
  aux_sym__string_content = 109,
  sym_escape_sequence = 110,
  sym_tuple_literal = 111,
  sym_record_literal = 112,
  sym_record_member = 113,
  sym__identifier = 114,
  sym__call_arg = 115,
  aux_sym_source_file_repeat1 = 116,
  aux_sym_call_repeat1 = 117,
  aux_sym_array_literal_repeat1 = 118,
  aux_sym_array_literal_2d_repeat1 = 119,
  aux_sym_array_literal_2d_repeat2 = 120,
  aux_sym_array_literal_2d_row_repeat1 = 121,
  aux_sym_set_literal_repeat1 = 122,
  aux_sym_record_literal_repeat1 = 123,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_identifier] = "identifier",
  [anon_sym_SEMI] = ";",
  [anon_sym_annotation] = "annotation",
  [anon_sym_EQ] = "=",
  [anon_sym_constraint] = "constraint",
  [anon_sym_COLON] = ":",
  [anon_sym_enum] = "enum",
  [anon_sym_PLUS_PLUS] = "++",
  [anon_sym_function] = "function",
  [anon_sym_solve] = "solve",
  [anon_sym_satisfy] = "satisfy",
  [anon_sym_maximize] = "maximize",
  [anon_sym_minimize] = "minimize",
  [anon_sym_include] = "include",
  [anon_sym_output] = "output",
  [anon_sym_COLON_COLON] = "::",
  [anon_sym_predicate] = "predicate",
  [anon_sym_test] = "test",
  [anon_sym_LPAREN] = "(",
  [anon_sym_COMMA] = ",",
  [anon_sym_RPAREN] = ")",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_type] = "type",
  [anon_sym_LBRACK] = "[",
  [anon_sym_PIPE] = "|",
  [anon_sym_RBRACK] = "]",
  [anon_sym_in] = "in",
  [anon_sym_where] = "where",
  [anon_sym_if] = "if",
  [anon_sym_then] = "then",
  [anon_sym_elseif] = "elseif",
  [anon_sym_else] = "else",
  [anon_sym_endif] = "endif",
  [anon_sym_DOT_DOT] = "..",
  [anon_sym_LT_DOT_DOT] = "<..",
  [anon_sym_DOT] = ".",
  [aux_sym_tuple_access_token1] = "integer_literal",
  [anon_sym_union] = "union",
  [anon_sym_u222a] = "\u222a",
  [anon_sym_case] = "case",
  [anon_sym_of] = "of",
  [anon_sym_endcase] = "endcase",
  [anon_sym_EQ_GT] = "=>",
  [anon_sym_lambda] = "lambda",
  [anon_sym_let] = "let",
  [anon_sym_DASH] = "-",
  [anon_sym_not] = "not",
  [anon_sym_u00ac] = "\u00ac",
  [anon_sym_DQUOTE] = "\"",
  [anon_sym_BSLASH_LPAREN] = "\\(",
  [anon_sym_array] = "array",
  [anon_sym_var] = "var",
  [anon_sym_par] = "par",
  [anon_sym_opt] = "opt",
  [anon_sym_set] = "set",
  [anon_sym_tuple] = "tuple",
  [anon_sym_record] = "record",
  [anon_sym_op] = "op",
  [anon_sym_any] = "any",
  [anon_sym_ann] = "ann",
  [anon_sym_bool] = "bool",
  [anon_sym_float] = "float",
  [anon_sym_int] = "int",
  [anon_sym_string] = "string",
  [sym_type_inst_id] = "type_inst_id",
  [sym_type_inst_enum_id] = "type_inst_enum_id",
  [sym_absent] = "absent",
  [sym_anonymous] = "anonymous",
  [anon_sym_LBRACK_PIPE] = "[|",
  [anon_sym_PIPE_RBRACK] = "|]",
  [anon_sym_true] = "true",
  [anon_sym_false] = "false",
  [sym_float_literal] = "float_literal",
  [sym_integer_literal] = "integer_literal",
  [sym_infinity] = "infinity",
  [anon_sym_u2205] = "\u2205",
  [sym_string_characters] = "string_characters",
  [anon_sym_BSLASH_SQUOTE] = "'",
  [anon_sym_BSLASH_DQUOTE] = "\"",
  [anon_sym_BSLASH_BSLASH] = "\\",
  [anon_sym_BSLASHr] = "\r",
  [anon_sym_BSLASHn] = "\n",
  [anon_sym_BSLASHt] = "\t",
  [anon_sym_BSLASH] = "\\",
  [aux_sym_escape_sequence_token1] = "octal",
  [anon_sym_BSLASHx] = "\\x",
  [aux_sym_escape_sequence_token2] = "hexadecimal",
  [anon_sym_BSLASHu] = "\\u",
  [aux_sym_escape_sequence_token3] = "hexadecimal",
  [anon_sym_BSLASHU] = "\\U",
  [aux_sym_escape_sequence_token4] = "hexadecimal",
  [sym_quoted_identifier] = "quoted_identifier",
  [anon_sym_CARET_DASH1] = "^-1",
  [sym_line_comment] = "line_comment",
  [sym_block_comment] = "block_comment",
  [sym_source_file] = "source_file",
  [sym_assignment] = "assignment",
  [sym__expression] = "_expression",
  [sym_call] = "call",
  [sym_infix_operator] = "infix_operator",
  [sym_array_literal] = "array_literal",
  [sym_array_literal_member] = "array_literal_member",
  [sym_array_literal_2d] = "array_literal_2d",
  [sym_array_literal_2d_row] = "array_literal_2d_row",
  [sym_boolean_literal] = "boolean_literal",
  [sym_set_literal] = "set_literal",
  [sym_string_literal] = "string_literal",
  [aux_sym__string_content] = "_string_content",
  [sym_escape_sequence] = "escape_sequence",
  [sym_tuple_literal] = "tuple_literal",
  [sym_record_literal] = "record_literal",
  [sym_record_member] = "record_member",
  [sym__identifier] = "_identifier",
  [sym__call_arg] = "_call_arg",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_call_repeat1] = "call_repeat1",
  [aux_sym_array_literal_repeat1] = "array_literal_repeat1",
  [aux_sym_array_literal_2d_repeat1] = "array_literal_2d_repeat1",
  [aux_sym_array_literal_2d_repeat2] = "array_literal_2d_repeat2",
  [aux_sym_array_literal_2d_row_repeat1] = "array_literal_2d_row_repeat1",
  [aux_sym_set_literal_repeat1] = "set_literal_repeat1",
  [aux_sym_record_literal_repeat1] = "record_literal_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_identifier] = sym_identifier,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [anon_sym_annotation] = anon_sym_annotation,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_constraint] = anon_sym_constraint,
  [anon_sym_COLON] = anon_sym_COLON,
  [anon_sym_enum] = anon_sym_enum,
  [anon_sym_PLUS_PLUS] = anon_sym_PLUS_PLUS,
  [anon_sym_function] = anon_sym_function,
  [anon_sym_solve] = anon_sym_solve,
  [anon_sym_satisfy] = anon_sym_satisfy,
  [anon_sym_maximize] = anon_sym_maximize,
  [anon_sym_minimize] = anon_sym_minimize,
  [anon_sym_include] = anon_sym_include,
  [anon_sym_output] = anon_sym_output,
  [anon_sym_COLON_COLON] = anon_sym_COLON_COLON,
  [anon_sym_predicate] = anon_sym_predicate,
  [anon_sym_test] = anon_sym_test,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_type] = anon_sym_type,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_PIPE] = anon_sym_PIPE,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [anon_sym_in] = anon_sym_in,
  [anon_sym_where] = anon_sym_where,
  [anon_sym_if] = anon_sym_if,
  [anon_sym_then] = anon_sym_then,
  [anon_sym_elseif] = anon_sym_elseif,
  [anon_sym_else] = anon_sym_else,
  [anon_sym_endif] = anon_sym_endif,
  [anon_sym_DOT_DOT] = anon_sym_DOT_DOT,
  [anon_sym_LT_DOT_DOT] = anon_sym_LT_DOT_DOT,
  [anon_sym_DOT] = anon_sym_DOT,
  [aux_sym_tuple_access_token1] = sym_integer_literal,
  [anon_sym_union] = anon_sym_union,
  [anon_sym_u222a] = anon_sym_u222a,
  [anon_sym_case] = anon_sym_case,
  [anon_sym_of] = anon_sym_of,
  [anon_sym_endcase] = anon_sym_endcase,
  [anon_sym_EQ_GT] = anon_sym_EQ_GT,
  [anon_sym_lambda] = anon_sym_lambda,
  [anon_sym_let] = anon_sym_let,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_not] = anon_sym_not,
  [anon_sym_u00ac] = anon_sym_u00ac,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [anon_sym_BSLASH_LPAREN] = anon_sym_BSLASH_LPAREN,
  [anon_sym_array] = anon_sym_array,
  [anon_sym_var] = anon_sym_var,
  [anon_sym_par] = anon_sym_par,
  [anon_sym_opt] = anon_sym_opt,
  [anon_sym_set] = anon_sym_set,
  [anon_sym_tuple] = anon_sym_tuple,
  [anon_sym_record] = anon_sym_record,
  [anon_sym_op] = anon_sym_op,
  [anon_sym_any] = anon_sym_any,
  [anon_sym_ann] = anon_sym_ann,
  [anon_sym_bool] = anon_sym_bool,
  [anon_sym_float] = anon_sym_float,
  [anon_sym_int] = anon_sym_int,
  [anon_sym_string] = anon_sym_string,
  [sym_type_inst_id] = sym_type_inst_id,
  [sym_type_inst_enum_id] = sym_type_inst_enum_id,
  [sym_absent] = sym_absent,
  [sym_anonymous] = sym_anonymous,
  [anon_sym_LBRACK_PIPE] = anon_sym_LBRACK_PIPE,
  [anon_sym_PIPE_RBRACK] = anon_sym_PIPE_RBRACK,
  [anon_sym_true] = anon_sym_true,
  [anon_sym_false] = anon_sym_false,
  [sym_float_literal] = sym_float_literal,
  [sym_integer_literal] = sym_integer_literal,
  [sym_infinity] = sym_infinity,
  [anon_sym_u2205] = anon_sym_u2205,
  [sym_string_characters] = sym_string_characters,
  [anon_sym_BSLASH_SQUOTE] = anon_sym_BSLASH_SQUOTE,
  [anon_sym_BSLASH_DQUOTE] = anon_sym_DQUOTE,
  [anon_sym_BSLASH_BSLASH] = anon_sym_BSLASH,
  [anon_sym_BSLASHr] = anon_sym_BSLASHr,
  [anon_sym_BSLASHn] = anon_sym_BSLASHn,
  [anon_sym_BSLASHt] = anon_sym_BSLASHt,
  [anon_sym_BSLASH] = anon_sym_BSLASH,
  [aux_sym_escape_sequence_token1] = aux_sym_escape_sequence_token1,
  [anon_sym_BSLASHx] = anon_sym_BSLASHx,
  [aux_sym_escape_sequence_token2] = aux_sym_escape_sequence_token2,
  [anon_sym_BSLASHu] = anon_sym_BSLASHu,
  [aux_sym_escape_sequence_token3] = aux_sym_escape_sequence_token2,
  [anon_sym_BSLASHU] = anon_sym_BSLASHU,
  [aux_sym_escape_sequence_token4] = aux_sym_escape_sequence_token2,
  [sym_quoted_identifier] = sym_quoted_identifier,
  [anon_sym_CARET_DASH1] = anon_sym_CARET_DASH1,
  [sym_line_comment] = sym_line_comment,
  [sym_block_comment] = sym_block_comment,
  [sym_source_file] = sym_source_file,
  [sym_assignment] = sym_assignment,
  [sym__expression] = sym__expression,
  [sym_call] = sym_call,
  [sym_infix_operator] = sym_infix_operator,
  [sym_array_literal] = sym_array_literal,
  [sym_array_literal_member] = sym_array_literal_member,
  [sym_array_literal_2d] = sym_array_literal_2d,
  [sym_array_literal_2d_row] = sym_array_literal_2d_row,
  [sym_boolean_literal] = sym_boolean_literal,
  [sym_set_literal] = sym_set_literal,
  [sym_string_literal] = sym_string_literal,
  [aux_sym__string_content] = aux_sym__string_content,
  [sym_escape_sequence] = sym_escape_sequence,
  [sym_tuple_literal] = sym_tuple_literal,
  [sym_record_literal] = sym_record_literal,
  [sym_record_member] = sym_record_member,
  [sym__identifier] = sym__identifier,
  [sym__call_arg] = sym__call_arg,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_call_repeat1] = aux_sym_call_repeat1,
  [aux_sym_array_literal_repeat1] = aux_sym_array_literal_repeat1,
  [aux_sym_array_literal_2d_repeat1] = aux_sym_array_literal_2d_repeat1,
  [aux_sym_array_literal_2d_repeat2] = aux_sym_array_literal_2d_repeat2,
  [aux_sym_array_literal_2d_row_repeat1] = aux_sym_array_literal_2d_row_repeat1,
  [aux_sym_set_literal_repeat1] = aux_sym_set_literal_repeat1,
  [aux_sym_record_literal_repeat1] = aux_sym_record_literal_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_SEMI] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_annotation] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_constraint] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_enum] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PLUS_PLUS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_function] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_solve] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_satisfy] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_maximize] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minimize] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_include] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_output] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_predicate] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_test] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_type] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PIPE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_in] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_where] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_if] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_then] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_elseif] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_else] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_endif] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DOT_DOT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_DOT_DOT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DOT] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_tuple_access_token1] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_union] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_u222a] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_case] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_of] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_endcase] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_lambda] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_let] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_not] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_u00ac] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASH_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_array] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_var] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_par] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_opt] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_set] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_tuple] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_record] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_op] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_any] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ann] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_bool] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_float] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_int] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_string] = {
    .visible = true,
    .named = false,
  },
  [sym_type_inst_id] = {
    .visible = true,
    .named = true,
  },
  [sym_type_inst_enum_id] = {
    .visible = true,
    .named = true,
  },
  [sym_absent] = {
    .visible = true,
    .named = true,
  },
  [sym_anonymous] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_LBRACK_PIPE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PIPE_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_true] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_false] = {
    .visible = true,
    .named = false,
  },
  [sym_float_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_integer_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_infinity] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_u2205] = {
    .visible = true,
    .named = false,
  },
  [sym_string_characters] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_BSLASH_SQUOTE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASH_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASH_BSLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASHr] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASHn] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASHt] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASH] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_escape_sequence_token1] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASHx] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_escape_sequence_token2] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASHu] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_escape_sequence_token3] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASHU] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_escape_sequence_token4] = {
    .visible = true,
    .named = false,
  },
  [sym_quoted_identifier] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_CARET_DASH1] = {
    .visible = true,
    .named = false,
  },
  [sym_line_comment] = {
    .visible = true,
    .named = true,
  },
  [sym_block_comment] = {
    .visible = true,
    .named = true,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym_assignment] = {
    .visible = true,
    .named = true,
  },
  [sym__expression] = {
    .visible = false,
    .named = true,
    .supertype = true,
  },
  [sym_call] = {
    .visible = true,
    .named = true,
  },
  [sym_infix_operator] = {
    .visible = true,
    .named = true,
  },
  [sym_array_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_array_literal_member] = {
    .visible = true,
    .named = true,
  },
  [sym_array_literal_2d] = {
    .visible = true,
    .named = true,
  },
  [sym_array_literal_2d_row] = {
    .visible = true,
    .named = true,
  },
  [sym_boolean_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_set_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_string_literal] = {
    .visible = true,
    .named = true,
  },
  [aux_sym__string_content] = {
    .visible = false,
    .named = false,
  },
  [sym_escape_sequence] = {
    .visible = true,
    .named = true,
  },
  [sym_tuple_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_record_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_record_member] = {
    .visible = true,
    .named = true,
  },
  [sym__identifier] = {
    .visible = false,
    .named = true,
  },
  [sym__call_arg] = {
    .visible = false,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_call_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_array_literal_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_array_literal_2d_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_array_literal_2d_repeat2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_array_literal_2d_row_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_set_literal_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_record_literal_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_argument = 1,
  field_column_index = 2,
  field_content = 3,
  field_definition = 4,
  field_escape = 5,
  field_function = 6,
  field_index = 7,
  field_item = 8,
  field_left = 9,
  field_member = 10,
  field_name = 11,
  field_operator = 12,
  field_right = 13,
  field_row = 14,
  field_value = 15,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_argument] = "argument",
  [field_column_index] = "column_index",
  [field_content] = "content",
  [field_definition] = "definition",
  [field_escape] = "escape",
  [field_function] = "function",
  [field_index] = "index",
  [field_item] = "item",
  [field_left] = "left",
  [field_member] = "member",
  [field_name] = "name",
  [field_operator] = "operator",
  [field_right] = "right",
  [field_row] = "row",
  [field_value] = "value",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 1},
  [2] = {.index = 1, .length = 1},
  [3] = {.index = 2, .length = 2},
  [4] = {.index = 4, .length = 2},
  [5] = {.index = 6, .length = 2},
  [6] = {.index = 8, .length = 1},
  [7] = {.index = 9, .length = 1},
  [8] = {.index = 10, .length = 1},
  [9] = {.index = 11, .length = 1},
  [10] = {.index = 12, .length = 1},
  [11] = {.index = 13, .length = 1},
  [12] = {.index = 14, .length = 2},
  [13] = {.index = 16, .length = 1},
  [14] = {.index = 17, .length = 1},
  [15] = {.index = 18, .length = 2},
  [16] = {.index = 20, .length = 1},
  [17] = {.index = 21, .length = 2},
  [18] = {.index = 23, .length = 1},
  [19] = {.index = 24, .length = 1},
  [20] = {.index = 25, .length = 2},
  [21] = {.index = 27, .length = 3},
  [22] = {.index = 30, .length = 1},
  [23] = {.index = 31, .length = 2},
  [24] = {.index = 33, .length = 2},
  [25] = {.index = 35, .length = 2},
  [26] = {.index = 37, .length = 2},
  [27] = {.index = 39, .length = 2},
  [28] = {.index = 41, .length = 2},
  [29] = {.index = 43, .length = 2},
  [30] = {.index = 45, .length = 2},
  [31] = {.index = 47, .length = 1},
  [32] = {.index = 48, .length = 2},
  [33] = {.index = 50, .length = 2},
  [34] = {.index = 52, .length = 2},
  [35] = {.index = 54, .length = 2},
  [36] = {.index = 56, .length = 2},
  [37] = {.index = 58, .length = 3},
  [38] = {.index = 61, .length = 3},
  [39] = {.index = 64, .length = 3},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_item, 0},
  [1] =
    {field_item, 0, .inherited = true},
  [2] =
    {field_item, 0, .inherited = true},
    {field_item, 1},
  [4] =
    {field_item, 0, .inherited = true},
    {field_item, 1, .inherited = true},
  [6] =
    {field_definition, 2},
    {field_name, 0},
  [8] =
    {field_value, 0},
  [9] =
    {field_content, 0},
  [10] =
    {field_escape, 0},
  [11] =
    {field_member, 0},
  [12] =
    {field_member, 1},
  [13] =
    {field_member, 1, .inherited = true},
  [14] =
    {field_member, 0, .inherited = true},
    {field_member, 1, .inherited = true},
  [16] =
    {field_escape, 1},
  [17] =
    {field_content, 1, .inherited = true},
  [18] =
    {field_content, 0, .inherited = true},
    {field_content, 1, .inherited = true},
  [20] =
    {field_column_index, 0},
  [21] =
    {field_member, 0},
    {field_member, 1, .inherited = true},
  [23] =
    {field_row, 1},
  [24] =
    {field_column_index, 1, .inherited = true},
  [25] =
    {field_column_index, 0, .inherited = true},
    {field_column_index, 1, .inherited = true},
  [27] =
    {field_left, 0},
    {field_operator, 1},
    {field_right, 2},
  [30] =
    {field_function, 0},
  [31] =
    {field_member, 1},
    {field_member, 2, .inherited = true},
  [33] =
    {field_name, 0},
    {field_value, 2},
  [35] =
    {field_member, 1, .inherited = true},
    {field_member, 2},
  [37] =
    {field_index, 0},
    {field_value, 2},
  [39] =
    {field_index, 0},
    {field_member, 2},
  [41] =
    {field_row, 1},
    {field_row, 2, .inherited = true},
  [43] =
    {field_row, 0, .inherited = true},
    {field_row, 1, .inherited = true},
  [45] =
    {field_column_index, 1, .inherited = true},
    {field_row, 2, .inherited = true},
  [47] =
    {field_argument, 0},
  [48] =
    {field_argument, 2},
    {field_function, 0},
  [50] =
    {field_argument, 2, .inherited = true},
    {field_function, 0},
  [52] =
    {field_argument, 0, .inherited = true},
    {field_argument, 1, .inherited = true},
  [54] =
    {field_member, 1},
    {field_member, 3},
  [56] =
    {field_member, 1},
    {field_member, 3, .inherited = true},
  [58] =
    {field_index, 0},
    {field_member, 2},
    {field_member, 3, .inherited = true},
  [61] =
    {field_argument, 2, .inherited = true},
    {field_argument, 3},
    {field_function, 0},
  [64] =
    {field_member, 1},
    {field_member, 3, .inherited = true},
    {field_member, 4},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 7,
  [8] = 8,
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 54,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 58,
  [59] = 59,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 84,
  [85] = 85,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 95,
  [96] = 96,
  [97] = 97,
  [98] = 98,
  [99] = 99,
  [100] = 100,
  [101] = 101,
  [102] = 102,
  [103] = 103,
  [104] = 104,
  [105] = 105,
  [106] = 106,
  [107] = 107,
  [108] = 108,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 113,
  [114] = 114,
  [115] = 115,
  [116] = 116,
  [117] = 117,
  [118] = 118,
  [119] = 119,
  [120] = 120,
  [121] = 121,
  [122] = 122,
  [123] = 123,
  [124] = 124,
  [125] = 125,
  [126] = 126,
  [127] = 127,
  [128] = 128,
  [129] = 129,
  [130] = 130,
  [131] = 131,
  [132] = 132,
  [133] = 133,
  [134] = 134,
  [135] = 135,
};

static TSCharacterRange sym_identifier_character_set_1[] = {
  {0, 0x08}, {0x0e, 0x1f}, {'#', '#'}, {'0', '9'}, {'?', 'Z'}, {'\\', '\\'}, {'_', 'z'}, {0x7f, 0x218f},
  {0x2191, 0x2191}, {0x2193, 0x21cf}, {0x21d1, 0x21d1}, {0x21d3, 0x21d3}, {0x21d5, 0x2207}, {0x2209, 0x2215}, {0x2217, 0x221d}, {0x221f, 0x2226},
  {0x222b, 0x225f}, {0x2261, 0x2263}, {0x2266, 0x2285}, {0x2288, 0x22ba}, {0x22bc, 0x27f6}, {0x27f8, 0x10ffff},
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(52);
      ADVANCE_MAP(
        '"', 78,
        '$', 4,
        '%', 150,
        '\'', 9,
        '(', 60,
        ')', 62,
        '+', 14,
        ',', 61,
        '-', 76,
        '.', 70,
        '/', 10,
        '0', 122,
        ':', 57,
        ';', 53,
        '<', 17,
        '=', 55,
        '[', 65,
        '\\', 117,
        ']', 67,
        '^', 15,
        '{', 63,
        '|', 66,
        '}', 64,
        0xac, 77,
        0x2205, 100,
        0x222a, 74,
        '8', 73,
        '9', 73,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('1' <= lookahead && lookahead <= '7')) ADVANCE(72);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(3);
      if (lookahead == '"') ADVANCE(78);
      if (lookahead == '%') ADVANCE(106);
      if (lookahead == '/') ADVANCE(104);
      if (lookahead == '\\') ADVANCE(118);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(101);
      if (lookahead != 0) ADVANCE(106);
      END_STATE();
    case 2:
      ADVANCE_MAP(
        '"', 78,
        '%', 150,
        '\'', 9,
        '(', 60,
        ')', 62,
        '-', 22,
        '/', 10,
        '0', 88,
        '<', 24,
        '[', 65,
        ']', 67,
        'i', 140,
        '{', 63,
        '|', 66,
        '}', 64,
        0x2205, 100,
        0x221e, 98,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(2);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(90);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(78);
      if (lookahead == '%') ADVANCE(150);
      if (lookahead == '/') ADVANCE(10);
      if (lookahead == '\\') ADVANCE(118);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(3);
      END_STATE();
    case 4:
      if (lookahead == '$') ADVANCE(50);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(80);
      END_STATE();
    case 5:
      if (lookahead == '%') ADVANCE(150);
      if (lookahead == '/') ADVANCE(10);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(5);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(123);
      END_STATE();
    case 6:
      if (lookahead == '%') ADVANCE(150);
      if (lookahead == '/') ADVANCE(10);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(6);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(49);
      END_STATE();
    case 7:
      if (lookahead == '%') ADVANCE(150);
      if (lookahead == '/') ADVANCE(10);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(7);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(40);
      END_STATE();
    case 8:
      if (lookahead == '%') ADVANCE(150);
      if (lookahead == '/') ADVANCE(10);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(8);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(45);
      END_STATE();
    case 9:
      if (lookahead == '\'') ADVANCE(148);
      if (lookahead != 0) ADVANCE(9);
      END_STATE();
    case 10:
      if (lookahead == '*') ADVANCE(12);
      END_STATE();
    case 11:
      if (lookahead == '*') ADVANCE(13);
      if (lookahead == '/') ADVANCE(152);
      if (lookahead != 0) ADVANCE(12);
      END_STATE();
    case 12:
      if (lookahead == '*') ADVANCE(13);
      if (lookahead != 0) ADVANCE(12);
      END_STATE();
    case 13:
      if (lookahead == '*') ADVANCE(11);
      if (lookahead == '/') ADVANCE(151);
      if (lookahead != 0) ADVANCE(12);
      END_STATE();
    case 14:
      if (lookahead == '+') ADVANCE(58);
      END_STATE();
    case 15:
      if (lookahead == '-') ADVANCE(23);
      END_STATE();
    case 16:
      if (lookahead == '.') ADVANCE(68);
      END_STATE();
    case 17:
      if (lookahead == '.') ADVANCE(18);
      if (lookahead == '>') ADVANCE(82);
      END_STATE();
    case 18:
      if (lookahead == '.') ADVANCE(69);
      END_STATE();
    case 19:
      if (lookahead == '.') ADVANCE(38);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(21);
      END_STATE();
    case 20:
      if (lookahead == '.') ADVANCE(38);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(93);
      END_STATE();
    case 21:
      if (lookahead == '.') ADVANCE(33);
      if (lookahead == 'P' ||
          lookahead == 'p') ADVANCE(32);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(21);
      END_STATE();
    case 22:
      if (lookahead == '0') ADVANCE(89);
      if (lookahead == 'i') ADVANCE(28);
      if (lookahead == 0x221e) ADVANCE(98);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(91);
      END_STATE();
    case 23:
      if (lookahead == '1') ADVANCE(149);
      END_STATE();
    case 24:
      if (lookahead == '>') ADVANCE(82);
      END_STATE();
    case 25:
      if (lookahead == 'f') ADVANCE(27);
      END_STATE();
    case 26:
      if (lookahead == 'i') ADVANCE(30);
      END_STATE();
    case 27:
      if (lookahead == 'i') ADVANCE(29);
      END_STATE();
    case 28:
      if (lookahead == 'n') ADVANCE(25);
      END_STATE();
    case 29:
      if (lookahead == 'n') ADVANCE(26);
      END_STATE();
    case 30:
      if (lookahead == 't') ADVANCE(31);
      END_STATE();
    case 31:
      if (lookahead == 'y') ADVANCE(98);
      END_STATE();
    case 32:
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(37);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(87);
      END_STATE();
    case 33:
      if (lookahead == 'P' ||
          lookahead == 'p') ADVANCE(32);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(33);
      END_STATE();
    case 34:
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(95);
      END_STATE();
    case 35:
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(97);
      END_STATE();
    case 36:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(85);
      END_STATE();
    case 37:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(87);
      END_STATE();
    case 38:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(33);
      END_STATE();
    case 39:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(133);
      END_STATE();
    case 40:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(127);
      END_STATE();
    case 41:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(130);
      END_STATE();
    case 42:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(39);
      END_STATE();
    case 43:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(41);
      END_STATE();
    case 44:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(42);
      END_STATE();
    case 45:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(43);
      END_STATE();
    case 46:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(44);
      END_STATE();
    case 47:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(46);
      END_STATE();
    case 48:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(47);
      END_STATE();
    case 49:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(48);
      END_STATE();
    case 50:
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      END_STATE();
    case 51:
      if (eof) ADVANCE(52);
      ADVANCE_MAP(
        '%', 150,
        '\'', 9,
        '(', 60,
        ')', 62,
        '+', 14,
        ',', 61,
        '.', 16,
        '/', 10,
        ':', 56,
        ';', 53,
        '=', 54,
        ']', 67,
        '|', 66,
        '}', 64,
        0x222a, 74,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(51);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym_EQ);
      if (lookahead == '>') ADVANCE(75);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(anon_sym_COLON);
      if (lookahead == ':') ADVANCE(59);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(anon_sym_PLUS_PLUS);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(anon_sym_COLON_COLON);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      if (lookahead == '|') ADVANCE(83);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(anon_sym_PIPE);
      if (lookahead == ']') ADVANCE(84);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(anon_sym_LT_DOT_DOT);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(anon_sym_DOT);
      if (lookahead == '.') ADVANCE(68);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(aux_sym_tuple_access_token1);
      if (lookahead == '8' ||
          lookahead == '9') ADVANCE(73);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(73);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(aux_sym_tuple_access_token1);
      if (lookahead == '8' ||
          lookahead == '9') ADVANCE(73);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(71);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(aux_sym_tuple_access_token1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(73);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(anon_sym_u222a);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(anon_sym_EQ_GT);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(anon_sym_u00ac);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(anon_sym_BSLASH_LPAREN);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(sym_type_inst_id);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(80);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(sym_type_inst_enum_id);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(sym_absent);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(anon_sym_LBRACK_PIPE);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(anon_sym_PIPE_RBRACK);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(sym_float_literal);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(32);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(85);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(86);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(87);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(36);
      if (lookahead == 'X') ADVANCE(134);
      if (lookahead == 'b') ADVANCE(145);
      if (lookahead == 'o') ADVANCE(146);
      if (lookahead == 'x') ADVANCE(135);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(144);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(90);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(36);
      if (lookahead == 'X') ADVANCE(19);
      if (lookahead == 'b') ADVANCE(34);
      if (lookahead == 'o') ADVANCE(35);
      if (lookahead == 'x') ADVANCE(20);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(32);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(91);
      END_STATE();
    case 90:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(36);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(144);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(90);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 91:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(36);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(32);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(91);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(33);
      if (lookahead == 'P' ||
          lookahead == 'p') ADVANCE(144);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(92);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(33);
      if (lookahead == 'P' ||
          lookahead == 'p') ADVANCE(32);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(93);
      END_STATE();
    case 94:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(94);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 95:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(95);
      END_STATE();
    case 96:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(96);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 97:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(97);
      END_STATE();
    case 98:
      ACCEPT_TOKEN(sym_infinity);
      END_STATE();
    case 99:
      ACCEPT_TOKEN(sym_infinity);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 100:
      ACCEPT_TOKEN(anon_sym_u2205);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 101:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '%') ADVANCE(106);
      if (lookahead == '/') ADVANCE(104);
      if (lookahead == '\t' ||
          (0x0b <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(101);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(106);
      END_STATE();
    case 102:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(105);
      if (lookahead == '/') ADVANCE(103);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(103);
      END_STATE();
    case 103:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(105);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(103);
      END_STATE();
    case 104:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(103);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(106);
      END_STATE();
    case 105:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(102);
      if (lookahead == '/') ADVANCE(106);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(103);
      END_STATE();
    case 106:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(106);
      END_STATE();
    case 107:
      ACCEPT_TOKEN(anon_sym_BSLASH_SQUOTE);
      END_STATE();
    case 108:
      ACCEPT_TOKEN(anon_sym_BSLASH_DQUOTE);
      END_STATE();
    case 109:
      ACCEPT_TOKEN(anon_sym_BSLASH_BSLASH);
      END_STATE();
    case 110:
      ACCEPT_TOKEN(anon_sym_BSLASH_BSLASH);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 111:
      ACCEPT_TOKEN(anon_sym_BSLASHr);
      END_STATE();
    case 112:
      ACCEPT_TOKEN(anon_sym_BSLASHr);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 113:
      ACCEPT_TOKEN(anon_sym_BSLASHn);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(anon_sym_BSLASHn);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 115:
      ACCEPT_TOKEN(anon_sym_BSLASHt);
      END_STATE();
    case 116:
      ACCEPT_TOKEN(anon_sym_BSLASHt);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 117:
      ACCEPT_TOKEN(anon_sym_BSLASH);
      ADVANCE_MAP(
        '"', 108,
        '\'', 107,
        '(', 79,
        'U', 132,
        '\\', 110,
        'n', 114,
        'r', 112,
        't', 116,
        'u', 129,
        'x', 126,
      );
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 118:
      ACCEPT_TOKEN(anon_sym_BSLASH);
      ADVANCE_MAP(
        '"', 108,
        '\'', 107,
        'U', 131,
        '\\', 109,
        'n', 113,
        'r', 111,
        't', 115,
        'u', 128,
        'x', 125,
      );
      END_STATE();
    case 119:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      END_STATE();
    case 120:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(124);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 121:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(119);
      END_STATE();
    case 122:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(120);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 123:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(121);
      END_STATE();
    case 124:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 125:
      ACCEPT_TOKEN(anon_sym_BSLASHx);
      END_STATE();
    case 126:
      ACCEPT_TOKEN(anon_sym_BSLASHx);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 127:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token2);
      END_STATE();
    case 128:
      ACCEPT_TOKEN(anon_sym_BSLASHu);
      END_STATE();
    case 129:
      ACCEPT_TOKEN(anon_sym_BSLASHu);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 130:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token3);
      END_STATE();
    case 131:
      ACCEPT_TOKEN(anon_sym_BSLASHU);
      END_STATE();
    case 132:
      ACCEPT_TOKEN(anon_sym_BSLASHU);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 133:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token4);
      END_STATE();
    case 134:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '.') ADVANCE(38);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(136);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 135:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '.') ADVANCE(38);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(92);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 136:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '.') ADVANCE(33);
      if (lookahead == 'P' ||
          lookahead == 'p') ADVANCE(144);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(136);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 137:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(139);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 138:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(142);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 139:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(141);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 140:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(137);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 141:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(138);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 142:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(143);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 143:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'y') ADVANCE(99);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 144:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(37);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(86);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 145:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(94);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 146:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(96);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 147:
      ACCEPT_TOKEN(sym_identifier);
      if ((!eof && set_contains(sym_identifier_character_set_1, 22, lookahead))) ADVANCE(147);
      END_STATE();
    case 148:
      ACCEPT_TOKEN(sym_quoted_identifier);
      END_STATE();
    case 149:
      ACCEPT_TOKEN(anon_sym_CARET_DASH1);
      END_STATE();
    case 150:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(150);
      END_STATE();
    case 151:
      ACCEPT_TOKEN(sym_block_comment);
      END_STATE();
    case 152:
      ACCEPT_TOKEN(sym_block_comment);
      if (lookahead == '*') ADVANCE(13);
      if (lookahead != 0) ADVANCE(12);
      END_STATE();
    default:
      return false;
  }
}

static bool ts_lex_keywords(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      ADVANCE_MAP(
        '_', 1,
        'a', 2,
        'b', 3,
        'c', 4,
        'e', 5,
        'f', 6,
        'i', 7,
        'l', 8,
        'm', 9,
        'n', 10,
        'o', 11,
        'p', 12,
        'r', 13,
        's', 14,
        't', 15,
        'u', 16,
        'v', 17,
        'w', 18,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      END_STATE();
    case 1:
      ACCEPT_TOKEN(sym_anonymous);
      END_STATE();
    case 2:
      if (lookahead == 'n') ADVANCE(19);
      if (lookahead == 'r') ADVANCE(20);
      END_STATE();
    case 3:
      if (lookahead == 'o') ADVANCE(21);
      END_STATE();
    case 4:
      if (lookahead == 'a') ADVANCE(22);
      if (lookahead == 'o') ADVANCE(23);
      END_STATE();
    case 5:
      if (lookahead == 'l') ADVANCE(24);
      if (lookahead == 'n') ADVANCE(25);
      END_STATE();
    case 6:
      if (lookahead == 'a') ADVANCE(26);
      if (lookahead == 'l') ADVANCE(27);
      if (lookahead == 'u') ADVANCE(28);
      END_STATE();
    case 7:
      if (lookahead == 'f') ADVANCE(29);
      if (lookahead == 'n') ADVANCE(30);
      END_STATE();
    case 8:
      if (lookahead == 'a') ADVANCE(31);
      if (lookahead == 'e') ADVANCE(32);
      END_STATE();
    case 9:
      if (lookahead == 'a') ADVANCE(33);
      if (lookahead == 'i') ADVANCE(34);
      END_STATE();
    case 10:
      if (lookahead == 'o') ADVANCE(35);
      END_STATE();
    case 11:
      if (lookahead == 'f') ADVANCE(36);
      if (lookahead == 'p') ADVANCE(37);
      if (lookahead == 'u') ADVANCE(38);
      END_STATE();
    case 12:
      if (lookahead == 'a') ADVANCE(39);
      if (lookahead == 'r') ADVANCE(40);
      END_STATE();
    case 13:
      if (lookahead == 'e') ADVANCE(41);
      END_STATE();
    case 14:
      if (lookahead == 'a') ADVANCE(42);
      if (lookahead == 'e') ADVANCE(43);
      if (lookahead == 'o') ADVANCE(44);
      if (lookahead == 't') ADVANCE(45);
      END_STATE();
    case 15:
      if (lookahead == 'e') ADVANCE(46);
      if (lookahead == 'h') ADVANCE(47);
      if (lookahead == 'r') ADVANCE(48);
      if (lookahead == 'u') ADVANCE(49);
      if (lookahead == 'y') ADVANCE(50);
      END_STATE();
    case 16:
      if (lookahead == 'n') ADVANCE(51);
      END_STATE();
    case 17:
      if (lookahead == 'a') ADVANCE(52);
      END_STATE();
    case 18:
      if (lookahead == 'h') ADVANCE(53);
      END_STATE();
    case 19:
      if (lookahead == 'n') ADVANCE(54);
      if (lookahead == 'y') ADVANCE(55);
      END_STATE();
    case 20:
      if (lookahead == 'r') ADVANCE(56);
      END_STATE();
    case 21:
      if (lookahead == 'o') ADVANCE(57);
      END_STATE();
    case 22:
      if (lookahead == 's') ADVANCE(58);
      END_STATE();
    case 23:
      if (lookahead == 'n') ADVANCE(59);
      END_STATE();
    case 24:
      if (lookahead == 's') ADVANCE(60);
      END_STATE();
    case 25:
      if (lookahead == 'd') ADVANCE(61);
      if (lookahead == 'u') ADVANCE(62);
      END_STATE();
    case 26:
      if (lookahead == 'l') ADVANCE(63);
      END_STATE();
    case 27:
      if (lookahead == 'o') ADVANCE(64);
      END_STATE();
    case 28:
      if (lookahead == 'n') ADVANCE(65);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(anon_sym_in);
      if (lookahead == 'c') ADVANCE(66);
      if (lookahead == 't') ADVANCE(67);
      END_STATE();
    case 31:
      if (lookahead == 'm') ADVANCE(68);
      END_STATE();
    case 32:
      if (lookahead == 't') ADVANCE(69);
      END_STATE();
    case 33:
      if (lookahead == 'x') ADVANCE(70);
      END_STATE();
    case 34:
      if (lookahead == 'n') ADVANCE(71);
      END_STATE();
    case 35:
      if (lookahead == 't') ADVANCE(72);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_of);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_op);
      if (lookahead == 't') ADVANCE(73);
      END_STATE();
    case 38:
      if (lookahead == 't') ADVANCE(74);
      END_STATE();
    case 39:
      if (lookahead == 'r') ADVANCE(75);
      END_STATE();
    case 40:
      if (lookahead == 'e') ADVANCE(76);
      END_STATE();
    case 41:
      if (lookahead == 'c') ADVANCE(77);
      END_STATE();
    case 42:
      if (lookahead == 't') ADVANCE(78);
      END_STATE();
    case 43:
      if (lookahead == 't') ADVANCE(79);
      END_STATE();
    case 44:
      if (lookahead == 'l') ADVANCE(80);
      END_STATE();
    case 45:
      if (lookahead == 'r') ADVANCE(81);
      END_STATE();
    case 46:
      if (lookahead == 's') ADVANCE(82);
      END_STATE();
    case 47:
      if (lookahead == 'e') ADVANCE(83);
      END_STATE();
    case 48:
      if (lookahead == 'u') ADVANCE(84);
      END_STATE();
    case 49:
      if (lookahead == 'p') ADVANCE(85);
      END_STATE();
    case 50:
      if (lookahead == 'p') ADVANCE(86);
      END_STATE();
    case 51:
      if (lookahead == 'i') ADVANCE(87);
      END_STATE();
    case 52:
      if (lookahead == 'r') ADVANCE(88);
      END_STATE();
    case 53:
      if (lookahead == 'e') ADVANCE(89);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_ann);
      if (lookahead == 'o') ADVANCE(90);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym_any);
      END_STATE();
    case 56:
      if (lookahead == 'a') ADVANCE(91);
      END_STATE();
    case 57:
      if (lookahead == 'l') ADVANCE(92);
      END_STATE();
    case 58:
      if (lookahead == 'e') ADVANCE(93);
      END_STATE();
    case 59:
      if (lookahead == 's') ADVANCE(94);
      END_STATE();
    case 60:
      if (lookahead == 'e') ADVANCE(95);
      END_STATE();
    case 61:
      if (lookahead == 'c') ADVANCE(96);
      if (lookahead == 'i') ADVANCE(97);
      END_STATE();
    case 62:
      if (lookahead == 'm') ADVANCE(98);
      END_STATE();
    case 63:
      if (lookahead == 's') ADVANCE(99);
      END_STATE();
    case 64:
      if (lookahead == 'a') ADVANCE(100);
      END_STATE();
    case 65:
      if (lookahead == 'c') ADVANCE(101);
      END_STATE();
    case 66:
      if (lookahead == 'l') ADVANCE(102);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(anon_sym_int);
      END_STATE();
    case 68:
      if (lookahead == 'b') ADVANCE(103);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(anon_sym_let);
      END_STATE();
    case 70:
      if (lookahead == 'i') ADVANCE(104);
      END_STATE();
    case 71:
      if (lookahead == 'i') ADVANCE(105);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(anon_sym_not);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(anon_sym_opt);
      END_STATE();
    case 74:
      if (lookahead == 'p') ADVANCE(106);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(anon_sym_par);
      END_STATE();
    case 76:
      if (lookahead == 'd') ADVANCE(107);
      END_STATE();
    case 77:
      if (lookahead == 'o') ADVANCE(108);
      END_STATE();
    case 78:
      if (lookahead == 'i') ADVANCE(109);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 80:
      if (lookahead == 'v') ADVANCE(110);
      END_STATE();
    case 81:
      if (lookahead == 'i') ADVANCE(111);
      END_STATE();
    case 82:
      if (lookahead == 't') ADVANCE(112);
      END_STATE();
    case 83:
      if (lookahead == 'n') ADVANCE(113);
      END_STATE();
    case 84:
      if (lookahead == 'e') ADVANCE(114);
      END_STATE();
    case 85:
      if (lookahead == 'l') ADVANCE(115);
      END_STATE();
    case 86:
      if (lookahead == 'e') ADVANCE(116);
      END_STATE();
    case 87:
      if (lookahead == 'o') ADVANCE(117);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(anon_sym_var);
      END_STATE();
    case 89:
      if (lookahead == 'r') ADVANCE(118);
      END_STATE();
    case 90:
      if (lookahead == 't') ADVANCE(119);
      END_STATE();
    case 91:
      if (lookahead == 'y') ADVANCE(120);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(anon_sym_bool);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(anon_sym_case);
      END_STATE();
    case 94:
      if (lookahead == 't') ADVANCE(121);
      END_STATE();
    case 95:
      ACCEPT_TOKEN(anon_sym_else);
      if (lookahead == 'i') ADVANCE(122);
      END_STATE();
    case 96:
      if (lookahead == 'a') ADVANCE(123);
      END_STATE();
    case 97:
      if (lookahead == 'f') ADVANCE(124);
      END_STATE();
    case 98:
      ACCEPT_TOKEN(anon_sym_enum);
      END_STATE();
    case 99:
      if (lookahead == 'e') ADVANCE(125);
      END_STATE();
    case 100:
      if (lookahead == 't') ADVANCE(126);
      END_STATE();
    case 101:
      if (lookahead == 't') ADVANCE(127);
      END_STATE();
    case 102:
      if (lookahead == 'u') ADVANCE(128);
      END_STATE();
    case 103:
      if (lookahead == 'd') ADVANCE(129);
      END_STATE();
    case 104:
      if (lookahead == 'm') ADVANCE(130);
      END_STATE();
    case 105:
      if (lookahead == 'm') ADVANCE(131);
      END_STATE();
    case 106:
      if (lookahead == 'u') ADVANCE(132);
      END_STATE();
    case 107:
      if (lookahead == 'i') ADVANCE(133);
      END_STATE();
    case 108:
      if (lookahead == 'r') ADVANCE(134);
      END_STATE();
    case 109:
      if (lookahead == 's') ADVANCE(135);
      END_STATE();
    case 110:
      if (lookahead == 'e') ADVANCE(136);
      END_STATE();
    case 111:
      if (lookahead == 'n') ADVANCE(137);
      END_STATE();
    case 112:
      ACCEPT_TOKEN(anon_sym_test);
      END_STATE();
    case 113:
      ACCEPT_TOKEN(anon_sym_then);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(anon_sym_true);
      END_STATE();
    case 115:
      if (lookahead == 'e') ADVANCE(138);
      END_STATE();
    case 116:
      ACCEPT_TOKEN(anon_sym_type);
      END_STATE();
    case 117:
      if (lookahead == 'n') ADVANCE(139);
      END_STATE();
    case 118:
      if (lookahead == 'e') ADVANCE(140);
      END_STATE();
    case 119:
      if (lookahead == 'a') ADVANCE(141);
      END_STATE();
    case 120:
      ACCEPT_TOKEN(anon_sym_array);
      END_STATE();
    case 121:
      if (lookahead == 'r') ADVANCE(142);
      END_STATE();
    case 122:
      if (lookahead == 'f') ADVANCE(143);
      END_STATE();
    case 123:
      if (lookahead == 's') ADVANCE(144);
      END_STATE();
    case 124:
      ACCEPT_TOKEN(anon_sym_endif);
      END_STATE();
    case 125:
      ACCEPT_TOKEN(anon_sym_false);
      END_STATE();
    case 126:
      ACCEPT_TOKEN(anon_sym_float);
      END_STATE();
    case 127:
      if (lookahead == 'i') ADVANCE(145);
      END_STATE();
    case 128:
      if (lookahead == 'd') ADVANCE(146);
      END_STATE();
    case 129:
      if (lookahead == 'a') ADVANCE(147);
      END_STATE();
    case 130:
      if (lookahead == 'i') ADVANCE(148);
      END_STATE();
    case 131:
      if (lookahead == 'i') ADVANCE(149);
      END_STATE();
    case 132:
      if (lookahead == 't') ADVANCE(150);
      END_STATE();
    case 133:
      if (lookahead == 'c') ADVANCE(151);
      END_STATE();
    case 134:
      if (lookahead == 'd') ADVANCE(152);
      END_STATE();
    case 135:
      if (lookahead == 'f') ADVANCE(153);
      END_STATE();
    case 136:
      ACCEPT_TOKEN(anon_sym_solve);
      END_STATE();
    case 137:
      if (lookahead == 'g') ADVANCE(154);
      END_STATE();
    case 138:
      ACCEPT_TOKEN(anon_sym_tuple);
      END_STATE();
    case 139:
      ACCEPT_TOKEN(anon_sym_union);
      END_STATE();
    case 140:
      ACCEPT_TOKEN(anon_sym_where);
      END_STATE();
    case 141:
      if (lookahead == 't') ADVANCE(155);
      END_STATE();
    case 142:
      if (lookahead == 'a') ADVANCE(156);
      END_STATE();
    case 143:
      ACCEPT_TOKEN(anon_sym_elseif);
      END_STATE();
    case 144:
      if (lookahead == 'e') ADVANCE(157);
      END_STATE();
    case 145:
      if (lookahead == 'o') ADVANCE(158);
      END_STATE();
    case 146:
      if (lookahead == 'e') ADVANCE(159);
      END_STATE();
    case 147:
      ACCEPT_TOKEN(anon_sym_lambda);
      END_STATE();
    case 148:
      if (lookahead == 'z') ADVANCE(160);
      END_STATE();
    case 149:
      if (lookahead == 'z') ADVANCE(161);
      END_STATE();
    case 150:
      ACCEPT_TOKEN(anon_sym_output);
      END_STATE();
    case 151:
      if (lookahead == 'a') ADVANCE(162);
      END_STATE();
    case 152:
      ACCEPT_TOKEN(anon_sym_record);
      END_STATE();
    case 153:
      if (lookahead == 'y') ADVANCE(163);
      END_STATE();
    case 154:
      ACCEPT_TOKEN(anon_sym_string);
      END_STATE();
    case 155:
      if (lookahead == 'i') ADVANCE(164);
      END_STATE();
    case 156:
      if (lookahead == 'i') ADVANCE(165);
      END_STATE();
    case 157:
      ACCEPT_TOKEN(anon_sym_endcase);
      END_STATE();
    case 158:
      if (lookahead == 'n') ADVANCE(166);
      END_STATE();
    case 159:
      ACCEPT_TOKEN(anon_sym_include);
      END_STATE();
    case 160:
      if (lookahead == 'e') ADVANCE(167);
      END_STATE();
    case 161:
      if (lookahead == 'e') ADVANCE(168);
      END_STATE();
    case 162:
      if (lookahead == 't') ADVANCE(169);
      END_STATE();
    case 163:
      ACCEPT_TOKEN(anon_sym_satisfy);
      END_STATE();
    case 164:
      if (lookahead == 'o') ADVANCE(170);
      END_STATE();
    case 165:
      if (lookahead == 'n') ADVANCE(171);
      END_STATE();
    case 166:
      ACCEPT_TOKEN(anon_sym_function);
      END_STATE();
    case 167:
      ACCEPT_TOKEN(anon_sym_maximize);
      END_STATE();
    case 168:
      ACCEPT_TOKEN(anon_sym_minimize);
      END_STATE();
    case 169:
      if (lookahead == 'e') ADVANCE(172);
      END_STATE();
    case 170:
      if (lookahead == 'n') ADVANCE(173);
      END_STATE();
    case 171:
      if (lookahead == 't') ADVANCE(174);
      END_STATE();
    case 172:
      ACCEPT_TOKEN(anon_sym_predicate);
      END_STATE();
    case 173:
      ACCEPT_TOKEN(anon_sym_annotation);
      END_STATE();
    case 174:
      ACCEPT_TOKEN(anon_sym_constraint);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 51},
  [2] = {.lex_state = 2},
  [3] = {.lex_state = 2},
  [4] = {.lex_state = 2},
  [5] = {.lex_state = 2},
  [6] = {.lex_state = 2},
  [7] = {.lex_state = 2},
  [8] = {.lex_state = 2},
  [9] = {.lex_state = 2},
  [10] = {.lex_state = 2},
  [11] = {.lex_state = 2},
  [12] = {.lex_state = 2},
  [13] = {.lex_state = 2},
  [14] = {.lex_state = 2},
  [15] = {.lex_state = 2},
  [16] = {.lex_state = 2},
  [17] = {.lex_state = 2},
  [18] = {.lex_state = 2},
  [19] = {.lex_state = 2},
  [20] = {.lex_state = 2},
  [21] = {.lex_state = 2},
  [22] = {.lex_state = 2},
  [23] = {.lex_state = 2},
  [24] = {.lex_state = 2},
  [25] = {.lex_state = 2},
  [26] = {.lex_state = 2},
  [27] = {.lex_state = 2},
  [28] = {.lex_state = 2},
  [29] = {.lex_state = 2},
  [30] = {.lex_state = 2},
  [31] = {.lex_state = 2},
  [32] = {.lex_state = 2},
  [33] = {.lex_state = 2},
  [34] = {.lex_state = 2},
  [35] = {.lex_state = 2},
  [36] = {.lex_state = 2},
  [37] = {.lex_state = 2},
  [38] = {.lex_state = 2},
  [39] = {.lex_state = 1},
  [40] = {.lex_state = 1},
  [41] = {.lex_state = 1},
  [42] = {.lex_state = 51},
  [43] = {.lex_state = 51},
  [44] = {.lex_state = 51},
  [45] = {.lex_state = 51},
  [46] = {.lex_state = 51},
  [47] = {.lex_state = 51},
  [48] = {.lex_state = 51},
  [49] = {.lex_state = 51},
  [50] = {.lex_state = 51},
  [51] = {.lex_state = 51},
  [52] = {.lex_state = 51},
  [53] = {.lex_state = 51},
  [54] = {.lex_state = 51},
  [55] = {.lex_state = 51},
  [56] = {.lex_state = 51},
  [57] = {.lex_state = 51},
  [58] = {.lex_state = 51},
  [59] = {.lex_state = 51},
  [60] = {.lex_state = 51},
  [61] = {.lex_state = 51},
  [62] = {.lex_state = 51},
  [63] = {.lex_state = 51},
  [64] = {.lex_state = 51},
  [65] = {.lex_state = 51},
  [66] = {.lex_state = 51},
  [67] = {.lex_state = 51},
  [68] = {.lex_state = 51},
  [69] = {.lex_state = 51},
  [70] = {.lex_state = 51},
  [71] = {.lex_state = 51},
  [72] = {.lex_state = 51},
  [73] = {.lex_state = 51},
  [74] = {.lex_state = 51},
  [75] = {.lex_state = 51},
  [76] = {.lex_state = 51},
  [77] = {.lex_state = 51},
  [78] = {.lex_state = 51},
  [79] = {.lex_state = 51},
  [80] = {.lex_state = 1},
  [81] = {.lex_state = 1},
  [82] = {.lex_state = 1},
  [83] = {.lex_state = 51},
  [84] = {.lex_state = 51},
  [85] = {.lex_state = 51},
  [86] = {.lex_state = 51},
  [87] = {.lex_state = 51},
  [88] = {.lex_state = 51},
  [89] = {.lex_state = 51},
  [90] = {.lex_state = 51},
  [91] = {.lex_state = 51},
  [92] = {.lex_state = 51},
  [93] = {.lex_state = 51},
  [94] = {.lex_state = 51},
  [95] = {.lex_state = 51},
  [96] = {.lex_state = 51},
  [97] = {.lex_state = 51},
  [98] = {.lex_state = 51},
  [99] = {.lex_state = 51},
  [100] = {.lex_state = 51},
  [101] = {.lex_state = 51},
  [102] = {.lex_state = 51},
  [103] = {.lex_state = 51},
  [104] = {.lex_state = 51},
  [105] = {.lex_state = 0},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 0},
  [108] = {.lex_state = 51},
  [109] = {.lex_state = 51},
  [110] = {.lex_state = 0},
  [111] = {.lex_state = 0},
  [112] = {.lex_state = 0},
  [113] = {.lex_state = 51},
  [114] = {.lex_state = 0},
  [115] = {.lex_state = 0},
  [116] = {.lex_state = 0},
  [117] = {.lex_state = 0},
  [118] = {.lex_state = 0},
  [119] = {.lex_state = 0},
  [120] = {.lex_state = 0},
  [121] = {.lex_state = 0},
  [122] = {.lex_state = 0},
  [123] = {.lex_state = 0},
  [124] = {.lex_state = 0},
  [125] = {.lex_state = 0},
  [126] = {.lex_state = 5},
  [127] = {.lex_state = 0},
  [128] = {.lex_state = 6},
  [129] = {.lex_state = 0},
  [130] = {.lex_state = 0},
  [131] = {.lex_state = 7},
  [132] = {.lex_state = 51},
  [133] = {.lex_state = 8},
  [134] = {.lex_state = 0},
  [135] = {.lex_state = 51},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_identifier] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_annotation] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_constraint] = ACTIONS(1),
    [anon_sym_COLON] = ACTIONS(1),
    [anon_sym_enum] = ACTIONS(1),
    [anon_sym_PLUS_PLUS] = ACTIONS(1),
    [anon_sym_function] = ACTIONS(1),
    [anon_sym_solve] = ACTIONS(1),
    [anon_sym_satisfy] = ACTIONS(1),
    [anon_sym_maximize] = ACTIONS(1),
    [anon_sym_minimize] = ACTIONS(1),
    [anon_sym_include] = ACTIONS(1),
    [anon_sym_output] = ACTIONS(1),
    [anon_sym_COLON_COLON] = ACTIONS(1),
    [anon_sym_predicate] = ACTIONS(1),
    [anon_sym_test] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_type] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_PIPE] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_in] = ACTIONS(1),
    [anon_sym_where] = ACTIONS(1),
    [anon_sym_if] = ACTIONS(1),
    [anon_sym_then] = ACTIONS(1),
    [anon_sym_elseif] = ACTIONS(1),
    [anon_sym_else] = ACTIONS(1),
    [anon_sym_endif] = ACTIONS(1),
    [anon_sym_DOT_DOT] = ACTIONS(1),
    [anon_sym_LT_DOT_DOT] = ACTIONS(1),
    [anon_sym_DOT] = ACTIONS(1),
    [aux_sym_tuple_access_token1] = ACTIONS(1),
    [anon_sym_union] = ACTIONS(1),
    [anon_sym_u222a] = ACTIONS(1),
    [anon_sym_case] = ACTIONS(1),
    [anon_sym_of] = ACTIONS(1),
    [anon_sym_endcase] = ACTIONS(1),
    [anon_sym_EQ_GT] = ACTIONS(1),
    [anon_sym_lambda] = ACTIONS(1),
    [anon_sym_let] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_not] = ACTIONS(1),
    [anon_sym_u00ac] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [anon_sym_BSLASH_LPAREN] = ACTIONS(1),
    [anon_sym_array] = ACTIONS(1),
    [anon_sym_var] = ACTIONS(1),
    [anon_sym_par] = ACTIONS(1),
    [anon_sym_opt] = ACTIONS(1),
    [anon_sym_set] = ACTIONS(1),
    [anon_sym_tuple] = ACTIONS(1),
    [anon_sym_record] = ACTIONS(1),
    [anon_sym_op] = ACTIONS(1),
    [anon_sym_any] = ACTIONS(1),
    [anon_sym_ann] = ACTIONS(1),
    [anon_sym_bool] = ACTIONS(1),
    [anon_sym_float] = ACTIONS(1),
    [anon_sym_int] = ACTIONS(1),
    [anon_sym_string] = ACTIONS(1),
    [sym_type_inst_id] = ACTIONS(1),
    [sym_type_inst_enum_id] = ACTIONS(1),
    [sym_absent] = ACTIONS(1),
    [sym_anonymous] = ACTIONS(1),
    [anon_sym_LBRACK_PIPE] = ACTIONS(1),
    [anon_sym_PIPE_RBRACK] = ACTIONS(1),
    [anon_sym_true] = ACTIONS(1),
    [anon_sym_false] = ACTIONS(1),
    [anon_sym_u2205] = ACTIONS(1),
    [anon_sym_BSLASH_SQUOTE] = ACTIONS(1),
    [anon_sym_BSLASH_DQUOTE] = ACTIONS(1),
    [anon_sym_BSLASH_BSLASH] = ACTIONS(1),
    [anon_sym_BSLASHr] = ACTIONS(1),
    [anon_sym_BSLASHn] = ACTIONS(1),
    [anon_sym_BSLASHt] = ACTIONS(1),
    [anon_sym_BSLASH] = ACTIONS(1),
    [aux_sym_escape_sequence_token1] = ACTIONS(1),
    [anon_sym_BSLASHx] = ACTIONS(1),
    [anon_sym_BSLASHu] = ACTIONS(1),
    [anon_sym_BSLASHU] = ACTIONS(1),
    [sym_quoted_identifier] = ACTIONS(1),
    [anon_sym_CARET_DASH1] = ACTIONS(1),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [1] = {
    [sym_source_file] = STATE(129),
    [sym_assignment] = STATE(125),
    [sym__identifier] = STATE(135),
    [aux_sym_source_file_repeat1] = STATE(91),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_identifier] = ACTIONS(7),
    [sym_quoted_identifier] = ACTIONS(7),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 19,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_PIPE,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(25), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(6), 1,
      aux_sym_array_literal_2d_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(104), 1,
      sym__expression,
    STATE(111), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [70] = 18,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(35), 1,
      anon_sym_RBRACK,
    STATE(7), 1,
      aux_sym_array_literal_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(89), 1,
      sym__expression,
    STATE(118), 1,
      sym_array_literal_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [137] = 20,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(37), 1,
      sym_identifier,
    ACTIONS(39), 1,
      anon_sym_RPAREN,
    ACTIONS(41), 1,
      sym_integer_literal,
    ACTIONS(43), 1,
      sym_quoted_identifier,
    STATE(5), 1,
      aux_sym_call_repeat1,
    STATE(88), 1,
      sym__identifier,
    STATE(109), 1,
      sym__expression,
    STATE(120), 1,
      sym__call_arg,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 2,
      sym_float_literal,
      sym_infinity,
    STATE(92), 3,
      sym_call,
      sym_infix_operator,
      sym_set_literal,
    STATE(71), 6,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [208] = 20,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(37), 1,
      sym_identifier,
    ACTIONS(41), 1,
      sym_integer_literal,
    ACTIONS(43), 1,
      sym_quoted_identifier,
    ACTIONS(45), 1,
      anon_sym_RPAREN,
    STATE(9), 1,
      aux_sym_call_repeat1,
    STATE(88), 1,
      sym__identifier,
    STATE(109), 1,
      sym__expression,
    STATE(119), 1,
      sym__call_arg,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 2,
      sym_float_literal,
      sym_infinity,
    STATE(92), 3,
      sym_call,
      sym_infix_operator,
      sym_set_literal,
    STATE(71), 6,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [279] = 18,
    ACTIONS(47), 1,
      sym_identifier,
    ACTIONS(50), 1,
      anon_sym_LPAREN,
    ACTIONS(53), 1,
      anon_sym_LBRACE,
    ACTIONS(56), 1,
      anon_sym_LBRACK,
    ACTIONS(59), 1,
      anon_sym_PIPE,
    ACTIONS(61), 1,
      anon_sym_DQUOTE,
    ACTIONS(64), 1,
      sym_absent,
    ACTIONS(67), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(70), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(78), 1,
      anon_sym_u2205,
    ACTIONS(81), 1,
      sym_quoted_identifier,
    STATE(6), 1,
      aux_sym_array_literal_2d_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(104), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(72), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(75), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [346] = 18,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(84), 1,
      anon_sym_RBRACK,
    STATE(10), 1,
      aux_sym_array_literal_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(89), 1,
      sym__expression,
    STATE(122), 1,
      sym_array_literal_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [413] = 17,
    ACTIONS(86), 1,
      sym_identifier,
    ACTIONS(89), 1,
      anon_sym_LPAREN,
    ACTIONS(94), 1,
      anon_sym_LBRACE,
    ACTIONS(97), 1,
      anon_sym_LBRACK,
    ACTIONS(100), 1,
      anon_sym_DQUOTE,
    ACTIONS(103), 1,
      sym_absent,
    ACTIONS(106), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(115), 1,
      anon_sym_u2205,
    ACTIONS(118), 1,
      sym_quoted_identifier,
    STATE(8), 1,
      aux_sym_set_literal_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(100), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(92), 2,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
    ACTIONS(109), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(112), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [478] = 20,
    ACTIONS(121), 1,
      sym_identifier,
    ACTIONS(124), 1,
      anon_sym_LPAREN,
    ACTIONS(127), 1,
      anon_sym_RPAREN,
    ACTIONS(129), 1,
      anon_sym_LBRACE,
    ACTIONS(132), 1,
      anon_sym_LBRACK,
    ACTIONS(135), 1,
      anon_sym_DQUOTE,
    ACTIONS(138), 1,
      sym_absent,
    ACTIONS(141), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(150), 1,
      sym_integer_literal,
    ACTIONS(153), 1,
      anon_sym_u2205,
    ACTIONS(156), 1,
      sym_quoted_identifier,
    STATE(9), 1,
      aux_sym_call_repeat1,
    STATE(88), 1,
      sym__identifier,
    STATE(109), 1,
      sym__expression,
    STATE(134), 1,
      sym__call_arg,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(144), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(147), 2,
      sym_float_literal,
      sym_infinity,
    STATE(92), 3,
      sym_call,
      sym_infix_operator,
      sym_set_literal,
    STATE(71), 6,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [549] = 18,
    ACTIONS(159), 1,
      sym_identifier,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(165), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_LBRACK,
    ACTIONS(171), 1,
      anon_sym_RBRACK,
    ACTIONS(173), 1,
      anon_sym_DQUOTE,
    ACTIONS(176), 1,
      sym_absent,
    ACTIONS(179), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(188), 1,
      anon_sym_u2205,
    ACTIONS(191), 1,
      sym_quoted_identifier,
    STATE(10), 1,
      aux_sym_array_literal_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(89), 1,
      sym__expression,
    STATE(127), 1,
      sym_array_literal_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(182), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(185), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [616] = 18,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(194), 1,
      anon_sym_PIPE_RBRACK,
    STATE(2), 1,
      aux_sym_array_literal_2d_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(84), 1,
      sym__expression,
    STATE(112), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [683] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(196), 1,
      anon_sym_PIPE,
    ACTIONS(198), 1,
      anon_sym_PIPE_RBRACK,
    STATE(42), 1,
      sym__identifier,
    STATE(86), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [747] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(200), 1,
      anon_sym_RBRACE,
    STATE(24), 1,
      aux_sym_set_literal_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(96), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [811] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(202), 1,
      anon_sym_RPAREN,
    STATE(8), 1,
      aux_sym_set_literal_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(95), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [875] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(204), 1,
      anon_sym_PIPE_RBRACK,
    STATE(42), 1,
      sym__identifier,
    STATE(83), 1,
      sym__expression,
    STATE(121), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [939] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(206), 1,
      anon_sym_PIPE,
    ACTIONS(208), 1,
      anon_sym_PIPE_RBRACK,
    STATE(42), 1,
      sym__identifier,
    STATE(86), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1003] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(210), 1,
      anon_sym_PIPE,
    ACTIONS(212), 1,
      anon_sym_PIPE_RBRACK,
    STATE(42), 1,
      sym__identifier,
    STATE(86), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1067] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(214), 1,
      anon_sym_PIPE_RBRACK,
    STATE(42), 1,
      sym__identifier,
    STATE(83), 1,
      sym__expression,
    STATE(121), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1131] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(216), 1,
      anon_sym_RPAREN,
    STATE(14), 1,
      aux_sym_set_literal_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(90), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1195] = 17,
    ACTIONS(218), 1,
      sym_identifier,
    ACTIONS(221), 1,
      anon_sym_LPAREN,
    ACTIONS(224), 1,
      anon_sym_LBRACE,
    ACTIONS(227), 1,
      anon_sym_LBRACK,
    ACTIONS(230), 1,
      anon_sym_PIPE,
    ACTIONS(232), 1,
      anon_sym_DQUOTE,
    ACTIONS(235), 1,
      sym_absent,
    ACTIONS(238), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(241), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(249), 1,
      anon_sym_u2205,
    ACTIONS(252), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(85), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(243), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(246), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1259] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(255), 1,
      anon_sym_PIPE_RBRACK,
    STATE(42), 1,
      sym__identifier,
    STATE(83), 1,
      sym__expression,
    STATE(121), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1323] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(257), 1,
      anon_sym_PIPE,
    ACTIONS(259), 1,
      anon_sym_PIPE_RBRACK,
    STATE(42), 1,
      sym__identifier,
    STATE(86), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1387] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(261), 1,
      anon_sym_PIPE_RBRACK,
    STATE(42), 1,
      sym__identifier,
    STATE(83), 1,
      sym__expression,
    STATE(121), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1451] = 17,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    ACTIONS(263), 1,
      anon_sym_RBRACE,
    STATE(8), 1,
      aux_sym_set_literal_repeat1,
    STATE(42), 1,
      sym__identifier,
    STATE(99), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1515] = 16,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(83), 1,
      sym__expression,
    STATE(121), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1576] = 16,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(265), 1,
      sym_identifier,
    ACTIONS(267), 1,
      sym_quoted_identifier,
    STATE(87), 1,
      sym__identifier,
    STATE(103), 1,
      sym__expression,
    STATE(115), 1,
      sym_record_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1637] = 15,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(85), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1695] = 15,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(98), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1753] = 15,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(86), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1811] = 15,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(94), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1869] = 15,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(97), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1927] = 15,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(76), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1985] = 15,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(74), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [2043] = 15,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_DQUOTE,
    ACTIONS(21), 1,
      sym_absent,
    ACTIONS(23), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      anon_sym_u2205,
    ACTIONS(33), 1,
      sym_quoted_identifier,
    STATE(42), 1,
      sym__identifier,
    STATE(75), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(29), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
    STATE(71), 9,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [2101] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(269), 8,
      anon_sym_LBRACK,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
      anon_sym_u2205,
      sym_identifier,
    ACTIONS(271), 8,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      sym_quoted_identifier,
  [2126] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(241), 7,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      anon_sym_PIPE_RBRACK,
      sym_quoted_identifier,
    ACTIONS(230), 9,
      anon_sym_LBRACK,
      anon_sym_PIPE,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
      anon_sym_u2205,
      sym_identifier,
  [2151] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(275), 7,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_LBRACE,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      sym_quoted_identifier,
    ACTIONS(273), 8,
      anon_sym_LBRACK,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
      anon_sym_u2205,
      sym_identifier,
  [2175] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(279), 7,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACK,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      sym_quoted_identifier,
    ACTIONS(277), 8,
      anon_sym_LBRACK,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      sym_infinity,
      anon_sym_u2205,
      sym_identifier,
  [2199] = 10,
    ACTIONS(281), 1,
      anon_sym_DQUOTE,
    ACTIONS(283), 1,
      sym_string_characters,
    ACTIONS(287), 1,
      anon_sym_BSLASH,
    ACTIONS(289), 1,
      anon_sym_BSLASHx,
    ACTIONS(291), 1,
      anon_sym_BSLASHu,
    ACTIONS(293), 1,
      anon_sym_BSLASHU,
    STATE(41), 1,
      aux_sym__string_content,
    STATE(81), 1,
      sym_escape_sequence,
    ACTIONS(295), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(285), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [2236] = 10,
    ACTIONS(283), 1,
      sym_string_characters,
    ACTIONS(287), 1,
      anon_sym_BSLASH,
    ACTIONS(289), 1,
      anon_sym_BSLASHx,
    ACTIONS(291), 1,
      anon_sym_BSLASHu,
    ACTIONS(293), 1,
      anon_sym_BSLASHU,
    ACTIONS(297), 1,
      anon_sym_DQUOTE,
    STATE(39), 1,
      aux_sym__string_content,
    STATE(81), 1,
      sym_escape_sequence,
    ACTIONS(295), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(285), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [2273] = 10,
    ACTIONS(299), 1,
      anon_sym_DQUOTE,
    ACTIONS(301), 1,
      sym_string_characters,
    ACTIONS(307), 1,
      anon_sym_BSLASH,
    ACTIONS(310), 1,
      anon_sym_BSLASHx,
    ACTIONS(313), 1,
      anon_sym_BSLASHu,
    ACTIONS(316), 1,
      anon_sym_BSLASHU,
    STATE(41), 1,
      aux_sym__string_content,
    STATE(81), 1,
      sym_escape_sequence,
    ACTIONS(295), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(304), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [2310] = 4,
    ACTIONS(321), 1,
      anon_sym_LPAREN,
    ACTIONS(323), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(319), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2335] = 3,
    ACTIONS(327), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(325), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2357] = 3,
    ACTIONS(331), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(329), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2379] = 3,
    ACTIONS(335), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(333), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2401] = 3,
    ACTIONS(339), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(337), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2423] = 3,
    ACTIONS(343), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(341), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2445] = 3,
    ACTIONS(347), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(345), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2467] = 3,
    ACTIONS(351), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(349), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2489] = 3,
    ACTIONS(355), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(353), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2511] = 3,
    ACTIONS(359), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(357), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2533] = 3,
    ACTIONS(363), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(361), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2555] = 3,
    ACTIONS(367), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(365), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2577] = 3,
    ACTIONS(371), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(369), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2599] = 3,
    ACTIONS(375), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(373), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2621] = 3,
    ACTIONS(379), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(377), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2643] = 3,
    ACTIONS(383), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(381), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2665] = 3,
    ACTIONS(387), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(385), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2687] = 3,
    ACTIONS(391), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(389), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2709] = 3,
    ACTIONS(395), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(393), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2731] = 3,
    ACTIONS(399), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(397), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2753] = 3,
    ACTIONS(403), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(401), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2775] = 3,
    ACTIONS(407), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(405), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2797] = 3,
    ACTIONS(411), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(409), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2819] = 3,
    ACTIONS(415), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(413), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2841] = 3,
    ACTIONS(419), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(417), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2863] = 3,
    ACTIONS(423), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(421), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2885] = 3,
    ACTIONS(427), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(425), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2907] = 3,
    ACTIONS(431), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(429), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2929] = 3,
    ACTIONS(435), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(433), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2951] = 3,
    ACTIONS(323), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(319), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2973] = 3,
    ACTIONS(439), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(437), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [2995] = 3,
    ACTIONS(443), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(441), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [3017] = 3,
    ACTIONS(447), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(445), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [3039] = 4,
    ACTIONS(447), 1,
      anon_sym_PIPE,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(445), 11,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [3063] = 5,
    ACTIONS(447), 1,
      anon_sym_PIPE,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(445), 10,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [3089] = 3,
    ACTIONS(455), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(453), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [3111] = 3,
    ACTIONS(459), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(457), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [3133] = 3,
    ACTIONS(463), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(461), 12,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
      anon_sym_PIPE_RBRACK,
  [3155] = 3,
    ACTIONS(467), 1,
      sym_string_characters,
    ACTIONS(295), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(465), 11,
      anon_sym_DQUOTE,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
      anon_sym_BSLASH,
      anon_sym_BSLASHx,
      anon_sym_BSLASHu,
      anon_sym_BSLASHU,
  [3176] = 3,
    ACTIONS(471), 1,
      sym_string_characters,
    ACTIONS(295), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(469), 11,
      anon_sym_DQUOTE,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
      anon_sym_BSLASH,
      anon_sym_BSLASHx,
      anon_sym_BSLASHu,
      anon_sym_BSLASHU,
  [3197] = 3,
    ACTIONS(475), 1,
      sym_string_characters,
    ACTIONS(295), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(473), 11,
      anon_sym_DQUOTE,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
      anon_sym_BSLASH,
      anon_sym_BSLASHx,
      anon_sym_BSLASHu,
      anon_sym_BSLASHU,
  [3218] = 9,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(477), 1,
      anon_sym_COLON,
    ACTIONS(479), 1,
      anon_sym_COMMA,
    ACTIONS(481), 1,
      anon_sym_PIPE,
    ACTIONS(485), 1,
      anon_sym_PIPE_RBRACK,
    STATE(107), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3248] = 9,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(479), 1,
      anon_sym_COMMA,
    ACTIONS(481), 1,
      anon_sym_PIPE,
    ACTIONS(485), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(487), 1,
      anon_sym_COLON,
    STATE(107), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3278] = 8,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(489), 1,
      anon_sym_COMMA,
    ACTIONS(491), 1,
      anon_sym_PIPE,
    ACTIONS(493), 1,
      anon_sym_PIPE_RBRACK,
    STATE(105), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3305] = 6,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(497), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
    ACTIONS(495), 2,
      anon_sym_COMMA,
      anon_sym_PIPE_RBRACK,
  [3327] = 4,
    ACTIONS(321), 1,
      anon_sym_LPAREN,
    ACTIONS(499), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(319), 5,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
  [3345] = 4,
    ACTIONS(321), 1,
      anon_sym_LPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(501), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
    ACTIONS(319), 4,
      anon_sym_PLUS_PLUS,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
  [3363] = 6,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(503), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
    ACTIONS(505), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3385] = 6,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(507), 1,
      anon_sym_COMMA,
    ACTIONS(509), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3406] = 6,
    ACTIONS(511), 1,
      ts_builtin_sym_end,
    STATE(93), 1,
      aux_sym_source_file_repeat1,
    STATE(123), 1,
      sym_assignment,
    STATE(135), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(7), 2,
      sym_identifier,
      sym_quoted_identifier,
  [3427] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(501), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
    ACTIONS(319), 4,
      anon_sym_PLUS_PLUS,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_u222a,
  [3442] = 6,
    ACTIONS(513), 1,
      ts_builtin_sym_end,
    STATE(93), 1,
      aux_sym_source_file_repeat1,
    STATE(130), 1,
      sym_assignment,
    STATE(135), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(515), 2,
      sym_identifier,
      sym_quoted_identifier,
  [3463] = 5,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
    ACTIONS(518), 2,
      ts_builtin_sym_end,
      anon_sym_SEMI,
  [3482] = 6,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(507), 1,
      anon_sym_COMMA,
    ACTIONS(520), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3503] = 6,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(507), 1,
      anon_sym_COMMA,
    ACTIONS(522), 1,
      anon_sym_RBRACE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3524] = 5,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
    ACTIONS(524), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [3543] = 5,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
    ACTIONS(526), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3562] = 6,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(507), 1,
      anon_sym_COMMA,
    ACTIONS(528), 1,
      anon_sym_RBRACE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3583] = 5,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(507), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3601] = 5,
    ACTIONS(532), 1,
      anon_sym_RPAREN,
    STATE(124), 1,
      sym_record_member,
    STATE(132), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(530), 2,
      sym_identifier,
      sym_quoted_identifier,
  [3619] = 5,
    ACTIONS(534), 1,
      anon_sym_RPAREN,
    STATE(124), 1,
      sym_record_member,
    STATE(132), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(530), 2,
      sym_identifier,
      sym_quoted_identifier,
  [3637] = 5,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(536), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3655] = 5,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(538), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3673] = 5,
    ACTIONS(540), 1,
      anon_sym_COMMA,
    ACTIONS(542), 1,
      anon_sym_PIPE,
    ACTIONS(544), 1,
      anon_sym_PIPE_RBRACK,
    STATE(106), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3690] = 5,
    ACTIONS(546), 1,
      anon_sym_COMMA,
    ACTIONS(549), 1,
      anon_sym_PIPE,
    ACTIONS(551), 1,
      anon_sym_PIPE_RBRACK,
    STATE(106), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3707] = 5,
    ACTIONS(553), 1,
      anon_sym_COMMA,
    ACTIONS(555), 1,
      anon_sym_PIPE,
    ACTIONS(557), 1,
      anon_sym_PIPE_RBRACK,
    STATE(106), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3724] = 4,
    STATE(124), 1,
      sym_record_member,
    STATE(132), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(530), 2,
      sym_identifier,
      sym_quoted_identifier,
  [3739] = 4,
    ACTIONS(449), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(451), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(483), 2,
      anon_sym_union,
      anon_sym_u222a,
  [3754] = 4,
    ACTIONS(559), 1,
      anon_sym_COMMA,
    ACTIONS(561), 1,
      anon_sym_RPAREN,
    STATE(116), 1,
      aux_sym_record_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3768] = 4,
    ACTIONS(563), 1,
      anon_sym_PIPE,
    ACTIONS(565), 1,
      anon_sym_PIPE_RBRACK,
    STATE(117), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3782] = 4,
    ACTIONS(567), 1,
      anon_sym_PIPE,
    ACTIONS(569), 1,
      anon_sym_PIPE_RBRACK,
    STATE(114), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3796] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(571), 3,
      ts_builtin_sym_end,
      sym_identifier,
      sym_quoted_identifier,
  [3806] = 4,
    ACTIONS(573), 1,
      anon_sym_PIPE,
    ACTIONS(575), 1,
      anon_sym_PIPE_RBRACK,
    STATE(117), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3820] = 4,
    ACTIONS(577), 1,
      anon_sym_COMMA,
    ACTIONS(579), 1,
      anon_sym_RPAREN,
    STATE(110), 1,
      aux_sym_record_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3834] = 4,
    ACTIONS(581), 1,
      anon_sym_COMMA,
    ACTIONS(584), 1,
      anon_sym_RPAREN,
    STATE(116), 1,
      aux_sym_record_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3848] = 4,
    ACTIONS(586), 1,
      anon_sym_PIPE,
    ACTIONS(589), 1,
      anon_sym_PIPE_RBRACK,
    STATE(117), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3862] = 3,
    ACTIONS(591), 1,
      anon_sym_COMMA,
    ACTIONS(593), 1,
      anon_sym_RBRACK,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3873] = 3,
    ACTIONS(595), 1,
      anon_sym_COMMA,
    ACTIONS(597), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3884] = 3,
    ACTIONS(595), 1,
      anon_sym_COMMA,
    ACTIONS(599), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3895] = 3,
    ACTIONS(601), 1,
      anon_sym_PIPE,
    ACTIONS(603), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3906] = 3,
    ACTIONS(591), 1,
      anon_sym_COMMA,
    ACTIONS(605), 1,
      anon_sym_RBRACK,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3917] = 3,
    ACTIONS(607), 1,
      ts_builtin_sym_end,
    ACTIONS(609), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3928] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(611), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [3937] = 3,
    ACTIONS(609), 1,
      anon_sym_SEMI,
    ACTIONS(613), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3948] = 2,
    ACTIONS(615), 1,
      aux_sym_escape_sequence_token1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3956] = 2,
    ACTIONS(591), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3964] = 2,
    ACTIONS(615), 1,
      aux_sym_escape_sequence_token4,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3972] = 2,
    ACTIONS(617), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3980] = 2,
    ACTIONS(609), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3988] = 2,
    ACTIONS(615), 1,
      aux_sym_escape_sequence_token2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3996] = 2,
    ACTIONS(499), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4004] = 2,
    ACTIONS(615), 1,
      aux_sym_escape_sequence_token3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4012] = 2,
    ACTIONS(595), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4020] = 2,
    ACTIONS(619), 1,
      anon_sym_EQ,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 70,
  [SMALL_STATE(4)] = 137,
  [SMALL_STATE(5)] = 208,
  [SMALL_STATE(6)] = 279,
  [SMALL_STATE(7)] = 346,
  [SMALL_STATE(8)] = 413,
  [SMALL_STATE(9)] = 478,
  [SMALL_STATE(10)] = 549,
  [SMALL_STATE(11)] = 616,
  [SMALL_STATE(12)] = 683,
  [SMALL_STATE(13)] = 747,
  [SMALL_STATE(14)] = 811,
  [SMALL_STATE(15)] = 875,
  [SMALL_STATE(16)] = 939,
  [SMALL_STATE(17)] = 1003,
  [SMALL_STATE(18)] = 1067,
  [SMALL_STATE(19)] = 1131,
  [SMALL_STATE(20)] = 1195,
  [SMALL_STATE(21)] = 1259,
  [SMALL_STATE(22)] = 1323,
  [SMALL_STATE(23)] = 1387,
  [SMALL_STATE(24)] = 1451,
  [SMALL_STATE(25)] = 1515,
  [SMALL_STATE(26)] = 1576,
  [SMALL_STATE(27)] = 1637,
  [SMALL_STATE(28)] = 1695,
  [SMALL_STATE(29)] = 1753,
  [SMALL_STATE(30)] = 1811,
  [SMALL_STATE(31)] = 1869,
  [SMALL_STATE(32)] = 1927,
  [SMALL_STATE(33)] = 1985,
  [SMALL_STATE(34)] = 2043,
  [SMALL_STATE(35)] = 2101,
  [SMALL_STATE(36)] = 2126,
  [SMALL_STATE(37)] = 2151,
  [SMALL_STATE(38)] = 2175,
  [SMALL_STATE(39)] = 2199,
  [SMALL_STATE(40)] = 2236,
  [SMALL_STATE(41)] = 2273,
  [SMALL_STATE(42)] = 2310,
  [SMALL_STATE(43)] = 2335,
  [SMALL_STATE(44)] = 2357,
  [SMALL_STATE(45)] = 2379,
  [SMALL_STATE(46)] = 2401,
  [SMALL_STATE(47)] = 2423,
  [SMALL_STATE(48)] = 2445,
  [SMALL_STATE(49)] = 2467,
  [SMALL_STATE(50)] = 2489,
  [SMALL_STATE(51)] = 2511,
  [SMALL_STATE(52)] = 2533,
  [SMALL_STATE(53)] = 2555,
  [SMALL_STATE(54)] = 2577,
  [SMALL_STATE(55)] = 2599,
  [SMALL_STATE(56)] = 2621,
  [SMALL_STATE(57)] = 2643,
  [SMALL_STATE(58)] = 2665,
  [SMALL_STATE(59)] = 2687,
  [SMALL_STATE(60)] = 2709,
  [SMALL_STATE(61)] = 2731,
  [SMALL_STATE(62)] = 2753,
  [SMALL_STATE(63)] = 2775,
  [SMALL_STATE(64)] = 2797,
  [SMALL_STATE(65)] = 2819,
  [SMALL_STATE(66)] = 2841,
  [SMALL_STATE(67)] = 2863,
  [SMALL_STATE(68)] = 2885,
  [SMALL_STATE(69)] = 2907,
  [SMALL_STATE(70)] = 2929,
  [SMALL_STATE(71)] = 2951,
  [SMALL_STATE(72)] = 2973,
  [SMALL_STATE(73)] = 2995,
  [SMALL_STATE(74)] = 3017,
  [SMALL_STATE(75)] = 3039,
  [SMALL_STATE(76)] = 3063,
  [SMALL_STATE(77)] = 3089,
  [SMALL_STATE(78)] = 3111,
  [SMALL_STATE(79)] = 3133,
  [SMALL_STATE(80)] = 3155,
  [SMALL_STATE(81)] = 3176,
  [SMALL_STATE(82)] = 3197,
  [SMALL_STATE(83)] = 3218,
  [SMALL_STATE(84)] = 3248,
  [SMALL_STATE(85)] = 3278,
  [SMALL_STATE(86)] = 3305,
  [SMALL_STATE(87)] = 3327,
  [SMALL_STATE(88)] = 3345,
  [SMALL_STATE(89)] = 3363,
  [SMALL_STATE(90)] = 3385,
  [SMALL_STATE(91)] = 3406,
  [SMALL_STATE(92)] = 3427,
  [SMALL_STATE(93)] = 3442,
  [SMALL_STATE(94)] = 3463,
  [SMALL_STATE(95)] = 3482,
  [SMALL_STATE(96)] = 3503,
  [SMALL_STATE(97)] = 3524,
  [SMALL_STATE(98)] = 3543,
  [SMALL_STATE(99)] = 3562,
  [SMALL_STATE(100)] = 3583,
  [SMALL_STATE(101)] = 3601,
  [SMALL_STATE(102)] = 3619,
  [SMALL_STATE(103)] = 3637,
  [SMALL_STATE(104)] = 3655,
  [SMALL_STATE(105)] = 3673,
  [SMALL_STATE(106)] = 3690,
  [SMALL_STATE(107)] = 3707,
  [SMALL_STATE(108)] = 3724,
  [SMALL_STATE(109)] = 3739,
  [SMALL_STATE(110)] = 3754,
  [SMALL_STATE(111)] = 3768,
  [SMALL_STATE(112)] = 3782,
  [SMALL_STATE(113)] = 3796,
  [SMALL_STATE(114)] = 3806,
  [SMALL_STATE(115)] = 3820,
  [SMALL_STATE(116)] = 3834,
  [SMALL_STATE(117)] = 3848,
  [SMALL_STATE(118)] = 3862,
  [SMALL_STATE(119)] = 3873,
  [SMALL_STATE(120)] = 3884,
  [SMALL_STATE(121)] = 3895,
  [SMALL_STATE(122)] = 3906,
  [SMALL_STATE(123)] = 3917,
  [SMALL_STATE(124)] = 3928,
  [SMALL_STATE(125)] = 3937,
  [SMALL_STATE(126)] = 3948,
  [SMALL_STATE(127)] = 3956,
  [SMALL_STATE(128)] = 3964,
  [SMALL_STATE(129)] = 3972,
  [SMALL_STATE(130)] = 3980,
  [SMALL_STATE(131)] = 3988,
  [SMALL_STATE(132)] = 3996,
  [SMALL_STATE(133)] = 4004,
  [SMALL_STATE(134)] = 4012,
  [SMALL_STATE(135)] = 4020,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(135),
  [9] = {.entry = {.count = 1, .reusable = false}}, SHIFT(42),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(3),
  [17] = {.entry = {.count = 1, .reusable = false}}, SHIFT(23),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [27] = {.entry = {.count = 1, .reusable = false}}, SHIFT(69),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(71),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(68),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(88),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [41] = {.entry = {.count = 1, .reusable = false}}, SHIFT(92),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(88),
  [45] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [47] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(42),
  [50] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(26),
  [53] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(13),
  [56] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(3),
  [59] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20),
  [61] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(40),
  [64] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(71),
  [67] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(11),
  [70] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20),
  [72] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(69),
  [75] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(71),
  [78] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(68),
  [81] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 20), SHIFT_REPEAT(42),
  [84] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [86] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(42),
  [89] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(26),
  [92] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12),
  [94] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(13),
  [97] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(3),
  [100] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(40),
  [103] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(71),
  [106] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(11),
  [109] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(69),
  [112] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(71),
  [115] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(68),
  [118] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(42),
  [121] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(88),
  [124] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(26),
  [127] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34),
  [129] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(13),
  [132] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(3),
  [135] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(40),
  [138] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(71),
  [141] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(11),
  [144] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(69),
  [147] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(71),
  [150] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(92),
  [153] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(68),
  [156] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, 0, 34), SHIFT_REPEAT(88),
  [159] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(42),
  [162] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(26),
  [165] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(13),
  [168] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(3),
  [171] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12),
  [173] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(40),
  [176] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(71),
  [179] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(11),
  [182] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(69),
  [185] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(71),
  [188] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(68),
  [191] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(42),
  [194] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [196] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 4, 0, 27),
  [198] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 4, 0, 27),
  [200] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [202] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [204] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [206] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 3, 0, 17),
  [208] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 3, 0, 17),
  [210] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 2, 0, 9),
  [212] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 2, 0, 9),
  [214] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [216] = {.entry = {.count = 1, .reusable = true}}, SHIFT(79),
  [218] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(42),
  [221] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(26),
  [224] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(13),
  [227] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(3),
  [230] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16),
  [232] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(40),
  [235] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(71),
  [238] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(11),
  [241] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16),
  [243] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(69),
  [246] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(71),
  [249] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(68),
  [252] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, 0, 16), SHIFT(42),
  [255] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [257] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 5, 0, 37),
  [259] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 5, 0, 37),
  [261] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [263] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [265] = {.entry = {.count = 1, .reusable = false}}, SHIFT(87),
  [267] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [269] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 9),
  [271] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, 0, 9),
  [273] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, 0, 31),
  [275] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, 0, 31),
  [277] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 9),
  [279] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, 0, 9),
  [281] = {.entry = {.count = 1, .reusable = false}}, SHIFT(44),
  [283] = {.entry = {.count = 1, .reusable = true}}, SHIFT(81),
  [285] = {.entry = {.count = 1, .reusable = false}}, SHIFT(82),
  [287] = {.entry = {.count = 1, .reusable = false}}, SHIFT(126),
  [289] = {.entry = {.count = 1, .reusable = false}}, SHIFT(131),
  [291] = {.entry = {.count = 1, .reusable = false}}, SHIFT(133),
  [293] = {.entry = {.count = 1, .reusable = false}}, SHIFT(128),
  [295] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [297] = {.entry = {.count = 1, .reusable = false}}, SHIFT(56),
  [299] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__string_content, 2, 0, 15),
  [301] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__string_content, 2, 0, 15), SHIFT_REPEAT(81),
  [304] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, 0, 15), SHIFT_REPEAT(82),
  [307] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, 0, 15), SHIFT_REPEAT(126),
  [310] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, 0, 15), SHIFT_REPEAT(131),
  [313] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, 0, 15), SHIFT_REPEAT(133),
  [316] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, 0, 15), SHIFT_REPEAT(128),
  [319] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__expression, 1, 0, 0),
  [321] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [323] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__expression, 1, 0, 0),
  [325] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 5, 0, 23),
  [327] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 5, 0, 23),
  [329] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 3, 0, 14),
  [331] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 3, 0, 14),
  [333] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, 0, 19),
  [335] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, 0, 19),
  [337] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 4, 0, 25),
  [339] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 4, 0, 25),
  [341] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 2, 0, 0),
  [343] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 2, 0, 0),
  [345] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, 0, 28),
  [347] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, 0, 28),
  [349] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 3, 0, 10),
  [351] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 3, 0, 10),
  [353] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, 0, 30),
  [355] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, 0, 30),
  [357] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 4, 0, 25),
  [359] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 4, 0, 25),
  [361] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 4, 0, 23),
  [363] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 4, 0, 23),
  [365] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 3, 0, 10),
  [367] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 3, 0, 10),
  [369] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 3, 0, 11),
  [371] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 3, 0, 11),
  [373] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 4, 0, 10),
  [375] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 4, 0, 10),
  [377] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 2, 0, 0),
  [379] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 2, 0, 0),
  [381] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 4, 0, 32),
  [383] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 4, 0, 32),
  [385] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 4, 0, 33),
  [387] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 4, 0, 33),
  [389] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 3, 0, 10),
  [391] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 3, 0, 10),
  [393] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 3, 0, 11),
  [395] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 3, 0, 11),
  [397] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, 0, 18),
  [399] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, 0, 18),
  [401] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 2, 0, 0),
  [403] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 2, 0, 0),
  [405] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 5, 0, 38),
  [407] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 5, 0, 38),
  [409] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 6, 0, 39),
  [411] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 6, 0, 39),
  [413] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 2, 0, 0),
  [415] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 2, 0, 0),
  [417] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 5, 0, 35),
  [419] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 5, 0, 35),
  [421] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 5, 0, 36),
  [423] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 5, 0, 36),
  [425] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 1, 0, 0),
  [427] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 1, 0, 0),
  [429] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean_literal, 1, 0, 0),
  [431] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean_literal, 1, 0, 0),
  [433] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 3, 0, 18),
  [435] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 3, 0, 18),
  [437] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 3, 0, 19),
  [439] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 3, 0, 19),
  [441] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 5, 0, 28),
  [443] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 5, 0, 28),
  [445] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_infix_operator, 3, 0, 21),
  [447] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_infix_operator, 3, 0, 21),
  [449] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [451] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [453] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 3, 0, 22),
  [455] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 3, 0, 22),
  [457] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 5, 0, 30),
  [459] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 5, 0, 30),
  [461] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 4, 0, 10),
  [463] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 4, 0, 10),
  [465] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_escape_sequence, 2, 0, 13),
  [467] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_escape_sequence, 2, 0, 13),
  [469] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__string_content, 1, 0, 7),
  [471] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__string_content, 1, 0, 7),
  [473] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_escape_sequence, 1, 0, 8),
  [475] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_escape_sequence, 1, 0, 8),
  [477] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [479] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [481] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 1, 0, 9),
  [483] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [485] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 1, 0, 9),
  [487] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [489] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [491] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 3, 0, 27),
  [493] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 3, 0, 27),
  [495] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, 0, 10),
  [497] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, 0, 10),
  [499] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [501] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__call_arg, 1, 0, 0),
  [503] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [505] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_member, 1, 0, 6),
  [507] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [509] = {.entry = {.count = 1, .reusable = true}}, SHIFT(66),
  [511] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 2),
  [513] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 4),
  [515] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 4), SHIFT_REPEAT(135),
  [518] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 3, 0, 5),
  [520] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [522] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [524] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_member, 3, 0, 24),
  [526] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_member, 3, 0, 26),
  [528] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [530] = {.entry = {.count = 1, .reusable = true}}, SHIFT(132),
  [532] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [534] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [536] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [538] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [540] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [542] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 4, 0, 37),
  [544] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 4, 0, 37),
  [546] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, 0, 12), SHIFT_REPEAT(29),
  [549] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, 0, 12),
  [551] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, 0, 12),
  [553] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [555] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 2, 0, 17),
  [557] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 2, 0, 17),
  [559] = {.entry = {.count = 1, .reusable = true}}, SHIFT(102),
  [561] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [563] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [565] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [567] = {.entry = {.count = 1, .reusable = false}}, SHIFT(15),
  [569] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [571] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 1),
  [573] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [575] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [577] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [579] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [581] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_record_literal_repeat1, 2, 0, 12), SHIFT_REPEAT(108),
  [584] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_record_literal_repeat1, 2, 0, 12),
  [586] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, 0, 29), SHIFT_REPEAT(25),
  [589] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, 0, 29),
  [591] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [593] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [595] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [597] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [599] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [601] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, 0, 18),
  [603] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, 0, 18),
  [605] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [607] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 2, 0, 3),
  [609] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [611] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_record_literal_repeat1, 2, 0, 10),
  [613] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 1),
  [615] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [617] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [619] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_datazinc(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .keyword_lex_fn = ts_lex_keywords,
    .keyword_capture_token = sym_identifier,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
