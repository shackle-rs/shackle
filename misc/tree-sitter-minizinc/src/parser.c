#include <tree_sitter/parser.h>

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 11
#define STATE_COUNT 73
#define LARGE_STATE_COUNT 35
#define SYMBOL_COUNT 66
#define ALIAS_COUNT 0
#define TOKEN_COUNT 50
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 7
#define MAX_ALIAS_SEQUENCE_LENGTH 5

enum {
  sym_identifier = 1,
  anon_sym_SEMI = 2,
  anon_sym_EQ = 3,
  anon_sym_LBRACK = 4,
  anon_sym_COMMA = 5,
  anon_sym_RBRACK = 6,
  anon_sym_LT_DASH_GT = 7,
  anon_sym_DASH_GT = 8,
  anon_sym_LT_DASH = 9,
  anon_sym_BSLASH_SLASH = 10,
  anon_sym_xor = 11,
  anon_sym_SLASH_BSLASH = 12,
  anon_sym_EQ_EQ = 13,
  anon_sym_BANG_EQ = 14,
  anon_sym_LT = 15,
  anon_sym_LT_EQ = 16,
  anon_sym_GT = 17,
  anon_sym_GT_EQ = 18,
  anon_sym_in = 19,
  anon_sym_subset = 20,
  anon_sym_superset = 21,
  anon_sym_union = 22,
  anon_sym_diff = 23,
  anon_sym_symdiff = 24,
  anon_sym_intersect = 25,
  anon_sym_DOT_DOT = 26,
  anon_sym_PLUS = 27,
  anon_sym_DASH = 28,
  anon_sym_PLUS_PLUS = 29,
  anon_sym_STAR = 30,
  anon_sym_SLASH = 31,
  anon_sym_div = 32,
  anon_sym_mod = 33,
  anon_sym_CARET = 34,
  anon_sym_COLON_COLON = 35,
  anon_sym_not = 36,
  anon_sym_ = 37,
  sym_absent = 38,
  anon_sym_true = 39,
  anon_sym_false = 40,
  sym_float_literal = 41,
  sym_integer_literal = 42,
  anon_sym_LBRACE = 43,
  anon_sym_RBRACE = 44,
  anon_sym_DQUOTE = 45,
  aux_sym_string_literal_token1 = 46,
  sym_escape_sequence = 47,
  sym_line_comment = 48,
  sym_block_comment = 49,
  sym_source_file = 50,
  sym__items = 51,
  sym_assignment_item = 52,
  sym__expression = 53,
  sym_index_expression = 54,
  sym_binary_operation = 55,
  sym_unary_operation = 56,
  sym__literal = 57,
  sym_array_literal = 58,
  sym_boolean_literal = 59,
  sym_set_literal = 60,
  sym_string_literal = 61,
  aux_sym_source_file_repeat1 = 62,
  aux_sym_index_expression_repeat1 = 63,
  aux_sym_array_literal_repeat1 = 64,
  aux_sym_string_literal_repeat1 = 65,
};

static const char *ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_identifier] = "identifier",
  [anon_sym_SEMI] = ";",
  [anon_sym_EQ] = "=",
  [anon_sym_LBRACK] = "[",
  [anon_sym_COMMA] = ",",
  [anon_sym_RBRACK] = "]",
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
  [sym_index_expression] = "index_expression",
  [sym_binary_operation] = "binary_operation",
  [sym_unary_operation] = "unary_operation",
  [sym__literal] = "_literal",
  [sym_array_literal] = "array_literal",
  [sym_boolean_literal] = "boolean_literal",
  [sym_set_literal] = "set_literal",
  [sym_string_literal] = "string_literal",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_index_expression_repeat1] = "index_expression_repeat1",
  [aux_sym_array_literal_repeat1] = "array_literal_repeat1",
  [aux_sym_string_literal_repeat1] = "string_literal_repeat1",
};

static TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_identifier] = sym_identifier,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
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
  [sym_index_expression] = sym_index_expression,
  [sym_binary_operation] = sym_binary_operation,
  [sym_unary_operation] = sym_unary_operation,
  [sym__literal] = sym__literal,
  [sym_array_literal] = sym_array_literal,
  [sym_boolean_literal] = sym_boolean_literal,
  [sym_set_literal] = sym_set_literal,
  [sym_string_literal] = sym_string_literal,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_index_expression_repeat1] = aux_sym_index_expression_repeat1,
  [aux_sym_array_literal_repeat1] = aux_sym_array_literal_repeat1,
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
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
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
  [sym_index_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_binary_operation] = {
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
  [aux_sym_index_expression_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_array_literal_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_literal_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum {
  field_collection = 1,
  field_expr = 2,
  field_indices = 3,
  field_left = 4,
  field_name = 5,
  field_operator = 6,
  field_right = 7,
};

static const char *ts_field_names[] = {
  [0] = NULL,
  [field_collection] = "collection",
  [field_expr] = "expr",
  [field_indices] = "indices",
  [field_left] = "left",
  [field_name] = "name",
  [field_operator] = "operator",
  [field_right] = "right",
};

static const TSFieldMapSlice ts_field_map_slices[6] = {
  [1] = {.index = 0, .length = 2},
  [2] = {.index = 2, .length = 1},
  [3] = {.index = 3, .length = 3},
  [4] = {.index = 6, .length = 2},
  [5] = {.index = 8, .length = 3},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_expr, 2},
    {field_name, 0},
  [2] =
    {field_operator, 0},
  [3] =
    {field_left, 0},
    {field_operator, 1},
    {field_right, 2},
  [6] =
    {field_collection, 0},
    {field_indices, 2},
  [8] =
    {field_collection, 0},
    {field_indices, 2},
    {field_indices, 3},
};

static TSSymbol ts_alias_sequences[6][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(31);
      if (lookahead == '!') ADVANCE(10);
      if (lookahead == '"') ADVANCE(69);
      if (lookahead == '%') ADVANCE(80);
      if (lookahead == '*') ADVANCE(54);
      if (lookahead == '+') ADVANCE(50);
      if (lookahead == ',') ADVANCE(35);
      if (lookahead == '-') ADVANCE(52);
      if (lookahead == '.') ADVANCE(6);
      if (lookahead == '/') ADVANCE(55);
      if (lookahead == '0') ADVANCE(62);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == ';') ADVANCE(32);
      if (lookahead == '<') ADVANCE(45);
      if (lookahead == '=') ADVANCE(33);
      if (lookahead == '>') ADVANCE(47);
      if (lookahead == '[') ADVANCE(34);
      if (lookahead == '\\') ADVANCE(8);
      if (lookahead == ']') ADVANCE(36);
      if (lookahead == '^') ADVANCE(56);
      if (lookahead == '{') ADVANCE(67);
      if (lookahead == '}') ADVANCE(68);
      if (lookahead == 172) ADVANCE(58);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(29)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(63);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(79);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(3)
      if (lookahead == '"') ADVANCE(69);
      if (lookahead == '%') ADVANCE(75);
      if (lookahead == '/') ADVANCE(73);
      if (lookahead == '\\') ADVANCE(12);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(70);
      if (lookahead != 0) ADVANCE(75);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(69);
      if (lookahead == '%') ADVANCE(80);
      if (lookahead == '-') ADVANCE(51);
      if (lookahead == '/') ADVANCE(4);
      if (lookahead == '0') ADVANCE(62);
      if (lookahead == '<') ADVANCE(11);
      if (lookahead == '[') ADVANCE(34);
      if (lookahead == ']') ADVANCE(36);
      if (lookahead == '{') ADVANCE(67);
      if (lookahead == '}') ADVANCE(68);
      if (lookahead == 172) ADVANCE(58);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(2)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(63);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(79);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(69);
      if (lookahead == '%') ADVANCE(80);
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
      if (lookahead == '/') ADVANCE(81);
      if (lookahead != 0) ADVANCE(27);
      END_STATE();
    case 6:
      if (lookahead == '.') ADVANCE(49);
      END_STATE();
    case 7:
      if (lookahead == '/') ADVANCE(40);
      END_STATE();
    case 8:
      if (lookahead == '/') ADVANCE(40);
      if (lookahead == 'U') ADVANCE(26);
      if (lookahead == 'u') ADVANCE(22);
      if (lookahead == 'x') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(78);
      if (lookahead != 0) ADVANCE(76);
      END_STATE();
    case 9:
      if (lookahead == ':') ADVANCE(57);
      END_STATE();
    case 10:
      if (lookahead == '=') ADVANCE(43);
      END_STATE();
    case 11:
      if (lookahead == '>') ADVANCE(59);
      END_STATE();
    case 12:
      if (lookahead == 'U') ADVANCE(26);
      if (lookahead == 'u') ADVANCE(22);
      if (lookahead == 'x') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(78);
      if (lookahead != 0) ADVANCE(76);
      END_STATE();
    case 13:
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(17);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(61);
      END_STATE();
    case 14:
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(64);
      END_STATE();
    case 15:
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(65);
      END_STATE();
    case 16:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(60);
      END_STATE();
    case 17:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(61);
      END_STATE();
    case 18:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(76);
      END_STATE();
    case 19:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(66);
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
      if (lookahead == '/') ADVANCE(82);
      END_STATE();
    case 29:
      if (eof) ADVANCE(31);
      if (lookahead == '!') ADVANCE(10);
      if (lookahead == '"') ADVANCE(69);
      if (lookahead == '%') ADVANCE(80);
      if (lookahead == '*') ADVANCE(54);
      if (lookahead == '+') ADVANCE(50);
      if (lookahead == ',') ADVANCE(35);
      if (lookahead == '-') ADVANCE(52);
      if (lookahead == '.') ADVANCE(6);
      if (lookahead == '/') ADVANCE(55);
      if (lookahead == '0') ADVANCE(62);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == ';') ADVANCE(32);
      if (lookahead == '<') ADVANCE(45);
      if (lookahead == '=') ADVANCE(33);
      if (lookahead == '>') ADVANCE(47);
      if (lookahead == '[') ADVANCE(34);
      if (lookahead == '\\') ADVANCE(7);
      if (lookahead == ']') ADVANCE(36);
      if (lookahead == '^') ADVANCE(56);
      if (lookahead == '{') ADVANCE(67);
      if (lookahead == '}') ADVANCE(68);
      if (lookahead == 172) ADVANCE(58);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(29)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(63);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(79);
      END_STATE();
    case 30:
      if (eof) ADVANCE(31);
      if (lookahead == '!') ADVANCE(10);
      if (lookahead == '%') ADVANCE(80);
      if (lookahead == '*') ADVANCE(54);
      if (lookahead == '+') ADVANCE(50);
      if (lookahead == ',') ADVANCE(35);
      if (lookahead == '-') ADVANCE(52);
      if (lookahead == '.') ADVANCE(6);
      if (lookahead == '/') ADVANCE(55);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == ';') ADVANCE(32);
      if (lookahead == '<') ADVANCE(44);
      if (lookahead == '=') ADVANCE(33);
      if (lookahead == '>') ADVANCE(47);
      if (lookahead == '[') ADVANCE(34);
      if (lookahead == '\\') ADVANCE(7);
      if (lookahead == ']') ADVANCE(36);
      if (lookahead == '^') ADVANCE(56);
      if (lookahead == '}') ADVANCE(68);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(30)
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(79);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(anon_sym_EQ);
      if (lookahead == '=') ADVANCE(42);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 35:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_LT_DASH_GT);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_DASH_GT);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(anon_sym_LT_DASH);
      if (lookahead == '>') ADVANCE(37);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(anon_sym_BSLASH_SLASH);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(anon_sym_SLASH_BSLASH);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(anon_sym_EQ_EQ);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(anon_sym_BANG_EQ);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '-') ADVANCE(39);
      if (lookahead == '=') ADVANCE(46);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '-') ADVANCE(39);
      if (lookahead == '=') ADVANCE(46);
      if (lookahead == '>') ADVANCE(59);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(anon_sym_LT_EQ);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(anon_sym_GT);
      if (lookahead == '=') ADVANCE(48);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(anon_sym_GT_EQ);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(anon_sym_PLUS);
      if (lookahead == '+') ADVANCE(53);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(anon_sym_DASH);
      if (lookahead == '>') ADVANCE(38);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(anon_sym_PLUS_PLUS);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_STAR);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym_SLASH);
      if (lookahead == '*') ADVANCE(27);
      if (lookahead == '\\') ADVANCE(41);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(anon_sym_CARET);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(anon_sym_COLON_COLON);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(anon_sym_);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(sym_absent);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(sym_float_literal);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(13);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(60);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(61);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(16);
      if (lookahead == 'b') ADVANCE(14);
      if (lookahead == 'o') ADVANCE(15);
      if (lookahead == 'x') ADVANCE(19);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(13);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(63);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(16);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(13);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(63);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(64);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(65);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(66);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '%') ADVANCE(75);
      if (lookahead == '/') ADVANCE(73);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(70);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(75);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(74);
      if (lookahead == '/') ADVANCE(72);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(72);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(74);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(72);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(72);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(75);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(71);
      if (lookahead == '/') ADVANCE(75);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(72);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(75);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(sym_escape_sequence);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(76);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(77);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(79);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(80);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(sym_block_comment);
      END_STATE();
    case 82:
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
  [35] = {.lex_state = 2},
  [36] = {.lex_state = 2},
  [37] = {.lex_state = 2},
  [38] = {.lex_state = 2},
  [39] = {.lex_state = 2},
  [40] = {.lex_state = 2},
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
  [58] = {.lex_state = 1},
  [59] = {.lex_state = 0},
  [60] = {.lex_state = 1},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 1},
  [63] = {.lex_state = 0},
  [64] = {.lex_state = 0},
  [65] = {.lex_state = 0},
  [66] = {.lex_state = 0},
  [67] = {.lex_state = 0},
  [68] = {.lex_state = 0},
  [69] = {.lex_state = 0},
  [70] = {.lex_state = 0},
  [71] = {.lex_state = 0},
  [72] = {.lex_state = 0},
};

static uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_identifier] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
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
    [sym_source_file] = STATE(71),
    [sym__items] = STATE(65),
    [sym_assignment_item] = STATE(65),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_identifier] = ACTIONS(7),
    [anon_sym_SEMI] = ACTIONS(9),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [2] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [3] = {
    [ts_builtin_sym_end] = ACTIONS(35),
    [anon_sym_SEMI] = ACTIONS(35),
    [anon_sym_EQ] = ACTIONS(37),
    [anon_sym_LBRACK] = ACTIONS(35),
    [anon_sym_COMMA] = ACTIONS(35),
    [anon_sym_RBRACK] = ACTIONS(35),
    [anon_sym_LT_DASH_GT] = ACTIONS(35),
    [anon_sym_DASH_GT] = ACTIONS(35),
    [anon_sym_LT_DASH] = ACTIONS(37),
    [anon_sym_BSLASH_SLASH] = ACTIONS(35),
    [anon_sym_xor] = ACTIONS(35),
    [anon_sym_SLASH_BSLASH] = ACTIONS(35),
    [anon_sym_EQ_EQ] = ACTIONS(35),
    [anon_sym_BANG_EQ] = ACTIONS(35),
    [anon_sym_LT] = ACTIONS(37),
    [anon_sym_LT_EQ] = ACTIONS(35),
    [anon_sym_GT] = ACTIONS(37),
    [anon_sym_GT_EQ] = ACTIONS(35),
    [anon_sym_in] = ACTIONS(37),
    [anon_sym_subset] = ACTIONS(35),
    [anon_sym_superset] = ACTIONS(35),
    [anon_sym_union] = ACTIONS(35),
    [anon_sym_diff] = ACTIONS(35),
    [anon_sym_symdiff] = ACTIONS(35),
    [anon_sym_intersect] = ACTIONS(35),
    [anon_sym_DOT_DOT] = ACTIONS(35),
    [anon_sym_PLUS] = ACTIONS(37),
    [anon_sym_DASH] = ACTIONS(37),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(35),
    [anon_sym_SLASH] = ACTIONS(37),
    [anon_sym_div] = ACTIONS(35),
    [anon_sym_mod] = ACTIONS(35),
    [anon_sym_CARET] = ACTIONS(35),
    [anon_sym_COLON_COLON] = ACTIONS(35),
    [anon_sym_RBRACE] = ACTIONS(35),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [4] = {
    [ts_builtin_sym_end] = ACTIONS(39),
    [anon_sym_SEMI] = ACTIONS(39),
    [anon_sym_EQ] = ACTIONS(41),
    [anon_sym_LBRACK] = ACTIONS(39),
    [anon_sym_COMMA] = ACTIONS(39),
    [anon_sym_RBRACK] = ACTIONS(39),
    [anon_sym_LT_DASH_GT] = ACTIONS(39),
    [anon_sym_DASH_GT] = ACTIONS(39),
    [anon_sym_LT_DASH] = ACTIONS(41),
    [anon_sym_BSLASH_SLASH] = ACTIONS(39),
    [anon_sym_xor] = ACTIONS(39),
    [anon_sym_SLASH_BSLASH] = ACTIONS(39),
    [anon_sym_EQ_EQ] = ACTIONS(39),
    [anon_sym_BANG_EQ] = ACTIONS(39),
    [anon_sym_LT] = ACTIONS(41),
    [anon_sym_LT_EQ] = ACTIONS(39),
    [anon_sym_GT] = ACTIONS(41),
    [anon_sym_GT_EQ] = ACTIONS(39),
    [anon_sym_in] = ACTIONS(41),
    [anon_sym_subset] = ACTIONS(39),
    [anon_sym_superset] = ACTIONS(39),
    [anon_sym_union] = ACTIONS(39),
    [anon_sym_diff] = ACTIONS(39),
    [anon_sym_symdiff] = ACTIONS(39),
    [anon_sym_intersect] = ACTIONS(39),
    [anon_sym_DOT_DOT] = ACTIONS(39),
    [anon_sym_PLUS] = ACTIONS(41),
    [anon_sym_DASH] = ACTIONS(41),
    [anon_sym_PLUS_PLUS] = ACTIONS(39),
    [anon_sym_STAR] = ACTIONS(39),
    [anon_sym_SLASH] = ACTIONS(41),
    [anon_sym_div] = ACTIONS(39),
    [anon_sym_mod] = ACTIONS(39),
    [anon_sym_CARET] = ACTIONS(39),
    [anon_sym_COLON_COLON] = ACTIONS(39),
    [anon_sym_RBRACE] = ACTIONS(39),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [5] = {
    [ts_builtin_sym_end] = ACTIONS(43),
    [anon_sym_SEMI] = ACTIONS(43),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LBRACK] = ACTIONS(43),
    [anon_sym_COMMA] = ACTIONS(43),
    [anon_sym_RBRACK] = ACTIONS(43),
    [anon_sym_LT_DASH_GT] = ACTIONS(43),
    [anon_sym_DASH_GT] = ACTIONS(43),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(43),
    [anon_sym_xor] = ACTIONS(43),
    [anon_sym_SLASH_BSLASH] = ACTIONS(43),
    [anon_sym_EQ_EQ] = ACTIONS(43),
    [anon_sym_BANG_EQ] = ACTIONS(43),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(43),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(43),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(43),
    [anon_sym_superset] = ACTIONS(43),
    [anon_sym_union] = ACTIONS(43),
    [anon_sym_diff] = ACTIONS(43),
    [anon_sym_symdiff] = ACTIONS(43),
    [anon_sym_intersect] = ACTIONS(43),
    [anon_sym_DOT_DOT] = ACTIONS(43),
    [anon_sym_PLUS] = ACTIONS(45),
    [anon_sym_DASH] = ACTIONS(45),
    [anon_sym_PLUS_PLUS] = ACTIONS(43),
    [anon_sym_STAR] = ACTIONS(43),
    [anon_sym_SLASH] = ACTIONS(45),
    [anon_sym_div] = ACTIONS(43),
    [anon_sym_mod] = ACTIONS(43),
    [anon_sym_CARET] = ACTIONS(43),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [anon_sym_RBRACE] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [6] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(13),
    [anon_sym_DASH] = ACTIONS(13),
    [anon_sym_PLUS_PLUS] = ACTIONS(11),
    [anon_sym_STAR] = ACTIONS(11),
    [anon_sym_SLASH] = ACTIONS(13),
    [anon_sym_div] = ACTIONS(11),
    [anon_sym_mod] = ACTIONS(11),
    [anon_sym_CARET] = ACTIONS(11),
    [anon_sym_COLON_COLON] = ACTIONS(11),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [7] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(13),
    [anon_sym_DASH] = ACTIONS(13),
    [anon_sym_PLUS_PLUS] = ACTIONS(11),
    [anon_sym_STAR] = ACTIONS(11),
    [anon_sym_SLASH] = ACTIONS(13),
    [anon_sym_div] = ACTIONS(11),
    [anon_sym_mod] = ACTIONS(11),
    [anon_sym_CARET] = ACTIONS(11),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [8] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(13),
    [anon_sym_DASH] = ACTIONS(13),
    [anon_sym_PLUS_PLUS] = ACTIONS(11),
    [anon_sym_STAR] = ACTIONS(11),
    [anon_sym_SLASH] = ACTIONS(13),
    [anon_sym_div] = ACTIONS(11),
    [anon_sym_mod] = ACTIONS(11),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [9] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(13),
    [anon_sym_DASH] = ACTIONS(13),
    [anon_sym_PLUS_PLUS] = ACTIONS(11),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [10] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [11] = {
    [ts_builtin_sym_end] = ACTIONS(47),
    [anon_sym_SEMI] = ACTIONS(47),
    [anon_sym_EQ] = ACTIONS(49),
    [anon_sym_LBRACK] = ACTIONS(47),
    [anon_sym_COMMA] = ACTIONS(47),
    [anon_sym_RBRACK] = ACTIONS(47),
    [anon_sym_LT_DASH_GT] = ACTIONS(47),
    [anon_sym_DASH_GT] = ACTIONS(47),
    [anon_sym_LT_DASH] = ACTIONS(49),
    [anon_sym_BSLASH_SLASH] = ACTIONS(47),
    [anon_sym_xor] = ACTIONS(47),
    [anon_sym_SLASH_BSLASH] = ACTIONS(47),
    [anon_sym_EQ_EQ] = ACTIONS(47),
    [anon_sym_BANG_EQ] = ACTIONS(47),
    [anon_sym_LT] = ACTIONS(49),
    [anon_sym_LT_EQ] = ACTIONS(47),
    [anon_sym_GT] = ACTIONS(49),
    [anon_sym_GT_EQ] = ACTIONS(47),
    [anon_sym_in] = ACTIONS(49),
    [anon_sym_subset] = ACTIONS(47),
    [anon_sym_superset] = ACTIONS(47),
    [anon_sym_union] = ACTIONS(47),
    [anon_sym_diff] = ACTIONS(47),
    [anon_sym_symdiff] = ACTIONS(47),
    [anon_sym_intersect] = ACTIONS(47),
    [anon_sym_DOT_DOT] = ACTIONS(47),
    [anon_sym_PLUS] = ACTIONS(49),
    [anon_sym_DASH] = ACTIONS(49),
    [anon_sym_PLUS_PLUS] = ACTIONS(47),
    [anon_sym_STAR] = ACTIONS(47),
    [anon_sym_SLASH] = ACTIONS(49),
    [anon_sym_div] = ACTIONS(47),
    [anon_sym_mod] = ACTIONS(47),
    [anon_sym_CARET] = ACTIONS(47),
    [anon_sym_COLON_COLON] = ACTIONS(47),
    [anon_sym_RBRACE] = ACTIONS(47),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [12] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [13] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [14] = {
    [ts_builtin_sym_end] = ACTIONS(51),
    [anon_sym_SEMI] = ACTIONS(51),
    [anon_sym_EQ] = ACTIONS(53),
    [anon_sym_LBRACK] = ACTIONS(51),
    [anon_sym_COMMA] = ACTIONS(51),
    [anon_sym_RBRACK] = ACTIONS(51),
    [anon_sym_LT_DASH_GT] = ACTIONS(51),
    [anon_sym_DASH_GT] = ACTIONS(51),
    [anon_sym_LT_DASH] = ACTIONS(53),
    [anon_sym_BSLASH_SLASH] = ACTIONS(51),
    [anon_sym_xor] = ACTIONS(51),
    [anon_sym_SLASH_BSLASH] = ACTIONS(51),
    [anon_sym_EQ_EQ] = ACTIONS(51),
    [anon_sym_BANG_EQ] = ACTIONS(51),
    [anon_sym_LT] = ACTIONS(53),
    [anon_sym_LT_EQ] = ACTIONS(51),
    [anon_sym_GT] = ACTIONS(53),
    [anon_sym_GT_EQ] = ACTIONS(51),
    [anon_sym_in] = ACTIONS(53),
    [anon_sym_subset] = ACTIONS(51),
    [anon_sym_superset] = ACTIONS(51),
    [anon_sym_union] = ACTIONS(51),
    [anon_sym_diff] = ACTIONS(51),
    [anon_sym_symdiff] = ACTIONS(51),
    [anon_sym_intersect] = ACTIONS(51),
    [anon_sym_DOT_DOT] = ACTIONS(51),
    [anon_sym_PLUS] = ACTIONS(53),
    [anon_sym_DASH] = ACTIONS(53),
    [anon_sym_PLUS_PLUS] = ACTIONS(51),
    [anon_sym_STAR] = ACTIONS(51),
    [anon_sym_SLASH] = ACTIONS(53),
    [anon_sym_div] = ACTIONS(51),
    [anon_sym_mod] = ACTIONS(51),
    [anon_sym_CARET] = ACTIONS(51),
    [anon_sym_COLON_COLON] = ACTIONS(51),
    [anon_sym_RBRACE] = ACTIONS(51),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [15] = {
    [ts_builtin_sym_end] = ACTIONS(55),
    [anon_sym_SEMI] = ACTIONS(55),
    [anon_sym_EQ] = ACTIONS(57),
    [anon_sym_LBRACK] = ACTIONS(55),
    [anon_sym_COMMA] = ACTIONS(55),
    [anon_sym_RBRACK] = ACTIONS(55),
    [anon_sym_LT_DASH_GT] = ACTIONS(55),
    [anon_sym_DASH_GT] = ACTIONS(55),
    [anon_sym_LT_DASH] = ACTIONS(57),
    [anon_sym_BSLASH_SLASH] = ACTIONS(55),
    [anon_sym_xor] = ACTIONS(55),
    [anon_sym_SLASH_BSLASH] = ACTIONS(55),
    [anon_sym_EQ_EQ] = ACTIONS(55),
    [anon_sym_BANG_EQ] = ACTIONS(55),
    [anon_sym_LT] = ACTIONS(57),
    [anon_sym_LT_EQ] = ACTIONS(55),
    [anon_sym_GT] = ACTIONS(57),
    [anon_sym_GT_EQ] = ACTIONS(55),
    [anon_sym_in] = ACTIONS(57),
    [anon_sym_subset] = ACTIONS(55),
    [anon_sym_superset] = ACTIONS(55),
    [anon_sym_union] = ACTIONS(55),
    [anon_sym_diff] = ACTIONS(55),
    [anon_sym_symdiff] = ACTIONS(55),
    [anon_sym_intersect] = ACTIONS(55),
    [anon_sym_DOT_DOT] = ACTIONS(55),
    [anon_sym_PLUS] = ACTIONS(57),
    [anon_sym_DASH] = ACTIONS(57),
    [anon_sym_PLUS_PLUS] = ACTIONS(55),
    [anon_sym_STAR] = ACTIONS(55),
    [anon_sym_SLASH] = ACTIONS(57),
    [anon_sym_div] = ACTIONS(55),
    [anon_sym_mod] = ACTIONS(55),
    [anon_sym_CARET] = ACTIONS(55),
    [anon_sym_COLON_COLON] = ACTIONS(55),
    [anon_sym_RBRACE] = ACTIONS(55),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [16] = {
    [ts_builtin_sym_end] = ACTIONS(59),
    [anon_sym_SEMI] = ACTIONS(59),
    [anon_sym_EQ] = ACTIONS(61),
    [anon_sym_LBRACK] = ACTIONS(59),
    [anon_sym_COMMA] = ACTIONS(59),
    [anon_sym_RBRACK] = ACTIONS(59),
    [anon_sym_LT_DASH_GT] = ACTIONS(59),
    [anon_sym_DASH_GT] = ACTIONS(59),
    [anon_sym_LT_DASH] = ACTIONS(61),
    [anon_sym_BSLASH_SLASH] = ACTIONS(59),
    [anon_sym_xor] = ACTIONS(59),
    [anon_sym_SLASH_BSLASH] = ACTIONS(59),
    [anon_sym_EQ_EQ] = ACTIONS(59),
    [anon_sym_BANG_EQ] = ACTIONS(59),
    [anon_sym_LT] = ACTIONS(61),
    [anon_sym_LT_EQ] = ACTIONS(59),
    [anon_sym_GT] = ACTIONS(61),
    [anon_sym_GT_EQ] = ACTIONS(59),
    [anon_sym_in] = ACTIONS(61),
    [anon_sym_subset] = ACTIONS(59),
    [anon_sym_superset] = ACTIONS(59),
    [anon_sym_union] = ACTIONS(59),
    [anon_sym_diff] = ACTIONS(59),
    [anon_sym_symdiff] = ACTIONS(59),
    [anon_sym_intersect] = ACTIONS(59),
    [anon_sym_DOT_DOT] = ACTIONS(59),
    [anon_sym_PLUS] = ACTIONS(61),
    [anon_sym_DASH] = ACTIONS(61),
    [anon_sym_PLUS_PLUS] = ACTIONS(59),
    [anon_sym_STAR] = ACTIONS(59),
    [anon_sym_SLASH] = ACTIONS(61),
    [anon_sym_div] = ACTIONS(59),
    [anon_sym_mod] = ACTIONS(59),
    [anon_sym_CARET] = ACTIONS(59),
    [anon_sym_COLON_COLON] = ACTIONS(59),
    [anon_sym_RBRACE] = ACTIONS(59),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [17] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [18] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [19] = {
    [ts_builtin_sym_end] = ACTIONS(73),
    [anon_sym_SEMI] = ACTIONS(73),
    [anon_sym_EQ] = ACTIONS(75),
    [anon_sym_LBRACK] = ACTIONS(73),
    [anon_sym_COMMA] = ACTIONS(73),
    [anon_sym_RBRACK] = ACTIONS(73),
    [anon_sym_LT_DASH_GT] = ACTIONS(73),
    [anon_sym_DASH_GT] = ACTIONS(73),
    [anon_sym_LT_DASH] = ACTIONS(75),
    [anon_sym_BSLASH_SLASH] = ACTIONS(73),
    [anon_sym_xor] = ACTIONS(73),
    [anon_sym_SLASH_BSLASH] = ACTIONS(73),
    [anon_sym_EQ_EQ] = ACTIONS(73),
    [anon_sym_BANG_EQ] = ACTIONS(73),
    [anon_sym_LT] = ACTIONS(75),
    [anon_sym_LT_EQ] = ACTIONS(73),
    [anon_sym_GT] = ACTIONS(75),
    [anon_sym_GT_EQ] = ACTIONS(73),
    [anon_sym_in] = ACTIONS(75),
    [anon_sym_subset] = ACTIONS(73),
    [anon_sym_superset] = ACTIONS(73),
    [anon_sym_union] = ACTIONS(73),
    [anon_sym_diff] = ACTIONS(73),
    [anon_sym_symdiff] = ACTIONS(73),
    [anon_sym_intersect] = ACTIONS(73),
    [anon_sym_DOT_DOT] = ACTIONS(73),
    [anon_sym_PLUS] = ACTIONS(75),
    [anon_sym_DASH] = ACTIONS(75),
    [anon_sym_PLUS_PLUS] = ACTIONS(73),
    [anon_sym_STAR] = ACTIONS(73),
    [anon_sym_SLASH] = ACTIONS(75),
    [anon_sym_div] = ACTIONS(73),
    [anon_sym_mod] = ACTIONS(73),
    [anon_sym_CARET] = ACTIONS(73),
    [anon_sym_COLON_COLON] = ACTIONS(73),
    [anon_sym_RBRACE] = ACTIONS(73),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [20] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(77),
    [anon_sym_LT_DASH] = ACTIONS(79),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(77),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [21] = {
    [ts_builtin_sym_end] = ACTIONS(81),
    [anon_sym_SEMI] = ACTIONS(81),
    [anon_sym_EQ] = ACTIONS(83),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(81),
    [anon_sym_RBRACK] = ACTIONS(81),
    [anon_sym_LT_DASH_GT] = ACTIONS(81),
    [anon_sym_DASH_GT] = ACTIONS(81),
    [anon_sym_LT_DASH] = ACTIONS(83),
    [anon_sym_BSLASH_SLASH] = ACTIONS(81),
    [anon_sym_xor] = ACTIONS(81),
    [anon_sym_SLASH_BSLASH] = ACTIONS(81),
    [anon_sym_EQ_EQ] = ACTIONS(81),
    [anon_sym_BANG_EQ] = ACTIONS(81),
    [anon_sym_LT] = ACTIONS(83),
    [anon_sym_LT_EQ] = ACTIONS(81),
    [anon_sym_GT] = ACTIONS(83),
    [anon_sym_GT_EQ] = ACTIONS(81),
    [anon_sym_in] = ACTIONS(83),
    [anon_sym_subset] = ACTIONS(81),
    [anon_sym_superset] = ACTIONS(81),
    [anon_sym_union] = ACTIONS(81),
    [anon_sym_diff] = ACTIONS(81),
    [anon_sym_symdiff] = ACTIONS(81),
    [anon_sym_intersect] = ACTIONS(81),
    [anon_sym_DOT_DOT] = ACTIONS(81),
    [anon_sym_PLUS] = ACTIONS(83),
    [anon_sym_DASH] = ACTIONS(83),
    [anon_sym_PLUS_PLUS] = ACTIONS(81),
    [anon_sym_STAR] = ACTIONS(81),
    [anon_sym_SLASH] = ACTIONS(83),
    [anon_sym_div] = ACTIONS(81),
    [anon_sym_mod] = ACTIONS(81),
    [anon_sym_CARET] = ACTIONS(81),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(81),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [22] = {
    [ts_builtin_sym_end] = ACTIONS(85),
    [anon_sym_SEMI] = ACTIONS(85),
    [anon_sym_EQ] = ACTIONS(87),
    [anon_sym_LBRACK] = ACTIONS(85),
    [anon_sym_COMMA] = ACTIONS(85),
    [anon_sym_RBRACK] = ACTIONS(85),
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
    [anon_sym_RBRACE] = ACTIONS(85),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [23] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [24] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(11),
    [anon_sym_RBRACK] = ACTIONS(11),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(13),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [25] = {
    [ts_builtin_sym_end] = ACTIONS(89),
    [anon_sym_SEMI] = ACTIONS(89),
    [anon_sym_EQ] = ACTIONS(91),
    [anon_sym_LBRACK] = ACTIONS(89),
    [anon_sym_COMMA] = ACTIONS(89),
    [anon_sym_RBRACK] = ACTIONS(89),
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
    [anon_sym_RBRACE] = ACTIONS(89),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [26] = {
    [ts_builtin_sym_end] = ACTIONS(93),
    [anon_sym_SEMI] = ACTIONS(93),
    [anon_sym_EQ] = ACTIONS(95),
    [anon_sym_LBRACK] = ACTIONS(93),
    [anon_sym_COMMA] = ACTIONS(93),
    [anon_sym_RBRACK] = ACTIONS(93),
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
    [anon_sym_RBRACE] = ACTIONS(93),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [27] = {
    [aux_sym_index_expression_repeat1] = STATE(66),
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(97),
    [anon_sym_RBRACK] = ACTIONS(99),
    [anon_sym_LT_DASH_GT] = ACTIONS(101),
    [anon_sym_DASH_GT] = ACTIONS(77),
    [anon_sym_LT_DASH] = ACTIONS(79),
    [anon_sym_BSLASH_SLASH] = ACTIONS(101),
    [anon_sym_xor] = ACTIONS(77),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [28] = {
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(103),
    [anon_sym_RBRACK] = ACTIONS(105),
    [anon_sym_LT_DASH_GT] = ACTIONS(101),
    [anon_sym_DASH_GT] = ACTIONS(77),
    [anon_sym_LT_DASH] = ACTIONS(79),
    [anon_sym_BSLASH_SLASH] = ACTIONS(101),
    [anon_sym_xor] = ACTIONS(77),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [29] = {
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(103),
    [anon_sym_LT_DASH_GT] = ACTIONS(101),
    [anon_sym_DASH_GT] = ACTIONS(77),
    [anon_sym_LT_DASH] = ACTIONS(79),
    [anon_sym_BSLASH_SLASH] = ACTIONS(101),
    [anon_sym_xor] = ACTIONS(77),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(107),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [30] = {
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(109),
    [anon_sym_RBRACK] = ACTIONS(109),
    [anon_sym_LT_DASH_GT] = ACTIONS(101),
    [anon_sym_DASH_GT] = ACTIONS(77),
    [anon_sym_LT_DASH] = ACTIONS(79),
    [anon_sym_BSLASH_SLASH] = ACTIONS(101),
    [anon_sym_xor] = ACTIONS(77),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [31] = {
    [ts_builtin_sym_end] = ACTIONS(111),
    [anon_sym_SEMI] = ACTIONS(111),
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_LT_DASH_GT] = ACTIONS(101),
    [anon_sym_DASH_GT] = ACTIONS(77),
    [anon_sym_LT_DASH] = ACTIONS(79),
    [anon_sym_BSLASH_SLASH] = ACTIONS(101),
    [anon_sym_xor] = ACTIONS(77),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [32] = {
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(103),
    [anon_sym_LT_DASH_GT] = ACTIONS(101),
    [anon_sym_DASH_GT] = ACTIONS(77),
    [anon_sym_LT_DASH] = ACTIONS(79),
    [anon_sym_BSLASH_SLASH] = ACTIONS(101),
    [anon_sym_xor] = ACTIONS(77),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [anon_sym_RBRACE] = ACTIONS(113),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [33] = {
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(103),
    [anon_sym_RBRACK] = ACTIONS(115),
    [anon_sym_LT_DASH_GT] = ACTIONS(101),
    [anon_sym_DASH_GT] = ACTIONS(77),
    [anon_sym_LT_DASH] = ACTIONS(79),
    [anon_sym_BSLASH_SLASH] = ACTIONS(101),
    [anon_sym_xor] = ACTIONS(77),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [34] = {
    [anon_sym_EQ] = ACTIONS(63),
    [anon_sym_LBRACK] = ACTIONS(15),
    [anon_sym_COMMA] = ACTIONS(103),
    [anon_sym_LT_DASH_GT] = ACTIONS(101),
    [anon_sym_DASH_GT] = ACTIONS(77),
    [anon_sym_LT_DASH] = ACTIONS(79),
    [anon_sym_BSLASH_SLASH] = ACTIONS(101),
    [anon_sym_xor] = ACTIONS(77),
    [anon_sym_SLASH_BSLASH] = ACTIONS(65),
    [anon_sym_EQ_EQ] = ACTIONS(67),
    [anon_sym_BANG_EQ] = ACTIONS(67),
    [anon_sym_LT] = ACTIONS(63),
    [anon_sym_LT_EQ] = ACTIONS(67),
    [anon_sym_GT] = ACTIONS(63),
    [anon_sym_GT_EQ] = ACTIONS(67),
    [anon_sym_in] = ACTIONS(63),
    [anon_sym_subset] = ACTIONS(67),
    [anon_sym_superset] = ACTIONS(67),
    [anon_sym_union] = ACTIONS(69),
    [anon_sym_diff] = ACTIONS(71),
    [anon_sym_symdiff] = ACTIONS(17),
    [anon_sym_intersect] = ACTIONS(19),
    [anon_sym_DOT_DOT] = ACTIONS(21),
    [anon_sym_PLUS] = ACTIONS(23),
    [anon_sym_DASH] = ACTIONS(23),
    [anon_sym_PLUS_PLUS] = ACTIONS(25),
    [anon_sym_STAR] = ACTIONS(27),
    [anon_sym_SLASH] = ACTIONS(29),
    [anon_sym_div] = ACTIONS(27),
    [anon_sym_mod] = ACTIONS(27),
    [anon_sym_CARET] = ACTIONS(31),
    [anon_sym_COLON_COLON] = ACTIONS(33),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
};

static uint16_t ts_small_parse_table[] = {
  [0] = 12,
    ACTIONS(120), 1,
      anon_sym_LBRACK,
    ACTIONS(128), 1,
      anon_sym_not,
    ACTIONS(137), 1,
      anon_sym_LBRACE,
    ACTIONS(140), 1,
      anon_sym_DQUOTE,
    STATE(35), 1,
      aux_sym_array_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(117), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(123), 2,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
    ACTIONS(125), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(131), 2,
      sym_absent,
      sym_float_literal,
    ACTIONS(134), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(34), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [51] = 12,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(147), 1,
      anon_sym_RBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    STATE(38), 1,
      aux_sym_array_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(143), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(153), 2,
      sym_absent,
      sym_float_literal,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(28), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [101] = 12,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(165), 1,
      anon_sym_RBRACE,
    STATE(39), 1,
      aux_sym_array_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(161), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(163), 2,
      sym_absent,
      sym_float_literal,
    STATE(32), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [151] = 12,
    ACTIONS(105), 1,
      anon_sym_RBRACK,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    STATE(35), 1,
      aux_sym_array_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(167), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(169), 2,
      sym_absent,
      sym_float_literal,
    STATE(33), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [201] = 12,
    ACTIONS(113), 1,
      anon_sym_RBRACE,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    STATE(35), 1,
      aux_sym_array_literal_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(171), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(173), 2,
      sym_absent,
      sym_float_literal,
    STATE(29), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [251] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(175), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(177), 2,
      sym_absent,
      sym_float_literal,
    STATE(6), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [295] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(179), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(181), 2,
      sym_absent,
      sym_float_literal,
    STATE(18), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [339] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(183), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(185), 2,
      sym_absent,
      sym_float_literal,
    STATE(7), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [383] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(187), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(189), 2,
      sym_absent,
      sym_float_literal,
    STATE(20), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [427] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(191), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(193), 2,
      sym_absent,
      sym_float_literal,
    STATE(23), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [471] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(195), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(197), 2,
      sym_absent,
      sym_float_literal,
    STATE(30), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [515] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(199), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(201), 2,
      sym_absent,
      sym_float_literal,
    STATE(8), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [559] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(203), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(205), 2,
      sym_absent,
      sym_float_literal,
    STATE(27), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [603] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(207), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(209), 2,
      sym_absent,
      sym_float_literal,
    STATE(24), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [647] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(211), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(213), 2,
      sym_absent,
      sym_float_literal,
    STATE(31), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [691] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(215), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(217), 2,
      sym_absent,
      sym_float_literal,
    STATE(2), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [735] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(219), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(221), 2,
      sym_absent,
      sym_float_literal,
    STATE(17), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [779] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(223), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(225), 2,
      sym_absent,
      sym_float_literal,
    STATE(9), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [823] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(227), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(229), 2,
      sym_absent,
      sym_float_literal,
    STATE(10), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [867] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(231), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(233), 2,
      sym_absent,
      sym_float_literal,
    STATE(21), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [911] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(235), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(237), 2,
      sym_absent,
      sym_float_literal,
    STATE(13), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [955] = 10,
    ACTIONS(145), 1,
      anon_sym_LBRACK,
    ACTIONS(151), 1,
      anon_sym_not,
    ACTIONS(157), 1,
      anon_sym_LBRACE,
    ACTIONS(159), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(149), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(155), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(239), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(241), 2,
      sym_absent,
      sym_float_literal,
    STATE(12), 9,
      sym__expression,
      sym_index_expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_array_literal,
      sym_boolean_literal,
      sym_set_literal,
      sym_string_literal,
  [999] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(243), 5,
      anon_sym_not,
      anon_sym_true,
      anon_sym_false,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(123), 9,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      anon_sym_DASH,
      anon_sym_,
      sym_absent,
      sym_float_literal,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DQUOTE,
  [1022] = 4,
    ACTIONS(245), 1,
      anon_sym_DQUOTE,
    STATE(60), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(247), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
    ACTIONS(249), 2,
      sym_line_comment,
      sym_block_comment,
  [1037] = 4,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(251), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(69), 2,
      sym__items,
      sym_assignment_item,
  [1052] = 4,
    ACTIONS(253), 1,
      anon_sym_DQUOTE,
    STATE(62), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(249), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(255), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
  [1067] = 4,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(257), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(69), 2,
      sym__items,
      sym_assignment_item,
  [1082] = 4,
    ACTIONS(259), 1,
      anon_sym_DQUOTE,
    STATE(62), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(249), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(261), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
  [1097] = 4,
    ACTIONS(264), 1,
      ts_builtin_sym_end,
    ACTIONS(266), 1,
      anon_sym_SEMI,
    STATE(63), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1111] = 4,
    ACTIONS(257), 1,
      ts_builtin_sym_end,
    ACTIONS(269), 1,
      anon_sym_SEMI,
    STATE(63), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1125] = 4,
    ACTIONS(271), 1,
      ts_builtin_sym_end,
    ACTIONS(273), 1,
      anon_sym_SEMI,
    STATE(64), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1139] = 4,
    ACTIONS(97), 1,
      anon_sym_COMMA,
    ACTIONS(275), 1,
      anon_sym_RBRACK,
    STATE(68), 1,
      aux_sym_index_expression_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1153] = 3,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(69), 2,
      sym__items,
      sym_assignment_item,
  [1165] = 4,
    ACTIONS(109), 1,
      anon_sym_RBRACK,
    ACTIONS(277), 1,
      anon_sym_COMMA,
    STATE(68), 1,
      aux_sym_index_expression_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1179] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(264), 2,
      ts_builtin_sym_end,
      anon_sym_SEMI,
  [1188] = 2,
    ACTIONS(271), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1196] = 2,
    ACTIONS(280), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1204] = 2,
    ACTIONS(282), 1,
      anon_sym_EQ,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
};

static uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(35)] = 0,
  [SMALL_STATE(36)] = 51,
  [SMALL_STATE(37)] = 101,
  [SMALL_STATE(38)] = 151,
  [SMALL_STATE(39)] = 201,
  [SMALL_STATE(40)] = 251,
  [SMALL_STATE(41)] = 295,
  [SMALL_STATE(42)] = 339,
  [SMALL_STATE(43)] = 383,
  [SMALL_STATE(44)] = 427,
  [SMALL_STATE(45)] = 471,
  [SMALL_STATE(46)] = 515,
  [SMALL_STATE(47)] = 559,
  [SMALL_STATE(48)] = 603,
  [SMALL_STATE(49)] = 647,
  [SMALL_STATE(50)] = 691,
  [SMALL_STATE(51)] = 735,
  [SMALL_STATE(52)] = 779,
  [SMALL_STATE(53)] = 823,
  [SMALL_STATE(54)] = 867,
  [SMALL_STATE(55)] = 911,
  [SMALL_STATE(56)] = 955,
  [SMALL_STATE(57)] = 999,
  [SMALL_STATE(58)] = 1022,
  [SMALL_STATE(59)] = 1037,
  [SMALL_STATE(60)] = 1052,
  [SMALL_STATE(61)] = 1067,
  [SMALL_STATE(62)] = 1082,
  [SMALL_STATE(63)] = 1097,
  [SMALL_STATE(64)] = 1111,
  [SMALL_STATE(65)] = 1125,
  [SMALL_STATE(66)] = 1139,
  [SMALL_STATE(67)] = 1153,
  [SMALL_STATE(68)] = 1165,
  [SMALL_STATE(69)] = 1179,
  [SMALL_STATE(70)] = 1188,
  [SMALL_STATE(71)] = 1196,
  [SMALL_STATE(72)] = 1204,
};

static TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [11] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_binary_operation, 3, .production_id = 3),
  [13] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_binary_operation, 3, .production_id = 3),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [23] = {.entry = {.count = 1, .reusable = false}}, SHIFT(52),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(46),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [35] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 2),
  [37] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 2),
  [39] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 3),
  [41] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 3),
  [43] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array_literal, 4),
  [45] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array_literal, 4),
  [47] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean_literal, 1),
  [49] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean_literal, 1),
  [51] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 4),
  [53] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 4),
  [55] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_index_expression, 4, .production_id = 4),
  [57] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_index_expression, 4, .production_id = 4),
  [59] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 3),
  [61] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 3),
  [63] = {.entry = {.count = 1, .reusable = false}}, SHIFT(48),
  [65] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [67] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [69] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [71] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [73] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_index_expression, 5, .production_id = 5),
  [75] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_index_expression, 5, .production_id = 5),
  [77] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [79] = {.entry = {.count = 1, .reusable = false}}, SHIFT(51),
  [81] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_unary_operation, 2, .production_id = 2),
  [83] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_unary_operation, 2, .production_id = 2),
  [85] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_literal, 2),
  [87] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_set_literal, 2),
  [89] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 2),
  [91] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 2),
  [93] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 3),
  [95] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 3),
  [97] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [99] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [101] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [103] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [105] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [107] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [109] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_index_expression_repeat1, 2),
  [111] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment_item, 3, .production_id = 1),
  [113] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [115] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [117] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2), SHIFT_REPEAT(34),
  [120] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2), SHIFT_REPEAT(36),
  [123] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2),
  [125] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2), SHIFT_REPEAT(54),
  [128] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2), SHIFT_REPEAT(54),
  [131] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2), SHIFT_REPEAT(34),
  [134] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2), SHIFT_REPEAT(11),
  [137] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2), SHIFT_REPEAT(37),
  [140] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_literal_repeat1, 2), SHIFT_REPEAT(58),
  [143] = {.entry = {.count = 1, .reusable = false}}, SHIFT(28),
  [145] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [147] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [149] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [151] = {.entry = {.count = 1, .reusable = false}}, SHIFT(54),
  [153] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [155] = {.entry = {.count = 1, .reusable = false}}, SHIFT(11),
  [157] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [159] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [161] = {.entry = {.count = 1, .reusable = false}}, SHIFT(32),
  [163] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [165] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [167] = {.entry = {.count = 1, .reusable = false}}, SHIFT(33),
  [169] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [171] = {.entry = {.count = 1, .reusable = false}}, SHIFT(29),
  [173] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [175] = {.entry = {.count = 1, .reusable = false}}, SHIFT(6),
  [177] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [179] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [181] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [183] = {.entry = {.count = 1, .reusable = false}}, SHIFT(7),
  [185] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [187] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [189] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [191] = {.entry = {.count = 1, .reusable = false}}, SHIFT(23),
  [193] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [195] = {.entry = {.count = 1, .reusable = false}}, SHIFT(30),
  [197] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [199] = {.entry = {.count = 1, .reusable = false}}, SHIFT(8),
  [201] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [203] = {.entry = {.count = 1, .reusable = false}}, SHIFT(27),
  [205] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [207] = {.entry = {.count = 1, .reusable = false}}, SHIFT(24),
  [209] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [211] = {.entry = {.count = 1, .reusable = false}}, SHIFT(31),
  [213] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [215] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [217] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [219] = {.entry = {.count = 1, .reusable = false}}, SHIFT(17),
  [221] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [223] = {.entry = {.count = 1, .reusable = false}}, SHIFT(9),
  [225] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [227] = {.entry = {.count = 1, .reusable = false}}, SHIFT(10),
  [229] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [231] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [233] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [235] = {.entry = {.count = 1, .reusable = false}}, SHIFT(13),
  [237] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [239] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [241] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [243] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_array_literal_repeat1, 2),
  [245] = {.entry = {.count = 1, .reusable = false}}, SHIFT(25),
  [247] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [249] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [251] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 3),
  [253] = {.entry = {.count = 1, .reusable = false}}, SHIFT(26),
  [255] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [257] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 2),
  [259] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_literal_repeat1, 2),
  [261] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_literal_repeat1, 2), SHIFT_REPEAT(62),
  [264] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2),
  [266] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(67),
  [269] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [271] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1),
  [273] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [275] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [277] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_index_expression_repeat1, 2), SHIFT_REPEAT(45),
  [280] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [282] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
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
