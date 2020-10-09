#include <tree_sitter/parser.h>

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 11
#define STATE_COUNT 48
#define LARGE_STATE_COUNT 20
#define SYMBOL_COUNT 56
#define ALIAS_COUNT 0
#define TOKEN_COUNT 45
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 5
#define MAX_ALIAS_SEQUENCE_LENGTH 3

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
  anon_sym_not = 33,
  anon_sym_ = 34,
  sym_absent = 35,
  anon_sym_true = 36,
  anon_sym_false = 37,
  sym_float_literal = 38,
  sym_integer_literal = 39,
  anon_sym_DQUOTE = 40,
  aux_sym_string_literal_token1 = 41,
  sym_escape_sequence = 42,
  sym_line_comment = 43,
  sym_block_comment = 44,
  sym_source_file = 45,
  sym__items = 46,
  sym_assignment_item = 47,
  sym__expression = 48,
  sym_binary_operation = 49,
  sym_unary_operation = 50,
  sym__literal = 51,
  sym_boolean_literal = 52,
  sym_string_literal = 53,
  aux_sym_source_file_repeat1 = 54,
  aux_sym_string_literal_repeat1 = 55,
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
  [anon_sym_not] = "not",
  [anon_sym_] = "¬",
  [sym_absent] = "absent",
  [anon_sym_true] = "true",
  [anon_sym_false] = "false",
  [sym_float_literal] = "float_literal",
  [sym_integer_literal] = "integer_literal",
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
  [sym_unary_operation] = "unary_operation",
  [sym__literal] = "_literal",
  [sym_boolean_literal] = "boolean_literal",
  [sym_string_literal] = "string_literal",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
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
  [anon_sym_not] = anon_sym_not,
  [anon_sym_] = anon_sym_,
  [sym_absent] = sym_absent,
  [anon_sym_true] = anon_sym_true,
  [anon_sym_false] = anon_sym_false,
  [sym_float_literal] = sym_float_literal,
  [sym_integer_literal] = sym_integer_literal,
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
  [sym_unary_operation] = sym_unary_operation,
  [sym__literal] = sym__literal,
  [sym_boolean_literal] = sym_boolean_literal,
  [sym_string_literal] = sym_string_literal,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
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
  [sym_unary_operation] = {
    .visible = true,
    .named = true,
  },
  [sym__literal] = {
    .visible = false,
    .named = true,
  },
  [sym_boolean_literal] = {
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
  [aux_sym_string_literal_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum {
  field_expr = 1,
  field_left = 2,
  field_name = 3,
  field_operator = 4,
  field_right = 5,
};

static const char *ts_field_names[] = {
  [0] = NULL,
  [field_expr] = "expr",
  [field_left] = "left",
  [field_name] = "name",
  [field_operator] = "operator",
  [field_right] = "right",
};

static const TSFieldMapSlice ts_field_map_slices[4] = {
  [1] = {.index = 0, .length = 2},
  [2] = {.index = 2, .length = 1},
  [3] = {.index = 3, .length = 3},
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
};

static TSSymbol ts_alias_sequences[4][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(31);
      if (lookahead == '!') ADVANCE(10);
      if (lookahead == '"') ADVANCE(64);
      if (lookahead == '%') ADVANCE(75);
      if (lookahead == '*') ADVANCE(51);
      if (lookahead == '+') ADVANCE(47);
      if (lookahead == '-') ADVANCE(49);
      if (lookahead == '.') ADVANCE(6);
      if (lookahead == '/') ADVANCE(52);
      if (lookahead == '0') ADVANCE(59);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == ';') ADVANCE(32);
      if (lookahead == '<') ADVANCE(42);
      if (lookahead == '=') ADVANCE(33);
      if (lookahead == '>') ADVANCE(44);
      if (lookahead == '\\') ADVANCE(8);
      if (lookahead == '^') ADVANCE(53);
      if (lookahead == 172) ADVANCE(55);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(29)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(60);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(74);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(3)
      if (lookahead == '"') ADVANCE(64);
      if (lookahead == '%') ADVANCE(70);
      if (lookahead == '/') ADVANCE(68);
      if (lookahead == '\\') ADVANCE(12);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(65);
      if (lookahead != 0) ADVANCE(70);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(64);
      if (lookahead == '%') ADVANCE(75);
      if (lookahead == '-') ADVANCE(48);
      if (lookahead == '/') ADVANCE(4);
      if (lookahead == '0') ADVANCE(59);
      if (lookahead == '<') ADVANCE(11);
      if (lookahead == 172) ADVANCE(55);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(2)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(60);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(74);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(64);
      if (lookahead == '%') ADVANCE(75);
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
      if (lookahead == '/') ADVANCE(76);
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
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(73);
      if (lookahead != 0) ADVANCE(71);
      END_STATE();
    case 9:
      if (lookahead == ':') ADVANCE(54);
      END_STATE();
    case 10:
      if (lookahead == '=') ADVANCE(40);
      END_STATE();
    case 11:
      if (lookahead == '>') ADVANCE(56);
      END_STATE();
    case 12:
      if (lookahead == 'U') ADVANCE(26);
      if (lookahead == 'u') ADVANCE(22);
      if (lookahead == 'x') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(73);
      if (lookahead != 0) ADVANCE(71);
      END_STATE();
    case 13:
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(17);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(58);
      END_STATE();
    case 14:
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(61);
      END_STATE();
    case 15:
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(62);
      END_STATE();
    case 16:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(57);
      END_STATE();
    case 17:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(58);
      END_STATE();
    case 18:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(71);
      END_STATE();
    case 19:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(63);
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
      if (lookahead == '/') ADVANCE(77);
      END_STATE();
    case 29:
      if (eof) ADVANCE(31);
      if (lookahead == '!') ADVANCE(10);
      if (lookahead == '"') ADVANCE(64);
      if (lookahead == '%') ADVANCE(75);
      if (lookahead == '*') ADVANCE(51);
      if (lookahead == '+') ADVANCE(47);
      if (lookahead == '-') ADVANCE(49);
      if (lookahead == '.') ADVANCE(6);
      if (lookahead == '/') ADVANCE(52);
      if (lookahead == '0') ADVANCE(59);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == ';') ADVANCE(32);
      if (lookahead == '<') ADVANCE(42);
      if (lookahead == '=') ADVANCE(33);
      if (lookahead == '>') ADVANCE(44);
      if (lookahead == '\\') ADVANCE(7);
      if (lookahead == '^') ADVANCE(53);
      if (lookahead == 172) ADVANCE(55);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(29)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(60);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(74);
      END_STATE();
    case 30:
      if (eof) ADVANCE(31);
      if (lookahead == '!') ADVANCE(10);
      if (lookahead == '%') ADVANCE(75);
      if (lookahead == '*') ADVANCE(51);
      if (lookahead == '+') ADVANCE(47);
      if (lookahead == '-') ADVANCE(49);
      if (lookahead == '.') ADVANCE(6);
      if (lookahead == '/') ADVANCE(52);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == ';') ADVANCE(32);
      if (lookahead == '<') ADVANCE(41);
      if (lookahead == '=') ADVANCE(33);
      if (lookahead == '>') ADVANCE(44);
      if (lookahead == '\\') ADVANCE(7);
      if (lookahead == '^') ADVANCE(53);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(30)
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(74);
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
      if (lookahead == '>') ADVANCE(56);
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
      ACCEPT_TOKEN(anon_sym_);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(sym_absent);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(sym_float_literal);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(13);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(57);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(58);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(16);
      if (lookahead == 'b') ADVANCE(14);
      if (lookahead == 'o') ADVANCE(15);
      if (lookahead == 'x') ADVANCE(19);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(13);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(60);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(16);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(13);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(60);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(61);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(62);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(sym_integer_literal);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(63);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '%') ADVANCE(70);
      if (lookahead == '/') ADVANCE(68);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(65);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(70);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(69);
      if (lookahead == '/') ADVANCE(67);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(67);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(69);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(67);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(67);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(70);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '*') ADVANCE(66);
      if (lookahead == '/') ADVANCE(70);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(67);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(70);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(sym_escape_sequence);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(71);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(72);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(74);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(75);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(sym_block_comment);
      END_STATE();
    case 77:
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
  [35] = {.lex_state = 1},
  [36] = {.lex_state = 0},
  [37] = {.lex_state = 1},
  [38] = {.lex_state = 1},
  [39] = {.lex_state = 0},
  [40] = {.lex_state = 0},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 0},
  [43] = {.lex_state = 0},
  [44] = {.lex_state = 0},
  [45] = {.lex_state = 0},
  [46] = {.lex_state = 0},
  [47] = {.lex_state = 0},
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
    [anon_sym_not] = ACTIONS(1),
    [anon_sym_] = ACTIONS(1),
    [sym_absent] = ACTIONS(1),
    [anon_sym_true] = ACTIONS(1),
    [anon_sym_false] = ACTIONS(1),
    [sym_float_literal] = ACTIONS(1),
    [sym_integer_literal] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [sym_escape_sequence] = ACTIONS(1),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [1] = {
    [sym_source_file] = STATE(46),
    [sym__items] = STATE(43),
    [sym_assignment_item] = STATE(43),
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
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(19),
    [anon_sym_EQ_EQ] = ACTIONS(21),
    [anon_sym_BANG_EQ] = ACTIONS(21),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(21),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(21),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(21),
    [anon_sym_superset] = ACTIONS(21),
    [anon_sym_union] = ACTIONS(23),
    [anon_sym_diff] = ACTIONS(25),
    [anon_sym_symdiff] = ACTIONS(27),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(31),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [3] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [4] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(45),
    [anon_sym_DASH] = ACTIONS(45),
    [anon_sym_PLUS_PLUS] = ACTIONS(11),
    [anon_sym_STAR] = ACTIONS(11),
    [anon_sym_SLASH] = ACTIONS(45),
    [anon_sym_div] = ACTIONS(11),
    [anon_sym_mod] = ACTIONS(11),
    [anon_sym_CARET] = ACTIONS(11),
    [anon_sym_COLON_COLON] = ACTIONS(11),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [5] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(31),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [6] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(27),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(31),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [7] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(25),
    [anon_sym_symdiff] = ACTIONS(27),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(31),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [8] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(21),
    [anon_sym_BANG_EQ] = ACTIONS(21),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(21),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(21),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(21),
    [anon_sym_superset] = ACTIONS(21),
    [anon_sym_union] = ACTIONS(23),
    [anon_sym_diff] = ACTIONS(25),
    [anon_sym_symdiff] = ACTIONS(27),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(31),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [9] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(23),
    [anon_sym_diff] = ACTIONS(25),
    [anon_sym_symdiff] = ACTIONS(27),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(31),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [10] = {
    [ts_builtin_sym_end] = ACTIONS(47),
    [anon_sym_SEMI] = ACTIONS(47),
    [anon_sym_EQ] = ACTIONS(49),
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
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [11] = {
    [ts_builtin_sym_end] = ACTIONS(51),
    [anon_sym_SEMI] = ACTIONS(51),
    [anon_sym_EQ] = ACTIONS(53),
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
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [12] = {
    [ts_builtin_sym_end] = ACTIONS(55),
    [anon_sym_SEMI] = ACTIONS(55),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LT_DASH_GT] = ACTIONS(57),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(57),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(19),
    [anon_sym_EQ_EQ] = ACTIONS(21),
    [anon_sym_BANG_EQ] = ACTIONS(21),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(21),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(21),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(21),
    [anon_sym_superset] = ACTIONS(21),
    [anon_sym_union] = ACTIONS(23),
    [anon_sym_diff] = ACTIONS(25),
    [anon_sym_symdiff] = ACTIONS(27),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(31),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [13] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(45),
    [anon_sym_DASH] = ACTIONS(45),
    [anon_sym_PLUS_PLUS] = ACTIONS(11),
    [anon_sym_STAR] = ACTIONS(11),
    [anon_sym_SLASH] = ACTIONS(45),
    [anon_sym_div] = ACTIONS(11),
    [anon_sym_mod] = ACTIONS(11),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [14] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(19),
    [anon_sym_EQ_EQ] = ACTIONS(21),
    [anon_sym_BANG_EQ] = ACTIONS(21),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(21),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(21),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(21),
    [anon_sym_superset] = ACTIONS(21),
    [anon_sym_union] = ACTIONS(23),
    [anon_sym_diff] = ACTIONS(25),
    [anon_sym_symdiff] = ACTIONS(27),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(31),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [15] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(45),
    [anon_sym_DASH] = ACTIONS(45),
    [anon_sym_PLUS_PLUS] = ACTIONS(11),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [16] = {
    [ts_builtin_sym_end] = ACTIONS(59),
    [anon_sym_SEMI] = ACTIONS(59),
    [anon_sym_EQ] = ACTIONS(13),
    [anon_sym_LT_DASH_GT] = ACTIONS(57),
    [anon_sym_DASH_GT] = ACTIONS(15),
    [anon_sym_LT_DASH] = ACTIONS(17),
    [anon_sym_BSLASH_SLASH] = ACTIONS(57),
    [anon_sym_xor] = ACTIONS(15),
    [anon_sym_SLASH_BSLASH] = ACTIONS(19),
    [anon_sym_EQ_EQ] = ACTIONS(21),
    [anon_sym_BANG_EQ] = ACTIONS(21),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_EQ] = ACTIONS(21),
    [anon_sym_GT] = ACTIONS(13),
    [anon_sym_GT_EQ] = ACTIONS(21),
    [anon_sym_in] = ACTIONS(13),
    [anon_sym_subset] = ACTIONS(21),
    [anon_sym_superset] = ACTIONS(21),
    [anon_sym_union] = ACTIONS(23),
    [anon_sym_diff] = ACTIONS(25),
    [anon_sym_symdiff] = ACTIONS(27),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(31),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [17] = {
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
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [18] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(29),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(33),
    [anon_sym_DASH] = ACTIONS(33),
    [anon_sym_PLUS_PLUS] = ACTIONS(35),
    [anon_sym_STAR] = ACTIONS(37),
    [anon_sym_SLASH] = ACTIONS(39),
    [anon_sym_div] = ACTIONS(37),
    [anon_sym_mod] = ACTIONS(37),
    [anon_sym_CARET] = ACTIONS(41),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [19] = {
    [ts_builtin_sym_end] = ACTIONS(11),
    [anon_sym_SEMI] = ACTIONS(11),
    [anon_sym_EQ] = ACTIONS(45),
    [anon_sym_LT_DASH_GT] = ACTIONS(11),
    [anon_sym_DASH_GT] = ACTIONS(11),
    [anon_sym_LT_DASH] = ACTIONS(45),
    [anon_sym_BSLASH_SLASH] = ACTIONS(11),
    [anon_sym_xor] = ACTIONS(11),
    [anon_sym_SLASH_BSLASH] = ACTIONS(11),
    [anon_sym_EQ_EQ] = ACTIONS(11),
    [anon_sym_BANG_EQ] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(45),
    [anon_sym_LT_EQ] = ACTIONS(11),
    [anon_sym_GT] = ACTIONS(45),
    [anon_sym_GT_EQ] = ACTIONS(11),
    [anon_sym_in] = ACTIONS(45),
    [anon_sym_subset] = ACTIONS(11),
    [anon_sym_superset] = ACTIONS(11),
    [anon_sym_union] = ACTIONS(11),
    [anon_sym_diff] = ACTIONS(11),
    [anon_sym_symdiff] = ACTIONS(11),
    [anon_sym_intersect] = ACTIONS(11),
    [anon_sym_DOT_DOT] = ACTIONS(11),
    [anon_sym_PLUS] = ACTIONS(45),
    [anon_sym_DASH] = ACTIONS(45),
    [anon_sym_PLUS_PLUS] = ACTIONS(11),
    [anon_sym_STAR] = ACTIONS(11),
    [anon_sym_SLASH] = ACTIONS(45),
    [anon_sym_div] = ACTIONS(11),
    [anon_sym_mod] = ACTIONS(11),
    [anon_sym_CARET] = ACTIONS(11),
    [anon_sym_COLON_COLON] = ACTIONS(43),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
};

static uint16_t ts_small_parse_table[] = {
  [0] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(65), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(71), 2,
      sym_absent,
      sym_float_literal,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(14), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [35] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(77), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(79), 2,
      sym_absent,
      sym_float_literal,
    STATE(2), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [70] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(81), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(83), 2,
      sym_absent,
      sym_float_literal,
    STATE(8), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [105] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(85), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(87), 2,
      sym_absent,
      sym_float_literal,
    STATE(7), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [140] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(89), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(91), 2,
      sym_absent,
      sym_float_literal,
    STATE(6), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [175] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(93), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(95), 2,
      sym_absent,
      sym_float_literal,
    STATE(9), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [210] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(97), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(99), 2,
      sym_absent,
      sym_float_literal,
    STATE(18), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [245] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(101), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(103), 2,
      sym_absent,
      sym_float_literal,
    STATE(15), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [280] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(105), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(107), 2,
      sym_absent,
      sym_float_literal,
    STATE(13), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [315] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(109), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(111), 2,
      sym_absent,
      sym_float_literal,
    STATE(19), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [350] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(113), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(115), 2,
      sym_absent,
      sym_float_literal,
    STATE(4), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [385] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(117), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(119), 2,
      sym_absent,
      sym_float_literal,
    STATE(3), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [420] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(121), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(123), 2,
      sym_absent,
      sym_float_literal,
    STATE(16), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [455] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(125), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(127), 2,
      sym_absent,
      sym_float_literal,
    STATE(5), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [490] = 8,
    ACTIONS(69), 1,
      anon_sym_not,
    ACTIONS(75), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(67), 2,
      anon_sym_DASH,
      anon_sym_,
    ACTIONS(73), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(129), 2,
      sym_integer_literal,
      sym_identifier,
    ACTIONS(131), 2,
      sym_absent,
      sym_float_literal,
    STATE(12), 6,
      sym__expression,
      sym_binary_operation,
      sym_unary_operation,
      sym__literal,
      sym_boolean_literal,
      sym_string_literal,
  [525] = 4,
    ACTIONS(133), 1,
      anon_sym_DQUOTE,
    STATE(37), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(135), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
    ACTIONS(137), 2,
      sym_line_comment,
      sym_block_comment,
  [540] = 4,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(139), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(44), 2,
      sym__items,
      sym_assignment_item,
  [555] = 4,
    ACTIONS(141), 1,
      anon_sym_DQUOTE,
    STATE(37), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(137), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(143), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
  [570] = 4,
    ACTIONS(146), 1,
      anon_sym_DQUOTE,
    STATE(35), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(137), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(148), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
  [585] = 4,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(150), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(44), 2,
      sym__items,
      sym_assignment_item,
  [600] = 4,
    ACTIONS(150), 1,
      ts_builtin_sym_end,
    ACTIONS(152), 1,
      anon_sym_SEMI,
    STATE(42), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [614] = 3,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(44), 2,
      sym__items,
      sym_assignment_item,
  [626] = 4,
    ACTIONS(154), 1,
      ts_builtin_sym_end,
    ACTIONS(156), 1,
      anon_sym_SEMI,
    STATE(42), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [640] = 4,
    ACTIONS(159), 1,
      ts_builtin_sym_end,
    ACTIONS(161), 1,
      anon_sym_SEMI,
    STATE(40), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [654] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(154), 2,
      ts_builtin_sym_end,
      anon_sym_SEMI,
  [663] = 2,
    ACTIONS(159), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [671] = 2,
    ACTIONS(163), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [679] = 2,
    ACTIONS(165), 1,
      anon_sym_EQ,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
};

static uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(20)] = 0,
  [SMALL_STATE(21)] = 35,
  [SMALL_STATE(22)] = 70,
  [SMALL_STATE(23)] = 105,
  [SMALL_STATE(24)] = 140,
  [SMALL_STATE(25)] = 175,
  [SMALL_STATE(26)] = 210,
  [SMALL_STATE(27)] = 245,
  [SMALL_STATE(28)] = 280,
  [SMALL_STATE(29)] = 315,
  [SMALL_STATE(30)] = 350,
  [SMALL_STATE(31)] = 385,
  [SMALL_STATE(32)] = 420,
  [SMALL_STATE(33)] = 455,
  [SMALL_STATE(34)] = 490,
  [SMALL_STATE(35)] = 525,
  [SMALL_STATE(36)] = 540,
  [SMALL_STATE(37)] = 555,
  [SMALL_STATE(38)] = 570,
  [SMALL_STATE(39)] = 585,
  [SMALL_STATE(40)] = 600,
  [SMALL_STATE(41)] = 614,
  [SMALL_STATE(42)] = 626,
  [SMALL_STATE(43)] = 640,
  [SMALL_STATE(44)] = 654,
  [SMALL_STATE(45)] = 663,
  [SMALL_STATE(46)] = 671,
  [SMALL_STATE(47)] = 679,
};

static TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [11] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_binary_operation, 3, .production_id = 3),
  [13] = {.entry = {.count = 1, .reusable = false}}, SHIFT(25),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [17] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [29] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [33] = {.entry = {.count = 1, .reusable = false}}, SHIFT(27),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(28),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [45] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_binary_operation, 3, .production_id = 3),
  [47] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean_literal, 1),
  [49] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean_literal, 1),
  [51] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 3),
  [53] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 3),
  [55] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment_item, 3, .production_id = 1),
  [57] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [59] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_unary_operation, 2, .production_id = 2),
  [61] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 2),
  [63] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 2),
  [65] = {.entry = {.count = 1, .reusable = false}}, SHIFT(14),
  [67] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [69] = {.entry = {.count = 1, .reusable = false}}, SHIFT(32),
  [71] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [73] = {.entry = {.count = 1, .reusable = false}}, SHIFT(10),
  [75] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [77] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [79] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [81] = {.entry = {.count = 1, .reusable = false}}, SHIFT(8),
  [83] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [85] = {.entry = {.count = 1, .reusable = false}}, SHIFT(7),
  [87] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [89] = {.entry = {.count = 1, .reusable = false}}, SHIFT(6),
  [91] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [93] = {.entry = {.count = 1, .reusable = false}}, SHIFT(9),
  [95] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [97] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [99] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [101] = {.entry = {.count = 1, .reusable = false}}, SHIFT(15),
  [103] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [105] = {.entry = {.count = 1, .reusable = false}}, SHIFT(13),
  [107] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [109] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [111] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [113] = {.entry = {.count = 1, .reusable = false}}, SHIFT(4),
  [115] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [117] = {.entry = {.count = 1, .reusable = false}}, SHIFT(3),
  [119] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [121] = {.entry = {.count = 1, .reusable = false}}, SHIFT(16),
  [123] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [125] = {.entry = {.count = 1, .reusable = false}}, SHIFT(5),
  [127] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [129] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [131] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [133] = {.entry = {.count = 1, .reusable = false}}, SHIFT(11),
  [135] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [137] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [139] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 3),
  [141] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_literal_repeat1, 2),
  [143] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_literal_repeat1, 2), SHIFT_REPEAT(37),
  [146] = {.entry = {.count = 1, .reusable = false}}, SHIFT(17),
  [148] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [150] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 2),
  [152] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [154] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2),
  [156] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(41),
  [159] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1),
  [161] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [163] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [165] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
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
