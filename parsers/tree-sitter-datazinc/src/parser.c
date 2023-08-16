#include <tree_sitter/parser.h>

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 137
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 126
#define ALIAS_COUNT 0
#define TOKEN_COUNT 98
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 15
#define MAX_ALIAS_SEQUENCE_LENGTH 6
#define PRODUCTION_ID_COUNT 40

enum {
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
  anon_sym_ = 40,
  anon_sym_case = 41,
  anon_sym_of = 42,
  anon_sym_endcase = 43,
  anon_sym_EQ_GT = 44,
  anon_sym_lambda = 45,
  anon_sym_let = 46,
  anon_sym_DASH = 47,
  anon_sym_not = 48,
  anon_sym_2 = 49,
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
  anon_sym_infinity = 76,
  anon_sym_3 = 77,
  anon_sym_4 = 78,
  sym_string_characters = 79,
  anon_sym_BSLASH_SQUOTE = 80,
  anon_sym_BSLASH_DQUOTE = 81,
  anon_sym_BSLASH_BSLASH = 82,
  anon_sym_BSLASHr = 83,
  anon_sym_BSLASHn = 84,
  anon_sym_BSLASHt = 85,
  anon_sym_BSLASH = 86,
  aux_sym_escape_sequence_token1 = 87,
  anon_sym_BSLASHx = 88,
  aux_sym_escape_sequence_token2 = 89,
  anon_sym_BSLASHu = 90,
  aux_sym_escape_sequence_token3 = 91,
  anon_sym_BSLASHU = 92,
  aux_sym_escape_sequence_token4 = 93,
  sym_quoted_identifier = 94,
  anon_sym_CARET_DASH1 = 95,
  sym_line_comment = 96,
  sym_block_comment = 97,
  sym_source_file = 98,
  sym_assignment = 99,
  sym__expression = 100,
  sym_call = 101,
  sym_infix_operator = 102,
  sym_array_literal = 103,
  sym_array_literal_member = 104,
  sym_array_literal_2d = 105,
  sym_array_literal_2d_row = 106,
  sym_boolean_literal = 107,
  sym_infinity = 108,
  sym_set_literal = 109,
  sym_string_literal = 110,
  aux_sym__string_content = 111,
  sym_escape_sequence = 112,
  sym_tuple_literal = 113,
  sym_record_literal = 114,
  sym_record_member = 115,
  sym__identifier = 116,
  sym__call_arg = 117,
  aux_sym_source_file_repeat1 = 118,
  aux_sym_call_repeat1 = 119,
  aux_sym_array_literal_repeat1 = 120,
  aux_sym_array_literal_2d_repeat1 = 121,
  aux_sym_array_literal_2d_repeat2 = 122,
  aux_sym_array_literal_2d_row_repeat1 = 123,
  aux_sym_set_literal_repeat1 = 124,
  aux_sym_record_literal_repeat1 = 125,
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
  [anon_sym_] = "∪",
  [anon_sym_case] = "case",
  [anon_sym_of] = "of",
  [anon_sym_endcase] = "endcase",
  [anon_sym_EQ_GT] = "=>",
  [anon_sym_lambda] = "lambda",
  [anon_sym_let] = "let",
  [anon_sym_DASH] = "-",
  [anon_sym_not] = "not",
  [anon_sym_2] = "¬",
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
  [anon_sym_infinity] = "infinity",
  [anon_sym_3] = "∞",
  [anon_sym_4] = "∅",
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
  [sym_infinity] = "infinity",
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
  [anon_sym_] = anon_sym_,
  [anon_sym_case] = anon_sym_case,
  [anon_sym_of] = anon_sym_of,
  [anon_sym_endcase] = anon_sym_endcase,
  [anon_sym_EQ_GT] = anon_sym_EQ_GT,
  [anon_sym_lambda] = anon_sym_lambda,
  [anon_sym_let] = anon_sym_let,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_not] = anon_sym_not,
  [anon_sym_2] = anon_sym_2,
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
  [anon_sym_infinity] = anon_sym_infinity,
  [anon_sym_3] = anon_sym_3,
  [anon_sym_4] = anon_sym_4,
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
  [sym_infinity] = sym_infinity,
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
  [anon_sym_] = {
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
  [anon_sym_2] = {
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
  [anon_sym_infinity] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_3] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_4] = {
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
  [sym_infinity] = {
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

enum {
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
  [136] = 136,
};

static inline bool sym_identifier_character_set_1(int32_t c) {
  return (c < 8660
    ? (c < 8592
      ? (c < '&'
        ? (c < '!'
          ? c == 0
          : c <= '!')
        : (c <= '>' || c == '~'))
      : (c <= 8592 || (c < 8656
        ? c == 8594
        : (c <= 8656 || c == 8658))))
    : (c <= 8660 || (c < 8804
      ? (c < 8743
        ? (c < 8726
          ? c == 8712
          : c <= 8726)
        : (c <= 8745 || c == 8800))
      : (c <= 8805 || (c < 8891
        ? (c >= 8838 && c <= 8839)
        : (c <= 8891 || c == 10231))))));
}

static inline bool sym_identifier_character_set_2(int32_t c) {
  return (c < 8660
    ? (c < '~'
      ? (c < '$'
        ? (c < '!'
          ? c == 0
          : c <= '!')
        : (c <= '>' || c == '^'))
      : (c <= '~' || (c < 8656
        ? (c < 8594
          ? c == 8592
          : c <= 8594)
        : (c <= 8656 || c == 8658))))
    : (c <= 8660 || (c < 8804
      ? (c < 8743
        ? (c < 8726
          ? c == 8712
          : c <= 8726)
        : (c <= 8746 || c == 8800))
      : (c <= 8805 || (c < 8891
        ? (c >= 8838 && c <= 8839)
        : (c <= 8891 || c == 10231))))));
}

static inline bool sym_identifier_character_set_3(int32_t c) {
  return (c < 8658
    ? (c < '^'
      ? (c < '$'
        ? (c < '!'
          ? c == 0
          : c <= '"')
        : (c <= '-' || (c < '['
          ? (c >= '<' && c <= '>')
          : c <= '[')))
      : (c <= '^' || (c < 8594
        ? (c < 8592
          ? (c >= '{' && c <= '~')
          : c <= 8592)
        : (c <= 8594 || c == 8656))))
    : (c <= 8658 || (c < 8800
      ? (c < 8726
        ? (c < 8712
          ? c == 8660
          : c <= 8712)
        : (c <= 8726 || (c < 8743
          ? c == 8734
          : c <= 8745)))
      : (c <= 8800 || (c < 8891
        ? (c < 8838
          ? (c >= 8804 && c <= 8805)
          : c <= 8839)
        : (c <= 8891 || c == 10231))))));
}

static inline bool sym_identifier_character_set_4(int32_t c) {
  return (c < 8656
    ? (c < ':'
      ? (c < '\r'
        ? (c < '\t'
          ? c == 0
          : c <= '\n')
        : (c <= '\r' || (c < '$'
          ? (c >= ' ' && c <= '"')
          : c <= '.')))
      : (c <= '>' || (c < '{'
        ? (c < ']'
          ? c == '['
          : c <= '^')
        : (c <= '~' || (c < 8594
          ? c == 8592
          : c <= 8594)))))
    : (c <= 8656 || (c < 8743
      ? (c < 8712
        ? (c < 8660
          ? c == 8658
          : c <= 8660)
        : (c <= 8712 || (c < 8734
          ? c == 8726
          : c <= 8734)))
      : (c <= 8746 || (c < 8838
        ? (c < 8804
          ? c == 8800
          : c <= 8805)
        : (c <= 8839 || (c < 10231
          ? c == 8891
          : c <= 10231)))))));
}

static inline bool sym_identifier_character_set_5(int32_t c) {
  return (c < 8656
    ? (c < ':'
      ? (c < '\r'
        ? (c < '\t'
          ? c == 0
          : c <= '\n')
        : (c <= '\r' || (c < '$'
          ? (c >= ' ' && c <= '"')
          : c <= '-')))
      : (c <= '>' || (c < '{'
        ? (c < ']'
          ? c == '['
          : c <= '^')
        : (c <= '~' || (c < 8594
          ? c == 8592
          : c <= 8594)))))
    : (c <= 8656 || (c < 8743
      ? (c < 8712
        ? (c < 8660
          ? c == 8658
          : c <= 8660)
        : (c <= 8712 || (c < 8734
          ? c == 8726
          : c <= 8734)))
      : (c <= 8746 || (c < 8838
        ? (c < 8804
          ? c == 8800
          : c <= 8805)
        : (c <= 8839 || (c < 10231
          ? c == 8891
          : c <= 10231)))))));
}

static inline bool sym_identifier_character_set_6(int32_t c) {
  return (c < 8658
    ? (c < ':'
      ? (c < '\r'
        ? (c < '\t'
          ? c == 0
          : c <= '\n')
        : (c <= '\r' || (c < '$'
          ? (c >= ' ' && c <= '!')
          : c <= '.')))
      : (c <= '>' || (c < 8592
        ? (c < '{'
          ? (c >= '[' && c <= '^')
          : c <= '~')
        : (c <= 8592 || (c < 8656
          ? c == 8594
          : c <= 8656)))))
    : (c <= 8658 || (c < 8800
      ? (c < 8726
        ? (c < 8712
          ? c == 8660
          : c <= 8712)
        : (c <= 8726 || (c < 8743
          ? c == 8734
          : c <= 8746)))
      : (c <= 8800 || (c < 8891
        ? (c < 8838
          ? (c >= 8804 && c <= 8805)
          : c <= 8839)
        : (c <= 8891 || c == 10231))))));
}

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(41);
      if (lookahead == '"') ADVANCE(67);
      if (lookahead == '$') ADVANCE(4);
      if (lookahead == '%') ADVANCE(130);
      if (lookahead == '\'') ADVANCE(9);
      if (lookahead == '(') ADVANCE(49);
      if (lookahead == ')') ADVANCE(51);
      if (lookahead == '+') ADVANCE(12);
      if (lookahead == ',') ADVANCE(50);
      if (lookahead == '-') ADVANCE(65);
      if (lookahead == '.') ADVANCE(59);
      if (lookahead == '/') ADVANCE(122);
      if (lookahead == '0') ADVANCE(110);
      if (lookahead == ':') ADVANCE(46);
      if (lookahead == ';') ADVANCE(42);
      if (lookahead == '<') ADVANCE(15);
      if (lookahead == '=') ADVANCE(44);
      if (lookahead == '[') ADVANCE(54);
      if (lookahead == '\\') ADVANCE(105);
      if (lookahead == ']') ADVANCE(56);
      if (lookahead == '^') ADVANCE(13);
      if (lookahead == '{') ADVANCE(52);
      if (lookahead == '|') ADVANCE(55);
      if (lookahead == '}') ADVANCE(53);
      if (lookahead == 172) ADVANCE(66);
      if (lookahead == 8709) ADVANCE(88);
      if (lookahead == 8734) ADVANCE(87);
      if (lookahead == 8746) ADVANCE(63);
      if (lookahead == '8' ||
          lookahead == '9') ADVANCE(62);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
      if (('1' <= lookahead && lookahead <= '7')) ADVANCE(61);
      if (!sym_identifier_character_set_1(lookahead)) ADVANCE(127);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(3)
      if (lookahead == '"') ADVANCE(67);
      if (lookahead == '%') ADVANCE(94);
      if (lookahead == '/') ADVANCE(92);
      if (lookahead == '\\') ADVANCE(106);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(89);
      if (lookahead != 0) ADVANCE(94);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(67);
      if (lookahead == '%') ADVANCE(130);
      if (lookahead == '\'') ADVANCE(9);
      if (lookahead == '(') ADVANCE(49);
      if (lookahead == ')') ADVANCE(51);
      if (lookahead == '-') ADVANCE(17);
      if (lookahead == '/') ADVANCE(122);
      if (lookahead == '0') ADVANCE(77);
      if (lookahead == '<') ADVANCE(19);
      if (lookahead == '[') ADVANCE(54);
      if (lookahead == ']') ADVANCE(56);
      if (lookahead == '{') ADVANCE(52);
      if (lookahead == '|') ADVANCE(55);
      if (lookahead == '}') ADVANCE(53);
      if (lookahead == 8709) ADVANCE(88);
      if (lookahead == 8734) ADVANCE(87);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(2)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(79);
      if (!sym_identifier_character_set_2(lookahead)) ADVANCE(127);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(67);
      if (lookahead == '%') ADVANCE(130);
      if (lookahead == '/') ADVANCE(10);
      if (lookahead == '\\') ADVANCE(106);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(3)
      END_STATE();
    case 4:
      if (lookahead == '$') ADVANCE(37);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(69);
      END_STATE();
    case 5:
      if (lookahead == '%') ADVANCE(130);
      if (lookahead == '/') ADVANCE(10);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(5)
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(111);
      END_STATE();
    case 6:
      if (lookahead == '%') ADVANCE(130);
      if (lookahead == '/') ADVANCE(10);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(6)
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(26);
      END_STATE();
    case 7:
      if (lookahead == '%') ADVANCE(130);
      if (lookahead == '/') ADVANCE(10);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(7)
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(31);
      END_STATE();
    case 8:
      if (lookahead == '%') ADVANCE(130);
      if (lookahead == '/') ADVANCE(10);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(8)
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(36);
      END_STATE();
    case 9:
      if (lookahead == '\'') ADVANCE(128);
      if (lookahead != 0) ADVANCE(9);
      END_STATE();
    case 10:
      if (lookahead == '*') ADVANCE(39);
      END_STATE();
    case 11:
      if (lookahead == '*') ADVANCE(38);
      if (lookahead == '/') ADVANCE(131);
      if (lookahead != 0) ADVANCE(39);
      END_STATE();
    case 12:
      if (lookahead == '+') ADVANCE(47);
      END_STATE();
    case 13:
      if (lookahead == '-') ADVANCE(18);
      END_STATE();
    case 14:
      if (lookahead == '.') ADVANCE(57);
      END_STATE();
    case 15:
      if (lookahead == '.') ADVANCE(16);
      if (lookahead == '>') ADVANCE(71);
      END_STATE();
    case 16:
      if (lookahead == '.') ADVANCE(58);
      END_STATE();
    case 17:
      if (lookahead == '0') ADVANCE(78);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(80);
      END_STATE();
    case 18:
      if (lookahead == '1') ADVANCE(129);
      END_STATE();
    case 19:
      if (lookahead == '>') ADVANCE(71);
      END_STATE();
    case 20:
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(24);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(76);
      END_STATE();
    case 21:
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(82);
      END_STATE();
    case 22:
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(84);
      END_STATE();
    case 23:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(74);
      END_STATE();
    case 24:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(76);
      END_STATE();
    case 25:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(86);
      END_STATE();
    case 26:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(115);
      END_STATE();
    case 27:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(118);
      END_STATE();
    case 28:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(121);
      END_STATE();
    case 29:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(27);
      END_STATE();
    case 30:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(28);
      END_STATE();
    case 31:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(29);
      END_STATE();
    case 32:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(30);
      END_STATE();
    case 33:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(32);
      END_STATE();
    case 34:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(33);
      END_STATE();
    case 35:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(34);
      END_STATE();
    case 36:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(35);
      END_STATE();
    case 37:
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(70);
      END_STATE();
    case 38:
      if (lookahead != 0 &&
          lookahead != '*' &&
          lookahead != '/') ADVANCE(39);
      if (lookahead == '*') ADVANCE(11);
      if (lookahead == '/') ADVANCE(132);
      END_STATE();
    case 39:
      if (lookahead != 0 &&
          lookahead != '*') ADVANCE(39);
      if (lookahead == '*') ADVANCE(11);
      END_STATE();
    case 40:
      if (eof) ADVANCE(41);
      if (lookahead == '%') ADVANCE(130);
      if (lookahead == '\'') ADVANCE(9);
      if (lookahead == '(') ADVANCE(49);
      if (lookahead == ')') ADVANCE(51);
      if (lookahead == '+') ADVANCE(12);
      if (lookahead == ',') ADVANCE(50);
      if (lookahead == '.') ADVANCE(14);
      if (lookahead == '/') ADVANCE(122);
      if (lookahead == ':') ADVANCE(45);
      if (lookahead == ';') ADVANCE(42);
      if (lookahead == '=') ADVANCE(43);
      if (lookahead == ']') ADVANCE(56);
      if (lookahead == '|') ADVANCE(55);
      if (lookahead == '}') ADVANCE(53);
      if (lookahead == 8746) ADVANCE(63);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(40)
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(127);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(anon_sym_EQ);
      if (lookahead == '>') ADVANCE(64);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(anon_sym_COLON);
      if (lookahead == ':') ADVANCE(48);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(anon_sym_PLUS_PLUS);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(anon_sym_COLON_COLON);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      if (lookahead == '|') ADVANCE(72);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym_PIPE);
      if (lookahead == ']') ADVANCE(73);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(anon_sym_LT_DOT_DOT);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(anon_sym_DOT);
      if (lookahead == '.') ADVANCE(57);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(aux_sym_tuple_access_token1);
      if (lookahead == '8' ||
          lookahead == '9') ADVANCE(62);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(62);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(aux_sym_tuple_access_token1);
      if (lookahead == '8' ||
          lookahead == '9') ADVANCE(62);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(60);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(aux_sym_tuple_access_token1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(62);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(anon_sym_);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(anon_sym_EQ_GT);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(anon_sym_2);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(anon_sym_BSLASH_LPAREN);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(sym_type_inst_id);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(69);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(sym_type_inst_enum_id);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(70);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(sym_absent);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(anon_sym_LBRACK_PIPE);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(anon_sym_PIPE_RBRACK);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(sym_float_literal);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(74);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(75);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(76);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(23);
      if (lookahead == 'b') ADVANCE(124);
      if (lookahead == 'o') ADVANCE(125);
      if (lookahead == 'x') ADVANCE(126);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(123);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(79);
      if (!sym_identifier_character_set_5(lookahead)) ADVANCE(127);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(23);
      if (lookahead == 'b') ADVANCE(21);
      if (lookahead == 'o') ADVANCE(22);
      if (lookahead == 'x') ADVANCE(25);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(80);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(23);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(123);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(79);
      if (!sym_identifier_character_set_5(lookahead)) ADVANCE(127);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(23);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(80);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(81);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(82);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(83);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(84);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(85);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(86);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(anon_sym_3);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(anon_sym_4);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '%') ADVANCE(94);
      if (lookahead == '/') ADVANCE(92);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(89);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(94);
      END_STATE();
    case 90:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(93);
      if (lookahead == '/') ADVANCE(91);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(91);
      END_STATE();
    case 91:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(93);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(91);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(91);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(94);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(90);
      if (lookahead == '/') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(91);
      END_STATE();
    case 94:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(94);
      END_STATE();
    case 95:
      ACCEPT_TOKEN(anon_sym_BSLASH_SQUOTE);
      END_STATE();
    case 96:
      ACCEPT_TOKEN(anon_sym_BSLASH_DQUOTE);
      END_STATE();
    case 97:
      ACCEPT_TOKEN(anon_sym_BSLASH_BSLASH);
      END_STATE();
    case 98:
      ACCEPT_TOKEN(anon_sym_BSLASH_BSLASH);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 99:
      ACCEPT_TOKEN(anon_sym_BSLASHr);
      END_STATE();
    case 100:
      ACCEPT_TOKEN(anon_sym_BSLASHr);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 101:
      ACCEPT_TOKEN(anon_sym_BSLASHn);
      END_STATE();
    case 102:
      ACCEPT_TOKEN(anon_sym_BSLASHn);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 103:
      ACCEPT_TOKEN(anon_sym_BSLASHt);
      END_STATE();
    case 104:
      ACCEPT_TOKEN(anon_sym_BSLASHt);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 105:
      ACCEPT_TOKEN(anon_sym_BSLASH);
      if (lookahead == '"') ADVANCE(96);
      if (lookahead == '\'') ADVANCE(95);
      if (lookahead == '(') ADVANCE(68);
      if (lookahead == 'U') ADVANCE(120);
      if (lookahead == '\\') ADVANCE(98);
      if (lookahead == 'n') ADVANCE(102);
      if (lookahead == 'r') ADVANCE(100);
      if (lookahead == 't') ADVANCE(104);
      if (lookahead == 'u') ADVANCE(117);
      if (lookahead == 'x') ADVANCE(114);
      if (!sym_identifier_character_set_6(lookahead)) ADVANCE(127);
      END_STATE();
    case 106:
      ACCEPT_TOKEN(anon_sym_BSLASH);
      if (lookahead == '"') ADVANCE(96);
      if (lookahead == '\'') ADVANCE(95);
      if (lookahead == 'U') ADVANCE(119);
      if (lookahead == '\\') ADVANCE(97);
      if (lookahead == 'n') ADVANCE(101);
      if (lookahead == 'r') ADVANCE(99);
      if (lookahead == 't') ADVANCE(103);
      if (lookahead == 'u') ADVANCE(116);
      if (lookahead == 'x') ADVANCE(113);
      END_STATE();
    case 107:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      END_STATE();
    case 108:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(112);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 109:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(107);
      END_STATE();
    case 110:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(108);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 111:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(109);
      END_STATE();
    case 112:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 113:
      ACCEPT_TOKEN(anon_sym_BSLASHx);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(anon_sym_BSLASHx);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 115:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token2);
      END_STATE();
    case 116:
      ACCEPT_TOKEN(anon_sym_BSLASHu);
      END_STATE();
    case 117:
      ACCEPT_TOKEN(anon_sym_BSLASHu);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 118:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token3);
      END_STATE();
    case 119:
      ACCEPT_TOKEN(anon_sym_BSLASHU);
      END_STATE();
    case 120:
      ACCEPT_TOKEN(anon_sym_BSLASHU);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 121:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token4);
      END_STATE();
    case 122:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '*') ADVANCE(39);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 123:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(24);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(75);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 124:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(81);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 125:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(83);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 126:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(85);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 127:
      ACCEPT_TOKEN(sym_identifier);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(127);
      END_STATE();
    case 128:
      ACCEPT_TOKEN(sym_quoted_identifier);
      END_STATE();
    case 129:
      ACCEPT_TOKEN(anon_sym_CARET_DASH1);
      END_STATE();
    case 130:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(130);
      END_STATE();
    case 131:
      ACCEPT_TOKEN(sym_block_comment);
      END_STATE();
    case 132:
      ACCEPT_TOKEN(sym_block_comment);
      if (lookahead != 0 &&
          lookahead != '*') ADVANCE(39);
      if (lookahead == '*') ADVANCE(11);
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
      if (lookahead == '_') ADVANCE(1);
      if (lookahead == 'a') ADVANCE(2);
      if (lookahead == 'b') ADVANCE(3);
      if (lookahead == 'c') ADVANCE(4);
      if (lookahead == 'e') ADVANCE(5);
      if (lookahead == 'f') ADVANCE(6);
      if (lookahead == 'i') ADVANCE(7);
      if (lookahead == 'l') ADVANCE(8);
      if (lookahead == 'm') ADVANCE(9);
      if (lookahead == 'n') ADVANCE(10);
      if (lookahead == 'o') ADVANCE(11);
      if (lookahead == 'p') ADVANCE(12);
      if (lookahead == 'r') ADVANCE(13);
      if (lookahead == 's') ADVANCE(14);
      if (lookahead == 't') ADVANCE(15);
      if (lookahead == 'u') ADVANCE(16);
      if (lookahead == 'v') ADVANCE(17);
      if (lookahead == 'w') ADVANCE(18);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
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
      if (lookahead == 'f') ADVANCE(67);
      if (lookahead == 't') ADVANCE(68);
      END_STATE();
    case 31:
      if (lookahead == 'm') ADVANCE(69);
      END_STATE();
    case 32:
      if (lookahead == 't') ADVANCE(70);
      END_STATE();
    case 33:
      if (lookahead == 'x') ADVANCE(71);
      END_STATE();
    case 34:
      if (lookahead == 'n') ADVANCE(72);
      END_STATE();
    case 35:
      if (lookahead == 't') ADVANCE(73);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_of);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_op);
      if (lookahead == 't') ADVANCE(74);
      END_STATE();
    case 38:
      if (lookahead == 't') ADVANCE(75);
      END_STATE();
    case 39:
      if (lookahead == 'r') ADVANCE(76);
      END_STATE();
    case 40:
      if (lookahead == 'e') ADVANCE(77);
      END_STATE();
    case 41:
      if (lookahead == 'c') ADVANCE(78);
      END_STATE();
    case 42:
      if (lookahead == 't') ADVANCE(79);
      END_STATE();
    case 43:
      if (lookahead == 't') ADVANCE(80);
      END_STATE();
    case 44:
      if (lookahead == 'l') ADVANCE(81);
      END_STATE();
    case 45:
      if (lookahead == 'r') ADVANCE(82);
      END_STATE();
    case 46:
      if (lookahead == 's') ADVANCE(83);
      END_STATE();
    case 47:
      if (lookahead == 'e') ADVANCE(84);
      END_STATE();
    case 48:
      if (lookahead == 'u') ADVANCE(85);
      END_STATE();
    case 49:
      if (lookahead == 'p') ADVANCE(86);
      END_STATE();
    case 50:
      if (lookahead == 'p') ADVANCE(87);
      END_STATE();
    case 51:
      if (lookahead == 'i') ADVANCE(88);
      END_STATE();
    case 52:
      if (lookahead == 'r') ADVANCE(89);
      END_STATE();
    case 53:
      if (lookahead == 'e') ADVANCE(90);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_ann);
      if (lookahead == 'o') ADVANCE(91);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym_any);
      END_STATE();
    case 56:
      if (lookahead == 'a') ADVANCE(92);
      END_STATE();
    case 57:
      if (lookahead == 'l') ADVANCE(93);
      END_STATE();
    case 58:
      if (lookahead == 'e') ADVANCE(94);
      END_STATE();
    case 59:
      if (lookahead == 's') ADVANCE(95);
      END_STATE();
    case 60:
      if (lookahead == 'e') ADVANCE(96);
      END_STATE();
    case 61:
      if (lookahead == 'c') ADVANCE(97);
      if (lookahead == 'i') ADVANCE(98);
      END_STATE();
    case 62:
      if (lookahead == 'm') ADVANCE(99);
      END_STATE();
    case 63:
      if (lookahead == 's') ADVANCE(100);
      END_STATE();
    case 64:
      if (lookahead == 'a') ADVANCE(101);
      END_STATE();
    case 65:
      if (lookahead == 'c') ADVANCE(102);
      END_STATE();
    case 66:
      if (lookahead == 'l') ADVANCE(103);
      END_STATE();
    case 67:
      if (lookahead == 'i') ADVANCE(104);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(anon_sym_int);
      END_STATE();
    case 69:
      if (lookahead == 'b') ADVANCE(105);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(anon_sym_let);
      END_STATE();
    case 71:
      if (lookahead == 'i') ADVANCE(106);
      END_STATE();
    case 72:
      if (lookahead == 'i') ADVANCE(107);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(anon_sym_not);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(anon_sym_opt);
      END_STATE();
    case 75:
      if (lookahead == 'p') ADVANCE(108);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(anon_sym_par);
      END_STATE();
    case 77:
      if (lookahead == 'd') ADVANCE(109);
      END_STATE();
    case 78:
      if (lookahead == 'o') ADVANCE(110);
      END_STATE();
    case 79:
      if (lookahead == 'i') ADVANCE(111);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 81:
      if (lookahead == 'v') ADVANCE(112);
      END_STATE();
    case 82:
      if (lookahead == 'i') ADVANCE(113);
      END_STATE();
    case 83:
      if (lookahead == 't') ADVANCE(114);
      END_STATE();
    case 84:
      if (lookahead == 'n') ADVANCE(115);
      END_STATE();
    case 85:
      if (lookahead == 'e') ADVANCE(116);
      END_STATE();
    case 86:
      if (lookahead == 'l') ADVANCE(117);
      END_STATE();
    case 87:
      if (lookahead == 'e') ADVANCE(118);
      END_STATE();
    case 88:
      if (lookahead == 'o') ADVANCE(119);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(anon_sym_var);
      END_STATE();
    case 90:
      if (lookahead == 'r') ADVANCE(120);
      END_STATE();
    case 91:
      if (lookahead == 't') ADVANCE(121);
      END_STATE();
    case 92:
      if (lookahead == 'y') ADVANCE(122);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(anon_sym_bool);
      END_STATE();
    case 94:
      ACCEPT_TOKEN(anon_sym_case);
      END_STATE();
    case 95:
      if (lookahead == 't') ADVANCE(123);
      END_STATE();
    case 96:
      ACCEPT_TOKEN(anon_sym_else);
      if (lookahead == 'i') ADVANCE(124);
      END_STATE();
    case 97:
      if (lookahead == 'a') ADVANCE(125);
      END_STATE();
    case 98:
      if (lookahead == 'f') ADVANCE(126);
      END_STATE();
    case 99:
      ACCEPT_TOKEN(anon_sym_enum);
      END_STATE();
    case 100:
      if (lookahead == 'e') ADVANCE(127);
      END_STATE();
    case 101:
      if (lookahead == 't') ADVANCE(128);
      END_STATE();
    case 102:
      if (lookahead == 't') ADVANCE(129);
      END_STATE();
    case 103:
      if (lookahead == 'u') ADVANCE(130);
      END_STATE();
    case 104:
      if (lookahead == 'n') ADVANCE(131);
      END_STATE();
    case 105:
      if (lookahead == 'd') ADVANCE(132);
      END_STATE();
    case 106:
      if (lookahead == 'm') ADVANCE(133);
      END_STATE();
    case 107:
      if (lookahead == 'm') ADVANCE(134);
      END_STATE();
    case 108:
      if (lookahead == 'u') ADVANCE(135);
      END_STATE();
    case 109:
      if (lookahead == 'i') ADVANCE(136);
      END_STATE();
    case 110:
      if (lookahead == 'r') ADVANCE(137);
      END_STATE();
    case 111:
      if (lookahead == 's') ADVANCE(138);
      END_STATE();
    case 112:
      if (lookahead == 'e') ADVANCE(139);
      END_STATE();
    case 113:
      if (lookahead == 'n') ADVANCE(140);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(anon_sym_test);
      END_STATE();
    case 115:
      ACCEPT_TOKEN(anon_sym_then);
      END_STATE();
    case 116:
      ACCEPT_TOKEN(anon_sym_true);
      END_STATE();
    case 117:
      if (lookahead == 'e') ADVANCE(141);
      END_STATE();
    case 118:
      ACCEPT_TOKEN(anon_sym_type);
      END_STATE();
    case 119:
      if (lookahead == 'n') ADVANCE(142);
      END_STATE();
    case 120:
      if (lookahead == 'e') ADVANCE(143);
      END_STATE();
    case 121:
      if (lookahead == 'a') ADVANCE(144);
      END_STATE();
    case 122:
      ACCEPT_TOKEN(anon_sym_array);
      END_STATE();
    case 123:
      if (lookahead == 'r') ADVANCE(145);
      END_STATE();
    case 124:
      if (lookahead == 'f') ADVANCE(146);
      END_STATE();
    case 125:
      if (lookahead == 's') ADVANCE(147);
      END_STATE();
    case 126:
      ACCEPT_TOKEN(anon_sym_endif);
      END_STATE();
    case 127:
      ACCEPT_TOKEN(anon_sym_false);
      END_STATE();
    case 128:
      ACCEPT_TOKEN(anon_sym_float);
      END_STATE();
    case 129:
      if (lookahead == 'i') ADVANCE(148);
      END_STATE();
    case 130:
      if (lookahead == 'd') ADVANCE(149);
      END_STATE();
    case 131:
      if (lookahead == 'i') ADVANCE(150);
      END_STATE();
    case 132:
      if (lookahead == 'a') ADVANCE(151);
      END_STATE();
    case 133:
      if (lookahead == 'i') ADVANCE(152);
      END_STATE();
    case 134:
      if (lookahead == 'i') ADVANCE(153);
      END_STATE();
    case 135:
      if (lookahead == 't') ADVANCE(154);
      END_STATE();
    case 136:
      if (lookahead == 'c') ADVANCE(155);
      END_STATE();
    case 137:
      if (lookahead == 'd') ADVANCE(156);
      END_STATE();
    case 138:
      if (lookahead == 'f') ADVANCE(157);
      END_STATE();
    case 139:
      ACCEPT_TOKEN(anon_sym_solve);
      END_STATE();
    case 140:
      if (lookahead == 'g') ADVANCE(158);
      END_STATE();
    case 141:
      ACCEPT_TOKEN(anon_sym_tuple);
      END_STATE();
    case 142:
      ACCEPT_TOKEN(anon_sym_union);
      END_STATE();
    case 143:
      ACCEPT_TOKEN(anon_sym_where);
      END_STATE();
    case 144:
      if (lookahead == 't') ADVANCE(159);
      END_STATE();
    case 145:
      if (lookahead == 'a') ADVANCE(160);
      END_STATE();
    case 146:
      ACCEPT_TOKEN(anon_sym_elseif);
      END_STATE();
    case 147:
      if (lookahead == 'e') ADVANCE(161);
      END_STATE();
    case 148:
      if (lookahead == 'o') ADVANCE(162);
      END_STATE();
    case 149:
      if (lookahead == 'e') ADVANCE(163);
      END_STATE();
    case 150:
      if (lookahead == 't') ADVANCE(164);
      END_STATE();
    case 151:
      ACCEPT_TOKEN(anon_sym_lambda);
      END_STATE();
    case 152:
      if (lookahead == 'z') ADVANCE(165);
      END_STATE();
    case 153:
      if (lookahead == 'z') ADVANCE(166);
      END_STATE();
    case 154:
      ACCEPT_TOKEN(anon_sym_output);
      END_STATE();
    case 155:
      if (lookahead == 'a') ADVANCE(167);
      END_STATE();
    case 156:
      ACCEPT_TOKEN(anon_sym_record);
      END_STATE();
    case 157:
      if (lookahead == 'y') ADVANCE(168);
      END_STATE();
    case 158:
      ACCEPT_TOKEN(anon_sym_string);
      END_STATE();
    case 159:
      if (lookahead == 'i') ADVANCE(169);
      END_STATE();
    case 160:
      if (lookahead == 'i') ADVANCE(170);
      END_STATE();
    case 161:
      ACCEPT_TOKEN(anon_sym_endcase);
      END_STATE();
    case 162:
      if (lookahead == 'n') ADVANCE(171);
      END_STATE();
    case 163:
      ACCEPT_TOKEN(anon_sym_include);
      END_STATE();
    case 164:
      if (lookahead == 'y') ADVANCE(172);
      END_STATE();
    case 165:
      if (lookahead == 'e') ADVANCE(173);
      END_STATE();
    case 166:
      if (lookahead == 'e') ADVANCE(174);
      END_STATE();
    case 167:
      if (lookahead == 't') ADVANCE(175);
      END_STATE();
    case 168:
      ACCEPT_TOKEN(anon_sym_satisfy);
      END_STATE();
    case 169:
      if (lookahead == 'o') ADVANCE(176);
      END_STATE();
    case 170:
      if (lookahead == 'n') ADVANCE(177);
      END_STATE();
    case 171:
      ACCEPT_TOKEN(anon_sym_function);
      END_STATE();
    case 172:
      ACCEPT_TOKEN(anon_sym_infinity);
      END_STATE();
    case 173:
      ACCEPT_TOKEN(anon_sym_maximize);
      END_STATE();
    case 174:
      ACCEPT_TOKEN(anon_sym_minimize);
      END_STATE();
    case 175:
      if (lookahead == 'e') ADVANCE(178);
      END_STATE();
    case 176:
      if (lookahead == 'n') ADVANCE(179);
      END_STATE();
    case 177:
      if (lookahead == 't') ADVANCE(180);
      END_STATE();
    case 178:
      ACCEPT_TOKEN(anon_sym_predicate);
      END_STATE();
    case 179:
      ACCEPT_TOKEN(anon_sym_annotation);
      END_STATE();
    case 180:
      ACCEPT_TOKEN(anon_sym_constraint);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 40},
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
  [39] = {.lex_state = 40},
  [40] = {.lex_state = 1},
  [41] = {.lex_state = 1},
  [42] = {.lex_state = 1},
  [43] = {.lex_state = 40},
  [44] = {.lex_state = 40},
  [45] = {.lex_state = 40},
  [46] = {.lex_state = 40},
  [47] = {.lex_state = 40},
  [48] = {.lex_state = 40},
  [49] = {.lex_state = 40},
  [50] = {.lex_state = 40},
  [51] = {.lex_state = 40},
  [52] = {.lex_state = 40},
  [53] = {.lex_state = 40},
  [54] = {.lex_state = 40},
  [55] = {.lex_state = 40},
  [56] = {.lex_state = 40},
  [57] = {.lex_state = 40},
  [58] = {.lex_state = 40},
  [59] = {.lex_state = 40},
  [60] = {.lex_state = 40},
  [61] = {.lex_state = 40},
  [62] = {.lex_state = 40},
  [63] = {.lex_state = 40},
  [64] = {.lex_state = 40},
  [65] = {.lex_state = 40},
  [66] = {.lex_state = 40},
  [67] = {.lex_state = 40},
  [68] = {.lex_state = 40},
  [69] = {.lex_state = 40},
  [70] = {.lex_state = 40},
  [71] = {.lex_state = 40},
  [72] = {.lex_state = 40},
  [73] = {.lex_state = 40},
  [74] = {.lex_state = 40},
  [75] = {.lex_state = 40},
  [76] = {.lex_state = 40},
  [77] = {.lex_state = 40},
  [78] = {.lex_state = 40},
  [79] = {.lex_state = 40},
  [80] = {.lex_state = 40},
  [81] = {.lex_state = 1},
  [82] = {.lex_state = 1},
  [83] = {.lex_state = 1},
  [84] = {.lex_state = 40},
  [85] = {.lex_state = 40},
  [86] = {.lex_state = 40},
  [87] = {.lex_state = 40},
  [88] = {.lex_state = 40},
  [89] = {.lex_state = 40},
  [90] = {.lex_state = 40},
  [91] = {.lex_state = 40},
  [92] = {.lex_state = 40},
  [93] = {.lex_state = 40},
  [94] = {.lex_state = 40},
  [95] = {.lex_state = 40},
  [96] = {.lex_state = 40},
  [97] = {.lex_state = 40},
  [98] = {.lex_state = 40},
  [99] = {.lex_state = 40},
  [100] = {.lex_state = 40},
  [101] = {.lex_state = 40},
  [102] = {.lex_state = 40},
  [103] = {.lex_state = 40},
  [104] = {.lex_state = 40},
  [105] = {.lex_state = 40},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 0},
  [108] = {.lex_state = 40},
  [109] = {.lex_state = 40},
  [110] = {.lex_state = 0},
  [111] = {.lex_state = 0},
  [112] = {.lex_state = 0},
  [113] = {.lex_state = 40},
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
  [126] = {.lex_state = 0},
  [127] = {.lex_state = 40},
  [128] = {.lex_state = 5},
  [129] = {.lex_state = 6},
  [130] = {.lex_state = 7},
  [131] = {.lex_state = 0},
  [132] = {.lex_state = 0},
  [133] = {.lex_state = 0},
  [134] = {.lex_state = 8},
  [135] = {.lex_state = 0},
  [136] = {.lex_state = 40},
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
    [anon_sym_] = ACTIONS(1),
    [anon_sym_case] = ACTIONS(1),
    [anon_sym_of] = ACTIONS(1),
    [anon_sym_endcase] = ACTIONS(1),
    [anon_sym_EQ_GT] = ACTIONS(1),
    [anon_sym_lambda] = ACTIONS(1),
    [anon_sym_let] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_not] = ACTIONS(1),
    [anon_sym_2] = ACTIONS(1),
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
    [anon_sym_infinity] = ACTIONS(1),
    [anon_sym_3] = ACTIONS(1),
    [anon_sym_4] = ACTIONS(1),
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
    [sym_source_file] = STATE(133),
    [sym_assignment] = STATE(126),
    [sym__identifier] = STATE(136),
    [aux_sym_source_file_repeat1] = STATE(91),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_identifier] = ACTIONS(7),
    [sym_quoted_identifier] = ACTIONS(9),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 21,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(19), 1,
      anon_sym_PIPE,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(27), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(7), 1,
      aux_sym_array_literal_2d_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(104), 1,
      sym__expression,
    STATE(118), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [76] = 20,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(41), 1,
      anon_sym_RBRACK,
    STATE(9), 1,
      aux_sym_array_literal_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(89), 1,
      sym__expression,
    STATE(121), 1,
      sym_array_literal_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [149] = 19,
    ACTIONS(43), 1,
      sym_identifier,
    ACTIONS(46), 1,
      anon_sym_LPAREN,
    ACTIONS(51), 1,
      anon_sym_LBRACE,
    ACTIONS(54), 1,
      anon_sym_LBRACK,
    ACTIONS(57), 1,
      anon_sym_DQUOTE,
    ACTIONS(60), 1,
      sym_absent,
    ACTIONS(63), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(72), 1,
      anon_sym_infinity,
    ACTIONS(75), 1,
      anon_sym_3,
    ACTIONS(78), 1,
      anon_sym_4,
    ACTIONS(81), 1,
      sym_quoted_identifier,
    STATE(4), 1,
      aux_sym_set_literal_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(103), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(49), 2,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
    ACTIONS(66), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(69), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [220] = 20,
    ACTIONS(84), 1,
      sym_identifier,
    ACTIONS(87), 1,
      anon_sym_LPAREN,
    ACTIONS(90), 1,
      anon_sym_LBRACE,
    ACTIONS(93), 1,
      anon_sym_LBRACK,
    ACTIONS(96), 1,
      anon_sym_RBRACK,
    ACTIONS(98), 1,
      anon_sym_DQUOTE,
    ACTIONS(101), 1,
      sym_absent,
    ACTIONS(104), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(113), 1,
      anon_sym_infinity,
    ACTIONS(116), 1,
      anon_sym_3,
    ACTIONS(119), 1,
      anon_sym_4,
    ACTIONS(122), 1,
      sym_quoted_identifier,
    STATE(5), 1,
      aux_sym_array_literal_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(89), 1,
      sym__expression,
    STATE(132), 1,
      sym_array_literal_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(107), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(110), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [293] = 22,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      sym_float_literal,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(125), 1,
      sym_identifier,
    ACTIONS(127), 1,
      anon_sym_RPAREN,
    ACTIONS(129), 1,
      sym_integer_literal,
    ACTIONS(131), 1,
      sym_quoted_identifier,
    STATE(8), 1,
      aux_sym_call_repeat1,
    STATE(87), 1,
      sym__identifier,
    STATE(109), 1,
      sym__expression,
    STATE(119), 1,
      sym__call_arg,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(97), 3,
      sym_call,
      sym_infix_operator,
      sym_set_literal,
    STATE(71), 7,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [370] = 20,
    ACTIONS(133), 1,
      sym_identifier,
    ACTIONS(136), 1,
      anon_sym_LPAREN,
    ACTIONS(139), 1,
      anon_sym_LBRACE,
    ACTIONS(142), 1,
      anon_sym_LBRACK,
    ACTIONS(145), 1,
      anon_sym_PIPE,
    ACTIONS(147), 1,
      anon_sym_DQUOTE,
    ACTIONS(150), 1,
      sym_absent,
    ACTIONS(153), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(156), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(164), 1,
      anon_sym_infinity,
    ACTIONS(167), 1,
      anon_sym_3,
    ACTIONS(170), 1,
      anon_sym_4,
    ACTIONS(173), 1,
      sym_quoted_identifier,
    STATE(7), 1,
      aux_sym_array_literal_2d_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(104), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(158), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(161), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [443] = 22,
    ACTIONS(176), 1,
      sym_identifier,
    ACTIONS(179), 1,
      anon_sym_LPAREN,
    ACTIONS(182), 1,
      anon_sym_RPAREN,
    ACTIONS(184), 1,
      anon_sym_LBRACE,
    ACTIONS(187), 1,
      anon_sym_LBRACK,
    ACTIONS(190), 1,
      anon_sym_DQUOTE,
    ACTIONS(193), 1,
      sym_absent,
    ACTIONS(196), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(202), 1,
      sym_float_literal,
    ACTIONS(205), 1,
      sym_integer_literal,
    ACTIONS(208), 1,
      anon_sym_infinity,
    ACTIONS(211), 1,
      anon_sym_3,
    ACTIONS(214), 1,
      anon_sym_4,
    ACTIONS(217), 1,
      sym_quoted_identifier,
    STATE(8), 1,
      aux_sym_call_repeat1,
    STATE(87), 1,
      sym__identifier,
    STATE(109), 1,
      sym__expression,
    STATE(135), 1,
      sym__call_arg,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(199), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(97), 3,
      sym_call,
      sym_infix_operator,
      sym_set_literal,
    STATE(71), 7,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [520] = 20,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(220), 1,
      anon_sym_RBRACK,
    STATE(5), 1,
      aux_sym_array_literal_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(89), 1,
      sym__expression,
    STATE(122), 1,
      sym_array_literal_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [593] = 20,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(222), 1,
      anon_sym_PIPE_RBRACK,
    STATE(2), 1,
      aux_sym_array_literal_2d_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(85), 1,
      sym__expression,
    STATE(111), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [666] = 22,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(31), 1,
      sym_float_literal,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(125), 1,
      sym_identifier,
    ACTIONS(129), 1,
      sym_integer_literal,
    ACTIONS(131), 1,
      sym_quoted_identifier,
    ACTIONS(224), 1,
      anon_sym_RPAREN,
    STATE(6), 1,
      aux_sym_call_repeat1,
    STATE(87), 1,
      sym__identifier,
    STATE(109), 1,
      sym__expression,
    STATE(125), 1,
      sym__call_arg,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(97), 3,
      sym_call,
      sym_infix_operator,
      sym_set_literal,
    STATE(71), 7,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [743] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(226), 1,
      anon_sym_PIPE_RBRACK,
    STATE(39), 1,
      sym__identifier,
    STATE(84), 1,
      sym__expression,
    STATE(123), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [813] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(228), 1,
      anon_sym_PIPE,
    ACTIONS(230), 1,
      anon_sym_PIPE_RBRACK,
    STATE(39), 1,
      sym__identifier,
    STATE(90), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [883] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(232), 1,
      anon_sym_PIPE_RBRACK,
    STATE(39), 1,
      sym__identifier,
    STATE(84), 1,
      sym__expression,
    STATE(123), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [953] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(234), 1,
      anon_sym_RBRACE,
    STATE(23), 1,
      aux_sym_set_literal_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(96), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1023] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(236), 1,
      anon_sym_PIPE,
    ACTIONS(238), 1,
      anon_sym_PIPE_RBRACK,
    STATE(39), 1,
      sym__identifier,
    STATE(90), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1093] = 19,
    ACTIONS(240), 1,
      sym_identifier,
    ACTIONS(243), 1,
      anon_sym_LPAREN,
    ACTIONS(246), 1,
      anon_sym_LBRACE,
    ACTIONS(249), 1,
      anon_sym_LBRACK,
    ACTIONS(252), 1,
      anon_sym_PIPE,
    ACTIONS(254), 1,
      anon_sym_DQUOTE,
    ACTIONS(257), 1,
      sym_absent,
    ACTIONS(260), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(263), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(271), 1,
      anon_sym_infinity,
    ACTIONS(274), 1,
      anon_sym_3,
    ACTIONS(277), 1,
      anon_sym_4,
    ACTIONS(280), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(86), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(265), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(268), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1163] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(283), 1,
      anon_sym_RPAREN,
    STATE(24), 1,
      aux_sym_set_literal_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(95), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1233] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(285), 1,
      anon_sym_PIPE_RBRACK,
    STATE(39), 1,
      sym__identifier,
    STATE(84), 1,
      sym__expression,
    STATE(123), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1303] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(287), 1,
      anon_sym_PIPE,
    ACTIONS(289), 1,
      anon_sym_PIPE_RBRACK,
    STATE(39), 1,
      sym__identifier,
    STATE(90), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1373] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(291), 1,
      anon_sym_PIPE,
    ACTIONS(293), 1,
      anon_sym_PIPE_RBRACK,
    STATE(39), 1,
      sym__identifier,
    STATE(90), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1443] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(295), 1,
      anon_sym_PIPE_RBRACK,
    STATE(39), 1,
      sym__identifier,
    STATE(84), 1,
      sym__expression,
    STATE(123), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1513] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(297), 1,
      anon_sym_RBRACE,
    STATE(4), 1,
      aux_sym_set_literal_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(100), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1583] = 19,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(299), 1,
      anon_sym_RPAREN,
    STATE(4), 1,
      aux_sym_set_literal_repeat1,
    STATE(39), 1,
      sym__identifier,
    STATE(94), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1653] = 18,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(301), 1,
      sym_identifier,
    ACTIONS(303), 1,
      sym_quoted_identifier,
    STATE(88), 1,
      sym__identifier,
    STATE(105), 1,
      sym__expression,
    STATE(116), 1,
      sym_record_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1720] = 18,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(84), 1,
      sym__expression,
    STATE(123), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1787] = 17,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(99), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1851] = 17,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(98), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1915] = 17,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(90), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1979] = 17,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(86), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [2043] = 17,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(76), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [2107] = 17,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(75), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [2171] = 17,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(74), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [2235] = 17,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(13), 1,
      anon_sym_LPAREN,
    ACTIONS(15), 1,
      anon_sym_LBRACE,
    ACTIONS(17), 1,
      anon_sym_LBRACK,
    ACTIONS(21), 1,
      anon_sym_DQUOTE,
    ACTIONS(23), 1,
      sym_absent,
    ACTIONS(25), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(33), 1,
      anon_sym_infinity,
    ACTIONS(35), 1,
      anon_sym_3,
    ACTIONS(37), 1,
      anon_sym_4,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(39), 1,
      sym__identifier,
    STATE(93), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(31), 2,
      sym_float_literal,
      sym_integer_literal,
    STATE(71), 10,
      sym_call,
      sym_infix_operator,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [2299] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(305), 8,
      anon_sym_LBRACK,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      anon_sym_infinity,
      anon_sym_4,
      sym_identifier,
    ACTIONS(307), 9,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      anon_sym_3,
      sym_quoted_identifier,
  [2325] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(263), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      anon_sym_PIPE_RBRACK,
      anon_sym_3,
      sym_quoted_identifier,
    ACTIONS(252), 9,
      anon_sym_LBRACK,
      anon_sym_PIPE,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      anon_sym_infinity,
      anon_sym_4,
      sym_identifier,
  [2351] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(309), 8,
      anon_sym_LBRACK,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      anon_sym_infinity,
      anon_sym_4,
      sym_identifier,
    ACTIONS(311), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACK,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      anon_sym_3,
      sym_quoted_identifier,
  [2376] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(313), 8,
      anon_sym_LBRACK,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      anon_sym_infinity,
      anon_sym_4,
      sym_identifier,
    ACTIONS(315), 8,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_LBRACE,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      anon_sym_3,
      sym_quoted_identifier,
  [2401] = 4,
    ACTIONS(319), 1,
      anon_sym_LPAREN,
    ACTIONS(321), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(317), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2426] = 10,
    ACTIONS(323), 1,
      anon_sym_DQUOTE,
    ACTIONS(325), 1,
      sym_string_characters,
    ACTIONS(331), 1,
      anon_sym_BSLASH,
    ACTIONS(334), 1,
      anon_sym_BSLASHx,
    ACTIONS(337), 1,
      anon_sym_BSLASHu,
    ACTIONS(340), 1,
      anon_sym_BSLASHU,
    STATE(40), 1,
      aux_sym__string_content,
    STATE(82), 1,
      sym_escape_sequence,
    ACTIONS(343), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(328), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [2463] = 10,
    ACTIONS(345), 1,
      anon_sym_DQUOTE,
    ACTIONS(347), 1,
      sym_string_characters,
    ACTIONS(351), 1,
      anon_sym_BSLASH,
    ACTIONS(353), 1,
      anon_sym_BSLASHx,
    ACTIONS(355), 1,
      anon_sym_BSLASHu,
    ACTIONS(357), 1,
      anon_sym_BSLASHU,
    STATE(42), 1,
      aux_sym__string_content,
    STATE(82), 1,
      sym_escape_sequence,
    ACTIONS(343), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(349), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [2500] = 10,
    ACTIONS(347), 1,
      sym_string_characters,
    ACTIONS(351), 1,
      anon_sym_BSLASH,
    ACTIONS(353), 1,
      anon_sym_BSLASHx,
    ACTIONS(355), 1,
      anon_sym_BSLASHu,
    ACTIONS(357), 1,
      anon_sym_BSLASHU,
    ACTIONS(359), 1,
      anon_sym_DQUOTE,
    STATE(40), 1,
      aux_sym__string_content,
    STATE(82), 1,
      sym_escape_sequence,
    ACTIONS(343), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(349), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [2537] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2559] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2581] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2603] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2625] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2647] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2669] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2691] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2713] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2735] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2757] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2779] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2801] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2823] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2845] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2867] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2889] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2911] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2933] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2955] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2977] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [2999] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3021] = 3,
    ACTIONS(451), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(449), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3043] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3065] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3087] = 3,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3109] = 3,
    ACTIONS(467), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(465), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3131] = 3,
    ACTIONS(471), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(469), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3153] = 3,
    ACTIONS(321), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(317), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3175] = 3,
    ACTIONS(475), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(473), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3197] = 3,
    ACTIONS(479), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(477), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3219] = 3,
    ACTIONS(483), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(481), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3241] = 4,
    ACTIONS(483), 1,
      anon_sym_PIPE,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(481), 11,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3265] = 5,
    ACTIONS(483), 1,
      anon_sym_PIPE,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(481), 10,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_union,
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3291] = 3,
    ACTIONS(491), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(489), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3313] = 3,
    ACTIONS(495), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(493), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3335] = 3,
    ACTIONS(499), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(497), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3357] = 3,
    ACTIONS(503), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(501), 12,
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
      anon_sym_,
      anon_sym_PIPE_RBRACK,
  [3379] = 3,
    ACTIONS(507), 1,
      sym_string_characters,
    ACTIONS(343), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(505), 11,
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
  [3400] = 3,
    ACTIONS(511), 1,
      sym_string_characters,
    ACTIONS(343), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(509), 11,
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
  [3421] = 3,
    ACTIONS(515), 1,
      sym_string_characters,
    ACTIONS(343), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(513), 11,
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
  [3442] = 9,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(517), 1,
      anon_sym_COLON,
    ACTIONS(519), 1,
      anon_sym_COMMA,
    ACTIONS(521), 1,
      anon_sym_PIPE,
    ACTIONS(525), 1,
      anon_sym_PIPE_RBRACK,
    STATE(110), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3472] = 9,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(519), 1,
      anon_sym_COMMA,
    ACTIONS(521), 1,
      anon_sym_PIPE,
    ACTIONS(525), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(527), 1,
      anon_sym_COLON,
    STATE(110), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3502] = 8,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(529), 1,
      anon_sym_COMMA,
    ACTIONS(531), 1,
      anon_sym_PIPE,
    ACTIONS(533), 1,
      anon_sym_PIPE_RBRACK,
    STATE(106), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3529] = 4,
    ACTIONS(319), 1,
      anon_sym_LPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(535), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
    ACTIONS(317), 4,
      anon_sym_PLUS_PLUS,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_,
  [3547] = 4,
    ACTIONS(319), 1,
      anon_sym_LPAREN,
    ACTIONS(537), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(317), 5,
      anon_sym_PLUS_PLUS,
      anon_sym_COMMA,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_,
  [3565] = 6,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(539), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
    ACTIONS(541), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3587] = 6,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(545), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
    ACTIONS(543), 2,
      anon_sym_COMMA,
      anon_sym_PIPE_RBRACK,
  [3609] = 7,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(9), 1,
      sym_quoted_identifier,
    ACTIONS(547), 1,
      ts_builtin_sym_end,
    STATE(92), 1,
      aux_sym_source_file_repeat1,
    STATE(124), 1,
      sym_assignment,
    STATE(136), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3632] = 7,
    ACTIONS(549), 1,
      ts_builtin_sym_end,
    ACTIONS(551), 1,
      sym_identifier,
    ACTIONS(554), 1,
      sym_quoted_identifier,
    STATE(92), 1,
      aux_sym_source_file_repeat1,
    STATE(131), 1,
      sym_assignment,
    STATE(136), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3655] = 5,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
    ACTIONS(557), 2,
      ts_builtin_sym_end,
      anon_sym_SEMI,
  [3674] = 6,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(559), 1,
      anon_sym_COMMA,
    ACTIONS(561), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3695] = 6,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(559), 1,
      anon_sym_COMMA,
    ACTIONS(563), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3716] = 6,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(559), 1,
      anon_sym_COMMA,
    ACTIONS(565), 1,
      anon_sym_RBRACE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3737] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(535), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
    ACTIONS(317), 4,
      anon_sym_PLUS_PLUS,
      anon_sym_DOT_DOT,
      anon_sym_union,
      anon_sym_,
  [3752] = 5,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
    ACTIONS(567), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [3771] = 5,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
    ACTIONS(569), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3790] = 6,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(559), 1,
      anon_sym_COMMA,
    ACTIONS(571), 1,
      anon_sym_RBRACE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3811] = 6,
    ACTIONS(573), 1,
      sym_identifier,
    ACTIONS(575), 1,
      anon_sym_RPAREN,
    ACTIONS(577), 1,
      sym_quoted_identifier,
    STATE(120), 1,
      sym_record_member,
    STATE(127), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3831] = 6,
    ACTIONS(573), 1,
      sym_identifier,
    ACTIONS(577), 1,
      sym_quoted_identifier,
    ACTIONS(579), 1,
      anon_sym_RPAREN,
    STATE(120), 1,
      sym_record_member,
    STATE(127), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3851] = 5,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(559), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3869] = 5,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(581), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3887] = 5,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(583), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3905] = 5,
    ACTIONS(585), 1,
      anon_sym_COMMA,
    ACTIONS(587), 1,
      anon_sym_PIPE,
    ACTIONS(589), 1,
      anon_sym_PIPE_RBRACK,
    STATE(107), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3922] = 5,
    ACTIONS(591), 1,
      anon_sym_COMMA,
    ACTIONS(594), 1,
      anon_sym_PIPE,
    ACTIONS(596), 1,
      anon_sym_PIPE_RBRACK,
    STATE(107), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3939] = 5,
    ACTIONS(573), 1,
      sym_identifier,
    ACTIONS(577), 1,
      sym_quoted_identifier,
    STATE(120), 1,
      sym_record_member,
    STATE(127), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3956] = 4,
    ACTIONS(485), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(487), 1,
      anon_sym_DOT_DOT,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(523), 2,
      anon_sym_union,
      anon_sym_,
  [3971] = 5,
    ACTIONS(598), 1,
      anon_sym_COMMA,
    ACTIONS(600), 1,
      anon_sym_PIPE,
    ACTIONS(602), 1,
      anon_sym_PIPE_RBRACK,
    STATE(107), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [3988] = 4,
    ACTIONS(604), 1,
      anon_sym_PIPE,
    ACTIONS(606), 1,
      anon_sym_PIPE_RBRACK,
    STATE(112), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4002] = 4,
    ACTIONS(608), 1,
      anon_sym_PIPE,
    ACTIONS(610), 1,
      anon_sym_PIPE_RBRACK,
    STATE(117), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4016] = 3,
    ACTIONS(614), 1,
      sym_identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(612), 2,
      ts_builtin_sym_end,
      sym_quoted_identifier,
  [4028] = 4,
    ACTIONS(616), 1,
      anon_sym_COMMA,
    ACTIONS(618), 1,
      anon_sym_RPAREN,
    STATE(115), 1,
      aux_sym_record_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4042] = 4,
    ACTIONS(620), 1,
      anon_sym_COMMA,
    ACTIONS(623), 1,
      anon_sym_RPAREN,
    STATE(115), 1,
      aux_sym_record_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4056] = 4,
    ACTIONS(625), 1,
      anon_sym_COMMA,
    ACTIONS(627), 1,
      anon_sym_RPAREN,
    STATE(114), 1,
      aux_sym_record_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4070] = 4,
    ACTIONS(629), 1,
      anon_sym_PIPE,
    ACTIONS(632), 1,
      anon_sym_PIPE_RBRACK,
    STATE(117), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4084] = 4,
    ACTIONS(634), 1,
      anon_sym_PIPE,
    ACTIONS(636), 1,
      anon_sym_PIPE_RBRACK,
    STATE(117), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4098] = 3,
    ACTIONS(638), 1,
      anon_sym_COMMA,
    ACTIONS(640), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4109] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(642), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [4118] = 3,
    ACTIONS(644), 1,
      anon_sym_COMMA,
    ACTIONS(646), 1,
      anon_sym_RBRACK,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4129] = 3,
    ACTIONS(644), 1,
      anon_sym_COMMA,
    ACTIONS(648), 1,
      anon_sym_RBRACK,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4140] = 3,
    ACTIONS(650), 1,
      anon_sym_PIPE,
    ACTIONS(652), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4151] = 3,
    ACTIONS(654), 1,
      ts_builtin_sym_end,
    ACTIONS(656), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4162] = 3,
    ACTIONS(638), 1,
      anon_sym_COMMA,
    ACTIONS(658), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4173] = 3,
    ACTIONS(656), 1,
      anon_sym_SEMI,
    ACTIONS(660), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4184] = 2,
    ACTIONS(537), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4192] = 2,
    ACTIONS(662), 1,
      aux_sym_escape_sequence_token1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4200] = 2,
    ACTIONS(662), 1,
      aux_sym_escape_sequence_token2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4208] = 2,
    ACTIONS(662), 1,
      aux_sym_escape_sequence_token3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4216] = 2,
    ACTIONS(656), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4224] = 2,
    ACTIONS(644), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4232] = 2,
    ACTIONS(664), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4240] = 2,
    ACTIONS(662), 1,
      aux_sym_escape_sequence_token4,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4248] = 2,
    ACTIONS(638), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [4256] = 2,
    ACTIONS(666), 1,
      anon_sym_EQ,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 76,
  [SMALL_STATE(4)] = 149,
  [SMALL_STATE(5)] = 220,
  [SMALL_STATE(6)] = 293,
  [SMALL_STATE(7)] = 370,
  [SMALL_STATE(8)] = 443,
  [SMALL_STATE(9)] = 520,
  [SMALL_STATE(10)] = 593,
  [SMALL_STATE(11)] = 666,
  [SMALL_STATE(12)] = 743,
  [SMALL_STATE(13)] = 813,
  [SMALL_STATE(14)] = 883,
  [SMALL_STATE(15)] = 953,
  [SMALL_STATE(16)] = 1023,
  [SMALL_STATE(17)] = 1093,
  [SMALL_STATE(18)] = 1163,
  [SMALL_STATE(19)] = 1233,
  [SMALL_STATE(20)] = 1303,
  [SMALL_STATE(21)] = 1373,
  [SMALL_STATE(22)] = 1443,
  [SMALL_STATE(23)] = 1513,
  [SMALL_STATE(24)] = 1583,
  [SMALL_STATE(25)] = 1653,
  [SMALL_STATE(26)] = 1720,
  [SMALL_STATE(27)] = 1787,
  [SMALL_STATE(28)] = 1851,
  [SMALL_STATE(29)] = 1915,
  [SMALL_STATE(30)] = 1979,
  [SMALL_STATE(31)] = 2043,
  [SMALL_STATE(32)] = 2107,
  [SMALL_STATE(33)] = 2171,
  [SMALL_STATE(34)] = 2235,
  [SMALL_STATE(35)] = 2299,
  [SMALL_STATE(36)] = 2325,
  [SMALL_STATE(37)] = 2351,
  [SMALL_STATE(38)] = 2376,
  [SMALL_STATE(39)] = 2401,
  [SMALL_STATE(40)] = 2426,
  [SMALL_STATE(41)] = 2463,
  [SMALL_STATE(42)] = 2500,
  [SMALL_STATE(43)] = 2537,
  [SMALL_STATE(44)] = 2559,
  [SMALL_STATE(45)] = 2581,
  [SMALL_STATE(46)] = 2603,
  [SMALL_STATE(47)] = 2625,
  [SMALL_STATE(48)] = 2647,
  [SMALL_STATE(49)] = 2669,
  [SMALL_STATE(50)] = 2691,
  [SMALL_STATE(51)] = 2713,
  [SMALL_STATE(52)] = 2735,
  [SMALL_STATE(53)] = 2757,
  [SMALL_STATE(54)] = 2779,
  [SMALL_STATE(55)] = 2801,
  [SMALL_STATE(56)] = 2823,
  [SMALL_STATE(57)] = 2845,
  [SMALL_STATE(58)] = 2867,
  [SMALL_STATE(59)] = 2889,
  [SMALL_STATE(60)] = 2911,
  [SMALL_STATE(61)] = 2933,
  [SMALL_STATE(62)] = 2955,
  [SMALL_STATE(63)] = 2977,
  [SMALL_STATE(64)] = 2999,
  [SMALL_STATE(65)] = 3021,
  [SMALL_STATE(66)] = 3043,
  [SMALL_STATE(67)] = 3065,
  [SMALL_STATE(68)] = 3087,
  [SMALL_STATE(69)] = 3109,
  [SMALL_STATE(70)] = 3131,
  [SMALL_STATE(71)] = 3153,
  [SMALL_STATE(72)] = 3175,
  [SMALL_STATE(73)] = 3197,
  [SMALL_STATE(74)] = 3219,
  [SMALL_STATE(75)] = 3241,
  [SMALL_STATE(76)] = 3265,
  [SMALL_STATE(77)] = 3291,
  [SMALL_STATE(78)] = 3313,
  [SMALL_STATE(79)] = 3335,
  [SMALL_STATE(80)] = 3357,
  [SMALL_STATE(81)] = 3379,
  [SMALL_STATE(82)] = 3400,
  [SMALL_STATE(83)] = 3421,
  [SMALL_STATE(84)] = 3442,
  [SMALL_STATE(85)] = 3472,
  [SMALL_STATE(86)] = 3502,
  [SMALL_STATE(87)] = 3529,
  [SMALL_STATE(88)] = 3547,
  [SMALL_STATE(89)] = 3565,
  [SMALL_STATE(90)] = 3587,
  [SMALL_STATE(91)] = 3609,
  [SMALL_STATE(92)] = 3632,
  [SMALL_STATE(93)] = 3655,
  [SMALL_STATE(94)] = 3674,
  [SMALL_STATE(95)] = 3695,
  [SMALL_STATE(96)] = 3716,
  [SMALL_STATE(97)] = 3737,
  [SMALL_STATE(98)] = 3752,
  [SMALL_STATE(99)] = 3771,
  [SMALL_STATE(100)] = 3790,
  [SMALL_STATE(101)] = 3811,
  [SMALL_STATE(102)] = 3831,
  [SMALL_STATE(103)] = 3851,
  [SMALL_STATE(104)] = 3869,
  [SMALL_STATE(105)] = 3887,
  [SMALL_STATE(106)] = 3905,
  [SMALL_STATE(107)] = 3922,
  [SMALL_STATE(108)] = 3939,
  [SMALL_STATE(109)] = 3956,
  [SMALL_STATE(110)] = 3971,
  [SMALL_STATE(111)] = 3988,
  [SMALL_STATE(112)] = 4002,
  [SMALL_STATE(113)] = 4016,
  [SMALL_STATE(114)] = 4028,
  [SMALL_STATE(115)] = 4042,
  [SMALL_STATE(116)] = 4056,
  [SMALL_STATE(117)] = 4070,
  [SMALL_STATE(118)] = 4084,
  [SMALL_STATE(119)] = 4098,
  [SMALL_STATE(120)] = 4109,
  [SMALL_STATE(121)] = 4118,
  [SMALL_STATE(122)] = 4129,
  [SMALL_STATE(123)] = 4140,
  [SMALL_STATE(124)] = 4151,
  [SMALL_STATE(125)] = 4162,
  [SMALL_STATE(126)] = 4173,
  [SMALL_STATE(127)] = 4184,
  [SMALL_STATE(128)] = 4192,
  [SMALL_STATE(129)] = 4200,
  [SMALL_STATE(130)] = 4208,
  [SMALL_STATE(131)] = 4216,
  [SMALL_STATE(132)] = 4224,
  [SMALL_STATE(133)] = 4232,
  [SMALL_STATE(134)] = 4240,
  [SMALL_STATE(135)] = 4248,
  [SMALL_STATE(136)] = 4256,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(136),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(136),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(39),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [17] = {.entry = {.count = 1, .reusable = false}}, SHIFT(3),
  [19] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(69),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(71),
  [33] = {.entry = {.count = 1, .reusable = false}}, SHIFT(68),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(67),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [43] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(39),
  [46] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(25),
  [49] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12),
  [51] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(15),
  [54] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(3),
  [57] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(41),
  [60] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(71),
  [63] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(10),
  [66] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(69),
  [69] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(71),
  [72] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(68),
  [75] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(68),
  [78] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(67),
  [81] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(39),
  [84] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(39),
  [87] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(25),
  [90] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(15),
  [93] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(3),
  [96] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12),
  [98] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(41),
  [101] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(71),
  [104] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(10),
  [107] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(69),
  [110] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(71),
  [113] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(68),
  [116] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(68),
  [119] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(67),
  [122] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(39),
  [125] = {.entry = {.count = 1, .reusable = false}}, SHIFT(87),
  [127] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [129] = {.entry = {.count = 1, .reusable = false}}, SHIFT(97),
  [131] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [133] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(39),
  [136] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(25),
  [139] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(15),
  [142] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(3),
  [145] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20),
  [147] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(41),
  [150] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(71),
  [153] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(10),
  [156] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20),
  [158] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(69),
  [161] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(71),
  [164] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(68),
  [167] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(68),
  [170] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(67),
  [173] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(39),
  [176] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(87),
  [179] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(25),
  [182] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34),
  [184] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(15),
  [187] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(3),
  [190] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(41),
  [193] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(71),
  [196] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(10),
  [199] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(69),
  [202] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(71),
  [205] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(97),
  [208] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(68),
  [211] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(68),
  [214] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(67),
  [217] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 34), SHIFT_REPEAT(87),
  [220] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [222] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [224] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [226] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [228] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 5, .production_id = 37),
  [230] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 5, .production_id = 37),
  [232] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [234] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [236] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 2, .production_id = 9),
  [238] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 2, .production_id = 9),
  [240] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(39),
  [243] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(25),
  [246] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(15),
  [249] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(3),
  [252] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16),
  [254] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(41),
  [257] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(71),
  [260] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(10),
  [263] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16),
  [265] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(69),
  [268] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(71),
  [271] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(68),
  [274] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(68),
  [277] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(67),
  [280] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(39),
  [283] = {.entry = {.count = 1, .reusable = true}}, SHIFT(79),
  [285] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [287] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 3, .production_id = 17),
  [289] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 3, .production_id = 17),
  [291] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 4, .production_id = 27),
  [293] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 4, .production_id = 27),
  [295] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [297] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [299] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [301] = {.entry = {.count = 1, .reusable = false}}, SHIFT(88),
  [303] = {.entry = {.count = 1, .reusable = true}}, SHIFT(88),
  [305] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 9),
  [307] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 9),
  [309] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 9),
  [311] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 9),
  [313] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 31),
  [315] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2, .production_id = 31),
  [317] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__expression, 1),
  [319] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [321] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__expression, 1),
  [323] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15),
  [325] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(82),
  [328] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(81),
  [331] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(128),
  [334] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(129),
  [337] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(130),
  [340] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(134),
  [343] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [345] = {.entry = {.count = 1, .reusable = false}}, SHIFT(51),
  [347] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [349] = {.entry = {.count = 1, .reusable = false}}, SHIFT(81),
  [351] = {.entry = {.count = 1, .reusable = false}}, SHIFT(128),
  [353] = {.entry = {.count = 1, .reusable = false}}, SHIFT(129),
  [355] = {.entry = {.count = 1, .reusable = false}}, SHIFT(130),
  [357] = {.entry = {.count = 1, .reusable = false}}, SHIFT(134),
  [359] = {.entry = {.count = 1, .reusable = false}}, SHIFT(65),
  [361] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 2),
  [363] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 2),
  [365] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 6, .production_id = 39),
  [367] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 6, .production_id = 39),
  [369] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, .production_id = 28),
  [371] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, .production_id = 28),
  [373] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 4, .production_id = 10),
  [375] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 4, .production_id = 10),
  [377] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, .production_id = 19),
  [379] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, .production_id = 19),
  [381] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, .production_id = 30),
  [383] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, .production_id = 30),
  [385] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, .production_id = 18),
  [387] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, .production_id = 18),
  [389] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 3, .production_id = 10),
  [391] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 3, .production_id = 10),
  [393] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 2),
  [395] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 2),
  [397] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 4, .production_id = 32),
  [399] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 4, .production_id = 32),
  [401] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 4, .production_id = 33),
  [403] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 4, .production_id = 33),
  [405] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 3, .production_id = 10),
  [407] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 3, .production_id = 10),
  [409] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 3, .production_id = 11),
  [411] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 3, .production_id = 11),
  [413] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 4, .production_id = 25),
  [415] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 4, .production_id = 25),
  [417] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 2),
  [419] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 2),
  [421] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 2),
  [423] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 2),
  [425] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 5, .production_id = 35),
  [427] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 5, .production_id = 35),
  [429] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 3, .production_id = 10),
  [431] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 3, .production_id = 10),
  [433] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 3, .production_id = 11),
  [435] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 3, .production_id = 11),
  [437] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 4, .production_id = 25),
  [439] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 4, .production_id = 25),
  [441] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 5, .production_id = 36),
  [443] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 5, .production_id = 36),
  [445] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 4, .production_id = 23),
  [447] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 4, .production_id = 23),
  [449] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 3, .production_id = 14),
  [451] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 3, .production_id = 14),
  [453] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 5, .production_id = 23),
  [455] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 5, .production_id = 23),
  [457] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 1),
  [459] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 1),
  [461] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_infinity, 1),
  [463] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_infinity, 1),
  [465] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean_literal, 1),
  [467] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean_literal, 1),
  [469] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 3, .production_id = 18),
  [471] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 3, .production_id = 18),
  [473] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 3, .production_id = 19),
  [475] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 3, .production_id = 19),
  [477] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 5, .production_id = 28),
  [479] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 5, .production_id = 28),
  [481] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_infix_operator, 3, .production_id = 21),
  [483] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_infix_operator, 3, .production_id = 21),
  [485] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [487] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [489] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 3, .production_id = 22),
  [491] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 3, .production_id = 22),
  [493] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 5, .production_id = 30),
  [495] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 5, .production_id = 30),
  [497] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 4, .production_id = 10),
  [499] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 4, .production_id = 10),
  [501] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 5, .production_id = 38),
  [503] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 5, .production_id = 38),
  [505] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_escape_sequence, 1, .production_id = 8),
  [507] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_escape_sequence, 1, .production_id = 8),
  [509] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__string_content, 1, .production_id = 7),
  [511] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__string_content, 1, .production_id = 7),
  [513] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_escape_sequence, 2, .production_id = 13),
  [515] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_escape_sequence, 2, .production_id = 13),
  [517] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [519] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [521] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 1, .production_id = 9),
  [523] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [525] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 1, .production_id = 9),
  [527] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [529] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [531] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 3, .production_id = 27),
  [533] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 3, .production_id = 27),
  [535] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__call_arg, 1),
  [537] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [539] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [541] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_member, 1, .production_id = 6),
  [543] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 10),
  [545] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 10),
  [547] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, .production_id = 2),
  [549] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 4),
  [551] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 4), SHIFT_REPEAT(136),
  [554] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 4), SHIFT_REPEAT(136),
  [557] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 3, .production_id = 5),
  [559] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [561] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [563] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [565] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [567] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_member, 3, .production_id = 24),
  [569] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_member, 3, .production_id = 26),
  [571] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [573] = {.entry = {.count = 1, .reusable = false}}, SHIFT(127),
  [575] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [577] = {.entry = {.count = 1, .reusable = true}}, SHIFT(127),
  [579] = {.entry = {.count = 1, .reusable = true}}, SHIFT(66),
  [581] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [583] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [585] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [587] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 4, .production_id = 37),
  [589] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 4, .production_id = 37),
  [591] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 12), SHIFT_REPEAT(29),
  [594] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 12),
  [596] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 12),
  [598] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [600] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 2, .production_id = 17),
  [602] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 2, .production_id = 17),
  [604] = {.entry = {.count = 1, .reusable = false}}, SHIFT(14),
  [606] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [608] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [610] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [612] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 1),
  [614] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 1),
  [616] = {.entry = {.count = 1, .reusable = true}}, SHIFT(102),
  [618] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [620] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_record_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(108),
  [623] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_record_literal_repeat1, 2, .production_id = 12),
  [625] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [627] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [629] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, .production_id = 29), SHIFT_REPEAT(26),
  [632] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, .production_id = 29),
  [634] = {.entry = {.count = 1, .reusable = false}}, SHIFT(22),
  [636] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [638] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [640] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [642] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_record_literal_repeat1, 2, .production_id = 10),
  [644] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [646] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [648] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [650] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, .production_id = 18),
  [652] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, .production_id = 18),
  [654] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 2, .production_id = 3),
  [656] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [658] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [660] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, .production_id = 1),
  [662] = {.entry = {.count = 1, .reusable = true}}, SHIFT(83),
  [664] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [666] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef _WIN32
#define extern __declspec(dllexport)
#endif

extern const TSLanguage *tree_sitter_datazinc(void) {
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
