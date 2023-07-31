#include <tree_sitter/parser.h>

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 116
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 168
#define ALIAS_COUNT 0
#define TOKEN_COUNT 144
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 10
#define MAX_ALIAS_SEQUENCE_LENGTH 6
#define PRODUCTION_ID_COUNT 33

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
  anon_sym_LT_DASH_GT = 39,
  anon_sym_ = 40,
  anon_sym_2 = 41,
  anon_sym_DASH_GT = 42,
  anon_sym_3 = 43,
  anon_sym_4 = 44,
  anon_sym_LT_DASH = 45,
  anon_sym_5 = 46,
  anon_sym_6 = 47,
  anon_sym_7 = 48,
  anon_sym_xor = 49,
  anon_sym_8 = 50,
  anon_sym_9 = 51,
  anon_sym_EQ_EQ = 52,
  anon_sym_BANG_EQ = 53,
  anon_sym_10 = 54,
  anon_sym_LT = 55,
  anon_sym_LT_EQ = 56,
  anon_sym_11 = 57,
  anon_sym_GT = 58,
  anon_sym_GT_EQ = 59,
  anon_sym_12 = 60,
  anon_sym_13 = 61,
  anon_sym_subset = 62,
  anon_sym_14 = 63,
  anon_sym_superset = 64,
  anon_sym_15 = 65,
  anon_sym_TILDE_EQ = 66,
  anon_sym_TILDE_BANG_EQ = 67,
  anon_sym_union = 68,
  anon_sym_16 = 69,
  anon_sym_diff = 70,
  anon_sym_17 = 71,
  anon_sym_symdiff = 72,
  anon_sym_intersect = 73,
  anon_sym_18 = 74,
  anon_sym_PLUS = 75,
  anon_sym_DASH = 76,
  anon_sym_TILDE_PLUS = 77,
  anon_sym_TILDE_DASH = 78,
  anon_sym_STAR = 79,
  anon_sym_SLASH = 80,
  anon_sym_div = 81,
  anon_sym_mod = 82,
  anon_sym_TILDE_STAR = 83,
  anon_sym_TILDEdiv = 84,
  anon_sym_TILDE_SLASH = 85,
  anon_sym_CARET = 86,
  anon_sym_default = 87,
  anon_sym_case = 88,
  anon_sym_of = 89,
  anon_sym_endcase = 90,
  anon_sym_EQ_GT = 91,
  anon_sym_lambda = 92,
  anon_sym_let = 93,
  anon_sym_not = 94,
  anon_sym_19 = 95,
  anon_sym_DQUOTE = 96,
  anon_sym_BSLASH_LPAREN = 97,
  anon_sym_array = 98,
  anon_sym_var = 99,
  anon_sym_par = 100,
  anon_sym_opt = 101,
  anon_sym_set = 102,
  anon_sym_tuple = 103,
  anon_sym_record = 104,
  anon_sym_op = 105,
  anon_sym_any = 106,
  anon_sym_ann = 107,
  anon_sym_bool = 108,
  anon_sym_float = 109,
  anon_sym_int = 110,
  anon_sym_string = 111,
  sym_type_inst_id = 112,
  sym_type_inst_enum_id = 113,
  sym_absent = 114,
  sym_anonymous = 115,
  anon_sym_LBRACK_PIPE = 116,
  anon_sym_PIPE_RBRACK = 117,
  anon_sym_true = 118,
  anon_sym_false = 119,
  sym_float_literal = 120,
  sym_integer_literal = 121,
  anon_sym_infinity = 122,
  anon_sym_20 = 123,
  anon_sym_21 = 124,
  sym_string_characters = 125,
  anon_sym_BSLASH_SQUOTE = 126,
  anon_sym_BSLASH_DQUOTE = 127,
  anon_sym_BSLASH_BSLASH = 128,
  anon_sym_BSLASHr = 129,
  anon_sym_BSLASHn = 130,
  anon_sym_BSLASHt = 131,
  anon_sym_BSLASH = 132,
  aux_sym_escape_sequence_token1 = 133,
  anon_sym_BSLASHx = 134,
  aux_sym_escape_sequence_token2 = 135,
  anon_sym_BSLASHu = 136,
  aux_sym_escape_sequence_token3 = 137,
  anon_sym_BSLASHU = 138,
  aux_sym_escape_sequence_token4 = 139,
  sym_quoted_identifier = 140,
  anon_sym_CARET_DASH1 = 141,
  sym_line_comment = 142,
  sym_block_comment = 143,
  sym_source_file = 144,
  sym_assignment = 145,
  sym__expression = 146,
  sym_array_literal = 147,
  sym_array_literal_member = 148,
  sym_array_literal_2d = 149,
  sym_array_literal_2d_row = 150,
  sym_boolean_literal = 151,
  sym_infinity = 152,
  sym_set_literal = 153,
  sym_string_literal = 154,
  aux_sym__string_content = 155,
  sym_escape_sequence = 156,
  sym_tuple_literal = 157,
  sym_record_literal = 158,
  sym_record_member = 159,
  sym__identifier = 160,
  aux_sym_source_file_repeat1 = 161,
  aux_sym_array_literal_repeat1 = 162,
  aux_sym_array_literal_2d_repeat1 = 163,
  aux_sym_array_literal_2d_repeat2 = 164,
  aux_sym_array_literal_2d_row_repeat1 = 165,
  aux_sym_set_literal_repeat1 = 166,
  aux_sym_record_literal_repeat1 = 167,
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
  [anon_sym_LT_DASH_GT] = "<->",
  [anon_sym_] = "⟷",
  [anon_sym_2] = "⇔",
  [anon_sym_DASH_GT] = "->",
  [anon_sym_3] = "→",
  [anon_sym_4] = "⇒",
  [anon_sym_LT_DASH] = "<-",
  [anon_sym_5] = "←",
  [anon_sym_6] = "⇐",
  [anon_sym_7] = "∨",
  [anon_sym_xor] = "xor",
  [anon_sym_8] = "⊻",
  [anon_sym_9] = "∧",
  [anon_sym_EQ_EQ] = "==",
  [anon_sym_BANG_EQ] = "!=",
  [anon_sym_10] = "≠",
  [anon_sym_LT] = "<",
  [anon_sym_LT_EQ] = "<=",
  [anon_sym_11] = "≤",
  [anon_sym_GT] = ">",
  [anon_sym_GT_EQ] = ">=",
  [anon_sym_12] = "≥",
  [anon_sym_13] = "∈",
  [anon_sym_subset] = "subset",
  [anon_sym_14] = "⊆",
  [anon_sym_superset] = "superset",
  [anon_sym_15] = "⊇",
  [anon_sym_TILDE_EQ] = "~=",
  [anon_sym_TILDE_BANG_EQ] = "~!=",
  [anon_sym_union] = "union",
  [anon_sym_16] = "∪",
  [anon_sym_diff] = "diff",
  [anon_sym_17] = "∖",
  [anon_sym_symdiff] = "symdiff",
  [anon_sym_intersect] = "intersect",
  [anon_sym_18] = "∩",
  [anon_sym_PLUS] = "+",
  [anon_sym_DASH] = "-",
  [anon_sym_TILDE_PLUS] = "~+",
  [anon_sym_TILDE_DASH] = "~-",
  [anon_sym_STAR] = "*",
  [anon_sym_SLASH] = "/",
  [anon_sym_div] = "div",
  [anon_sym_mod] = "mod",
  [anon_sym_TILDE_STAR] = "~*",
  [anon_sym_TILDEdiv] = "~div",
  [anon_sym_TILDE_SLASH] = "~/",
  [anon_sym_CARET] = "^",
  [anon_sym_default] = "default",
  [anon_sym_case] = "case",
  [anon_sym_of] = "of",
  [anon_sym_endcase] = "endcase",
  [anon_sym_EQ_GT] = "=>",
  [anon_sym_lambda] = "lambda",
  [anon_sym_let] = "let",
  [anon_sym_not] = "not",
  [anon_sym_19] = "¬",
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
  [anon_sym_20] = "∞",
  [anon_sym_21] = "∅",
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
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
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
  [anon_sym_LT_DASH_GT] = anon_sym_LT_DASH_GT,
  [anon_sym_] = anon_sym_,
  [anon_sym_2] = anon_sym_2,
  [anon_sym_DASH_GT] = anon_sym_DASH_GT,
  [anon_sym_3] = anon_sym_3,
  [anon_sym_4] = anon_sym_4,
  [anon_sym_LT_DASH] = anon_sym_LT_DASH,
  [anon_sym_5] = anon_sym_5,
  [anon_sym_6] = anon_sym_6,
  [anon_sym_7] = anon_sym_7,
  [anon_sym_xor] = anon_sym_xor,
  [anon_sym_8] = anon_sym_8,
  [anon_sym_9] = anon_sym_9,
  [anon_sym_EQ_EQ] = anon_sym_EQ_EQ,
  [anon_sym_BANG_EQ] = anon_sym_BANG_EQ,
  [anon_sym_10] = anon_sym_10,
  [anon_sym_LT] = anon_sym_LT,
  [anon_sym_LT_EQ] = anon_sym_LT_EQ,
  [anon_sym_11] = anon_sym_11,
  [anon_sym_GT] = anon_sym_GT,
  [anon_sym_GT_EQ] = anon_sym_GT_EQ,
  [anon_sym_12] = anon_sym_12,
  [anon_sym_13] = anon_sym_13,
  [anon_sym_subset] = anon_sym_subset,
  [anon_sym_14] = anon_sym_14,
  [anon_sym_superset] = anon_sym_superset,
  [anon_sym_15] = anon_sym_15,
  [anon_sym_TILDE_EQ] = anon_sym_TILDE_EQ,
  [anon_sym_TILDE_BANG_EQ] = anon_sym_TILDE_BANG_EQ,
  [anon_sym_union] = anon_sym_union,
  [anon_sym_16] = anon_sym_16,
  [anon_sym_diff] = anon_sym_diff,
  [anon_sym_17] = anon_sym_17,
  [anon_sym_symdiff] = anon_sym_symdiff,
  [anon_sym_intersect] = anon_sym_intersect,
  [anon_sym_18] = anon_sym_18,
  [anon_sym_PLUS] = anon_sym_PLUS,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_TILDE_PLUS] = anon_sym_TILDE_PLUS,
  [anon_sym_TILDE_DASH] = anon_sym_TILDE_DASH,
  [anon_sym_STAR] = anon_sym_STAR,
  [anon_sym_SLASH] = anon_sym_SLASH,
  [anon_sym_div] = anon_sym_div,
  [anon_sym_mod] = anon_sym_mod,
  [anon_sym_TILDE_STAR] = anon_sym_TILDE_STAR,
  [anon_sym_TILDEdiv] = anon_sym_TILDEdiv,
  [anon_sym_TILDE_SLASH] = anon_sym_TILDE_SLASH,
  [anon_sym_CARET] = anon_sym_CARET,
  [anon_sym_default] = anon_sym_default,
  [anon_sym_case] = anon_sym_case,
  [anon_sym_of] = anon_sym_of,
  [anon_sym_endcase] = anon_sym_endcase,
  [anon_sym_EQ_GT] = anon_sym_EQ_GT,
  [anon_sym_lambda] = anon_sym_lambda,
  [anon_sym_let] = anon_sym_let,
  [anon_sym_not] = anon_sym_not,
  [anon_sym_19] = anon_sym_19,
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
  [anon_sym_20] = anon_sym_20,
  [anon_sym_21] = anon_sym_21,
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
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
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
  [anon_sym_LT_DASH_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_2] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH_GT] = {
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
  [anon_sym_LT_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_5] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_6] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_7] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_xor] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_8] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_9] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BANG_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_10] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_11] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_12] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_13] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_subset] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_14] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_superset] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_15] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_TILDE_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_TILDE_BANG_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_union] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_16] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_diff] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_17] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_symdiff] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_intersect] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_18] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PLUS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_TILDE_PLUS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_TILDE_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_STAR] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_div] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_mod] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_TILDE_STAR] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_TILDEdiv] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_TILDE_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_CARET] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_default] = {
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
  [anon_sym_not] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_19] = {
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
  [anon_sym_20] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_21] = {
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
  [aux_sym_source_file_repeat1] = {
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
  field_column_index = 1,
  field_content = 2,
  field_definition = 3,
  field_escape = 4,
  field_index = 5,
  field_item = 6,
  field_member = 7,
  field_name = 8,
  field_row = 9,
  field_value = 10,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_column_index] = "column_index",
  [field_content] = "content",
  [field_definition] = "definition",
  [field_escape] = "escape",
  [field_index] = "index",
  [field_item] = "item",
  [field_member] = "member",
  [field_name] = "name",
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
  [21] = {.index = 27, .length = 2},
  [22] = {.index = 29, .length = 2},
  [23] = {.index = 31, .length = 2},
  [24] = {.index = 33, .length = 2},
  [25] = {.index = 35, .length = 2},
  [26] = {.index = 37, .length = 2},
  [27] = {.index = 39, .length = 2},
  [28] = {.index = 41, .length = 2},
  [29] = {.index = 43, .length = 2},
  [30] = {.index = 45, .length = 2},
  [31] = {.index = 47, .length = 3},
  [32] = {.index = 50, .length = 3},
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
    {field_member, 1},
    {field_member, 2, .inherited = true},
  [29] =
    {field_name, 0},
    {field_value, 2},
  [31] =
    {field_member, 1, .inherited = true},
    {field_member, 2},
  [33] =
    {field_index, 0},
    {field_value, 2},
  [35] =
    {field_index, 0},
    {field_member, 2},
  [37] =
    {field_row, 1},
    {field_row, 2, .inherited = true},
  [39] =
    {field_row, 0, .inherited = true},
    {field_row, 1, .inherited = true},
  [41] =
    {field_column_index, 1, .inherited = true},
    {field_row, 2, .inherited = true},
  [43] =
    {field_member, 1},
    {field_member, 3},
  [45] =
    {field_member, 1},
    {field_member, 3, .inherited = true},
  [47] =
    {field_index, 0},
    {field_member, 2},
    {field_member, 3, .inherited = true},
  [50] =
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
};

static inline bool sym_identifier_character_set_1(int32_t c) {
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

static inline bool sym_identifier_character_set_2(int32_t c) {
  return (c < 8658
    ? (c < '^'
      ? (c < '$'
        ? (c < '!'
          ? c == 0
          : c <= '"')
        : (c <= '.' || (c < '['
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
          : c <= 8746)))
      : (c <= 8800 || (c < 8891
        ? (c < 8838
          ? (c >= 8804 && c <= 8805)
          : c <= 8839)
        : (c <= 8891 || c == 10231))))));
}

static inline bool sym_identifier_character_set_3(int32_t c) {
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

static inline bool sym_identifier_character_set_4(int32_t c) {
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

static inline bool sym_identifier_character_set_5(int32_t c) {
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
      if (eof) ADVANCE(38);
      if (lookahead == '!') ADVANCE(15);
      if (lookahead == '"') ADVANCE(101);
      if (lookahead == '$') ADVANCE(5);
      if (lookahead == '%') ADVANCE(159);
      if (lookahead == '\'') ADVANCE(10);
      if (lookahead == '(') ADVANCE(46);
      if (lookahead == ')') ADVANCE(48);
      if (lookahead == '*') ADVANCE(93);
      if (lookahead == '+') ADVANCE(89);
      if (lookahead == ',') ADVANCE(47);
      if (lookahead == '-') ADVANCE(90);
      if (lookahead == '.') ADVANCE(56);
      if (lookahead == '/') ADVANCE(94);
      if (lookahead == '0') ADVANCE(139);
      if (lookahead == ':') ADVANCE(43);
      if (lookahead == ';') ADVANCE(39);
      if (lookahead == '<') ADVANCE(75);
      if (lookahead == '=') ADVANCE(41);
      if (lookahead == '>') ADVANCE(78);
      if (lookahead == '[') ADVANCE(51);
      if (lookahead == '\\') ADVANCE(134);
      if (lookahead == ']') ADVANCE(53);
      if (lookahead == '^') ADVANCE(98);
      if (lookahead == '{') ADVANCE(49);
      if (lookahead == '|') ADVANCE(52);
      if (lookahead == '}') ADVANCE(50);
      if (lookahead == '~') ADVANCE(2);
      if (lookahead == 172) ADVANCE(100);
      if (lookahead == 8592) ADVANCE(67);
      if (lookahead == 8594) ADVANCE(64);
      if (lookahead == 8656) ADVANCE(68);
      if (lookahead == 8658) ADVANCE(65);
      if (lookahead == 8660) ADVANCE(62);
      if (lookahead == 8709) ADVANCE(117);
      if (lookahead == 8712) ADVANCE(81);
      if (lookahead == 8726) ADVANCE(87);
      if (lookahead == 8734) ADVANCE(116);
      if (lookahead == 8743) ADVANCE(71);
      if (lookahead == 8744) ADVANCE(69);
      if (lookahead == 8745) ADVANCE(88);
      if (lookahead == 8746) ADVANCE(86);
      if (lookahead == 8800) ADVANCE(74);
      if (lookahead == 8804) ADVANCE(77);
      if (lookahead == 8805) ADVANCE(80);
      if (lookahead == 8838) ADVANCE(82);
      if (lookahead == 8839) ADVANCE(83);
      if (lookahead == 8891) ADVANCE(70);
      if (lookahead == 10231) ADVANCE(61);
      if (lookahead == '8' ||
          lookahead == '9') ADVANCE(59);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
      if (('1' <= lookahead && lookahead <= '7')) ADVANCE(58);
      if (lookahead != 0 &&
          lookahead != '&') ADVANCE(156);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(4)
      if (lookahead == '"') ADVANCE(101);
      if (lookahead == '%') ADVANCE(123);
      if (lookahead == '/') ADVANCE(121);
      if (lookahead == '\\') ADVANCE(135);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(118);
      if (lookahead != 0) ADVANCE(123);
      END_STATE();
    case 2:
      if (lookahead == '!') ADVANCE(16);
      if (lookahead == '*') ADVANCE(95);
      if (lookahead == '+') ADVANCE(91);
      if (lookahead == '-') ADVANCE(92);
      if (lookahead == '/') ADVANCE(97);
      if (lookahead == '=') ADVANCE(84);
      if (lookahead == 'd') ADVANCE(18);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(101);
      if (lookahead == '%') ADVANCE(159);
      if (lookahead == '\'') ADVANCE(10);
      if (lookahead == '(') ADVANCE(46);
      if (lookahead == ')') ADVANCE(48);
      if (lookahead == '/') ADVANCE(151);
      if (lookahead == '0') ADVANCE(111);
      if (lookahead == '<') ADVANCE(17);
      if (lookahead == '[') ADVANCE(51);
      if (lookahead == ']') ADVANCE(53);
      if (lookahead == '{') ADVANCE(49);
      if (lookahead == '|') ADVANCE(52);
      if (lookahead == '}') ADVANCE(50);
      if (lookahead == 8709) ADVANCE(117);
      if (lookahead == 8734) ADVANCE(116);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(3)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(112);
      if (!sym_identifier_character_set_1(lookahead)) ADVANCE(156);
      END_STATE();
    case 4:
      if (lookahead == '"') ADVANCE(101);
      if (lookahead == '%') ADVANCE(159);
      if (lookahead == '/') ADVANCE(11);
      if (lookahead == '\\') ADVANCE(135);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(4)
      END_STATE();
    case 5:
      if (lookahead == '$') ADVANCE(34);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(103);
      END_STATE();
    case 6:
      if (lookahead == '%') ADVANCE(159);
      if (lookahead == '/') ADVANCE(11);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(6)
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(140);
      END_STATE();
    case 7:
      if (lookahead == '%') ADVANCE(159);
      if (lookahead == '/') ADVANCE(11);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(7)
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(23);
      END_STATE();
    case 8:
      if (lookahead == '%') ADVANCE(159);
      if (lookahead == '/') ADVANCE(11);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(8)
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(28);
      END_STATE();
    case 9:
      if (lookahead == '%') ADVANCE(159);
      if (lookahead == '/') ADVANCE(11);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(9)
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(33);
      END_STATE();
    case 10:
      if (lookahead == '\'') ADVANCE(157);
      if (lookahead != 0) ADVANCE(10);
      END_STATE();
    case 11:
      if (lookahead == '*') ADVANCE(36);
      END_STATE();
    case 12:
      if (lookahead == '*') ADVANCE(35);
      if (lookahead == '/') ADVANCE(160);
      if (lookahead != 0) ADVANCE(36);
      END_STATE();
    case 13:
      if (lookahead == '.') ADVANCE(55);
      END_STATE();
    case 14:
      if (lookahead == '1') ADVANCE(158);
      END_STATE();
    case 15:
      if (lookahead == '=') ADVANCE(73);
      END_STATE();
    case 16:
      if (lookahead == '=') ADVANCE(85);
      END_STATE();
    case 17:
      if (lookahead == '>') ADVANCE(105);
      END_STATE();
    case 18:
      if (lookahead == 'i') ADVANCE(19);
      END_STATE();
    case 19:
      if (lookahead == 'v') ADVANCE(96);
      END_STATE();
    case 20:
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(22);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(110);
      END_STATE();
    case 21:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(108);
      END_STATE();
    case 22:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(110);
      END_STATE();
    case 23:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(144);
      END_STATE();
    case 24:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(147);
      END_STATE();
    case 25:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(150);
      END_STATE();
    case 26:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(24);
      END_STATE();
    case 27:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(25);
      END_STATE();
    case 28:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(26);
      END_STATE();
    case 29:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(27);
      END_STATE();
    case 30:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(29);
      END_STATE();
    case 31:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(30);
      END_STATE();
    case 32:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(31);
      END_STATE();
    case 33:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(32);
      END_STATE();
    case 34:
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(104);
      END_STATE();
    case 35:
      if (lookahead != 0 &&
          lookahead != '*' &&
          lookahead != '/') ADVANCE(36);
      if (lookahead == '*') ADVANCE(12);
      if (lookahead == '/') ADVANCE(161);
      END_STATE();
    case 36:
      if (lookahead != 0 &&
          lookahead != '*') ADVANCE(36);
      if (lookahead == '*') ADVANCE(12);
      END_STATE();
    case 37:
      if (eof) ADVANCE(38);
      if (lookahead == '%') ADVANCE(159);
      if (lookahead == '\'') ADVANCE(10);
      if (lookahead == ')') ADVANCE(48);
      if (lookahead == ',') ADVANCE(47);
      if (lookahead == '/') ADVANCE(151);
      if (lookahead == ':') ADVANCE(42);
      if (lookahead == ';') ADVANCE(39);
      if (lookahead == '=') ADVANCE(40);
      if (lookahead == ']') ADVANCE(53);
      if (lookahead == '|') ADVANCE(52);
      if (lookahead == '}') ADVANCE(50);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(37)
      if (!sym_identifier_character_set_2(lookahead)) ADVANCE(156);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(anon_sym_EQ);
      if (lookahead == '=') ADVANCE(72);
      if (lookahead == '>') ADVANCE(99);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(anon_sym_COLON);
      if (lookahead == ':') ADVANCE(45);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(anon_sym_PLUS_PLUS);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(anon_sym_COLON_COLON);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      if (lookahead == '|') ADVANCE(106);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(anon_sym_PIPE);
      if (lookahead == ']') ADVANCE(107);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym_LT_DOT_DOT);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(anon_sym_DOT);
      if (lookahead == '.') ADVANCE(54);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(aux_sym_tuple_access_token1);
      if (lookahead == '8' ||
          lookahead == '9') ADVANCE(59);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(59);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(aux_sym_tuple_access_token1);
      if (lookahead == '8' ||
          lookahead == '9') ADVANCE(59);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(57);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(aux_sym_tuple_access_token1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(59);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(anon_sym_LT_DASH_GT);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(anon_sym_);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(anon_sym_2);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(anon_sym_DASH_GT);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(anon_sym_3);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(anon_sym_4);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(anon_sym_LT_DASH);
      if (lookahead == '>') ADVANCE(60);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(anon_sym_5);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(anon_sym_6);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(anon_sym_7);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(anon_sym_8);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(anon_sym_9);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(anon_sym_EQ_EQ);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(anon_sym_BANG_EQ);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(anon_sym_10);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '-') ADVANCE(66);
      if (lookahead == '.') ADVANCE(13);
      if (lookahead == '=') ADVANCE(76);
      if (lookahead == '>') ADVANCE(105);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(anon_sym_LT_EQ);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(anon_sym_11);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(anon_sym_GT);
      if (lookahead == '=') ADVANCE(79);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(anon_sym_GT_EQ);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(anon_sym_12);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(anon_sym_13);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(anon_sym_14);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(anon_sym_15);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(anon_sym_TILDE_EQ);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(anon_sym_TILDE_BANG_EQ);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(anon_sym_16);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(anon_sym_17);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(anon_sym_18);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(anon_sym_PLUS);
      if (lookahead == '+') ADVANCE(44);
      END_STATE();
    case 90:
      ACCEPT_TOKEN(anon_sym_DASH);
      if (lookahead == '>') ADVANCE(63);
      END_STATE();
    case 91:
      ACCEPT_TOKEN(anon_sym_TILDE_PLUS);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(anon_sym_TILDE_DASH);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(anon_sym_STAR);
      END_STATE();
    case 94:
      ACCEPT_TOKEN(anon_sym_SLASH);
      if (lookahead == '*') ADVANCE(36);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 95:
      ACCEPT_TOKEN(anon_sym_TILDE_STAR);
      END_STATE();
    case 96:
      ACCEPT_TOKEN(anon_sym_TILDEdiv);
      END_STATE();
    case 97:
      ACCEPT_TOKEN(anon_sym_TILDE_SLASH);
      END_STATE();
    case 98:
      ACCEPT_TOKEN(anon_sym_CARET);
      if (lookahead == '-') ADVANCE(14);
      END_STATE();
    case 99:
      ACCEPT_TOKEN(anon_sym_EQ_GT);
      END_STATE();
    case 100:
      ACCEPT_TOKEN(anon_sym_19);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 101:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 102:
      ACCEPT_TOKEN(anon_sym_BSLASH_LPAREN);
      END_STATE();
    case 103:
      ACCEPT_TOKEN(sym_type_inst_id);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(103);
      END_STATE();
    case 104:
      ACCEPT_TOKEN(sym_type_inst_enum_id);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(104);
      END_STATE();
    case 105:
      ACCEPT_TOKEN(sym_absent);
      END_STATE();
    case 106:
      ACCEPT_TOKEN(anon_sym_LBRACK_PIPE);
      END_STATE();
    case 107:
      ACCEPT_TOKEN(anon_sym_PIPE_RBRACK);
      END_STATE();
    case 108:
      ACCEPT_TOKEN(sym_float_literal);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(108);
      END_STATE();
    case 109:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(109);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 110:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(110);
      END_STATE();
    case 111:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(21);
      if (lookahead == 'b') ADVANCE(153);
      if (lookahead == 'o') ADVANCE(154);
      if (lookahead == 'x') ADVANCE(155);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(152);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(112);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(156);
      END_STATE();
    case 112:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(21);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(152);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(112);
      if (!sym_identifier_character_set_4(lookahead)) ADVANCE(156);
      END_STATE();
    case 113:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(113);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(114);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 115:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(115);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 116:
      ACCEPT_TOKEN(anon_sym_20);
      END_STATE();
    case 117:
      ACCEPT_TOKEN(anon_sym_21);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 118:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '%') ADVANCE(123);
      if (lookahead == '/') ADVANCE(121);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(118);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(123);
      END_STATE();
    case 119:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(122);
      if (lookahead == '/') ADVANCE(120);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(120);
      END_STATE();
    case 120:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(122);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(120);
      END_STATE();
    case 121:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(120);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(123);
      END_STATE();
    case 122:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(119);
      if (lookahead == '/') ADVANCE(123);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(120);
      END_STATE();
    case 123:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(123);
      END_STATE();
    case 124:
      ACCEPT_TOKEN(anon_sym_BSLASH_SQUOTE);
      END_STATE();
    case 125:
      ACCEPT_TOKEN(anon_sym_BSLASH_DQUOTE);
      END_STATE();
    case 126:
      ACCEPT_TOKEN(anon_sym_BSLASH_BSLASH);
      END_STATE();
    case 127:
      ACCEPT_TOKEN(anon_sym_BSLASH_BSLASH);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 128:
      ACCEPT_TOKEN(anon_sym_BSLASHr);
      END_STATE();
    case 129:
      ACCEPT_TOKEN(anon_sym_BSLASHr);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 130:
      ACCEPT_TOKEN(anon_sym_BSLASHn);
      END_STATE();
    case 131:
      ACCEPT_TOKEN(anon_sym_BSLASHn);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 132:
      ACCEPT_TOKEN(anon_sym_BSLASHt);
      END_STATE();
    case 133:
      ACCEPT_TOKEN(anon_sym_BSLASHt);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 134:
      ACCEPT_TOKEN(anon_sym_BSLASH);
      if (lookahead == '"') ADVANCE(125);
      if (lookahead == '\'') ADVANCE(124);
      if (lookahead == '(') ADVANCE(102);
      if (lookahead == 'U') ADVANCE(149);
      if (lookahead == '\\') ADVANCE(127);
      if (lookahead == 'n') ADVANCE(131);
      if (lookahead == 'r') ADVANCE(129);
      if (lookahead == 't') ADVANCE(133);
      if (lookahead == 'u') ADVANCE(146);
      if (lookahead == 'x') ADVANCE(143);
      if (!sym_identifier_character_set_5(lookahead)) ADVANCE(156);
      END_STATE();
    case 135:
      ACCEPT_TOKEN(anon_sym_BSLASH);
      if (lookahead == '"') ADVANCE(125);
      if (lookahead == '\'') ADVANCE(124);
      if (lookahead == 'U') ADVANCE(148);
      if (lookahead == '\\') ADVANCE(126);
      if (lookahead == 'n') ADVANCE(130);
      if (lookahead == 'r') ADVANCE(128);
      if (lookahead == 't') ADVANCE(132);
      if (lookahead == 'u') ADVANCE(145);
      if (lookahead == 'x') ADVANCE(142);
      END_STATE();
    case 136:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      END_STATE();
    case 137:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(141);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 138:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(136);
      END_STATE();
    case 139:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(137);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 140:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(138);
      END_STATE();
    case 141:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 142:
      ACCEPT_TOKEN(anon_sym_BSLASHx);
      END_STATE();
    case 143:
      ACCEPT_TOKEN(anon_sym_BSLASHx);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 144:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token2);
      END_STATE();
    case 145:
      ACCEPT_TOKEN(anon_sym_BSLASHu);
      END_STATE();
    case 146:
      ACCEPT_TOKEN(anon_sym_BSLASHu);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 147:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token3);
      END_STATE();
    case 148:
      ACCEPT_TOKEN(anon_sym_BSLASHU);
      END_STATE();
    case 149:
      ACCEPT_TOKEN(anon_sym_BSLASHU);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 150:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token4);
      END_STATE();
    case 151:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '*') ADVANCE(36);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 152:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(22);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(109);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 153:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(113);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 154:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(114);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 155:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(115);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 156:
      ACCEPT_TOKEN(sym_identifier);
      if (!sym_identifier_character_set_3(lookahead)) ADVANCE(156);
      END_STATE();
    case 157:
      ACCEPT_TOKEN(sym_quoted_identifier);
      END_STATE();
    case 158:
      ACCEPT_TOKEN(anon_sym_CARET_DASH1);
      END_STATE();
    case 159:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(159);
      END_STATE();
    case 160:
      ACCEPT_TOKEN(sym_block_comment);
      END_STATE();
    case 161:
      ACCEPT_TOKEN(sym_block_comment);
      if (lookahead != 0 &&
          lookahead != '*') ADVANCE(36);
      if (lookahead == '*') ADVANCE(12);
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
      if (lookahead == 'd') ADVANCE(5);
      if (lookahead == 'e') ADVANCE(6);
      if (lookahead == 'f') ADVANCE(7);
      if (lookahead == 'i') ADVANCE(8);
      if (lookahead == 'l') ADVANCE(9);
      if (lookahead == 'm') ADVANCE(10);
      if (lookahead == 'n') ADVANCE(11);
      if (lookahead == 'o') ADVANCE(12);
      if (lookahead == 'p') ADVANCE(13);
      if (lookahead == 'r') ADVANCE(14);
      if (lookahead == 's') ADVANCE(15);
      if (lookahead == 't') ADVANCE(16);
      if (lookahead == 'u') ADVANCE(17);
      if (lookahead == 'v') ADVANCE(18);
      if (lookahead == 'w') ADVANCE(19);
      if (lookahead == 'x') ADVANCE(20);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
      END_STATE();
    case 1:
      ACCEPT_TOKEN(sym_anonymous);
      END_STATE();
    case 2:
      if (lookahead == 'n') ADVANCE(21);
      if (lookahead == 'r') ADVANCE(22);
      END_STATE();
    case 3:
      if (lookahead == 'o') ADVANCE(23);
      END_STATE();
    case 4:
      if (lookahead == 'a') ADVANCE(24);
      if (lookahead == 'o') ADVANCE(25);
      END_STATE();
    case 5:
      if (lookahead == 'e') ADVANCE(26);
      if (lookahead == 'i') ADVANCE(27);
      END_STATE();
    case 6:
      if (lookahead == 'l') ADVANCE(28);
      if (lookahead == 'n') ADVANCE(29);
      END_STATE();
    case 7:
      if (lookahead == 'a') ADVANCE(30);
      if (lookahead == 'l') ADVANCE(31);
      if (lookahead == 'u') ADVANCE(32);
      END_STATE();
    case 8:
      if (lookahead == 'f') ADVANCE(33);
      if (lookahead == 'n') ADVANCE(34);
      END_STATE();
    case 9:
      if (lookahead == 'a') ADVANCE(35);
      if (lookahead == 'e') ADVANCE(36);
      END_STATE();
    case 10:
      if (lookahead == 'a') ADVANCE(37);
      if (lookahead == 'i') ADVANCE(38);
      if (lookahead == 'o') ADVANCE(39);
      END_STATE();
    case 11:
      if (lookahead == 'o') ADVANCE(40);
      END_STATE();
    case 12:
      if (lookahead == 'f') ADVANCE(41);
      if (lookahead == 'p') ADVANCE(42);
      if (lookahead == 'u') ADVANCE(43);
      END_STATE();
    case 13:
      if (lookahead == 'a') ADVANCE(44);
      if (lookahead == 'r') ADVANCE(45);
      END_STATE();
    case 14:
      if (lookahead == 'e') ADVANCE(46);
      END_STATE();
    case 15:
      if (lookahead == 'a') ADVANCE(47);
      if (lookahead == 'e') ADVANCE(48);
      if (lookahead == 'o') ADVANCE(49);
      if (lookahead == 't') ADVANCE(50);
      if (lookahead == 'u') ADVANCE(51);
      if (lookahead == 'y') ADVANCE(52);
      END_STATE();
    case 16:
      if (lookahead == 'e') ADVANCE(53);
      if (lookahead == 'h') ADVANCE(54);
      if (lookahead == 'r') ADVANCE(55);
      if (lookahead == 'u') ADVANCE(56);
      if (lookahead == 'y') ADVANCE(57);
      END_STATE();
    case 17:
      if (lookahead == 'n') ADVANCE(58);
      END_STATE();
    case 18:
      if (lookahead == 'a') ADVANCE(59);
      END_STATE();
    case 19:
      if (lookahead == 'h') ADVANCE(60);
      END_STATE();
    case 20:
      if (lookahead == 'o') ADVANCE(61);
      END_STATE();
    case 21:
      if (lookahead == 'n') ADVANCE(62);
      if (lookahead == 'y') ADVANCE(63);
      END_STATE();
    case 22:
      if (lookahead == 'r') ADVANCE(64);
      END_STATE();
    case 23:
      if (lookahead == 'o') ADVANCE(65);
      END_STATE();
    case 24:
      if (lookahead == 's') ADVANCE(66);
      END_STATE();
    case 25:
      if (lookahead == 'n') ADVANCE(67);
      END_STATE();
    case 26:
      if (lookahead == 'f') ADVANCE(68);
      END_STATE();
    case 27:
      if (lookahead == 'f') ADVANCE(69);
      if (lookahead == 'v') ADVANCE(70);
      END_STATE();
    case 28:
      if (lookahead == 's') ADVANCE(71);
      END_STATE();
    case 29:
      if (lookahead == 'd') ADVANCE(72);
      if (lookahead == 'u') ADVANCE(73);
      END_STATE();
    case 30:
      if (lookahead == 'l') ADVANCE(74);
      END_STATE();
    case 31:
      if (lookahead == 'o') ADVANCE(75);
      END_STATE();
    case 32:
      if (lookahead == 'n') ADVANCE(76);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(anon_sym_in);
      if (lookahead == 'c') ADVANCE(77);
      if (lookahead == 'f') ADVANCE(78);
      if (lookahead == 't') ADVANCE(79);
      END_STATE();
    case 35:
      if (lookahead == 'm') ADVANCE(80);
      END_STATE();
    case 36:
      if (lookahead == 't') ADVANCE(81);
      END_STATE();
    case 37:
      if (lookahead == 'x') ADVANCE(82);
      END_STATE();
    case 38:
      if (lookahead == 'n') ADVANCE(83);
      END_STATE();
    case 39:
      if (lookahead == 'd') ADVANCE(84);
      END_STATE();
    case 40:
      if (lookahead == 't') ADVANCE(85);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(anon_sym_of);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(anon_sym_op);
      if (lookahead == 't') ADVANCE(86);
      END_STATE();
    case 43:
      if (lookahead == 't') ADVANCE(87);
      END_STATE();
    case 44:
      if (lookahead == 'r') ADVANCE(88);
      END_STATE();
    case 45:
      if (lookahead == 'e') ADVANCE(89);
      END_STATE();
    case 46:
      if (lookahead == 'c') ADVANCE(90);
      END_STATE();
    case 47:
      if (lookahead == 't') ADVANCE(91);
      END_STATE();
    case 48:
      if (lookahead == 't') ADVANCE(92);
      END_STATE();
    case 49:
      if (lookahead == 'l') ADVANCE(93);
      END_STATE();
    case 50:
      if (lookahead == 'r') ADVANCE(94);
      END_STATE();
    case 51:
      if (lookahead == 'b') ADVANCE(95);
      if (lookahead == 'p') ADVANCE(96);
      END_STATE();
    case 52:
      if (lookahead == 'm') ADVANCE(97);
      END_STATE();
    case 53:
      if (lookahead == 's') ADVANCE(98);
      END_STATE();
    case 54:
      if (lookahead == 'e') ADVANCE(99);
      END_STATE();
    case 55:
      if (lookahead == 'u') ADVANCE(100);
      END_STATE();
    case 56:
      if (lookahead == 'p') ADVANCE(101);
      END_STATE();
    case 57:
      if (lookahead == 'p') ADVANCE(102);
      END_STATE();
    case 58:
      if (lookahead == 'i') ADVANCE(103);
      END_STATE();
    case 59:
      if (lookahead == 'r') ADVANCE(104);
      END_STATE();
    case 60:
      if (lookahead == 'e') ADVANCE(105);
      END_STATE();
    case 61:
      if (lookahead == 'r') ADVANCE(106);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(anon_sym_ann);
      if (lookahead == 'o') ADVANCE(107);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(anon_sym_any);
      END_STATE();
    case 64:
      if (lookahead == 'a') ADVANCE(108);
      END_STATE();
    case 65:
      if (lookahead == 'l') ADVANCE(109);
      END_STATE();
    case 66:
      if (lookahead == 'e') ADVANCE(110);
      END_STATE();
    case 67:
      if (lookahead == 's') ADVANCE(111);
      END_STATE();
    case 68:
      if (lookahead == 'a') ADVANCE(112);
      END_STATE();
    case 69:
      if (lookahead == 'f') ADVANCE(113);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(anon_sym_div);
      END_STATE();
    case 71:
      if (lookahead == 'e') ADVANCE(114);
      END_STATE();
    case 72:
      if (lookahead == 'c') ADVANCE(115);
      if (lookahead == 'i') ADVANCE(116);
      END_STATE();
    case 73:
      if (lookahead == 'm') ADVANCE(117);
      END_STATE();
    case 74:
      if (lookahead == 's') ADVANCE(118);
      END_STATE();
    case 75:
      if (lookahead == 'a') ADVANCE(119);
      END_STATE();
    case 76:
      if (lookahead == 'c') ADVANCE(120);
      END_STATE();
    case 77:
      if (lookahead == 'l') ADVANCE(121);
      END_STATE();
    case 78:
      if (lookahead == 'i') ADVANCE(122);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(anon_sym_int);
      if (lookahead == 'e') ADVANCE(123);
      END_STATE();
    case 80:
      if (lookahead == 'b') ADVANCE(124);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(anon_sym_let);
      END_STATE();
    case 82:
      if (lookahead == 'i') ADVANCE(125);
      END_STATE();
    case 83:
      if (lookahead == 'i') ADVANCE(126);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(anon_sym_mod);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(anon_sym_not);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(anon_sym_opt);
      END_STATE();
    case 87:
      if (lookahead == 'p') ADVANCE(127);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(anon_sym_par);
      END_STATE();
    case 89:
      if (lookahead == 'd') ADVANCE(128);
      END_STATE();
    case 90:
      if (lookahead == 'o') ADVANCE(129);
      END_STATE();
    case 91:
      if (lookahead == 'i') ADVANCE(130);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 93:
      if (lookahead == 'v') ADVANCE(131);
      END_STATE();
    case 94:
      if (lookahead == 'i') ADVANCE(132);
      END_STATE();
    case 95:
      if (lookahead == 's') ADVANCE(133);
      END_STATE();
    case 96:
      if (lookahead == 'e') ADVANCE(134);
      END_STATE();
    case 97:
      if (lookahead == 'd') ADVANCE(135);
      END_STATE();
    case 98:
      if (lookahead == 't') ADVANCE(136);
      END_STATE();
    case 99:
      if (lookahead == 'n') ADVANCE(137);
      END_STATE();
    case 100:
      if (lookahead == 'e') ADVANCE(138);
      END_STATE();
    case 101:
      if (lookahead == 'l') ADVANCE(139);
      END_STATE();
    case 102:
      if (lookahead == 'e') ADVANCE(140);
      END_STATE();
    case 103:
      if (lookahead == 'o') ADVANCE(141);
      END_STATE();
    case 104:
      ACCEPT_TOKEN(anon_sym_var);
      END_STATE();
    case 105:
      if (lookahead == 'r') ADVANCE(142);
      END_STATE();
    case 106:
      ACCEPT_TOKEN(anon_sym_xor);
      END_STATE();
    case 107:
      if (lookahead == 't') ADVANCE(143);
      END_STATE();
    case 108:
      if (lookahead == 'y') ADVANCE(144);
      END_STATE();
    case 109:
      ACCEPT_TOKEN(anon_sym_bool);
      END_STATE();
    case 110:
      ACCEPT_TOKEN(anon_sym_case);
      END_STATE();
    case 111:
      if (lookahead == 't') ADVANCE(145);
      END_STATE();
    case 112:
      if (lookahead == 'u') ADVANCE(146);
      END_STATE();
    case 113:
      ACCEPT_TOKEN(anon_sym_diff);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(anon_sym_else);
      if (lookahead == 'i') ADVANCE(147);
      END_STATE();
    case 115:
      if (lookahead == 'a') ADVANCE(148);
      END_STATE();
    case 116:
      if (lookahead == 'f') ADVANCE(149);
      END_STATE();
    case 117:
      ACCEPT_TOKEN(anon_sym_enum);
      END_STATE();
    case 118:
      if (lookahead == 'e') ADVANCE(150);
      END_STATE();
    case 119:
      if (lookahead == 't') ADVANCE(151);
      END_STATE();
    case 120:
      if (lookahead == 't') ADVANCE(152);
      END_STATE();
    case 121:
      if (lookahead == 'u') ADVANCE(153);
      END_STATE();
    case 122:
      if (lookahead == 'n') ADVANCE(154);
      END_STATE();
    case 123:
      if (lookahead == 'r') ADVANCE(155);
      END_STATE();
    case 124:
      if (lookahead == 'd') ADVANCE(156);
      END_STATE();
    case 125:
      if (lookahead == 'm') ADVANCE(157);
      END_STATE();
    case 126:
      if (lookahead == 'm') ADVANCE(158);
      END_STATE();
    case 127:
      if (lookahead == 'u') ADVANCE(159);
      END_STATE();
    case 128:
      if (lookahead == 'i') ADVANCE(160);
      END_STATE();
    case 129:
      if (lookahead == 'r') ADVANCE(161);
      END_STATE();
    case 130:
      if (lookahead == 's') ADVANCE(162);
      END_STATE();
    case 131:
      if (lookahead == 'e') ADVANCE(163);
      END_STATE();
    case 132:
      if (lookahead == 'n') ADVANCE(164);
      END_STATE();
    case 133:
      if (lookahead == 'e') ADVANCE(165);
      END_STATE();
    case 134:
      if (lookahead == 'r') ADVANCE(166);
      END_STATE();
    case 135:
      if (lookahead == 'i') ADVANCE(167);
      END_STATE();
    case 136:
      ACCEPT_TOKEN(anon_sym_test);
      END_STATE();
    case 137:
      ACCEPT_TOKEN(anon_sym_then);
      END_STATE();
    case 138:
      ACCEPT_TOKEN(anon_sym_true);
      END_STATE();
    case 139:
      if (lookahead == 'e') ADVANCE(168);
      END_STATE();
    case 140:
      ACCEPT_TOKEN(anon_sym_type);
      END_STATE();
    case 141:
      if (lookahead == 'n') ADVANCE(169);
      END_STATE();
    case 142:
      if (lookahead == 'e') ADVANCE(170);
      END_STATE();
    case 143:
      if (lookahead == 'a') ADVANCE(171);
      END_STATE();
    case 144:
      ACCEPT_TOKEN(anon_sym_array);
      END_STATE();
    case 145:
      if (lookahead == 'r') ADVANCE(172);
      END_STATE();
    case 146:
      if (lookahead == 'l') ADVANCE(173);
      END_STATE();
    case 147:
      if (lookahead == 'f') ADVANCE(174);
      END_STATE();
    case 148:
      if (lookahead == 's') ADVANCE(175);
      END_STATE();
    case 149:
      ACCEPT_TOKEN(anon_sym_endif);
      END_STATE();
    case 150:
      ACCEPT_TOKEN(anon_sym_false);
      END_STATE();
    case 151:
      ACCEPT_TOKEN(anon_sym_float);
      END_STATE();
    case 152:
      if (lookahead == 'i') ADVANCE(176);
      END_STATE();
    case 153:
      if (lookahead == 'd') ADVANCE(177);
      END_STATE();
    case 154:
      if (lookahead == 'i') ADVANCE(178);
      END_STATE();
    case 155:
      if (lookahead == 's') ADVANCE(179);
      END_STATE();
    case 156:
      if (lookahead == 'a') ADVANCE(180);
      END_STATE();
    case 157:
      if (lookahead == 'i') ADVANCE(181);
      END_STATE();
    case 158:
      if (lookahead == 'i') ADVANCE(182);
      END_STATE();
    case 159:
      if (lookahead == 't') ADVANCE(183);
      END_STATE();
    case 160:
      if (lookahead == 'c') ADVANCE(184);
      END_STATE();
    case 161:
      if (lookahead == 'd') ADVANCE(185);
      END_STATE();
    case 162:
      if (lookahead == 'f') ADVANCE(186);
      END_STATE();
    case 163:
      ACCEPT_TOKEN(anon_sym_solve);
      END_STATE();
    case 164:
      if (lookahead == 'g') ADVANCE(187);
      END_STATE();
    case 165:
      if (lookahead == 't') ADVANCE(188);
      END_STATE();
    case 166:
      if (lookahead == 's') ADVANCE(189);
      END_STATE();
    case 167:
      if (lookahead == 'f') ADVANCE(190);
      END_STATE();
    case 168:
      ACCEPT_TOKEN(anon_sym_tuple);
      END_STATE();
    case 169:
      ACCEPT_TOKEN(anon_sym_union);
      END_STATE();
    case 170:
      ACCEPT_TOKEN(anon_sym_where);
      END_STATE();
    case 171:
      if (lookahead == 't') ADVANCE(191);
      END_STATE();
    case 172:
      if (lookahead == 'a') ADVANCE(192);
      END_STATE();
    case 173:
      if (lookahead == 't') ADVANCE(193);
      END_STATE();
    case 174:
      ACCEPT_TOKEN(anon_sym_elseif);
      END_STATE();
    case 175:
      if (lookahead == 'e') ADVANCE(194);
      END_STATE();
    case 176:
      if (lookahead == 'o') ADVANCE(195);
      END_STATE();
    case 177:
      if (lookahead == 'e') ADVANCE(196);
      END_STATE();
    case 178:
      if (lookahead == 't') ADVANCE(197);
      END_STATE();
    case 179:
      if (lookahead == 'e') ADVANCE(198);
      END_STATE();
    case 180:
      ACCEPT_TOKEN(anon_sym_lambda);
      END_STATE();
    case 181:
      if (lookahead == 'z') ADVANCE(199);
      END_STATE();
    case 182:
      if (lookahead == 'z') ADVANCE(200);
      END_STATE();
    case 183:
      ACCEPT_TOKEN(anon_sym_output);
      END_STATE();
    case 184:
      if (lookahead == 'a') ADVANCE(201);
      END_STATE();
    case 185:
      ACCEPT_TOKEN(anon_sym_record);
      END_STATE();
    case 186:
      if (lookahead == 'y') ADVANCE(202);
      END_STATE();
    case 187:
      ACCEPT_TOKEN(anon_sym_string);
      END_STATE();
    case 188:
      ACCEPT_TOKEN(anon_sym_subset);
      END_STATE();
    case 189:
      if (lookahead == 'e') ADVANCE(203);
      END_STATE();
    case 190:
      if (lookahead == 'f') ADVANCE(204);
      END_STATE();
    case 191:
      if (lookahead == 'i') ADVANCE(205);
      END_STATE();
    case 192:
      if (lookahead == 'i') ADVANCE(206);
      END_STATE();
    case 193:
      ACCEPT_TOKEN(anon_sym_default);
      END_STATE();
    case 194:
      ACCEPT_TOKEN(anon_sym_endcase);
      END_STATE();
    case 195:
      if (lookahead == 'n') ADVANCE(207);
      END_STATE();
    case 196:
      ACCEPT_TOKEN(anon_sym_include);
      END_STATE();
    case 197:
      if (lookahead == 'y') ADVANCE(208);
      END_STATE();
    case 198:
      if (lookahead == 'c') ADVANCE(209);
      END_STATE();
    case 199:
      if (lookahead == 'e') ADVANCE(210);
      END_STATE();
    case 200:
      if (lookahead == 'e') ADVANCE(211);
      END_STATE();
    case 201:
      if (lookahead == 't') ADVANCE(212);
      END_STATE();
    case 202:
      ACCEPT_TOKEN(anon_sym_satisfy);
      END_STATE();
    case 203:
      if (lookahead == 't') ADVANCE(213);
      END_STATE();
    case 204:
      ACCEPT_TOKEN(anon_sym_symdiff);
      END_STATE();
    case 205:
      if (lookahead == 'o') ADVANCE(214);
      END_STATE();
    case 206:
      if (lookahead == 'n') ADVANCE(215);
      END_STATE();
    case 207:
      ACCEPT_TOKEN(anon_sym_function);
      END_STATE();
    case 208:
      ACCEPT_TOKEN(anon_sym_infinity);
      END_STATE();
    case 209:
      if (lookahead == 't') ADVANCE(216);
      END_STATE();
    case 210:
      ACCEPT_TOKEN(anon_sym_maximize);
      END_STATE();
    case 211:
      ACCEPT_TOKEN(anon_sym_minimize);
      END_STATE();
    case 212:
      if (lookahead == 'e') ADVANCE(217);
      END_STATE();
    case 213:
      ACCEPT_TOKEN(anon_sym_superset);
      END_STATE();
    case 214:
      if (lookahead == 'n') ADVANCE(218);
      END_STATE();
    case 215:
      if (lookahead == 't') ADVANCE(219);
      END_STATE();
    case 216:
      ACCEPT_TOKEN(anon_sym_intersect);
      END_STATE();
    case 217:
      ACCEPT_TOKEN(anon_sym_predicate);
      END_STATE();
    case 218:
      ACCEPT_TOKEN(anon_sym_annotation);
      END_STATE();
    case 219:
      ACCEPT_TOKEN(anon_sym_constraint);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 37},
  [2] = {.lex_state = 3},
  [3] = {.lex_state = 3},
  [4] = {.lex_state = 3},
  [5] = {.lex_state = 3},
  [6] = {.lex_state = 3},
  [7] = {.lex_state = 3},
  [8] = {.lex_state = 3},
  [9] = {.lex_state = 3},
  [10] = {.lex_state = 3},
  [11] = {.lex_state = 3},
  [12] = {.lex_state = 3},
  [13] = {.lex_state = 3},
  [14] = {.lex_state = 3},
  [15] = {.lex_state = 3},
  [16] = {.lex_state = 3},
  [17] = {.lex_state = 3},
  [18] = {.lex_state = 3},
  [19] = {.lex_state = 3},
  [20] = {.lex_state = 3},
  [21] = {.lex_state = 3},
  [22] = {.lex_state = 3},
  [23] = {.lex_state = 3},
  [24] = {.lex_state = 3},
  [25] = {.lex_state = 3},
  [26] = {.lex_state = 3},
  [27] = {.lex_state = 3},
  [28] = {.lex_state = 3},
  [29] = {.lex_state = 3},
  [30] = {.lex_state = 3},
  [31] = {.lex_state = 3},
  [32] = {.lex_state = 1},
  [33] = {.lex_state = 1},
  [34] = {.lex_state = 1},
  [35] = {.lex_state = 1},
  [36] = {.lex_state = 1},
  [37] = {.lex_state = 1},
  [38] = {.lex_state = 37},
  [39] = {.lex_state = 37},
  [40] = {.lex_state = 37},
  [41] = {.lex_state = 37},
  [42] = {.lex_state = 37},
  [43] = {.lex_state = 37},
  [44] = {.lex_state = 37},
  [45] = {.lex_state = 37},
  [46] = {.lex_state = 37},
  [47] = {.lex_state = 37},
  [48] = {.lex_state = 37},
  [49] = {.lex_state = 37},
  [50] = {.lex_state = 37},
  [51] = {.lex_state = 37},
  [52] = {.lex_state = 37},
  [53] = {.lex_state = 37},
  [54] = {.lex_state = 37},
  [55] = {.lex_state = 37},
  [56] = {.lex_state = 37},
  [57] = {.lex_state = 37},
  [58] = {.lex_state = 37},
  [59] = {.lex_state = 37},
  [60] = {.lex_state = 37},
  [61] = {.lex_state = 37},
  [62] = {.lex_state = 37},
  [63] = {.lex_state = 37},
  [64] = {.lex_state = 37},
  [65] = {.lex_state = 37},
  [66] = {.lex_state = 37},
  [67] = {.lex_state = 37},
  [68] = {.lex_state = 37},
  [69] = {.lex_state = 37},
  [70] = {.lex_state = 37},
  [71] = {.lex_state = 37},
  [72] = {.lex_state = 37},
  [73] = {.lex_state = 37},
  [74] = {.lex_state = 37},
  [75] = {.lex_state = 37},
  [76] = {.lex_state = 37},
  [77] = {.lex_state = 37},
  [78] = {.lex_state = 37},
  [79] = {.lex_state = 37},
  [80] = {.lex_state = 37},
  [81] = {.lex_state = 37},
  [82] = {.lex_state = 37},
  [83] = {.lex_state = 37},
  [84] = {.lex_state = 37},
  [85] = {.lex_state = 37},
  [86] = {.lex_state = 37},
  [87] = {.lex_state = 37},
  [88] = {.lex_state = 37},
  [89] = {.lex_state = 37},
  [90] = {.lex_state = 37},
  [91] = {.lex_state = 37},
  [92] = {.lex_state = 37},
  [93] = {.lex_state = 37},
  [94] = {.lex_state = 37},
  [95] = {.lex_state = 37},
  [96] = {.lex_state = 37},
  [97] = {.lex_state = 37},
  [98] = {.lex_state = 37},
  [99] = {.lex_state = 37},
  [100] = {.lex_state = 37},
  [101] = {.lex_state = 37},
  [102] = {.lex_state = 37},
  [103] = {.lex_state = 37},
  [104] = {.lex_state = 37},
  [105] = {.lex_state = 37},
  [106] = {.lex_state = 37},
  [107] = {.lex_state = 37},
  [108] = {.lex_state = 6},
  [109] = {.lex_state = 7},
  [110] = {.lex_state = 37},
  [111] = {.lex_state = 8},
  [112] = {.lex_state = 9},
  [113] = {.lex_state = 37},
  [114] = {.lex_state = 37},
  [115] = {.lex_state = 37},
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
    [anon_sym_LT_DASH_GT] = ACTIONS(1),
    [anon_sym_] = ACTIONS(1),
    [anon_sym_2] = ACTIONS(1),
    [anon_sym_DASH_GT] = ACTIONS(1),
    [anon_sym_3] = ACTIONS(1),
    [anon_sym_4] = ACTIONS(1),
    [anon_sym_LT_DASH] = ACTIONS(1),
    [anon_sym_5] = ACTIONS(1),
    [anon_sym_6] = ACTIONS(1),
    [anon_sym_7] = ACTIONS(1),
    [anon_sym_xor] = ACTIONS(1),
    [anon_sym_8] = ACTIONS(1),
    [anon_sym_9] = ACTIONS(1),
    [anon_sym_EQ_EQ] = ACTIONS(1),
    [anon_sym_BANG_EQ] = ACTIONS(1),
    [anon_sym_10] = ACTIONS(1),
    [anon_sym_LT] = ACTIONS(1),
    [anon_sym_LT_EQ] = ACTIONS(1),
    [anon_sym_11] = ACTIONS(1),
    [anon_sym_GT] = ACTIONS(1),
    [anon_sym_GT_EQ] = ACTIONS(1),
    [anon_sym_12] = ACTIONS(1),
    [anon_sym_13] = ACTIONS(1),
    [anon_sym_subset] = ACTIONS(1),
    [anon_sym_14] = ACTIONS(1),
    [anon_sym_superset] = ACTIONS(1),
    [anon_sym_15] = ACTIONS(1),
    [anon_sym_TILDE_EQ] = ACTIONS(1),
    [anon_sym_TILDE_BANG_EQ] = ACTIONS(1),
    [anon_sym_union] = ACTIONS(1),
    [anon_sym_16] = ACTIONS(1),
    [anon_sym_diff] = ACTIONS(1),
    [anon_sym_17] = ACTIONS(1),
    [anon_sym_symdiff] = ACTIONS(1),
    [anon_sym_intersect] = ACTIONS(1),
    [anon_sym_18] = ACTIONS(1),
    [anon_sym_PLUS] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_TILDE_PLUS] = ACTIONS(1),
    [anon_sym_TILDE_DASH] = ACTIONS(1),
    [anon_sym_STAR] = ACTIONS(1),
    [anon_sym_SLASH] = ACTIONS(1),
    [anon_sym_div] = ACTIONS(1),
    [anon_sym_mod] = ACTIONS(1),
    [anon_sym_TILDE_STAR] = ACTIONS(1),
    [anon_sym_TILDEdiv] = ACTIONS(1),
    [anon_sym_TILDE_SLASH] = ACTIONS(1),
    [anon_sym_CARET] = ACTIONS(1),
    [anon_sym_default] = ACTIONS(1),
    [anon_sym_case] = ACTIONS(1),
    [anon_sym_of] = ACTIONS(1),
    [anon_sym_endcase] = ACTIONS(1),
    [anon_sym_EQ_GT] = ACTIONS(1),
    [anon_sym_lambda] = ACTIONS(1),
    [anon_sym_let] = ACTIONS(1),
    [anon_sym_not] = ACTIONS(1),
    [anon_sym_19] = ACTIONS(1),
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
    [anon_sym_20] = ACTIONS(1),
    [anon_sym_21] = ACTIONS(1),
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
    [sym_source_file] = STATE(104),
    [sym_assignment] = STATE(99),
    [sym__identifier] = STATE(114),
    [aux_sym_source_file_repeat1] = STATE(69),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_identifier] = ACTIONS(7),
    [sym_quoted_identifier] = ACTIONS(9),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 18,
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
    ACTIONS(31), 1,
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    STATE(5), 1,
      aux_sym_array_literal_2d_repeat1,
    STATE(81), 1,
      aux_sym_array_literal_2d_repeat2,
    STATE(115), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [66] = 18,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(37), 1,
      sym_identifier,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    STATE(84), 1,
      sym_record_member,
    STATE(105), 1,
      sym__identifier,
    STATE(107), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(11), 2,
      sym_float_literal,
      sym_integer_literal,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [131] = 17,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(41), 1,
      anon_sym_RBRACK,
    STATE(7), 1,
      aux_sym_array_literal_repeat1,
    STATE(80), 1,
      sym__expression,
    STATE(93), 1,
      sym_array_literal_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [194] = 17,
    ACTIONS(46), 1,
      anon_sym_LPAREN,
    ACTIONS(49), 1,
      anon_sym_LBRACE,
    ACTIONS(52), 1,
      anon_sym_LBRACK,
    ACTIONS(55), 1,
      anon_sym_PIPE,
    ACTIONS(57), 1,
      anon_sym_DQUOTE,
    ACTIONS(60), 1,
      sym_absent,
    ACTIONS(63), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(66), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(71), 1,
      anon_sym_infinity,
    ACTIONS(74), 1,
      anon_sym_20,
    ACTIONS(77), 1,
      anon_sym_21,
    STATE(5), 1,
      aux_sym_array_literal_2d_repeat1,
    STATE(115), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(68), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(43), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [257] = 16,
    ACTIONS(83), 1,
      anon_sym_LPAREN,
    ACTIONS(88), 1,
      anon_sym_LBRACE,
    ACTIONS(91), 1,
      anon_sym_LBRACK,
    ACTIONS(94), 1,
      anon_sym_DQUOTE,
    ACTIONS(97), 1,
      sym_absent,
    ACTIONS(100), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(106), 1,
      anon_sym_infinity,
    ACTIONS(109), 1,
      anon_sym_20,
    ACTIONS(112), 1,
      anon_sym_21,
    STATE(6), 1,
      aux_sym_set_literal_repeat1,
    STATE(106), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(86), 2,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
    ACTIONS(103), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(80), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [318] = 17,
    ACTIONS(118), 1,
      anon_sym_LPAREN,
    ACTIONS(121), 1,
      anon_sym_LBRACE,
    ACTIONS(124), 1,
      anon_sym_LBRACK,
    ACTIONS(127), 1,
      anon_sym_RBRACK,
    ACTIONS(129), 1,
      anon_sym_DQUOTE,
    ACTIONS(132), 1,
      sym_absent,
    ACTIONS(135), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(141), 1,
      anon_sym_infinity,
    ACTIONS(144), 1,
      anon_sym_20,
    ACTIONS(147), 1,
      anon_sym_21,
    STATE(7), 1,
      aux_sym_array_literal_repeat1,
    STATE(80), 1,
      sym__expression,
    STATE(113), 1,
      sym_array_literal_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(138), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(115), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [381] = 17,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(150), 1,
      anon_sym_RBRACK,
    STATE(4), 1,
      aux_sym_array_literal_repeat1,
    STATE(80), 1,
      sym__expression,
    STATE(100), 1,
      sym_array_literal_member,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [444] = 17,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(152), 1,
      anon_sym_PIPE_RBRACK,
    STATE(2), 1,
      aux_sym_array_literal_2d_repeat1,
    STATE(74), 1,
      sym__expression,
    STATE(87), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [507] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(154), 1,
      anon_sym_PIPE,
    ACTIONS(156), 1,
      anon_sym_PIPE_RBRACK,
    STATE(89), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [567] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(158), 1,
      anon_sym_PIPE,
    ACTIONS(160), 1,
      anon_sym_PIPE_RBRACK,
    STATE(89), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [627] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(162), 1,
      anon_sym_PIPE_RBRACK,
    STATE(71), 1,
      sym__expression,
    STATE(94), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [687] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(164), 1,
      anon_sym_PIPE_RBRACK,
    STATE(71), 1,
      sym__expression,
    STATE(94), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [747] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(166), 1,
      anon_sym_RBRACE,
    STATE(22), 1,
      aux_sym_set_literal_repeat1,
    STATE(102), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [807] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(168), 1,
      anon_sym_RPAREN,
    STATE(6), 1,
      aux_sym_set_literal_repeat1,
    STATE(103), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [867] = 16,
    ACTIONS(173), 1,
      anon_sym_LPAREN,
    ACTIONS(176), 1,
      anon_sym_LBRACE,
    ACTIONS(179), 1,
      anon_sym_LBRACK,
    ACTIONS(182), 1,
      anon_sym_PIPE,
    ACTIONS(184), 1,
      anon_sym_DQUOTE,
    ACTIONS(187), 1,
      sym_absent,
    ACTIONS(190), 1,
      anon_sym_LBRACK_PIPE,
    ACTIONS(193), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(198), 1,
      anon_sym_infinity,
    ACTIONS(201), 1,
      anon_sym_20,
    ACTIONS(204), 1,
      anon_sym_21,
    STATE(75), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(195), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(170), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [927] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(207), 1,
      anon_sym_RPAREN,
    STATE(15), 1,
      aux_sym_set_literal_repeat1,
    STATE(90), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [987] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(209), 1,
      anon_sym_PIPE,
    ACTIONS(211), 1,
      anon_sym_PIPE_RBRACK,
    STATE(89), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1047] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(213), 1,
      anon_sym_PIPE_RBRACK,
    STATE(71), 1,
      sym__expression,
    STATE(94), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1107] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(215), 1,
      anon_sym_PIPE_RBRACK,
    STATE(71), 1,
      sym__expression,
    STATE(94), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1167] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(217), 1,
      anon_sym_PIPE,
    ACTIONS(219), 1,
      anon_sym_PIPE_RBRACK,
    STATE(89), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1227] = 16,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    ACTIONS(221), 1,
      anon_sym_RBRACE,
    STATE(6), 1,
      aux_sym_set_literal_repeat1,
    STATE(91), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1287] = 15,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    STATE(71), 1,
      sym__expression,
    STATE(94), 1,
      sym_array_literal_2d_row,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1344] = 14,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    STATE(98), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1398] = 14,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    STATE(95), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1452] = 14,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    STATE(101), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1506] = 14,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    STATE(89), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1560] = 14,
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
      anon_sym_infinity,
    ACTIONS(33), 1,
      anon_sym_20,
    ACTIONS(35), 1,
      anon_sym_21,
    STATE(75), 1,
      sym__expression,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(29), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(11), 3,
      sym_float_literal,
      sym_integer_literal,
      sym_identifier,
    STATE(64), 8,
      sym_array_literal,
      sym_array_literal_2d,
      sym_boolean_literal,
      sym_infinity,
      sym_set_literal,
      sym_string_literal,
      sym_tuple_literal,
      sym_record_literal,
  [1614] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(223), 8,
      anon_sym_LBRACK,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      anon_sym_infinity,
      anon_sym_21,
      sym_identifier,
    ACTIONS(225), 8,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      anon_sym_20,
  [1639] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(193), 7,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      anon_sym_PIPE_RBRACK,
      anon_sym_20,
    ACTIONS(182), 9,
      anon_sym_LBRACK,
      anon_sym_PIPE,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      anon_sym_infinity,
      anon_sym_21,
      sym_identifier,
  [1664] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(229), 7,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACK,
      anon_sym_DQUOTE,
      sym_absent,
      anon_sym_LBRACK_PIPE,
      anon_sym_20,
    ACTIONS(227), 8,
      anon_sym_LBRACK,
      anon_sym_true,
      anon_sym_false,
      sym_float_literal,
      sym_integer_literal,
      anon_sym_infinity,
      anon_sym_21,
      sym_identifier,
  [1688] = 10,
    ACTIONS(231), 1,
      anon_sym_DQUOTE,
    ACTIONS(233), 1,
      sym_string_characters,
    ACTIONS(237), 1,
      anon_sym_BSLASH,
    ACTIONS(239), 1,
      anon_sym_BSLASHx,
    ACTIONS(241), 1,
      anon_sym_BSLASHu,
    ACTIONS(243), 1,
      anon_sym_BSLASHU,
    STATE(33), 1,
      aux_sym__string_content,
    STATE(35), 1,
      sym_escape_sequence,
    ACTIONS(245), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(235), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [1725] = 10,
    ACTIONS(247), 1,
      anon_sym_DQUOTE,
    ACTIONS(249), 1,
      sym_string_characters,
    ACTIONS(255), 1,
      anon_sym_BSLASH,
    ACTIONS(258), 1,
      anon_sym_BSLASHx,
    ACTIONS(261), 1,
      anon_sym_BSLASHu,
    ACTIONS(264), 1,
      anon_sym_BSLASHU,
    STATE(33), 1,
      aux_sym__string_content,
    STATE(35), 1,
      sym_escape_sequence,
    ACTIONS(245), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(252), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [1762] = 10,
    ACTIONS(233), 1,
      sym_string_characters,
    ACTIONS(237), 1,
      anon_sym_BSLASH,
    ACTIONS(239), 1,
      anon_sym_BSLASHx,
    ACTIONS(241), 1,
      anon_sym_BSLASHu,
    ACTIONS(243), 1,
      anon_sym_BSLASHU,
    ACTIONS(267), 1,
      anon_sym_DQUOTE,
    STATE(32), 1,
      aux_sym__string_content,
    STATE(35), 1,
      sym_escape_sequence,
    ACTIONS(245), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(235), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [1799] = 3,
    ACTIONS(271), 1,
      sym_string_characters,
    ACTIONS(245), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(269), 11,
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
  [1820] = 3,
    ACTIONS(275), 1,
      sym_string_characters,
    ACTIONS(245), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(273), 11,
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
  [1841] = 3,
    ACTIONS(279), 1,
      sym_string_characters,
    ACTIONS(245), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(277), 11,
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
  [1862] = 3,
    ACTIONS(283), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(281), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [1880] = 3,
    ACTIONS(287), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(285), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [1898] = 3,
    ACTIONS(291), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(289), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [1916] = 3,
    ACTIONS(295), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(293), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [1934] = 3,
    ACTIONS(299), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(297), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [1952] = 3,
    ACTIONS(303), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(301), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [1970] = 3,
    ACTIONS(307), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(305), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [1988] = 3,
    ACTIONS(311), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(309), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2006] = 3,
    ACTIONS(315), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(313), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2024] = 3,
    ACTIONS(319), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(317), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2042] = 3,
    ACTIONS(323), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(321), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2060] = 3,
    ACTIONS(327), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(325), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2078] = 3,
    ACTIONS(331), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(329), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2096] = 3,
    ACTIONS(335), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(333), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2114] = 3,
    ACTIONS(339), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(337), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2132] = 3,
    ACTIONS(343), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(341), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2150] = 3,
    ACTIONS(347), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(345), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2168] = 3,
    ACTIONS(351), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(349), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2186] = 3,
    ACTIONS(355), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(353), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2204] = 3,
    ACTIONS(359), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(357), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2222] = 3,
    ACTIONS(363), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(361), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2240] = 3,
    ACTIONS(367), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(365), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2258] = 3,
    ACTIONS(371), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(369), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2276] = 3,
    ACTIONS(375), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(373), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2294] = 3,
    ACTIONS(379), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(377), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2312] = 3,
    ACTIONS(383), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(381), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2330] = 3,
    ACTIONS(387), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(385), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2348] = 3,
    ACTIONS(391), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(389), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2366] = 3,
    ACTIONS(395), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(393), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2384] = 3,
    ACTIONS(399), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(397), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2402] = 3,
    ACTIONS(403), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(401), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_RBRACE,
      anon_sym_RBRACK,
      anon_sym_PIPE_RBRACK,
  [2420] = 7,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(9), 1,
      sym_quoted_identifier,
    ACTIONS(405), 1,
      ts_builtin_sym_end,
    STATE(70), 1,
      aux_sym_source_file_repeat1,
    STATE(97), 1,
      sym_assignment,
    STATE(114), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2443] = 7,
    ACTIONS(407), 1,
      ts_builtin_sym_end,
    ACTIONS(409), 1,
      sym_identifier,
    ACTIONS(412), 1,
      sym_quoted_identifier,
    STATE(70), 1,
      aux_sym_source_file_repeat1,
    STATE(110), 1,
      sym_assignment,
    STATE(114), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2466] = 6,
    ACTIONS(415), 1,
      anon_sym_COLON,
    ACTIONS(417), 1,
      anon_sym_COMMA,
    ACTIONS(419), 1,
      anon_sym_PIPE,
    ACTIONS(421), 1,
      anon_sym_PIPE_RBRACK,
    STATE(77), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2486] = 6,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(423), 1,
      sym_identifier,
    ACTIONS(425), 1,
      anon_sym_RPAREN,
    STATE(92), 1,
      sym_record_member,
    STATE(105), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2506] = 6,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(423), 1,
      sym_identifier,
    ACTIONS(427), 1,
      anon_sym_RPAREN,
    STATE(92), 1,
      sym_record_member,
    STATE(105), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2526] = 6,
    ACTIONS(417), 1,
      anon_sym_COMMA,
    ACTIONS(419), 1,
      anon_sym_PIPE,
    ACTIONS(421), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(429), 1,
      anon_sym_COLON,
    STATE(77), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2546] = 5,
    ACTIONS(431), 1,
      anon_sym_COMMA,
    ACTIONS(433), 1,
      anon_sym_PIPE,
    ACTIONS(435), 1,
      anon_sym_PIPE_RBRACK,
    STATE(79), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2563] = 5,
    ACTIONS(437), 1,
      anon_sym_COMMA,
    ACTIONS(440), 1,
      anon_sym_PIPE,
    ACTIONS(442), 1,
      anon_sym_PIPE_RBRACK,
    STATE(76), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2580] = 5,
    ACTIONS(444), 1,
      anon_sym_COMMA,
    ACTIONS(446), 1,
      anon_sym_PIPE,
    ACTIONS(448), 1,
      anon_sym_PIPE_RBRACK,
    STATE(76), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2597] = 5,
    ACTIONS(39), 1,
      sym_quoted_identifier,
    ACTIONS(423), 1,
      sym_identifier,
    STATE(92), 1,
      sym_record_member,
    STATE(105), 1,
      sym__identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2614] = 5,
    ACTIONS(450), 1,
      anon_sym_COMMA,
    ACTIONS(452), 1,
      anon_sym_PIPE,
    ACTIONS(454), 1,
      anon_sym_PIPE_RBRACK,
    STATE(76), 1,
      aux_sym_array_literal_2d_row_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2631] = 3,
    ACTIONS(456), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(458), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [2643] = 4,
    ACTIONS(460), 1,
      anon_sym_PIPE,
    ACTIONS(462), 1,
      anon_sym_PIPE_RBRACK,
    STATE(85), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2657] = 4,
    ACTIONS(464), 1,
      anon_sym_PIPE,
    ACTIONS(466), 1,
      anon_sym_PIPE_RBRACK,
    STATE(85), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2671] = 4,
    ACTIONS(468), 1,
      anon_sym_COMMA,
    ACTIONS(471), 1,
      anon_sym_RPAREN,
    STATE(83), 1,
      aux_sym_record_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2685] = 4,
    ACTIONS(473), 1,
      anon_sym_COMMA,
    ACTIONS(475), 1,
      anon_sym_RPAREN,
    STATE(88), 1,
      aux_sym_record_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2699] = 4,
    ACTIONS(477), 1,
      anon_sym_PIPE,
    ACTIONS(480), 1,
      anon_sym_PIPE_RBRACK,
    STATE(85), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2713] = 3,
    ACTIONS(484), 1,
      sym_identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(482), 2,
      ts_builtin_sym_end,
      sym_quoted_identifier,
  [2725] = 4,
    ACTIONS(486), 1,
      anon_sym_PIPE,
    ACTIONS(488), 1,
      anon_sym_PIPE_RBRACK,
    STATE(82), 1,
      aux_sym_array_literal_2d_repeat2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2739] = 4,
    ACTIONS(490), 1,
      anon_sym_COMMA,
    ACTIONS(492), 1,
      anon_sym_RPAREN,
    STATE(83), 1,
      aux_sym_record_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2753] = 3,
    ACTIONS(496), 1,
      anon_sym_PIPE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(494), 2,
      anon_sym_COMMA,
      anon_sym_PIPE_RBRACK,
  [2765] = 3,
    ACTIONS(498), 1,
      anon_sym_COMMA,
    ACTIONS(500), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2776] = 3,
    ACTIONS(498), 1,
      anon_sym_COMMA,
    ACTIONS(502), 1,
      anon_sym_RBRACE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2787] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(504), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [2796] = 3,
    ACTIONS(506), 1,
      anon_sym_COMMA,
    ACTIONS(508), 1,
      anon_sym_RBRACK,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2807] = 3,
    ACTIONS(510), 1,
      anon_sym_PIPE,
    ACTIONS(512), 1,
      anon_sym_PIPE_RBRACK,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2818] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(514), 2,
      ts_builtin_sym_end,
      anon_sym_SEMI,
  [2827] = 3,
    ACTIONS(385), 1,
      anon_sym_COMMA,
    ACTIONS(516), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2838] = 3,
    ACTIONS(518), 1,
      ts_builtin_sym_end,
    ACTIONS(520), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2849] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(522), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [2858] = 3,
    ACTIONS(520), 1,
      anon_sym_SEMI,
    ACTIONS(524), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2869] = 3,
    ACTIONS(506), 1,
      anon_sym_COMMA,
    ACTIONS(526), 1,
      anon_sym_RBRACK,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2880] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(528), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [2889] = 3,
    ACTIONS(498), 1,
      anon_sym_COMMA,
    ACTIONS(530), 1,
      anon_sym_RBRACE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2900] = 3,
    ACTIONS(498), 1,
      anon_sym_COMMA,
    ACTIONS(532), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2911] = 2,
    ACTIONS(534), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2919] = 2,
    ACTIONS(536), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2927] = 2,
    ACTIONS(498), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2935] = 2,
    ACTIONS(538), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2943] = 2,
    ACTIONS(540), 1,
      aux_sym_escape_sequence_token1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2951] = 2,
    ACTIONS(540), 1,
      aux_sym_escape_sequence_token2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2959] = 2,
    ACTIONS(520), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2967] = 2,
    ACTIONS(540), 1,
      aux_sym_escape_sequence_token3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2975] = 2,
    ACTIONS(540), 1,
      aux_sym_escape_sequence_token4,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2983] = 2,
    ACTIONS(506), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2991] = 2,
    ACTIONS(542), 1,
      anon_sym_EQ,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [2999] = 2,
    ACTIONS(544), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 66,
  [SMALL_STATE(4)] = 131,
  [SMALL_STATE(5)] = 194,
  [SMALL_STATE(6)] = 257,
  [SMALL_STATE(7)] = 318,
  [SMALL_STATE(8)] = 381,
  [SMALL_STATE(9)] = 444,
  [SMALL_STATE(10)] = 507,
  [SMALL_STATE(11)] = 567,
  [SMALL_STATE(12)] = 627,
  [SMALL_STATE(13)] = 687,
  [SMALL_STATE(14)] = 747,
  [SMALL_STATE(15)] = 807,
  [SMALL_STATE(16)] = 867,
  [SMALL_STATE(17)] = 927,
  [SMALL_STATE(18)] = 987,
  [SMALL_STATE(19)] = 1047,
  [SMALL_STATE(20)] = 1107,
  [SMALL_STATE(21)] = 1167,
  [SMALL_STATE(22)] = 1227,
  [SMALL_STATE(23)] = 1287,
  [SMALL_STATE(24)] = 1344,
  [SMALL_STATE(25)] = 1398,
  [SMALL_STATE(26)] = 1452,
  [SMALL_STATE(27)] = 1506,
  [SMALL_STATE(28)] = 1560,
  [SMALL_STATE(29)] = 1614,
  [SMALL_STATE(30)] = 1639,
  [SMALL_STATE(31)] = 1664,
  [SMALL_STATE(32)] = 1688,
  [SMALL_STATE(33)] = 1725,
  [SMALL_STATE(34)] = 1762,
  [SMALL_STATE(35)] = 1799,
  [SMALL_STATE(36)] = 1820,
  [SMALL_STATE(37)] = 1841,
  [SMALL_STATE(38)] = 1862,
  [SMALL_STATE(39)] = 1880,
  [SMALL_STATE(40)] = 1898,
  [SMALL_STATE(41)] = 1916,
  [SMALL_STATE(42)] = 1934,
  [SMALL_STATE(43)] = 1952,
  [SMALL_STATE(44)] = 1970,
  [SMALL_STATE(45)] = 1988,
  [SMALL_STATE(46)] = 2006,
  [SMALL_STATE(47)] = 2024,
  [SMALL_STATE(48)] = 2042,
  [SMALL_STATE(49)] = 2060,
  [SMALL_STATE(50)] = 2078,
  [SMALL_STATE(51)] = 2096,
  [SMALL_STATE(52)] = 2114,
  [SMALL_STATE(53)] = 2132,
  [SMALL_STATE(54)] = 2150,
  [SMALL_STATE(55)] = 2168,
  [SMALL_STATE(56)] = 2186,
  [SMALL_STATE(57)] = 2204,
  [SMALL_STATE(58)] = 2222,
  [SMALL_STATE(59)] = 2240,
  [SMALL_STATE(60)] = 2258,
  [SMALL_STATE(61)] = 2276,
  [SMALL_STATE(62)] = 2294,
  [SMALL_STATE(63)] = 2312,
  [SMALL_STATE(64)] = 2330,
  [SMALL_STATE(65)] = 2348,
  [SMALL_STATE(66)] = 2366,
  [SMALL_STATE(67)] = 2384,
  [SMALL_STATE(68)] = 2402,
  [SMALL_STATE(69)] = 2420,
  [SMALL_STATE(70)] = 2443,
  [SMALL_STATE(71)] = 2466,
  [SMALL_STATE(72)] = 2486,
  [SMALL_STATE(73)] = 2506,
  [SMALL_STATE(74)] = 2526,
  [SMALL_STATE(75)] = 2546,
  [SMALL_STATE(76)] = 2563,
  [SMALL_STATE(77)] = 2580,
  [SMALL_STATE(78)] = 2597,
  [SMALL_STATE(79)] = 2614,
  [SMALL_STATE(80)] = 2631,
  [SMALL_STATE(81)] = 2643,
  [SMALL_STATE(82)] = 2657,
  [SMALL_STATE(83)] = 2671,
  [SMALL_STATE(84)] = 2685,
  [SMALL_STATE(85)] = 2699,
  [SMALL_STATE(86)] = 2713,
  [SMALL_STATE(87)] = 2725,
  [SMALL_STATE(88)] = 2739,
  [SMALL_STATE(89)] = 2753,
  [SMALL_STATE(90)] = 2765,
  [SMALL_STATE(91)] = 2776,
  [SMALL_STATE(92)] = 2787,
  [SMALL_STATE(93)] = 2796,
  [SMALL_STATE(94)] = 2807,
  [SMALL_STATE(95)] = 2818,
  [SMALL_STATE(96)] = 2827,
  [SMALL_STATE(97)] = 2838,
  [SMALL_STATE(98)] = 2849,
  [SMALL_STATE(99)] = 2858,
  [SMALL_STATE(100)] = 2869,
  [SMALL_STATE(101)] = 2880,
  [SMALL_STATE(102)] = 2889,
  [SMALL_STATE(103)] = 2900,
  [SMALL_STATE(104)] = 2911,
  [SMALL_STATE(105)] = 2919,
  [SMALL_STATE(106)] = 2927,
  [SMALL_STATE(107)] = 2935,
  [SMALL_STATE(108)] = 2943,
  [SMALL_STATE(109)] = 2951,
  [SMALL_STATE(110)] = 2959,
  [SMALL_STATE(111)] = 2967,
  [SMALL_STATE(112)] = 2975,
  [SMALL_STATE(113)] = 2983,
  [SMALL_STATE(114)] = 2991,
  [SMALL_STATE(115)] = 2999,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(114),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(114),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(64),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [17] = {.entry = {.count = 1, .reusable = false}}, SHIFT(8),
  [19] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(63),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(38),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [35] = {.entry = {.count = 1, .reusable = false}}, SHIFT(60),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(96),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(105),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [43] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(64),
  [46] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(3),
  [49] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(14),
  [52] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(8),
  [55] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20),
  [57] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(34),
  [60] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(64),
  [63] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(9),
  [66] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20),
  [68] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(63),
  [71] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(38),
  [74] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(38),
  [77] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 20), SHIFT_REPEAT(60),
  [80] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(64),
  [83] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(3),
  [86] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12),
  [88] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(14),
  [91] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(8),
  [94] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(34),
  [97] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(64),
  [100] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(9),
  [103] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(63),
  [106] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(38),
  [109] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(38),
  [112] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(60),
  [115] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(64),
  [118] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(3),
  [121] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(14),
  [124] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(8),
  [127] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12),
  [129] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(34),
  [132] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(64),
  [135] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(9),
  [138] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(63),
  [141] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(38),
  [144] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(38),
  [147] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(60),
  [150] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [152] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [154] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 2, .production_id = 9),
  [156] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 2, .production_id = 9),
  [158] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 5, .production_id = 31),
  [160] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 5, .production_id = 31),
  [162] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [164] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [166] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [168] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [170] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(64),
  [173] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(3),
  [176] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(14),
  [179] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(8),
  [182] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16),
  [184] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(34),
  [187] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(64),
  [190] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(9),
  [193] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16),
  [195] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(63),
  [198] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(38),
  [201] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(38),
  [204] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat1, 2, .production_id = 16), SHIFT(60),
  [207] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [209] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 3, .production_id = 17),
  [211] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 3, .production_id = 17),
  [213] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [215] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [217] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 4, .production_id = 25),
  [219] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 4, .production_id = 25),
  [221] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [223] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 9),
  [225] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_set_literal_repeat1, 2, .production_id = 9),
  [227] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 9),
  [229] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2, .production_id = 9),
  [231] = {.entry = {.count = 1, .reusable = false}}, SHIFT(61),
  [233] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [235] = {.entry = {.count = 1, .reusable = false}}, SHIFT(36),
  [237] = {.entry = {.count = 1, .reusable = false}}, SHIFT(108),
  [239] = {.entry = {.count = 1, .reusable = false}}, SHIFT(109),
  [241] = {.entry = {.count = 1, .reusable = false}}, SHIFT(111),
  [243] = {.entry = {.count = 1, .reusable = false}}, SHIFT(112),
  [245] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [247] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15),
  [249] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(35),
  [252] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(36),
  [255] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(108),
  [258] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(109),
  [261] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(111),
  [264] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 15), SHIFT_REPEAT(112),
  [267] = {.entry = {.count = 1, .reusable = false}}, SHIFT(48),
  [269] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__string_content, 1, .production_id = 7),
  [271] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__string_content, 1, .production_id = 7),
  [273] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_escape_sequence, 1, .production_id = 8),
  [275] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_escape_sequence, 1, .production_id = 8),
  [277] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_escape_sequence, 2, .production_id = 13),
  [279] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_escape_sequence, 2, .production_id = 13),
  [281] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_infinity, 1),
  [283] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_infinity, 1),
  [285] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 3, .production_id = 11),
  [287] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 3, .production_id = 11),
  [289] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 2),
  [291] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 2),
  [293] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 4, .production_id = 21),
  [295] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 4, .production_id = 21),
  [297] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, .production_id = 18),
  [299] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, .production_id = 18),
  [301] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 6, .production_id = 32),
  [303] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 6, .production_id = 32),
  [305] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, .production_id = 26),
  [307] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, .production_id = 26),
  [309] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 4, .production_id = 10),
  [311] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 4, .production_id = 10),
  [313] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 3, .production_id = 10),
  [315] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 3, .production_id = 10),
  [317] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, .production_id = 19),
  [319] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, .production_id = 19),
  [321] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 2),
  [323] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 2),
  [325] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 2),
  [327] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 2),
  [329] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 3, .production_id = 10),
  [331] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 3, .production_id = 10),
  [333] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 3, .production_id = 11),
  [335] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 3, .production_id = 11),
  [337] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 4, .production_id = 28),
  [339] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 4, .production_id = 28),
  [341] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 2),
  [343] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 2),
  [345] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 5, .production_id = 29),
  [347] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 5, .production_id = 29),
  [349] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 5, .production_id = 30),
  [351] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 5, .production_id = 30),
  [353] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 3, .production_id = 10),
  [355] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 3, .production_id = 10),
  [357] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 5, .production_id = 28),
  [359] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 5, .production_id = 28),
  [361] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 4, .production_id = 23),
  [363] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 4, .production_id = 23),
  [365] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_literal, 5, .production_id = 21),
  [367] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_record_literal, 5, .production_id = 21),
  [369] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 1),
  [371] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 1),
  [373] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 3, .production_id = 14),
  [375] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 3, .production_id = 14),
  [377] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 4, .production_id = 23),
  [379] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 4, .production_id = 23),
  [381] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean_literal, 1),
  [383] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean_literal, 1),
  [385] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__expression, 1),
  [387] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__expression, 1),
  [389] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tuple_literal, 4, .production_id = 10),
  [391] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tuple_literal, 4, .production_id = 10),
  [393] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 3, .production_id = 18),
  [395] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 3, .production_id = 18),
  [397] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 5, .production_id = 26),
  [399] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 5, .production_id = 26),
  [401] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d, 3, .production_id = 19),
  [403] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d, 3, .production_id = 19),
  [405] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, .production_id = 2),
  [407] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 4),
  [409] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 4), SHIFT_REPEAT(114),
  [412] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 4), SHIFT_REPEAT(114),
  [415] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [417] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [419] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 1, .production_id = 9),
  [421] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 1, .production_id = 9),
  [423] = {.entry = {.count = 1, .reusable = false}}, SHIFT(105),
  [425] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [427] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [429] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [431] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [433] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 3, .production_id = 25),
  [435] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 3, .production_id = 25),
  [437] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 12), SHIFT_REPEAT(27),
  [440] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 12),
  [442] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 12),
  [444] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [446] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 2, .production_id = 17),
  [448] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 2, .production_id = 17),
  [450] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [452] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal_2d_row, 4, .production_id = 31),
  [454] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_2d_row, 4, .production_id = 31),
  [456] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [458] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_member, 1, .production_id = 6),
  [460] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [462] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [464] = {.entry = {.count = 1, .reusable = false}}, SHIFT(13),
  [466] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [468] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_record_literal_repeat1, 2, .production_id = 12), SHIFT_REPEAT(78),
  [471] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_record_literal_repeat1, 2, .production_id = 12),
  [473] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [475] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [477] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, .production_id = 27), SHIFT_REPEAT(23),
  [480] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, .production_id = 27),
  [482] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 1),
  [484] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 1),
  [486] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [488] = {.entry = {.count = 1, .reusable = true}}, SHIFT(66),
  [490] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [492] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [494] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 10),
  [496] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_row_repeat1, 2, .production_id = 10),
  [498] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [500] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [502] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [504] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_record_literal_repeat1, 2, .production_id = 10),
  [506] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [508] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [510] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, .production_id = 18),
  [512] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_2d_repeat2, 2, .production_id = 18),
  [514] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 3, .production_id = 5),
  [516] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__identifier, 1),
  [518] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 2, .production_id = 3),
  [520] = {.entry = {.count = 1, .reusable = true}}, SHIFT(86),
  [522] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal_member, 3, .production_id = 24),
  [524] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, .production_id = 1),
  [526] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [528] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_record_member, 3, .production_id = 22),
  [530] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [532] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [534] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [536] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [538] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [540] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [542] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [544] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
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
