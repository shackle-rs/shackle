#include <tree_sitter/parser.h>

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 11
#define STATE_COUNT 79
#define LARGE_STATE_COUNT 40
#define SYMBOL_COUNT 69
#define ALIAS_COUNT 0
#define TOKEN_COUNT 52
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 8
#define MAX_ALIAS_SEQUENCE_LENGTH 5

enum {
  sym_identifier = 1,
  anon_sym_SEMI = 2,
  anon_sym_EQ = 3,
  anon_sym_LT_DASH_GT = 4,
  anon_sym_DASH_GT = 5,
  anon_sym_LT_DASH = 6,
  anon_sym_BSLASH_SLASH = 7,
  anon_sym_xor = 8,
  anon_sym_SLASH_BSLASH = 9,
  anon_sym_EQ_EQ = 10,
  anon_sym_BANG_EQ = 11,
  anon_sym_LT = 12,
  anon_sym_LT_EQ = 13,
  anon_sym_GT = 14,
  anon_sym_GT_EQ = 15,
  anon_sym_in = 16,
  anon_sym_subset = 17,
  anon_sym_superset = 18,
  anon_sym_union = 19,
  anon_sym_diff = 20,
  anon_sym_symdiff = 21,
  anon_sym_intersect = 22,
  anon_sym_DOT_DOT = 23,
  anon_sym_PLUS = 24,
  anon_sym_DASH = 25,
  anon_sym_PLUS_PLUS = 26,
  anon_sym_STAR = 27,
  anon_sym_SLASH = 28,
  anon_sym_div = 29,
  anon_sym_mod = 30,
  anon_sym_CARET = 31,
  anon_sym_COLON_COLON = 32,
  anon_sym_LPAREN = 33,
  anon_sym_COMMA = 34,
  anon_sym_RPAREN = 35,
  anon_sym_LBRACK = 36,
  anon_sym_RBRACK = 37,
  anon_sym_not = 38,
  anon_sym_ = 39,
  sym_absent = 40,
  anon_sym_true = 41,
  anon_sym_false = 42,
  sym_float_literal = 43,
  sym_integer_literal = 44,
  anon_sym_LBRACE = 45,
  anon_sym_RBRACE = 46,
  anon_sym_DQUOTE = 47,
  aux_sym_string_literal_token1 = 48,
  sym_escape_sequence = 49,
  sym_line_comment = 50,
  sym_block_comment = 51,
  sym_source_file = 52,
  sym__items = 53,
  sym_assignment_item = 54,
  sym__expression = 55,
  sym_binary_operation = 56,
  sym_call = 57,
  sym_index_expression = 58,
  sym_unary_operation = 59,
  sym__literal = 60,
  sym_array_literal = 61,
  sym_boolean_literal = 62,
  sym_set_literal = 63,
  sym_string_literal = 64,
  aux_sym_source_file_repeat1 = 65,
  aux_sym_call_repeat1 = 66,
  aux_sym_index_expression_repeat1 = 67,
  aux_sym_string_literal_repeat1 = 68,
};

static const char *ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_identifier] = "identifier",
  [anon_sym_SEMI] = ";",
  [anon_sym_EQ] = "=",
  [anon_sym_LT_DASH_GT] = "<->",
  [anon_sym_DASH_GT] = "->",
  [anon_sym_LT_DASH] = "<-",
  [anon_sym_BSLASH_SLASH] = "\\/",
  [anon_sym_xor] = "xor",
  [anon_sym_SLASH_BSLASH] = "/\\",
  [anon_sym_EQ_EQ] = "==",
  [anon_sym_BANG_EQ] = "!=",
  [anon_sym_LT] = "<",
  [anon_sym_LT_EQ] = "<=",
  [anon_sym_GT] = ">",
  [anon_sym_GT_EQ] = ">=",
  [anon_sym_in] = "in",
  [anon_sym_subset] = "subset",
  [anon_sym_superset] = "superset",
  [anon_sym_union] = "union",
  [anon_sym_diff] = "diff",
  [anon_sym_symdiff] = "symdiff",
  [anon_sym_intersect] = "intersect",
  [anon_sym_DOT_DOT] = "..",
  [anon_sym_PLUS] = "+",
  [anon_sym_DASH] = "-",
  [anon_sym_PLUS_PLUS] = "++",
  [anon_sym_STAR] = "*",
  [anon_sym_SLASH] = "/",
  [anon_sym_div] = "div",
  [anon_sym_mod] = "mod",
  [anon_sym_CARET] = "^",
  [anon_sym_COLON_COLON] = "::",
  [anon_sym_LPAREN] = "(",
  [anon_sym_COMMA] = ",",
  [anon_sym_RPAREN] = ")",
  [anon_sym_LBRACK] = "[",
  [anon_sym_RBRACK] = "]",
  [anon_sym_not] = "not",
  [anon_sym_] = "¬",
  [sym_absent] = "absent",
  [anon_sym_true] = "true",
  [anon_sym_false] = "false",
  [sym_float_literal] = "float_literal",
  [sym_integer_literal] = "integer_literal",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_DQUOTE] = "\"",
  [aux_sym_string_literal_token1] = "string_literal_token1",
  [sym_escape_sequence] = "escape_sequence",
  [sym_line_comment] = "line_comment",
  [sym_block_comment] = "block_comment",
  [sym_source_file] = "source_file",
  [sym__items] = "_items",
  [sym_assignment_item] = "assignment_item",
  [sym__expression] = "_expression",
  [sym_binary_operation] = "binary_operation",
  [sym_call] = "call",
  [sym_index_expression] = "index_expression",
  [sym_unary_operation] = "unary_operation",
  [sym__literal] = "_literal",
  [sym_array_literal] = "array_literal",
  [sym_boolean_literal] = "boolean_literal",
  [sym_set_literal] = "set_literal",
  [sym_string_literal] = "string_literal",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_call_repeat1] = "call_repeat1",
  [aux_sym_index_expression_repeat1] = "index_expression_repeat1",
  [aux_sym_string_literal_repeat1] = "string_literal_repeat1",
};

static TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_identifier] = sym_identifier,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_LT_DASH_GT] = anon_sym_LT_DASH_GT,
  [anon_sym_DASH_GT] = anon_sym_DASH_GT,
  [anon_sym_LT_DASH] = anon_sym_LT_DASH,
  [anon_sym_BSLASH_SLASH] = anon_sym_BSLASH_SLASH,
  [anon_sym_xor] = anon_sym_xor,
  [anon_sym_SLASH_BSLASH] = anon_sym_SLASH_BSLASH,
  [anon_sym_EQ_EQ] = anon_sym_EQ_EQ,
  [anon_sym_BANG_EQ] = anon_sym_BANG_EQ,
  [anon_sym_LT] = anon_sym_LT,
  [anon_sym_LT_EQ] = anon_sym_LT_EQ,
  [anon_sym_GT] = anon_sym_GT,
  [anon_sym_GT_EQ] = anon_sym_GT_EQ,
  [anon_sym_in] = anon_sym_in,
  [anon_sym_subset] = anon_sym_subset,
  [anon_sym_superset] = anon_sym_superset,
  [anon_sym_union] = anon_sym_union,
  [anon_sym_diff] = anon_sym_diff,
  [anon_sym_symdiff] = anon_sym_symdiff,
  [anon_sym_intersect] = anon_sym_intersect,
  [anon_sym_DOT_DOT] = anon_sym_DOT_DOT,
  [anon_sym_PLUS] = anon_sym_PLUS,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_PLUS_PLUS] = anon_sym_PLUS_PLUS,
  [anon_sym_STAR] = anon_sym_STAR,
  [anon_sym_SLASH] = anon_sym_SLASH,
  [anon_sym_div] = anon_sym_div,
  [anon_sym_mod] = anon_sym_mod,
  [anon_sym_CARET] = anon_sym_CARET,
  [anon_sym_COLON_COLON] = anon_sym_COLON_COLON,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [anon_sym_not] = anon_sym_not,
  [anon_sym_] = anon_sym_,
  [sym_absent] = sym_absent,
  [anon_sym_true] = anon_sym_true,
  [anon_sym_false] = anon_sym_false,
  [sym_float_literal] = sym_float_literal,
  [sym_integer_literal] = sym_integer_literal,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [aux_sym_string_literal_token1] = aux_sym_string_literal_token1,
  [sym_escape_sequence] = sym_escape_sequence,
  [sym_line_comment] = sym_line_comment,
  [sym_block_comment] = sym_block_comment,
  [sym_source_file] = sym_source_file,
  [sym__items] = sym__items,
  [sym_assignment_item] = sym_assignment_item,
  [sym__expression] = sym__expression,
  [sym_binary_operation] = sym_binary_operation,
  [sym_call] = sym_call,
  [sym_index_expression] = sym_index_expression,
  [sym_unary_operation] = sym_unary_operation,
  [sym__literal] = sym__literal,
  [sym_array_literal] = sym_array_literal,
  [sym_boolean_literal] = sym_boolean_literal,
  [sym_set_literal] = sym_set_literal,
  [sym_string_literal] = sym_string_literal,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_call_repeat1] = aux_sym_call_repeat1,
  [aux_sym_index_expression_repeat1] = aux_sym_index_expression_repeat1,
  [aux_sym_string_literal_repeat1] = aux_sym_string_literal_repeat1,
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
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_DASH_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASH_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_xor] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SLASH_BSLASH] = {
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
  [anon_sym_LT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_EQ] = {
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
  [anon_sym_in] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_subset] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_superset] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_union] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_diff] = {
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
  [anon_sym_DOT_DOT] = {
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
  [anon_sym_PLUS_PLUS] = {
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
  [anon_sym_CARET] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON_COLON] = {
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
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_not] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_] = {
    .visible = true,
    .named = false,
  },
  [sym_absent] = {
    .visible = true,
    .named = true,
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
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_string_literal_token1] = {
    .visible = false,
    .named = false,
  },
  [sym_escape_sequence] = {
    .visible = true,
    .named = true,
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
  [sym__items] = {
    .visible = false,
    .named = true,
  },
  [sym_assignment_item] = {
    .visible = true,
    .named = true,
  },
  [sym__expression] = {
    .visible = false,
    .named = true,
  },
  [sym_binary_operation] = {
    .visible = true,
    .named = true,
  },
  [sym_call] = {
    .visible = true,
    .named = true,
  },
  [sym_index_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_unary_operation] = {
    .visible = true,
    .named = true,
  },
  [sym__literal] = {
    .visible = false,
    .named = true,
  },
  [sym_array_literal] = {
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
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_call_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_index_expression_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_literal_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum {
  field_arguments = 1,
  field_collection = 2,
  field_expr = 3,
  field_indices = 4,
  field_left = 5,
  field_name = 6,
  field_operator = 7,
  field_right = 8,
};

static const char *ts_field_names[] = {
  [0] = NULL,
  [field_arguments] = "arguments",
  [field_collection] = "collection",
  [field_expr] = "expr",
  [field_indices] = "indices",
  [field_left] = "left",
  [field_name] = "name",
  [field_operator] = "operator",
  [field_right] = "right",
};

static const TSFieldMapSlice ts_field_map_slices[9] = {
  [1] = {.index = 0, .length = 2},
  [2] = {.index = 2, .length = 1},
  [3] = {.index = 3, .length = 1},
  [4] = {.index = 4, .length = 3},
  [5] = {.index = 7, .length = 2},
  [6] = {.index = 9, .length = 2},
  [7] = {.index = 11, .length = 3},
  [8] = {.index = 14, .length = 3},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_expr, 2},
    {field_name, 0},
  [2] =
    {field_operator, 0},
  [3] =
    {field_name, 0},
  [4] =
    {field_left, 0},
    {field_operator, 1},
    {field_right, 2},
  [7] =
    {field_arguments, 2},
    {field_name, 0},
  [9] =
    {field_collection, 0},
    {field_indices, 2},
  [11] =
    {field_arguments, 2},
    {field_arguments, 3},
    {field_name, 0},
  [14] =
    {field_collection, 0},
    {field_indices, 2},
    {field_indices, 3},
};

static TSSymbol ts_alias_sequences[9][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(31);
      if (lookahead == '!') ADVANCE(10);
      if (lookahead == '"') ADVANCE(71);
      if (lookahead == '%') ADVANCE(82);
      if (lookahead == '(') ADVANCE(55);
      if (lookahead == ')') ADVANCE(57);
      if (lookahead == '*') ADVANCE(51);
      if (lookahead == '+') ADVANCE(47);
      if (lookahead == ',') ADVANCE(56);
      if (lookahead == '-') ADVANCE(49);
      if (lookahead == '.') ADVANCE(6);
      if (lookahead == '/') ADVANCE(52);
      if (lookahead == '0') ADVANCE(64);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == ';') ADVANCE(32);
      if (lookahead == '<') ADVANCE(42);
      if (lookahead == '=') ADVANCE(33);
      if (lookahead == '>') ADVANCE(44);
      if (lookahead == '[') ADVANCE(58);
      if (lookahead == '\\') ADVANCE(8);
      if (lookahead == ']') ADVANCE(59);
      if (lookahead == '^') ADVANCE(53);
      if (lookahead == '{') ADVANCE(69);
      if (lookahead == '}') ADVANCE(70);
      if (lookahead == 172) ADVANCE(60);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(29)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(65);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(3)
      if (lookahead == '"') ADVANCE(71);
      if (lookahead == '%') ADVANCE(77);
      if (lookahead == '/') ADVANCE(75);
      if (lookahead == '\\') ADVANCE(12);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(72);
      if (lookahead != 0) ADVANCE(77);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(71);
      if (lookahead == '%') ADVANCE(82);
      if (lookahead == ')') ADVANCE(57);
      if (lookahead == '-') ADVANCE(48);
      if (lookahead == '/') ADVANCE(4);
      if (lookahead == '0') ADVANCE(64);
      if (lookahead == '<') ADVANCE(11);
      if (lookahead == '[') ADVANCE(58);
      if (lookahead == ']') ADVANCE(59);
      if (lookahead == '{') ADVANCE(69);
      if (lookahead == '}') ADVANCE(70);
      if (lookahead == 172) ADVANCE(60);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(2)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(65);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(71);
      if (lookahead == '%') ADVANCE(82);
      if (lookahead == '/') ADVANCE(4);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(3)
      END_STATE();
    case 4:
      if (lookahead == '*') ADVANCE(27);
      END_STATE();
    case 5:
      if (lookahead == '*') ADVANCE(28);
      if (lookahead == '/') ADVANCE(83);
      if (lookahead != 0) ADVANCE(27);
      END_STATE();
    case 6:
      if (lookahead == '.') ADVANCE(46);
      END_STATE();
    case 7:
      if (lookahead == '/') ADVANCE(37);
      END_STATE();
    case 8:
      if (lookahead == '/') ADVANCE(37);
      if (lookahead == 'U') ADVANCE(26);
      if (lookahead == 'u') ADVANCE(22);
      if (lookahead == 'x') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(80);
      if (lookahead != 0) ADVANCE(78);
      END_STATE();
    case 9:
      if (lookahead == ':') ADVANCE(54);
      END_STATE();
    case 10:
      if (lookahead == '=') ADVANCE(40);
      END_STATE();
    case 11:
      if (lookahead == '>') ADVANCE(61);
      END_STATE();
    case 12:
      if (lookahead == 'U') ADVANCE(26);
      if (lookahead == 'u') ADVANCE(22);
      if (lookahead == 'x') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(80);
      if (lookahead != 0) ADVANCE(78);
      END_STATE();
    case 13:
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(17);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(63);
      END_STATE();
    case 14:
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(66);
      END_STATE();
    case 15:
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(67);
      END_STATE();
    case 16:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(62);
      END_STATE();
    case 17:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(63);
      END_STATE();
    case 18:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(78);
      END_STATE();
    case 19:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(68);
      END_STATE();
    case 20:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(18);
      END_STATE();
    case 21:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(20);
      END_STATE();
    case 22:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(21);
      END_STATE();
    case 23:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(22);
      END_STATE();
    case 24:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(23);
      END_STATE();
    case 25:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(24);
      END_STATE();
    case 26:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(25);
      END_STATE();
    case 27:
      if (lookahead != 0 &&
          lookahead != '*') ADVANCE(27);
      if (lookahead == '*') ADVANCE(5);
      END_STATE();
    case 28:
      if (lookahead != 0 &&
          lookahead != '*' &&
          lookahead != '/') ADVANCE(27);
      if (lookahead == '*') ADVANCE(5);
      if (lookahead == '/') ADVANCE(84);
      END_STATE();
    case 29:
      if (eof) ADVANCE(31);
      if (lookahead == '!') ADVANCE(10);
      if (lookahead == '"') ADVANCE(71);
      if (lookahead == '%') ADVANCE(82);
      if (lookahead == '(') ADVANCE(55);
      if (lookahead == ')') ADVANCE(57);
      if (lookahead == '*') ADVANCE(51);
      if (lookahead == '+') ADVANCE(47);
      if (lookahead == ',') ADVANCE(56);
      if (lookahead == '-') ADVANCE(49);
      if (lookahead == '.') ADVANCE(6);
      if (lookahead == '/') ADVANCE(52);
      if (lookahead == '0') ADVANCE(64);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == ';') ADVANCE(32);
      if (lookahead == '<') ADVANCE(42);
      if (lookahead == '=') ADVANCE(33);
      if (lookahead == '>') ADVANCE(44);
      if (lookahead == '[') ADVANCE(58);
      if (lookahead == '\\') ADVANCE(7);
      if (lookahead == ']') ADVANCE(59);
      if (lookahead == '^') ADVANCE(53);
      if (lookahead == '{') ADVANCE(69);
      if (lookahead == '}') ADVANCE(70);
      if (lookahead == 172) ADVANCE(60);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(29)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(65);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      END_STATE();
    case 30:
      if (eof) ADVANCE(31);
      if (lookahead == '!') ADVANCE(10);
      if (lookahead == '%') ADVANCE(82);
      if (lookahead == '(') ADVANCE(55);
      if (lookahead == ')') ADVANCE(57);
      if (lookahead == '*') ADVANCE(51);
      if (lookahead == '+') ADVANCE(47);
      if (lookahead == ',') ADVANCE(56);
      if (lookahead == '-') ADVANCE(49);
      if (lookahead == '.') ADVANCE(6);
      if (lookahead == '/') ADVANCE(52);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == ';') ADVANCE(32);
      if (lookahead == '<') ADVANCE(41);
      if (lookahead == '=') ADVANCE(33);
      if (lookahead == '>') ADVANCE(44);
      if (lookahead == '[') ADVANCE(58);
      if (lookahead == '\\') ADVANCE(7);
      if (lookahead == ']') ADVANCE(59);
      if (lookahead == '^') ADVANCE(53);
      if (lookahead == '}') ADVANCE(70);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(30)
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(anon_sym_EQ);
      if (lookahead == '=') ADVANCE(39);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(anon_sym_LT_DASH_GT);
      END_STATE();
    case 35:
      ACCEPT_TOKEN(anon_sym_DASH_GT);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_LT_DASH);
      if (lookahead == '>') ADVANCE(34);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_BSLASH_SLASH);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_SLASH_BSLASH);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(anon_sym_EQ_EQ);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(anon_sym_BANG_EQ);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '-') ADVANCE(36);
      if (lookahead == '=') ADVANCE(43);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '-') ADVANCE(36);
      if (lookahead == '=') ADVANCE(43);
      if (lookahead == '>') ADVANCE(61);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(anon_sym_LT_EQ);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(anon_sym_GT);
      if (lookahead == '=') ADVANCE(45);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(anon_sym_GT_EQ);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(anon_sym_PLUS);
      if (lookahead == '+') ADVANCE(50);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(anon_sym_DASH);
      if (lookahead == '>') ADVANCE(35);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(anon_sym_PLUS_PLUS);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(anon_sym_STAR);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(anon_sym_SLASH);
      if (lookahead == '*') ADVANCE(27);
      if (lookahead == '\\') ADVANCE(38);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(anon_sym_CARET);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_COLON_COLON);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(anon_sym_);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(sym_absent);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(sym_float_literal);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(13);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(62);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(63);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(16);
      if (lookahead == 'b') ADVANCE(14);
      if (lookahead == 'o') ADVANCE(15);
      if (lookahead == 'x') ADVANCE(19);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(13);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(65);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(16);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(13);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(65);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(66);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(67);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(68);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '%') ADVANCE(77);
      if (lookahead == '/') ADVANCE(75);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(72);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(77);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(76);
      if (lookahead == '/') ADVANCE(74);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(74);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(76);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(74);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(74);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(77);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(73);
      if (lookahead == '/') ADVANCE(77);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(74);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(77);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(sym_escape_sequence);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(78);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(79);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(82);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(sym_block_comment);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(sym_block_comment);
      if (lookahead != 0 &&
          lookahead != '*') ADVANCE(27);
      if (lookahead == '*') ADVANCE(5);
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
      if (lookahead == 'd') ADVANCE(1);
      if (lookahead == 'f') ADVANCE(2);
      if (lookahead == 'i') ADVANCE(3);
      if (lookahead == 'm') ADVANCE(4);
      if (lookahead == 'n') ADVANCE(5);
      if (lookahead == 's') ADVANCE(6);
      if (lookahead == 't') ADVANCE(7);
      if (lookahead == 'u') ADVANCE(8);
      if (lookahead == 'x') ADVANCE(9);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
      END_STATE();
    case 1:
      if (lookahead == 'i') ADVANCE(10);
      END_STATE();
    case 2:
      if (lookahead == 'a') ADVANCE(11);
      END_STATE();
    case 3:
      if (lookahead == 'n') ADVANCE(12);
      END_STATE();
    case 4:
      if (lookahead == 'o') ADVANCE(13);
      END_STATE();
    case 5:
      if (lookahead == 'o') ADVANCE(14);
      END_STATE();
    case 6:
      if (lookahead == 'u') ADVANCE(15);
      if (lookahead == 'y') ADVANCE(16);
      END_STATE();
    case 7:
      if (lookahead == 'r') ADVANCE(17);
      END_STATE();
    case 8:
      if (lookahead == 'n') ADVANCE(18);
      END_STATE();
    case 9:
      if (lookahead == 'o') ADVANCE(19);
      END_STATE();
    case 10:
      if (lookahead == 'f') ADVANCE(20);
      if (lookahead == 'v') ADVANCE(21);
      END_STATE();
    case 11:
      if (lookahead == 'l') ADVANCE(22);
      END_STATE();
    case 12:
      ACCEPT_TOKEN(anon_sym_in);
      if (lookahead == 't') ADVANCE(23);
      END_STATE();
    case 13:
      if (lookahead == 'd') ADVANCE(24);
      END_STATE();
    case 14:
      if (lookahead == 't') ADVANCE(25);
      END_STATE();
    case 15:
      if (lookahead == 'b') ADVANCE(26);
      if (lookahead == 'p') ADVANCE(27);
      END_STATE();
    case 16:
      if (lookahead == 'm') ADVANCE(28);
      END_STATE();
    case 17:
      if (lookahead == 'u') ADVANCE(29);
      END_STATE();
    case 18:
      if (lookahead == 'i') ADVANCE(30);
      END_STATE();
    case 19:
      if (lookahead == 'r') ADVANCE(31);
      END_STATE();
    case 20:
      if (lookahead == 'f') ADVANCE(32);
      END_STATE();
    case 21:
      ACCEPT_TOKEN(anon_sym_div);
      END_STATE();
    case 22:
      if (lookahead == 's') ADVANCE(33);
      END_STATE();
    case 23:
      if (lookahead == 'e') ADVANCE(34);
      END_STATE();
    case 24:
      ACCEPT_TOKEN(anon_sym_mod);
      END_STATE();
    case 25:
      ACCEPT_TOKEN(anon_sym_not);
      END_STATE();
    case 26:
      if (lookahead == 's') ADVANCE(35);
      END_STATE();
    case 27:
      if (lookahead == 'e') ADVANCE(36);
      END_STATE();
    case 28:
      if (lookahead == 'd') ADVANCE(37);
      END_STATE();
    case 29:
      if (lookahead == 'e') ADVANCE(38);
      END_STATE();
    case 30:
      if (lookahead == 'o') ADVANCE(39);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(anon_sym_xor);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_diff);
      END_STATE();
    case 33:
      if (lookahead == 'e') ADVANCE(40);
      END_STATE();
    case 34:
      if (lookahead == 'r') ADVANCE(41);
      END_STATE();
    case 35:
      if (lookahead == 'e') ADVANCE(42);
      END_STATE();
    case 36:
      if (lookahead == 'r') ADVANCE(43);
      END_STATE();
    case 37:
      if (lookahead == 'i') ADVANCE(44);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_true);
      END_STATE();
    case 39:
      if (lookahead == 'n') ADVANCE(45);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(anon_sym_false);
      END_STATE();
    case 41:
      if (lookahead == 's') ADVANCE(46);
      END_STATE();
    case 42:
      if (lookahead == 't') ADVANCE(47);
      END_STATE();
    case 43:
      if (lookahead == 's') ADVANCE(48);
      END_STATE();
    case 44:
      if (lookahead == 'f') ADVANCE(49);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(anon_sym_union);
      END_STATE();
    case 46:
      if (lookahead == 'e') ADVANCE(50);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(anon_sym_subset);
      END_STATE();
    case 48:
      if (lookahead == 'e') ADVANCE(51);
      END_STATE();
    case 49:
      if (lookahead == 'f') ADVANCE(52);
      END_STATE();
    case 50:
      if (lookahead == 'c') ADVANCE(53);
      END_STATE();
    case 51:
      if (lookahead == 't') ADVANCE(54);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(anon_sym_symdiff);
      END_STATE();
    case 53:
      if (lookahead == 't') ADVANCE(55);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_superset);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym_intersect);
      END_STATE();
    default:
      return false;
  }
}

static TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 0},
  [2] = {.lex_state = 30},
  [3] = {.lex_state = 30},
  [4] = {.lex_state = 30},
  [5] = {.lex_state = 30},
  [6] = {.lex_state = 30},
  [7] = {.lex_state = 30},
  [8] = {.lex_state = 30},
  [9] = {.lex_state = 30},
  [10] = {.lex_state = 30},
  [11] = {.lex_state = 30},
  [12] = {.lex_state = 30},
  [13] = {.lex_state = 30},
  [14] = {.lex_state = 30},
  [15] = {.lex_state = 30},
  [16] = {.lex_state = 30},
  [17] = {.lex_state = 30},
  [18] = {.lex_state = 30},
  [19] = {.lex_state = 30},
  [20] = {.lex_state = 30},
  [21] = {.lex_state = 30},
  [22] = {.lex_state = 30},
  [23] = {.lex_state = 30},
  [24] = {.lex_state = 30},
  [25] = {.lex_state = 30},
  [26] = {.lex_state = 30},
  [27] = {.lex_state = 30},
  [28] = {.lex_state = 30},
  [29] = {.lex_state = 30},
  [30] = {.lex_state = 30},
  [31] = {.lex_state = 30},
  [32] = {.lex_state = 30},
  [33] = {.lex_state = 30},
  [34] = {.lex_state = 30},
  [35] = {.lex_state = 30},
  [36] = {.lex_state = 30},
  [37] = {.lex_state = 30},
  [38] = {.lex_state = 30},
  [39] = {.lex_state = 30},
  [40] = {.lex_state = 30},
  [41] = {.lex_state = 2},
  [42] = {.lex_state = 2},
  [43] = {.lex_state = 2},
  [44] = {.lex_state = 2},
  [45] = {.lex_state = 2},
  [46] = {.lex_state = 2},
  [47] = {.lex_state = 2},
  [48] = {.lex_state = 2},
  [49] = {.lex_state = 2},
  [50] = {.lex_state = 2},
  [51] = {.lex_state = 2},
  [52] = {.lex_state = 2},
  [53] = {.lex_state = 2},
  [54] = {.lex_state = 2},
  [55] = {.lex_state = 2},
  [56] = {.lex_state = 2},
  [57] = {.lex_state = 2},
  [58] = {.lex_state = 2},
  [59] = {.lex_state = 2},
  [60] = {.lex_state = 2},
  [61] = {.lex_state = 2},
  [62] = {.lex_state = 2},
  [63] = {.lex_state = 2},
  [64] = {.lex_state = 2},
  [65] = {.lex_state = 2},
  [66] = {.lex_state = 0},
  [67] = {.lex_state = 0},
  [68] = {.lex_state = 1},
  [69] = {.lex_state = 1},
  [70] = {.lex_state = 1},
  [71] = {.lex_state = 0},
  [72] = {.lex_state = 0},
  [73] = {.lex_state = 0},
  [74] = {.lex_state = 0},
  [75] = {.lex_state = 0},
  [76] = {.lex_state = 0},
  [77] = {.lex_state = 0},
  [78] = {.lex_state = 0},
};

static uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_identifier] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_LT_DASH_GT] = ACTIONS(1),
    [anon_sym_DASH_GT] = ACTIONS(1),
    [anon_sym_LT_DASH] = ACTIONS(1),
    [anon_sym_BSLASH_SLASH] = ACTIONS(1),
    [anon_sym_xor] = ACTIONS(1),
    [anon_sym_SLASH_BSLASH] = ACTIONS(1),
    [anon_sym_EQ_EQ] = ACTIONS(1),
    [anon_sym_BANG_EQ] = ACTIONS(1),
    [anon_sym_LT] = ACTIONS(1),
    [anon_sym_LT_EQ] = ACTIONS(1),
    [anon_sym_GT] = ACTIONS(1),
    [anon_sym_GT_EQ] = ACTIONS(1),
    [anon_sym_in] = ACTIONS(1),
    [anon_sym_subset] = ACTIONS(1),
    [anon_sym_superset] = ACTIONS(1),
    [anon_sym_union] = ACTIONS(1),
    [anon_sym_diff] = ACTIONS(1),
    [anon_sym_symdiff] = ACTIONS(1),
    [anon_sym_intersect] = ACTIONS(1),
    [anon_sym_DOT_DOT] = ACTIONS(1),
    [anon_sym_PLUS] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_PLUS_PLUS] = ACTIONS(1),
    [anon_sym_STAR] = ACTIONS(1),
    [anon_sym_SLASH] = ACTIONS(1),
    [anon_sym_div] = ACTIONS(1),
    [anon_sym_mod] = ACTIONS(1),
    [anon_sym_CARET] = ACTIONS(1),
    [anon_sym_COLON_COLON] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_not] = ACTIONS(1),
    [anon_sym_] = ACTIONS(1),
    [sym_absent] = ACTIONS(1),
    [anon_sym_true] = ACTIONS(1),
    [anon_sym_false] = ACTIONS(1),
    [sym_float_literal] = ACTIONS(1),
    [sym_integer_literal] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [sym_escape_sequence] = ACTIONS(1),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [1] = {
    [sym_source_file] = STATE(78),
    [sym__items] = STATE(75),
    [sym_assignment_item] = STATE(75),
    [aux_sym_source_file_repeat1] = STATE(67),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_identifier] = ACTIONS(7),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [2] = {
    [ts_builtin_sym_end] = ACTIONS(9),
    [anon_sym_SEMI] = ACTIONS(9),
    [anon_sym_EQ] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(9),
    [anon_sym_DASH_GT] = ACTIONS(9),
    [anon_sym_LT_DASH] = ACTIONS(11),
    [anon_sym_BSLASH_SLASH] = ACTIONS(9),
    [anon_sym_xor] = ACTIONS(9),
    [anon_sym_SLASH_BSLASH] = ACTIONS(9),
    [anon_sym_EQ_EQ] = ACTIONS(9),
    [anon_sym_BANG_EQ] = ACTIONS(9),
    [anon_sym_LT] = ACTIONS(11),
    [anon_sym_LT_EQ] = ACTIONS(9),
    [anon_sym_GT] = ACTIONS(11),
    [anon_sym_GT_EQ] = ACTIONS(9),
    [anon_sym_in] = ACTIONS(11),
    [anon_sym_subset] = ACTIONS(9),
    [anon_sym_superset] = ACTIONS(9),
    [anon_sym_union] = ACTIONS(9),
    [anon_sym_diff] = ACTIONS(9),
    [anon_sym_symdiff] = ACTIONS(9),
    [anon_sym_intersect] = ACTIONS(9),
    [anon_sym_DOT_DOT] = ACTIONS(9),
    [anon_sym_PLUS] = ACTIONS(11),
    [anon_sym_DASH] = ACTIONS(11),
    [anon_sym_PLUS_PLUS] = ACTIONS(9),
    [anon_sym_STAR] = ACTIONS(9),
    [anon_sym_SLASH] = ACTIONS(11),
    [anon_sym_div] = ACTIONS(9),
    [anon_sym_mod] = ACTIONS(9),
    [anon_sym_CARET] = ACTIONS(9),
    [anon_sym_COLON_COLON] = ACTIONS(9),
    [anon_sym_LPAREN] = ACTIONS(13),
    [anon_sym_COMMA] = ACTIONS(9),
    [anon_sym_RPAREN] = ACTIONS(9),
    [anon_sym_LBRACK] = ACTIONS(9),
    [anon_sym_RBRACK] = ACTIONS(9),
    [anon_sym_RBRACE] = ACTIONS(9),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [3] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(15),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [4] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(15),
    [anon_sym_diff] = ACTIONS(15),
    [anon_sym_symdiff] = ACTIONS(15),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [5] = {
    [ts_builtin_sym_end] = ACTIONS(41),
    [anon_sym_SEMI] = ACTIONS(41),
    [anon_sym_EQ] = ACTIONS(43),
    [anon_sym_LT_DASH_GT] = ACTIONS(41),
    [anon_sym_DASH_GT] = ACTIONS(41),
    [anon_sym_LT_DASH] = ACTIONS(43),
    [anon_sym_BSLASH_SLASH] = ACTIONS(41),
    [anon_sym_xor] = ACTIONS(41),
    [anon_sym_SLASH_BSLASH] = ACTIONS(41),
    [anon_sym_EQ_EQ] = ACTIONS(41),
    [anon_sym_BANG_EQ] = ACTIONS(41),
    [anon_sym_LT] = ACTIONS(43),
    [anon_sym_LT_EQ] = ACTIONS(41),
    [anon_sym_GT] = ACTIONS(43),
    [anon_sym_GT_EQ] = ACTIONS(41),
    [anon_sym_in] = ACTIONS(43),
    [anon_sym_subset] = ACTIONS(41),
    [anon_sym_superset] = ACTIONS(41),
    [anon_sym_union] = ACTIONS(41),
    [anon_sym_diff] = ACTIONS(41),
    [anon_sym_symdiff] = ACTIONS(41),
    [anon_sym_intersect] = ACTIONS(41),
    [anon_sym_DOT_DOT] = ACTIONS(41),
    [anon_sym_PLUS] = ACTIONS(43),
    [anon_sym_DASH] = ACTIONS(43),
    [anon_sym_PLUS_PLUS] = ACTIONS(41),
    [anon_sym_STAR] = ACTIONS(41),
    [anon_sym_SLASH] = ACTIONS(43),
    [anon_sym_div] = ACTIONS(41),
    [anon_sym_mod] = ACTIONS(41),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(41),
    [anon_sym_COMMA] = ACTIONS(41),
    [anon_sym_RPAREN] = ACTIONS(41),
    [anon_sym_LBRACK] = ACTIONS(41),
    [anon_sym_RBRACK] = ACTIONS(41),
    [anon_sym_RBRACE] = ACTIONS(41),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [6] = {
    [ts_builtin_sym_end] = ACTIONS(45),
    [anon_sym_SEMI] = ACTIONS(45),
    [anon_sym_EQ] = ACTIONS(47),
    [anon_sym_LT_DASH_GT] = ACTIONS(45),
    [anon_sym_DASH_GT] = ACTIONS(45),
    [anon_sym_LT_DASH] = ACTIONS(47),
    [anon_sym_BSLASH_SLASH] = ACTIONS(45),
    [anon_sym_xor] = ACTIONS(45),
    [anon_sym_SLASH_BSLASH] = ACTIONS(45),
    [anon_sym_EQ_EQ] = ACTIONS(45),
    [anon_sym_BANG_EQ] = ACTIONS(45),
    [anon_sym_LT] = ACTIONS(47),
    [anon_sym_LT_EQ] = ACTIONS(45),
    [anon_sym_GT] = ACTIONS(47),
    [anon_sym_GT_EQ] = ACTIONS(45),
    [anon_sym_in] = ACTIONS(47),
    [anon_sym_subset] = ACTIONS(45),
    [anon_sym_superset] = ACTIONS(45),
    [anon_sym_union] = ACTIONS(45),
    [anon_sym_diff] = ACTIONS(45),
    [anon_sym_symdiff] = ACTIONS(45),
    [anon_sym_intersect] = ACTIONS(45),
    [anon_sym_DOT_DOT] = ACTIONS(45),
    [anon_sym_PLUS] = ACTIONS(47),
    [anon_sym_DASH] = ACTIONS(47),
    [anon_sym_PLUS_PLUS] = ACTIONS(45),
    [anon_sym_STAR] = ACTIONS(45),
    [anon_sym_SLASH] = ACTIONS(47),
    [anon_sym_div] = ACTIONS(45),
    [anon_sym_mod] = ACTIONS(45),
    [anon_sym_CARET] = ACTIONS(45),
    [anon_sym_COLON_COLON] = ACTIONS(45),
    [anon_sym_COMMA] = ACTIONS(45),
    [anon_sym_RPAREN] = ACTIONS(45),
    [anon_sym_LBRACK] = ACTIONS(45),
    [anon_sym_RBRACK] = ACTIONS(45),
    [anon_sym_RBRACE] = ACTIONS(45),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [7] = {
    [ts_builtin_sym_end] = ACTIONS(49),
    [anon_sym_SEMI] = ACTIONS(49),
    [anon_sym_EQ] = ACTIONS(51),
    [anon_sym_LT_DASH_GT] = ACTIONS(49),
    [anon_sym_DASH_GT] = ACTIONS(49),
    [anon_sym_LT_DASH] = ACTIONS(51),
    [anon_sym_BSLASH_SLASH] = ACTIONS(49),
    [anon_sym_xor] = ACTIONS(49),
    [anon_sym_SLASH_BSLASH] = ACTIONS(49),
    [anon_sym_EQ_EQ] = ACTIONS(49),
    [anon_sym_BANG_EQ] = ACTIONS(49),
    [anon_sym_LT] = ACTIONS(51),
    [anon_sym_LT_EQ] = ACTIONS(49),
    [anon_sym_GT] = ACTIONS(51),
    [anon_sym_GT_EQ] = ACTIONS(49),
    [anon_sym_in] = ACTIONS(51),
    [anon_sym_subset] = ACTIONS(49),
    [anon_sym_superset] = ACTIONS(49),
    [anon_sym_union] = ACTIONS(49),
    [anon_sym_diff] = ACTIONS(49),
    [anon_sym_symdiff] = ACTIONS(49),
    [anon_sym_intersect] = ACTIONS(49),
    [anon_sym_DOT_DOT] = ACTIONS(49),
    [anon_sym_PLUS] = ACTIONS(51),
    [anon_sym_DASH] = ACTIONS(51),
    [anon_sym_PLUS_PLUS] = ACTIONS(49),
    [anon_sym_STAR] = ACTIONS(49),
    [anon_sym_SLASH] = ACTIONS(51),
    [anon_sym_div] = ACTIONS(49),
    [anon_sym_mod] = ACTIONS(49),
    [anon_sym_CARET] = ACTIONS(49),
    [anon_sym_COLON_COLON] = ACTIONS(49),
    [anon_sym_COMMA] = ACTIONS(49),
    [anon_sym_RPAREN] = ACTIONS(49),
    [anon_sym_LBRACK] = ACTIONS(49),
    [anon_sym_RBRACK] = ACTIONS(49),
    [anon_sym_RBRACE] = ACTIONS(49),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [8] = {
    [ts_builtin_sym_end] = ACTIONS(53),
    [anon_sym_SEMI] = ACTIONS(53),
    [anon_sym_EQ] = ACTIONS(55),
    [anon_sym_LT_DASH_GT] = ACTIONS(53),
    [anon_sym_DASH_GT] = ACTIONS(53),
    [anon_sym_LT_DASH] = ACTIONS(55),
    [anon_sym_BSLASH_SLASH] = ACTIONS(53),
    [anon_sym_xor] = ACTIONS(53),
    [anon_sym_SLASH_BSLASH] = ACTIONS(53),
    [anon_sym_EQ_EQ] = ACTIONS(53),
    [anon_sym_BANG_EQ] = ACTIONS(53),
    [anon_sym_LT] = ACTIONS(55),
    [anon_sym_LT_EQ] = ACTIONS(53),
    [anon_sym_GT] = ACTIONS(55),
    [anon_sym_GT_EQ] = ACTIONS(53),
    [anon_sym_in] = ACTIONS(55),
    [anon_sym_subset] = ACTIONS(53),
    [anon_sym_superset] = ACTIONS(53),
    [anon_sym_union] = ACTIONS(53),
    [anon_sym_diff] = ACTIONS(53),
    [anon_sym_symdiff] = ACTIONS(53),
    [anon_sym_intersect] = ACTIONS(53),
    [anon_sym_DOT_DOT] = ACTIONS(53),
    [anon_sym_PLUS] = ACTIONS(55),
    [anon_sym_DASH] = ACTIONS(55),
    [anon_sym_PLUS_PLUS] = ACTIONS(53),
    [anon_sym_STAR] = ACTIONS(53),
    [anon_sym_SLASH] = ACTIONS(55),
    [anon_sym_div] = ACTIONS(53),
    [anon_sym_mod] = ACTIONS(53),
    [anon_sym_CARET] = ACTIONS(53),
    [anon_sym_COLON_COLON] = ACTIONS(53),
    [anon_sym_COMMA] = ACTIONS(53),
    [anon_sym_RPAREN] = ACTIONS(53),
    [anon_sym_LBRACK] = ACTIONS(53),
    [anon_sym_RBRACK] = ACTIONS(53),
    [anon_sym_RBRACE] = ACTIONS(53),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [9] = {
    [ts_builtin_sym_end] = ACTIONS(57),
    [anon_sym_SEMI] = ACTIONS(57),
    [anon_sym_EQ] = ACTIONS(59),
    [anon_sym_LT_DASH_GT] = ACTIONS(57),
    [anon_sym_DASH_GT] = ACTIONS(57),
    [anon_sym_LT_DASH] = ACTIONS(59),
    [anon_sym_BSLASH_SLASH] = ACTIONS(57),
    [anon_sym_xor] = ACTIONS(57),
    [anon_sym_SLASH_BSLASH] = ACTIONS(57),
    [anon_sym_EQ_EQ] = ACTIONS(57),
    [anon_sym_BANG_EQ] = ACTIONS(57),
    [anon_sym_LT] = ACTIONS(59),
    [anon_sym_LT_EQ] = ACTIONS(57),
    [anon_sym_GT] = ACTIONS(59),
    [anon_sym_GT_EQ] = ACTIONS(57),
    [anon_sym_in] = ACTIONS(59),
    [anon_sym_subset] = ACTIONS(57),
    [anon_sym_superset] = ACTIONS(57),
    [anon_sym_union] = ACTIONS(57),
    [anon_sym_diff] = ACTIONS(57),
    [anon_sym_symdiff] = ACTIONS(57),
    [anon_sym_intersect] = ACTIONS(57),
    [anon_sym_DOT_DOT] = ACTIONS(57),
    [anon_sym_PLUS] = ACTIONS(59),
    [anon_sym_DASH] = ACTIONS(59),
    [anon_sym_PLUS_PLUS] = ACTIONS(57),
    [anon_sym_STAR] = ACTIONS(57),
    [anon_sym_SLASH] = ACTIONS(59),
    [anon_sym_div] = ACTIONS(57),
    [anon_sym_mod] = ACTIONS(57),
    [anon_sym_CARET] = ACTIONS(57),
    [anon_sym_COLON_COLON] = ACTIONS(57),
    [anon_sym_COMMA] = ACTIONS(57),
    [anon_sym_RPAREN] = ACTIONS(57),
    [anon_sym_LBRACK] = ACTIONS(57),
    [anon_sym_RBRACK] = ACTIONS(57),
    [anon_sym_RBRACE] = ACTIONS(57),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [10] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(15),
    [anon_sym_diff] = ACTIONS(15),
    [anon_sym_symdiff] = ACTIONS(15),
    [anon_sym_intersect] = ACTIONS(15),
    [anon_sym_DOT_DOT] = ACTIONS(15),
    [anon_sym_PLUS] = ACTIONS(17),
    [anon_sym_DASH] = ACTIONS(17),
    [anon_sym_PLUS_PLUS] = ACTIONS(15),
    [anon_sym_STAR] = ACTIONS(15),
    [anon_sym_SLASH] = ACTIONS(17),
    [anon_sym_div] = ACTIONS(15),
    [anon_sym_mod] = ACTIONS(15),
    [anon_sym_CARET] = ACTIONS(15),
    [anon_sym_COLON_COLON] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [11] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(15),
    [anon_sym_diff] = ACTIONS(15),
    [anon_sym_symdiff] = ACTIONS(15),
    [anon_sym_intersect] = ACTIONS(15),
    [anon_sym_DOT_DOT] = ACTIONS(15),
    [anon_sym_PLUS] = ACTIONS(17),
    [anon_sym_DASH] = ACTIONS(17),
    [anon_sym_PLUS_PLUS] = ACTIONS(15),
    [anon_sym_STAR] = ACTIONS(15),
    [anon_sym_SLASH] = ACTIONS(17),
    [anon_sym_div] = ACTIONS(15),
    [anon_sym_mod] = ACTIONS(15),
    [anon_sym_CARET] = ACTIONS(15),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [12] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(15),
    [anon_sym_diff] = ACTIONS(15),
    [anon_sym_symdiff] = ACTIONS(15),
    [anon_sym_intersect] = ACTIONS(15),
    [anon_sym_DOT_DOT] = ACTIONS(15),
    [anon_sym_PLUS] = ACTIONS(17),
    [anon_sym_DASH] = ACTIONS(17),
    [anon_sym_PLUS_PLUS] = ACTIONS(15),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [13] = {
    [ts_builtin_sym_end] = ACTIONS(61),
    [anon_sym_SEMI] = ACTIONS(61),
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LT_DASH_GT] = ACTIONS(61),
    [anon_sym_DASH_GT] = ACTIONS(61),
    [anon_sym_LT_DASH] = ACTIONS(63),
    [anon_sym_BSLASH_SLASH] = ACTIONS(61),
    [anon_sym_xor] = ACTIONS(61),
    [anon_sym_SLASH_BSLASH] = ACTIONS(61),
    [anon_sym_EQ_EQ] = ACTIONS(61),
    [anon_sym_BANG_EQ] = ACTIONS(61),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(61),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(61),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(61),
    [anon_sym_superset] = ACTIONS(61),
    [anon_sym_union] = ACTIONS(61),
    [anon_sym_diff] = ACTIONS(61),
    [anon_sym_symdiff] = ACTIONS(61),
    [anon_sym_intersect] = ACTIONS(61),
    [anon_sym_DOT_DOT] = ACTIONS(61),
    [anon_sym_PLUS] = ACTIONS(63),
    [anon_sym_DASH] = ACTIONS(63),
    [anon_sym_PLUS_PLUS] = ACTIONS(61),
    [anon_sym_STAR] = ACTIONS(61),
    [anon_sym_SLASH] = ACTIONS(63),
    [anon_sym_div] = ACTIONS(61),
    [anon_sym_mod] = ACTIONS(61),
    [anon_sym_CARET] = ACTIONS(61),
    [anon_sym_COLON_COLON] = ACTIONS(61),
    [anon_sym_COMMA] = ACTIONS(61),
    [anon_sym_RPAREN] = ACTIONS(61),
    [anon_sym_LBRACK] = ACTIONS(61),
    [anon_sym_RBRACK] = ACTIONS(61),
    [anon_sym_RBRACE] = ACTIONS(61),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [14] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(15),
    [anon_sym_diff] = ACTIONS(15),
    [anon_sym_symdiff] = ACTIONS(15),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(15),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [15] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(15),
    [anon_sym_diff] = ACTIONS(15),
    [anon_sym_symdiff] = ACTIONS(15),
    [anon_sym_intersect] = ACTIONS(15),
    [anon_sym_DOT_DOT] = ACTIONS(15),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [16] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(15),
    [anon_sym_diff] = ACTIONS(15),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [17] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [18] = {
    [ts_builtin_sym_end] = ACTIONS(71),
    [anon_sym_SEMI] = ACTIONS(71),
    [anon_sym_EQ] = ACTIONS(73),
    [anon_sym_LT_DASH_GT] = ACTIONS(71),
    [anon_sym_DASH_GT] = ACTIONS(71),
    [anon_sym_LT_DASH] = ACTIONS(73),
    [anon_sym_BSLASH_SLASH] = ACTIONS(71),
    [anon_sym_xor] = ACTIONS(71),
    [anon_sym_SLASH_BSLASH] = ACTIONS(71),
    [anon_sym_EQ_EQ] = ACTIONS(71),
    [anon_sym_BANG_EQ] = ACTIONS(71),
    [anon_sym_LT] = ACTIONS(73),
    [anon_sym_LT_EQ] = ACTIONS(71),
    [anon_sym_GT] = ACTIONS(73),
    [anon_sym_GT_EQ] = ACTIONS(71),
    [anon_sym_in] = ACTIONS(73),
    [anon_sym_subset] = ACTIONS(71),
    [anon_sym_superset] = ACTIONS(71),
    [anon_sym_union] = ACTIONS(71),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(71),
    [anon_sym_intersect] = ACTIONS(71),
    [anon_sym_DOT_DOT] = ACTIONS(71),
    [anon_sym_PLUS] = ACTIONS(73),
    [anon_sym_DASH] = ACTIONS(73),
    [anon_sym_PLUS_PLUS] = ACTIONS(71),
    [anon_sym_STAR] = ACTIONS(71),
    [anon_sym_SLASH] = ACTIONS(73),
    [anon_sym_div] = ACTIONS(71),
    [anon_sym_mod] = ACTIONS(71),
    [anon_sym_CARET] = ACTIONS(71),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(71),
    [anon_sym_RPAREN] = ACTIONS(71),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(71),
    [anon_sym_RBRACE] = ACTIONS(71),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [19] = {
    [ts_builtin_sym_end] = ACTIONS(75),
    [anon_sym_SEMI] = ACTIONS(75),
    [anon_sym_EQ] = ACTIONS(77),
    [anon_sym_LT_DASH_GT] = ACTIONS(75),
    [anon_sym_DASH_GT] = ACTIONS(75),
    [anon_sym_LT_DASH] = ACTIONS(77),
    [anon_sym_BSLASH_SLASH] = ACTIONS(75),
    [anon_sym_xor] = ACTIONS(75),
    [anon_sym_SLASH_BSLASH] = ACTIONS(75),
    [anon_sym_EQ_EQ] = ACTIONS(75),
    [anon_sym_BANG_EQ] = ACTIONS(75),
    [anon_sym_LT] = ACTIONS(77),
    [anon_sym_LT_EQ] = ACTIONS(75),
    [anon_sym_GT] = ACTIONS(77),
    [anon_sym_GT_EQ] = ACTIONS(75),
    [anon_sym_in] = ACTIONS(77),
    [anon_sym_subset] = ACTIONS(75),
    [anon_sym_superset] = ACTIONS(75),
    [anon_sym_union] = ACTIONS(75),
    [anon_sym_diff] = ACTIONS(75),
    [anon_sym_symdiff] = ACTIONS(75),
    [anon_sym_intersect] = ACTIONS(75),
    [anon_sym_DOT_DOT] = ACTIONS(75),
    [anon_sym_PLUS] = ACTIONS(77),
    [anon_sym_DASH] = ACTIONS(77),
    [anon_sym_PLUS_PLUS] = ACTIONS(75),
    [anon_sym_STAR] = ACTIONS(75),
    [anon_sym_SLASH] = ACTIONS(77),
    [anon_sym_div] = ACTIONS(75),
    [anon_sym_mod] = ACTIONS(75),
    [anon_sym_CARET] = ACTIONS(75),
    [anon_sym_COLON_COLON] = ACTIONS(75),
    [anon_sym_COMMA] = ACTIONS(75),
    [anon_sym_RPAREN] = ACTIONS(75),
    [anon_sym_LBRACK] = ACTIONS(75),
    [anon_sym_RBRACK] = ACTIONS(75),
    [anon_sym_RBRACE] = ACTIONS(75),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [20] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [21] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [22] = {
    [ts_builtin_sym_end] = ACTIONS(85),
    [anon_sym_SEMI] = ACTIONS(85),
    [anon_sym_EQ] = ACTIONS(87),
    [anon_sym_LT_DASH_GT] = ACTIONS(85),
    [anon_sym_DASH_GT] = ACTIONS(85),
    [anon_sym_LT_DASH] = ACTIONS(87),
    [anon_sym_BSLASH_SLASH] = ACTIONS(85),
    [anon_sym_xor] = ACTIONS(85),
    [anon_sym_SLASH_BSLASH] = ACTIONS(85),
    [anon_sym_EQ_EQ] = ACTIONS(85),
    [anon_sym_BANG_EQ] = ACTIONS(85),
    [anon_sym_LT] = ACTIONS(87),
    [anon_sym_LT_EQ] = ACTIONS(85),
    [anon_sym_GT] = ACTIONS(87),
    [anon_sym_GT_EQ] = ACTIONS(85),
    [anon_sym_in] = ACTIONS(87),
    [anon_sym_subset] = ACTIONS(85),
    [anon_sym_superset] = ACTIONS(85),
    [anon_sym_union] = ACTIONS(85),
    [anon_sym_diff] = ACTIONS(85),
    [anon_sym_symdiff] = ACTIONS(85),
    [anon_sym_intersect] = ACTIONS(85),
    [anon_sym_DOT_DOT] = ACTIONS(85),
    [anon_sym_PLUS] = ACTIONS(87),
    [anon_sym_DASH] = ACTIONS(87),
    [anon_sym_PLUS_PLUS] = ACTIONS(85),
    [anon_sym_STAR] = ACTIONS(85),
    [anon_sym_SLASH] = ACTIONS(87),
    [anon_sym_div] = ACTIONS(85),
    [anon_sym_mod] = ACTIONS(85),
    [anon_sym_CARET] = ACTIONS(85),
    [anon_sym_COLON_COLON] = ACTIONS(85),
    [anon_sym_COMMA] = ACTIONS(85),
    [anon_sym_RPAREN] = ACTIONS(85),
    [anon_sym_LBRACK] = ACTIONS(85),
    [anon_sym_RBRACK] = ACTIONS(85),
    [anon_sym_RBRACE] = ACTIONS(85),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [23] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [24] = {
    [ts_builtin_sym_end] = ACTIONS(89),
    [anon_sym_SEMI] = ACTIONS(89),
    [anon_sym_EQ] = ACTIONS(91),
    [anon_sym_LT_DASH_GT] = ACTIONS(89),
    [anon_sym_DASH_GT] = ACTIONS(89),
    [anon_sym_LT_DASH] = ACTIONS(91),
    [anon_sym_BSLASH_SLASH] = ACTIONS(89),
    [anon_sym_xor] = ACTIONS(89),
    [anon_sym_SLASH_BSLASH] = ACTIONS(89),
    [anon_sym_EQ_EQ] = ACTIONS(89),
    [anon_sym_BANG_EQ] = ACTIONS(89),
    [anon_sym_LT] = ACTIONS(91),
    [anon_sym_LT_EQ] = ACTIONS(89),
    [anon_sym_GT] = ACTIONS(91),
    [anon_sym_GT_EQ] = ACTIONS(89),
    [anon_sym_in] = ACTIONS(91),
    [anon_sym_subset] = ACTIONS(89),
    [anon_sym_superset] = ACTIONS(89),
    [anon_sym_union] = ACTIONS(89),
    [anon_sym_diff] = ACTIONS(89),
    [anon_sym_symdiff] = ACTIONS(89),
    [anon_sym_intersect] = ACTIONS(89),
    [anon_sym_DOT_DOT] = ACTIONS(89),
    [anon_sym_PLUS] = ACTIONS(91),
    [anon_sym_DASH] = ACTIONS(91),
    [anon_sym_PLUS_PLUS] = ACTIONS(89),
    [anon_sym_STAR] = ACTIONS(89),
    [anon_sym_SLASH] = ACTIONS(91),
    [anon_sym_div] = ACTIONS(89),
    [anon_sym_mod] = ACTIONS(89),
    [anon_sym_CARET] = ACTIONS(89),
    [anon_sym_COLON_COLON] = ACTIONS(89),
    [anon_sym_COMMA] = ACTIONS(89),
    [anon_sym_RPAREN] = ACTIONS(89),
    [anon_sym_LBRACK] = ACTIONS(89),
    [anon_sym_RBRACK] = ACTIONS(89),
    [anon_sym_RBRACE] = ACTIONS(89),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [25] = {
    [ts_builtin_sym_end] = ACTIONS(93),
    [anon_sym_SEMI] = ACTIONS(93),
    [anon_sym_EQ] = ACTIONS(95),
    [anon_sym_LT_DASH_GT] = ACTIONS(93),
    [anon_sym_DASH_GT] = ACTIONS(93),
    [anon_sym_LT_DASH] = ACTIONS(95),
    [anon_sym_BSLASH_SLASH] = ACTIONS(93),
    [anon_sym_xor] = ACTIONS(93),
    [anon_sym_SLASH_BSLASH] = ACTIONS(93),
    [anon_sym_EQ_EQ] = ACTIONS(93),
    [anon_sym_BANG_EQ] = ACTIONS(93),
    [anon_sym_LT] = ACTIONS(95),
    [anon_sym_LT_EQ] = ACTIONS(93),
    [anon_sym_GT] = ACTIONS(95),
    [anon_sym_GT_EQ] = ACTIONS(93),
    [anon_sym_in] = ACTIONS(95),
    [anon_sym_subset] = ACTIONS(93),
    [anon_sym_superset] = ACTIONS(93),
    [anon_sym_union] = ACTIONS(93),
    [anon_sym_diff] = ACTIONS(93),
    [anon_sym_symdiff] = ACTIONS(93),
    [anon_sym_intersect] = ACTIONS(93),
    [anon_sym_DOT_DOT] = ACTIONS(93),
    [anon_sym_PLUS] = ACTIONS(95),
    [anon_sym_DASH] = ACTIONS(95),
    [anon_sym_PLUS_PLUS] = ACTIONS(93),
    [anon_sym_STAR] = ACTIONS(93),
    [anon_sym_SLASH] = ACTIONS(95),
    [anon_sym_div] = ACTIONS(93),
    [anon_sym_mod] = ACTIONS(93),
    [anon_sym_CARET] = ACTIONS(93),
    [anon_sym_COLON_COLON] = ACTIONS(93),
    [anon_sym_COMMA] = ACTIONS(93),
    [anon_sym_RPAREN] = ACTIONS(93),
    [anon_sym_LBRACK] = ACTIONS(93),
    [anon_sym_RBRACK] = ACTIONS(93),
    [anon_sym_RBRACE] = ACTIONS(93),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [26] = {
    [ts_builtin_sym_end] = ACTIONS(97),
    [anon_sym_SEMI] = ACTIONS(97),
    [anon_sym_EQ] = ACTIONS(99),
    [anon_sym_LT_DASH_GT] = ACTIONS(97),
    [anon_sym_DASH_GT] = ACTIONS(97),
    [anon_sym_LT_DASH] = ACTIONS(99),
    [anon_sym_BSLASH_SLASH] = ACTIONS(97),
    [anon_sym_xor] = ACTIONS(97),
    [anon_sym_SLASH_BSLASH] = ACTIONS(97),
    [anon_sym_EQ_EQ] = ACTIONS(97),
    [anon_sym_BANG_EQ] = ACTIONS(97),
    [anon_sym_LT] = ACTIONS(99),
    [anon_sym_LT_EQ] = ACTIONS(97),
    [anon_sym_GT] = ACTIONS(99),
    [anon_sym_GT_EQ] = ACTIONS(97),
    [anon_sym_in] = ACTIONS(99),
    [anon_sym_subset] = ACTIONS(97),
    [anon_sym_superset] = ACTIONS(97),
    [anon_sym_union] = ACTIONS(97),
    [anon_sym_diff] = ACTIONS(97),
    [anon_sym_symdiff] = ACTIONS(97),
    [anon_sym_intersect] = ACTIONS(97),
    [anon_sym_DOT_DOT] = ACTIONS(97),
    [anon_sym_PLUS] = ACTIONS(99),
    [anon_sym_DASH] = ACTIONS(99),
    [anon_sym_PLUS_PLUS] = ACTIONS(97),
    [anon_sym_STAR] = ACTIONS(97),
    [anon_sym_SLASH] = ACTIONS(99),
    [anon_sym_div] = ACTIONS(97),
    [anon_sym_mod] = ACTIONS(97),
    [anon_sym_CARET] = ACTIONS(97),
    [anon_sym_COLON_COLON] = ACTIONS(97),
    [anon_sym_COMMA] = ACTIONS(97),
    [anon_sym_RPAREN] = ACTIONS(97),
    [anon_sym_LBRACK] = ACTIONS(97),
    [anon_sym_RBRACK] = ACTIONS(97),
    [anon_sym_RBRACE] = ACTIONS(97),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [27] = {
    [ts_builtin_sym_end] = ACTIONS(101),
    [anon_sym_SEMI] = ACTIONS(101),
    [anon_sym_EQ] = ACTIONS(103),
    [anon_sym_LT_DASH_GT] = ACTIONS(101),
    [anon_sym_DASH_GT] = ACTIONS(101),
    [anon_sym_LT_DASH] = ACTIONS(103),
    [anon_sym_BSLASH_SLASH] = ACTIONS(101),
    [anon_sym_xor] = ACTIONS(101),
    [anon_sym_SLASH_BSLASH] = ACTIONS(101),
    [anon_sym_EQ_EQ] = ACTIONS(101),
    [anon_sym_BANG_EQ] = ACTIONS(101),
    [anon_sym_LT] = ACTIONS(103),
    [anon_sym_LT_EQ] = ACTIONS(101),
    [anon_sym_GT] = ACTIONS(103),
    [anon_sym_GT_EQ] = ACTIONS(101),
    [anon_sym_in] = ACTIONS(103),
    [anon_sym_subset] = ACTIONS(101),
    [anon_sym_superset] = ACTIONS(101),
    [anon_sym_union] = ACTIONS(101),
    [anon_sym_diff] = ACTIONS(101),
    [anon_sym_symdiff] = ACTIONS(101),
    [anon_sym_intersect] = ACTIONS(101),
    [anon_sym_DOT_DOT] = ACTIONS(101),
    [anon_sym_PLUS] = ACTIONS(103),
    [anon_sym_DASH] = ACTIONS(103),
    [anon_sym_PLUS_PLUS] = ACTIONS(101),
    [anon_sym_STAR] = ACTIONS(101),
    [anon_sym_SLASH] = ACTIONS(103),
    [anon_sym_div] = ACTIONS(101),
    [anon_sym_mod] = ACTIONS(101),
    [anon_sym_CARET] = ACTIONS(101),
    [anon_sym_COLON_COLON] = ACTIONS(101),
    [anon_sym_COMMA] = ACTIONS(101),
    [anon_sym_RPAREN] = ACTIONS(101),
    [anon_sym_LBRACK] = ACTIONS(101),
    [anon_sym_RBRACK] = ACTIONS(101),
    [anon_sym_RBRACE] = ACTIONS(101),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [28] = {
    [ts_builtin_sym_end] = ACTIONS(105),
    [anon_sym_SEMI] = ACTIONS(105),
    [anon_sym_EQ] = ACTIONS(107),
    [anon_sym_LT_DASH_GT] = ACTIONS(105),
    [anon_sym_DASH_GT] = ACTIONS(105),
    [anon_sym_LT_DASH] = ACTIONS(107),
    [anon_sym_BSLASH_SLASH] = ACTIONS(105),
    [anon_sym_xor] = ACTIONS(105),
    [anon_sym_SLASH_BSLASH] = ACTIONS(105),
    [anon_sym_EQ_EQ] = ACTIONS(105),
    [anon_sym_BANG_EQ] = ACTIONS(105),
    [anon_sym_LT] = ACTIONS(107),
    [anon_sym_LT_EQ] = ACTIONS(105),
    [anon_sym_GT] = ACTIONS(107),
    [anon_sym_GT_EQ] = ACTIONS(105),
    [anon_sym_in] = ACTIONS(107),
    [anon_sym_subset] = ACTIONS(105),
    [anon_sym_superset] = ACTIONS(105),
    [anon_sym_union] = ACTIONS(105),
    [anon_sym_diff] = ACTIONS(105),
    [anon_sym_symdiff] = ACTIONS(105),
    [anon_sym_intersect] = ACTIONS(105),
    [anon_sym_DOT_DOT] = ACTIONS(105),
    [anon_sym_PLUS] = ACTIONS(107),
    [anon_sym_DASH] = ACTIONS(107),
    [anon_sym_PLUS_PLUS] = ACTIONS(105),
    [anon_sym_STAR] = ACTIONS(105),
    [anon_sym_SLASH] = ACTIONS(107),
    [anon_sym_div] = ACTIONS(105),
    [anon_sym_mod] = ACTIONS(105),
    [anon_sym_CARET] = ACTIONS(105),
    [anon_sym_COLON_COLON] = ACTIONS(105),
    [anon_sym_COMMA] = ACTIONS(105),
    [anon_sym_RPAREN] = ACTIONS(105),
    [anon_sym_LBRACK] = ACTIONS(105),
    [anon_sym_RBRACK] = ACTIONS(105),
    [anon_sym_RBRACE] = ACTIONS(105),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [29] = {
    [ts_builtin_sym_end] = ACTIONS(15),
    [anon_sym_SEMI] = ACTIONS(15),
    [anon_sym_EQ] = ACTIONS(17),
    [anon_sym_LT_DASH_GT] = ACTIONS(15),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(15),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(15),
    [anon_sym_EQ_EQ] = ACTIONS(15),
    [anon_sym_BANG_EQ] = ACTIONS(15),
    [anon_sym_LT] = ACTIONS(17),
    [anon_sym_LT_EQ] = ACTIONS(15),
    [anon_sym_GT] = ACTIONS(17),
    [anon_sym_GT_EQ] = ACTIONS(15),
    [anon_sym_in] = ACTIONS(17),
    [anon_sym_subset] = ACTIONS(15),
    [anon_sym_superset] = ACTIONS(15),
    [anon_sym_union] = ACTIONS(15),
    [anon_sym_diff] = ACTIONS(15),
    [anon_sym_symdiff] = ACTIONS(15),
    [anon_sym_intersect] = ACTIONS(15),
    [anon_sym_DOT_DOT] = ACTIONS(15),
    [anon_sym_PLUS] = ACTIONS(17),
    [anon_sym_DASH] = ACTIONS(17),
    [anon_sym_PLUS_PLUS] = ACTIONS(15),
    [anon_sym_STAR] = ACTIONS(15),
    [anon_sym_SLASH] = ACTIONS(17),
    [anon_sym_div] = ACTIONS(15),
    [anon_sym_mod] = ACTIONS(15),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(15),
    [anon_sym_RPAREN] = ACTIONS(15),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(15),
    [anon_sym_RBRACE] = ACTIONS(15),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [30] = {
    [ts_builtin_sym_end] = ACTIONS(109),
    [anon_sym_SEMI] = ACTIONS(109),
    [anon_sym_EQ] = ACTIONS(111),
    [anon_sym_LT_DASH_GT] = ACTIONS(109),
    [anon_sym_DASH_GT] = ACTIONS(109),
    [anon_sym_LT_DASH] = ACTIONS(111),
    [anon_sym_BSLASH_SLASH] = ACTIONS(109),
    [anon_sym_xor] = ACTIONS(109),
    [anon_sym_SLASH_BSLASH] = ACTIONS(109),
    [anon_sym_EQ_EQ] = ACTIONS(109),
    [anon_sym_BANG_EQ] = ACTIONS(109),
    [anon_sym_LT] = ACTIONS(111),
    [anon_sym_LT_EQ] = ACTIONS(109),
    [anon_sym_GT] = ACTIONS(111),
    [anon_sym_GT_EQ] = ACTIONS(109),
    [anon_sym_in] = ACTIONS(111),
    [anon_sym_subset] = ACTIONS(109),
    [anon_sym_superset] = ACTIONS(109),
    [anon_sym_union] = ACTIONS(109),
    [anon_sym_diff] = ACTIONS(109),
    [anon_sym_symdiff] = ACTIONS(109),
    [anon_sym_intersect] = ACTIONS(109),
    [anon_sym_DOT_DOT] = ACTIONS(109),
    [anon_sym_PLUS] = ACTIONS(111),
    [anon_sym_DASH] = ACTIONS(111),
    [anon_sym_PLUS_PLUS] = ACTIONS(109),
    [anon_sym_STAR] = ACTIONS(109),
    [anon_sym_SLASH] = ACTIONS(111),
    [anon_sym_div] = ACTIONS(109),
    [anon_sym_mod] = ACTIONS(109),
    [anon_sym_CARET] = ACTIONS(109),
    [anon_sym_COLON_COLON] = ACTIONS(109),
    [anon_sym_COMMA] = ACTIONS(109),
    [anon_sym_RPAREN] = ACTIONS(109),
    [anon_sym_LBRACK] = ACTIONS(109),
    [anon_sym_RBRACK] = ACTIONS(109),
    [anon_sym_RBRACE] = ACTIONS(109),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [31] = {
    [aux_sym_index_expression_repeat1] = STATE(71),
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(113),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(113),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(115),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(117),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [32] = {
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(113),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(113),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(119),
    [anon_sym_RPAREN] = ACTIONS(121),
    [anon_sym_LBRACK] = ACTIONS(39),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [33] = {
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(113),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(113),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(119),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(123),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [34] = {
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(113),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(113),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(125),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(125),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [35] = {
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(113),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(113),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(119),
    [anon_sym_RPAREN] = ACTIONS(127),
    [anon_sym_LBRACK] = ACTIONS(39),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [36] = {
    [ts_builtin_sym_end] = ACTIONS(129),
    [anon_sym_SEMI] = ACTIONS(129),
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(113),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(113),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_LBRACK] = ACTIONS(39),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [37] = {
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(113),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(113),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(119),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACE] = ACTIONS(131),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [38] = {
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(113),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(113),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(119),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(133),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [39] = {
    [anon_sym_EQ] = ACTIONS(65),
    [anon_sym_LT_DASH_GT] = ACTIONS(113),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(113),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(79),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(65),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(65),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(65),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(19),
    [anon_sym_symdiff] = ACTIONS(21),
    [anon_sym_intersect] = ACTIONS(23),
    [anon_sym_DOT_DOT] = ACTIONS(25),
    [anon_sym_PLUS] = ACTIONS(27),
    [anon_sym_DASH] = ACTIONS(27),
    [anon_sym_PLUS_PLUS] = ACTIONS(29),
    [anon_sym_STAR] = ACTIONS(31),
    [anon_sym_SLASH] = ACTIONS(33),
    [anon_sym_div] = ACTIONS(31),
    [anon_sym_mod] = ACTIONS(31),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(37),
    [anon_sym_COMMA] = ACTIONS(119),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_RBRACE] = ACTIONS(135),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
};

static uint16_t ts_small_parse_table[] = {
  [0] = 20,
    ACTIONS(19), 1,
      anon_sym_diff,
    ACTIONS(21), 1,
      anon_sym_symdiff,
    ACTIONS(23), 1,
      anon_sym_intersect,
    ACTIONS(25), 1,
      anon_sym_DOT_DOT,
    ACTIONS(29), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(33), 1,
      anon_sym_SLASH,
    ACTIONS(35), 1,
      anon_sym_CARET,
    ACTIONS(37), 1,
      anon_sym_COLON_COLON,
    ACTIONS(39), 1,
      anon_sym_LBRACK,
    ACTIONS(69), 1,
      anon_sym_union,
    ACTIONS(79), 1,
      anon_sym_SLASH_BSLASH,
    ACTIONS(83), 1,
      anon_sym_LT_DASH,
    ACTIONS(119), 1,
      anon_sym_COMMA,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(27), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(81), 2,
      anon_sym_DASH_GT,
      anon_sym_xor,
    ACTIONS(113), 2,
      anon_sym_LT_DASH_GT,
      anon_sym_BSLASH_SLASH,
    ACTIONS(31), 3,
      anon_sym_STAR,
      anon_sym_div,
      anon_sym_mod,
    ACTIONS(65), 4,
      anon_sym_EQ,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_in,
    ACTIONS(67), 6,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_subset,
      anon_sym_superset,
  [75] = 13,
    ACTIONS(137), 1,
      sym_identifier,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(148), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      sym_integer_literal,
    ACTIONS(160), 1,
      anon_sym_LBRACE,
    ACTIONS(163), 1,
      anon_sym_DQUOTE,
    STATE(41), 1,
      aux_sym_call_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(140), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(151), 2,
      sym_absent,
      sym_float_literal,
    ACTIONS(154), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(143), 3,
      anon_sym_RPAREN,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
    STATE(40), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [130] = 13,
    ACTIONS(131), 1,
      anon_sym_RBRACE,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(178), 1,
      sym_integer_literal,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    STATE(41), 1,
      aux_sym_call_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(174), 2,
      sym_absent,
      sym_float_literal,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(39), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [183] = 13,
    ACTIONS(123), 1,
      anon_sym_RBRACK,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(186), 1,
      sym_integer_literal,
    STATE(41), 1,
      aux_sym_call_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(184), 2,
      sym_absent,
      sym_float_literal,
    STATE(38), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [236] = 13,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(188), 1,
      anon_sym_RBRACK,
    ACTIONS(192), 1,
      sym_integer_literal,
    STATE(43), 1,
      aux_sym_call_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(190), 2,
      sym_absent,
      sym_float_literal,
    STATE(33), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [289] = 13,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(196), 1,
      sym_integer_literal,
    ACTIONS(198), 1,
      anon_sym_RBRACE,
    STATE(42), 1,
      aux_sym_call_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(194), 2,
      sym_absent,
      sym_float_literal,
    STATE(37), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [342] = 13,
    ACTIONS(127), 1,
      anon_sym_RPAREN,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_integer_literal,
    STATE(41), 1,
      aux_sym_call_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(200), 2,
      sym_absent,
      sym_float_literal,
    STATE(32), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [395] = 13,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(204), 1,
      anon_sym_RPAREN,
    ACTIONS(208), 1,
      sym_integer_literal,
    STATE(46), 1,
      aux_sym_call_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_absent,
      sym_float_literal,
    STATE(35), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [448] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(212), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(210), 2,
      sym_absent,
      sym_float_literal,
    STATE(21), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [495] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(216), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(214), 2,
      sym_absent,
      sym_float_literal,
    STATE(20), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [542] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(220), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(218), 2,
      sym_absent,
      sym_float_literal,
    STATE(34), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [589] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(224), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(222), 2,
      sym_absent,
      sym_float_literal,
    STATE(10), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [636] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(228), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(226), 2,
      sym_absent,
      sym_float_literal,
    STATE(3), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [683] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(232), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(230), 2,
      sym_absent,
      sym_float_literal,
    STATE(23), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [730] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(236), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(234), 2,
      sym_absent,
      sym_float_literal,
    STATE(14), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [777] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(240), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(238), 2,
      sym_absent,
      sym_float_literal,
    STATE(17), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [824] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(244), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(242), 2,
      sym_absent,
      sym_float_literal,
    STATE(16), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [871] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(248), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(246), 2,
      sym_absent,
      sym_float_literal,
    STATE(36), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [918] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(252), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(250), 2,
      sym_absent,
      sym_float_literal,
    STATE(4), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [965] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(256), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(254), 2,
      sym_absent,
      sym_float_literal,
    STATE(15), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [1012] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(260), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(258), 2,
      sym_absent,
      sym_float_literal,
    STATE(11), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [1059] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(264), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(262), 2,
      sym_absent,
      sym_float_literal,
    STATE(12), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [1106] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(268), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(266), 2,
      sym_absent,
      sym_float_literal,
    STATE(29), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [1153] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(272), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(270), 2,
      sym_absent,
      sym_float_literal,
    STATE(31), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [1200] = 11,
    ACTIONS(166), 1,
      sym_identifier,
    ACTIONS(170), 1,
      anon_sym_LBRACK,
    ACTIONS(172), 1,
      anon_sym_not,
    ACTIONS(180), 1,
      anon_sym_LBRACE,
    ACTIONS(182), 1,
      anon_sym_DQUOTE,
    ACTIONS(276), 1,
      sym_integer_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(168), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(176), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(274), 2,
      sym_absent,
      sym_float_literal,
    STATE(18), 10,
      sym__expression,
      sym_binary_operation,
      sym_call,
      sym_index_expression,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [1247] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(278), 5,
      anon_sym_not,
      anon_sym_true,
      anon_sym_false,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(143), 10,
      anon_sym_DASH,
      anon_sym_RPAREN,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      anon_sym_,
      sym_absent,
      sym_float_literal,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DQUOTE,
  [1271] = 5,
    ACTIONS(280), 1,
      ts_builtin_sym_end,
    ACTIONS(282), 1,
      sym_identifier,
    STATE(66), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(77), 2,
      sym__items,
      sym_assignment_item,
  [1289] = 5,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(285), 1,
      ts_builtin_sym_end,
    STATE(66), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(74), 2,
      sym__items,
      sym_assignment_item,
  [1307] = 4,
    ACTIONS(287), 1,
      anon_sym_DQUOTE,
    STATE(70), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(289), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
    ACTIONS(291), 2,
      sym_line_comment,
      sym_block_comment,
  [1322] = 4,
    ACTIONS(293), 1,
      anon_sym_DQUOTE,
    STATE(68), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(291), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(295), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
  [1337] = 4,
    ACTIONS(297), 1,
      anon_sym_DQUOTE,
    STATE(70), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(291), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(299), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
  [1352] = 4,
    ACTIONS(115), 1,
      anon_sym_COMMA,
    ACTIONS(302), 1,
      anon_sym_RBRACK,
    STATE(72), 1,
      aux_sym_index_expression_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1366] = 4,
    ACTIONS(125), 1,
      anon_sym_RBRACK,
    ACTIONS(304), 1,
      anon_sym_COMMA,
    STATE(72), 1,
      aux_sym_index_expression_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1380] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(280), 2,
      ts_builtin_sym_end,
      sym_identifier,
  [1389] = 3,
    ACTIONS(307), 1,
      ts_builtin_sym_end,
    ACTIONS(309), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1400] = 3,
    ACTIONS(285), 1,
      ts_builtin_sym_end,
    ACTIONS(309), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1411] = 2,
    ACTIONS(311), 1,
      anon_sym_EQ,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1419] = 2,
    ACTIONS(309), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1427] = 2,
    ACTIONS(313), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
};

static uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(40)] = 0,
  [SMALL_STATE(41)] = 75,
  [SMALL_STATE(42)] = 130,
  [SMALL_STATE(43)] = 183,
  [SMALL_STATE(44)] = 236,
  [SMALL_STATE(45)] = 289,
  [SMALL_STATE(46)] = 342,
  [SMALL_STATE(47)] = 395,
  [SMALL_STATE(48)] = 448,
  [SMALL_STATE(49)] = 495,
  [SMALL_STATE(50)] = 542,
  [SMALL_STATE(51)] = 589,
  [SMALL_STATE(52)] = 636,
  [SMALL_STATE(53)] = 683,
  [SMALL_STATE(54)] = 730,
  [SMALL_STATE(55)] = 777,
  [SMALL_STATE(56)] = 824,
  [SMALL_STATE(57)] = 871,
  [SMALL_STATE(58)] = 918,
  [SMALL_STATE(59)] = 965,
  [SMALL_STATE(60)] = 1012,
  [SMALL_STATE(61)] = 1059,
  [SMALL_STATE(62)] = 1106,
  [SMALL_STATE(63)] = 1153,
  [SMALL_STATE(64)] = 1200,
  [SMALL_STATE(65)] = 1247,
  [SMALL_STATE(66)] = 1271,
  [SMALL_STATE(67)] = 1289,
  [SMALL_STATE(68)] = 1307,
  [SMALL_STATE(69)] = 1322,
  [SMALL_STATE(70)] = 1337,
  [SMALL_STATE(71)] = 1352,
  [SMALL_STATE(72)] = 1366,
  [SMALL_STATE(73)] = 1380,
  [SMALL_STATE(74)] = 1389,
  [SMALL_STATE(75)] = 1400,
  [SMALL_STATE(76)] = 1411,
  [SMALL_STATE(77)] = 1419,
  [SMALL_STATE(78)] = 1427,
};

static TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
  [9] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__expression, 1),
  [11] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__expression, 1),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [15] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_binary_operation, 3, .production_id = 4),
  [17] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_binary_operation, 3, .production_id = 4),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [27] = {.entry = {.count = 1, .reusable = false}}, SHIFT(61),
  [29] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [33] = {.entry = {.count = 1, .reusable = false}}, SHIFT(62),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [41] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_index_expression, 5, .production_id = 8),
  [43] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_index_expression, 5, .production_id = 8),
  [45] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_index_expression, 4, .production_id = 6),
  [47] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_index_expression, 4, .production_id = 6),
  [49] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 4, .production_id = 5),
  [51] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 4, .production_id = 5),
  [53] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 4),
  [55] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 4),
  [57] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 4),
  [59] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 4),
  [61] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean_literal, 1),
  [63] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean_literal, 1),
  [65] = {.entry = {.count = 1, .reusable = false}}, SHIFT(53),
  [67] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [69] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [71] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_unary_operation, 2, .production_id = 2),
  [73] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_unary_operation, 2, .production_id = 2),
  [75] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 2),
  [77] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 2),
  [79] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [81] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [83] = {.entry = {.count = 1, .reusable = false}}, SHIFT(49),
  [85] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 2),
  [87] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 2),
  [89] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 3, .production_id = 3),
  [91] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 3, .production_id = 3),
  [93] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 2),
  [95] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 2),
  [97] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 3),
  [99] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 3),
  [101] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 3),
  [103] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 3),
  [105] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 3),
  [107] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 3),
  [109] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call, 5, .production_id = 7),
  [111] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call, 5, .production_id = 7),
  [113] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [115] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [117] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [119] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [121] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [123] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [125] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_index_expression_repeat1, 2),
  [127] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [129] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment_item, 3, .production_id = 1),
  [131] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [133] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [135] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [137] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2), SHIFT_REPEAT(2),
  [140] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2), SHIFT_REPEAT(64),
  [143] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2),
  [145] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2), SHIFT_REPEAT(44),
  [148] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2), SHIFT_REPEAT(64),
  [151] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2), SHIFT_REPEAT(40),
  [154] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2), SHIFT_REPEAT(13),
  [157] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2), SHIFT_REPEAT(40),
  [160] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2), SHIFT_REPEAT(45),
  [163] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_call_repeat1, 2), SHIFT_REPEAT(69),
  [166] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [168] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [170] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [172] = {.entry = {.count = 1, .reusable = false}}, SHIFT(64),
  [174] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [176] = {.entry = {.count = 1, .reusable = false}}, SHIFT(13),
  [178] = {.entry = {.count = 1, .reusable = false}}, SHIFT(39),
  [180] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [182] = {.entry = {.count = 1, .reusable = true}}, SHIFT(69),
  [184] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [186] = {.entry = {.count = 1, .reusable = false}}, SHIFT(38),
  [188] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [190] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [192] = {.entry = {.count = 1, .reusable = false}}, SHIFT(33),
  [194] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [196] = {.entry = {.count = 1, .reusable = false}}, SHIFT(37),
  [198] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [200] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [202] = {.entry = {.count = 1, .reusable = false}}, SHIFT(32),
  [204] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [206] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [208] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [210] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [212] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [214] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [216] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [218] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [220] = {.entry = {.count = 1, .reusable = false}}, SHIFT(34),
  [222] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [224] = {.entry = {.count = 1, .reusable = false}}, SHIFT(10),
  [226] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [228] = {.entry = {.count = 1, .reusable = false}}, SHIFT(3),
  [230] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [232] = {.entry = {.count = 1, .reusable = false}}, SHIFT(23),
  [234] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [236] = {.entry = {.count = 1, .reusable = false}}, SHIFT(14),
  [238] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [240] = {.entry = {.count = 1, .reusable = false}}, SHIFT(17),
  [242] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [244] = {.entry = {.count = 1, .reusable = false}}, SHIFT(16),
  [246] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [248] = {.entry = {.count = 1, .reusable = false}}, SHIFT(36),
  [250] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [252] = {.entry = {.count = 1, .reusable = false}}, SHIFT(4),
  [254] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [256] = {.entry = {.count = 1, .reusable = false}}, SHIFT(15),
  [258] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [260] = {.entry = {.count = 1, .reusable = false}}, SHIFT(11),
  [262] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [264] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [266] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [268] = {.entry = {.count = 1, .reusable = false}}, SHIFT(29),
  [270] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [272] = {.entry = {.count = 1, .reusable = false}}, SHIFT(31),
  [274] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [276] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [278] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_call_repeat1, 2),
  [280] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2),
  [282] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(76),
  [285] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1),
  [287] = {.entry = {.count = 1, .reusable = false}}, SHIFT(26),
  [289] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [291] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [293] = {.entry = {.count = 1, .reusable = false}}, SHIFT(25),
  [295] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [297] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_literal_repeat1, 2),
  [299] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_literal_repeat1, 2), SHIFT_REPEAT(70),
  [302] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [304] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_index_expression_repeat1, 2), SHIFT_REPEAT(50),
  [307] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 2),
  [309] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [311] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [313] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef _WIN32
#define extern __declspec(dllexport)
#endif

extern const TSLanguage *tree_sitter_minizinc(void) {
  static TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .symbol_metadata = ts_symbol_metadata,
    .parse_table = (const unsigned short *)ts_parse_table,
    .small_parse_table = (const uint16_t *)ts_small_parse_table,
    .small_parse_table_map = (const uint32_t *)ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .lex_modes = ts_lex_modes,
    .symbol_names = ts_symbol_names,
    .public_symbol_map = ts_symbol_map,
    .alias_sequences = (const TSSymbol *)ts_alias_sequences,
    .field_count = FIELD_COUNT,
    .field_names = ts_field_names,
    .field_map_slices = (const TSFieldMapSlice *)ts_field_map_slices,
    .field_map_entries = (const TSFieldMapEntry *)ts_field_map_entries,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .lex_fn = ts_lex,
    .keyword_lex_fn = ts_lex_keywords,
    .keyword_capture_token = sym_identifier,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
