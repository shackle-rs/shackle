#include <tree_sitter/parser.h>

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 63
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 60
#define ALIAS_COUNT 0
#define TOKEN_COUNT 45
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 12
#define MAX_ALIAS_SEQUENCE_LENGTH 6
#define PRODUCTION_ID_COUNT 16

enum {
  sym_identifier = 1,
  anon_sym_SEMI = 2,
  anon_sym_LBRACE = 3,
  anon_sym_RBRACE = 4,
  anon_sym_COLON = 5,
  anon_sym_COMMA = 6,
  anon_sym_LPAREN = 7,
  anon_sym_RPAREN = 8,
  anon_sym_PLUS = 9,
  anon_sym_DASH = 10,
  anon_sym_not = 11,
  anon_sym_BANG = 12,
  anon_sym_less = 13,
  anon_sym_STAR = 14,
  anon_sym_SLASH = 15,
  anon_sym_mod = 16,
  anon_sym_div = 17,
  anon_sym_let = 18,
  anon_sym_COLON_EQ = 19,
  anon_sym_if = 20,
  anon_sym_then = 21,
  anon_sym_else = 22,
  sym_number_literal = 23,
  anon_sym_true = 24,
  anon_sym_false = 25,
  anon_sym_SQUOTE = 26,
  anon_sym_DQUOTE = 27,
  sym_string_characters = 28,
  anon_sym_BSLASH_SQUOTE = 29,
  anon_sym_BSLASH_DQUOTE = 30,
  anon_sym_BSLASH_BSLASH = 31,
  anon_sym_BSLASHr = 32,
  anon_sym_BSLASHn = 33,
  anon_sym_BSLASHt = 34,
  anon_sym_BSLASH = 35,
  aux_sym_escape_sequence_token1 = 36,
  anon_sym_BSLASHx = 37,
  aux_sym_escape_sequence_token2 = 38,
  anon_sym_BSLASHu = 39,
  aux_sym_escape_sequence_token3 = 40,
  anon_sym_BSLASHU = 41,
  aux_sym_escape_sequence_token4 = 42,
  sym_line_comment = 43,
  sym_block_comment = 44,
  sym_source_file = 45,
  sym__item = 46,
  sym_indexing = 47,
  sym__sexpr_list = 48,
  sym__expr = 49,
  sym_unary_operator = 50,
  sym_infix_operator = 51,
  sym_let_decl = 52,
  sym_if_then_else = 53,
  sym_boolean_literal = 54,
  sym_string_literal = 55,
  aux_sym__string_content = 56,
  sym_escape_sequence = 57,
  aux_sym_source_file_repeat1 = 58,
  aux_sym__sexpr_list_repeat1 = 59,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_identifier] = "identifier",
  [anon_sym_SEMI] = ";",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_COLON] = ":",
  [anon_sym_COMMA] = ",",
  [anon_sym_LPAREN] = "(",
  [anon_sym_RPAREN] = ")",
  [anon_sym_PLUS] = "+",
  [anon_sym_DASH] = "-",
  [anon_sym_not] = "not",
  [anon_sym_BANG] = "!",
  [anon_sym_less] = "less",
  [anon_sym_STAR] = "*",
  [anon_sym_SLASH] = "/",
  [anon_sym_mod] = "mod",
  [anon_sym_div] = "div",
  [anon_sym_let] = "let",
  [anon_sym_COLON_EQ] = ":=",
  [anon_sym_if] = "if",
  [anon_sym_then] = "then",
  [anon_sym_else] = "else",
  [sym_number_literal] = "number_literal",
  [anon_sym_true] = "true",
  [anon_sym_false] = "false",
  [anon_sym_SQUOTE] = "'",
  [anon_sym_DQUOTE] = "\"",
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
  [sym_line_comment] = "line_comment",
  [sym_block_comment] = "block_comment",
  [sym_source_file] = "source_file",
  [sym__item] = "_item",
  [sym_indexing] = "indexing",
  [sym__sexpr_list] = "_sexpr_list",
  [sym__expr] = "_expr",
  [sym_unary_operator] = "unary_operator",
  [sym_infix_operator] = "infix_operator",
  [sym_let_decl] = "let_decl",
  [sym_if_then_else] = "if_then_else",
  [sym_boolean_literal] = "boolean_literal",
  [sym_string_literal] = "string_literal",
  [aux_sym__string_content] = "_string_content",
  [sym_escape_sequence] = "escape_sequence",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym__sexpr_list_repeat1] = "_sexpr_list_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_identifier] = sym_identifier,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_COLON] = anon_sym_COLON,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [anon_sym_PLUS] = anon_sym_PLUS,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_not] = anon_sym_not,
  [anon_sym_BANG] = anon_sym_BANG,
  [anon_sym_less] = anon_sym_less,
  [anon_sym_STAR] = anon_sym_STAR,
  [anon_sym_SLASH] = anon_sym_SLASH,
  [anon_sym_mod] = anon_sym_mod,
  [anon_sym_div] = anon_sym_div,
  [anon_sym_let] = anon_sym_let,
  [anon_sym_COLON_EQ] = anon_sym_COLON_EQ,
  [anon_sym_if] = anon_sym_if,
  [anon_sym_then] = anon_sym_then,
  [anon_sym_else] = anon_sym_else,
  [sym_number_literal] = sym_number_literal,
  [anon_sym_true] = anon_sym_true,
  [anon_sym_false] = anon_sym_false,
  [anon_sym_SQUOTE] = anon_sym_SQUOTE,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [sym_string_characters] = sym_string_characters,
  [anon_sym_BSLASH_SQUOTE] = anon_sym_SQUOTE,
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
  [sym_line_comment] = sym_line_comment,
  [sym_block_comment] = sym_block_comment,
  [sym_source_file] = sym_source_file,
  [sym__item] = sym__item,
  [sym_indexing] = sym_indexing,
  [sym__sexpr_list] = sym__sexpr_list,
  [sym__expr] = sym__expr,
  [sym_unary_operator] = sym_unary_operator,
  [sym_infix_operator] = sym_infix_operator,
  [sym_let_decl] = sym_let_decl,
  [sym_if_then_else] = sym_if_then_else,
  [sym_boolean_literal] = sym_boolean_literal,
  [sym_string_literal] = sym_string_literal,
  [aux_sym__string_content] = aux_sym__string_content,
  [sym_escape_sequence] = sym_escape_sequence,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym__sexpr_list_repeat1] = aux_sym__sexpr_list_repeat1,
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
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RPAREN] = {
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
  [anon_sym_not] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BANG] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_less] = {
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
  [anon_sym_mod] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_div] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_let] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON_EQ] = {
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
  [anon_sym_else] = {
    .visible = true,
    .named = false,
  },
  [sym_number_literal] = {
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
  [anon_sym_SQUOTE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DQUOTE] = {
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
  [sym__item] = {
    .visible = false,
    .named = true,
  },
  [sym_indexing] = {
    .visible = true,
    .named = true,
  },
  [sym__sexpr_list] = {
    .visible = false,
    .named = true,
  },
  [sym__expr] = {
    .visible = false,
    .named = true,
  },
  [sym_unary_operator] = {
    .visible = true,
    .named = true,
  },
  [sym_infix_operator] = {
    .visible = true,
    .named = true,
  },
  [sym_let_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_if_then_else] = {
    .visible = true,
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
  [aux_sym__string_content] = {
    .visible = false,
    .named = false,
  },
  [sym_escape_sequence] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__sexpr_list_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum {
  field_condition = 1,
  field_content = 2,
  field_else = 3,
  field_escape = 4,
  field_indexing = 5,
  field_item = 6,
  field_left = 7,
  field_name = 8,
  field_operand = 9,
  field_operator = 10,
  field_result = 11,
  field_right = 12,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_condition] = "condition",
  [field_content] = "content",
  [field_else] = "else",
  [field_escape] = "escape",
  [field_indexing] = "indexing",
  [field_item] = "item",
  [field_left] = "left",
  [field_name] = "name",
  [field_operand] = "operand",
  [field_operator] = "operator",
  [field_result] = "result",
  [field_right] = "right",
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
  [10] = {.index = 12, .length = 2},
  [11] = {.index = 14, .length = 3},
  [12] = {.index = 17, .length = 1},
  [13] = {.index = 18, .length = 2},
  [14] = {.index = 20, .length = 2},
  [15] = {.index = 22, .length = 3},
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
    {field_operand, 1},
    {field_operator, 0},
  [8] =
    {field_content, 0},
  [9] =
    {field_escape, 0},
  [10] =
    {field_escape, 1},
  [11] =
    {field_content, 1, .inherited = true},
  [12] =
    {field_content, 0, .inherited = true},
    {field_content, 1, .inherited = true},
  [14] =
    {field_left, 0},
    {field_operator, 1},
    {field_right, 2},
  [17] =
    {field_name, 1},
  [18] =
    {field_condition, 1},
    {field_result, 3},
  [20] =
    {field_indexing, 1},
    {field_name, 2},
  [22] =
    {field_condition, 1},
    {field_else, 5},
    {field_result, 3},
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
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(26);
      if (lookahead == '!') ADVANCE(37);
      if (lookahead == '"') ADVANCE(45);
      if (lookahead == '#') ADVANCE(68);
      if (lookahead == '\'') ADVANCE(44);
      if (lookahead == '(') ADVANCE(33);
      if (lookahead == ')') ADVANCE(34);
      if (lookahead == '*') ADVANCE(38);
      if (lookahead == '+') ADVANCE(35);
      if (lookahead == ',') ADVANCE(32);
      if (lookahead == '-') ADVANCE(36);
      if (lookahead == '/') ADVANCE(39);
      if (lookahead == ':') ADVANCE(31);
      if (lookahead == ';') ADVANCE(27);
      if (lookahead == '\\') ADVANCE(58);
      if (lookahead == '{') ADVANCE(28);
      if (lookahead == '}') ADVANCE(29);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(61);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(71);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(2)
      if (lookahead == '"') ADVANCE(45);
      if (lookahead == '#') ADVANCE(51);
      if (lookahead == '\'') ADVANCE(51);
      if (lookahead == '/') ADVANCE(49);
      if (lookahead == '\\') ADVANCE(58);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(46);
      if (lookahead != 0) ADVANCE(51);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(45);
      if (lookahead == '#') ADVANCE(68);
      if (lookahead == '\'') ADVANCE(44);
      if (lookahead == '/') ADVANCE(6);
      if (lookahead == '\\') ADVANCE(58);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(2)
      END_STATE();
    case 3:
      if (lookahead == '#') ADVANCE(68);
      if (lookahead == '/') ADVANCE(6);
      if (lookahead == ':') ADVANCE(9);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(3)
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(22);
      END_STATE();
    case 4:
      if (lookahead == '#') ADVANCE(68);
      if (lookahead == '/') ADVANCE(6);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(4)
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(18);
      END_STATE();
    case 5:
      if (lookahead == '#') ADVANCE(68);
      if (lookahead == '/') ADVANCE(6);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(5)
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(14);
      END_STATE();
    case 6:
      if (lookahead == '*') ADVANCE(24);
      END_STATE();
    case 7:
      if (lookahead == '*') ADVANCE(23);
      if (lookahead == '/') ADVANCE(69);
      if (lookahead != 0) ADVANCE(24);
      END_STATE();
    case 8:
      if (lookahead == '-') ADVANCE(11);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(43);
      END_STATE();
    case 9:
      if (lookahead == '=') ADVANCE(40);
      END_STATE();
    case 10:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(42);
      END_STATE();
    case 11:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(43);
      END_STATE();
    case 12:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(67);
      END_STATE();
    case 13:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(65);
      END_STATE();
    case 14:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(63);
      END_STATE();
    case 15:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(12);
      END_STATE();
    case 16:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(13);
      END_STATE();
    case 17:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(15);
      END_STATE();
    case 18:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(16);
      END_STATE();
    case 19:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(17);
      END_STATE();
    case 20:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(19);
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
      if (lookahead != 0 &&
          lookahead != '*' &&
          lookahead != '/') ADVANCE(24);
      if (lookahead == '*') ADVANCE(7);
      if (lookahead == '/') ADVANCE(70);
      END_STATE();
    case 24:
      if (lookahead != 0 &&
          lookahead != '*') ADVANCE(24);
      if (lookahead == '*') ADVANCE(7);
      END_STATE();
    case 25:
      if (eof) ADVANCE(26);
      if (lookahead == '!') ADVANCE(37);
      if (lookahead == '"') ADVANCE(45);
      if (lookahead == '#') ADVANCE(68);
      if (lookahead == '\'') ADVANCE(44);
      if (lookahead == '(') ADVANCE(33);
      if (lookahead == ')') ADVANCE(34);
      if (lookahead == '*') ADVANCE(38);
      if (lookahead == '+') ADVANCE(35);
      if (lookahead == ',') ADVANCE(32);
      if (lookahead == '-') ADVANCE(36);
      if (lookahead == '/') ADVANCE(39);
      if (lookahead == ':') ADVANCE(30);
      if (lookahead == ';') ADVANCE(27);
      if (lookahead == '{') ADVANCE(28);
      if (lookahead == '}') ADVANCE(29);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(25)
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(41);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(71);
      END_STATE();
    case 26:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 27:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 28:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(anon_sym_COLON);
      if (lookahead == '=') ADVANCE(40);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 35:
      ACCEPT_TOKEN(anon_sym_PLUS);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_BANG);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_STAR);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(anon_sym_SLASH);
      if (lookahead == '*') ADVANCE(24);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(anon_sym_COLON_EQ);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(sym_number_literal);
      if (lookahead == '.') ADVANCE(10);
      if (lookahead == 'D' ||
          lookahead == 'E' ||
          lookahead == 'd' ||
          lookahead == 'e') ADVANCE(8);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(41);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(sym_number_literal);
      if (lookahead == 'D' ||
          lookahead == 'E' ||
          lookahead == 'd' ||
          lookahead == 'e') ADVANCE(8);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(42);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(sym_number_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(43);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(anon_sym_SQUOTE);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '#') ADVANCE(51);
      if (lookahead == '\'') ADVANCE(51);
      if (lookahead == '/') ADVANCE(49);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(46);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(51);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(50);
      if (lookahead == '/') ADVANCE(48);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(48);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(50);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(48);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(48);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(51);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead == '*') ADVANCE(47);
      if (lookahead == '/') ADVANCE(51);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(48);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(sym_string_characters);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(51);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(anon_sym_BSLASH_SQUOTE);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(anon_sym_BSLASH_DQUOTE);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_BSLASH_BSLASH);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym_BSLASHr);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(anon_sym_BSLASHn);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(anon_sym_BSLASHt);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(anon_sym_BSLASH);
      if (lookahead == '"') ADVANCE(53);
      if (lookahead == '\'') ADVANCE(52);
      if (lookahead == 'U') ADVANCE(66);
      if (lookahead == '\\') ADVANCE(54);
      if (lookahead == 'n') ADVANCE(56);
      if (lookahead == 'r') ADVANCE(55);
      if (lookahead == 't') ADVANCE(57);
      if (lookahead == 'u') ADVANCE(64);
      if (lookahead == 'x') ADVANCE(62);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(59);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token1);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(60);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(anon_sym_BSLASHx);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token2);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(anon_sym_BSLASHu);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token3);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(anon_sym_BSLASHU);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(aux_sym_escape_sequence_token4);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(sym_line_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(68);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(sym_block_comment);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(sym_block_comment);
      if (lookahead != 0 &&
          lookahead != '*') ADVANCE(24);
      if (lookahead == '*') ADVANCE(7);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(71);
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
      if (lookahead == 'e') ADVANCE(2);
      if (lookahead == 'f') ADVANCE(3);
      if (lookahead == 'i') ADVANCE(4);
      if (lookahead == 'l') ADVANCE(5);
      if (lookahead == 'm') ADVANCE(6);
      if (lookahead == 'n') ADVANCE(7);
      if (lookahead == 't') ADVANCE(8);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
      END_STATE();
    case 1:
      if (lookahead == 'i') ADVANCE(9);
      END_STATE();
    case 2:
      if (lookahead == 'l') ADVANCE(10);
      END_STATE();
    case 3:
      if (lookahead == 'a') ADVANCE(11);
      END_STATE();
    case 4:
      if (lookahead == 'f') ADVANCE(12);
      END_STATE();
    case 5:
      if (lookahead == 'e') ADVANCE(13);
      END_STATE();
    case 6:
      if (lookahead == 'o') ADVANCE(14);
      END_STATE();
    case 7:
      if (lookahead == 'o') ADVANCE(15);
      END_STATE();
    case 8:
      if (lookahead == 'h') ADVANCE(16);
      if (lookahead == 'r') ADVANCE(17);
      END_STATE();
    case 9:
      if (lookahead == 'v') ADVANCE(18);
      END_STATE();
    case 10:
      if (lookahead == 's') ADVANCE(19);
      END_STATE();
    case 11:
      if (lookahead == 'l') ADVANCE(20);
      END_STATE();
    case 12:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 13:
      if (lookahead == 's') ADVANCE(21);
      if (lookahead == 't') ADVANCE(22);
      END_STATE();
    case 14:
      if (lookahead == 'd') ADVANCE(23);
      END_STATE();
    case 15:
      if (lookahead == 't') ADVANCE(24);
      END_STATE();
    case 16:
      if (lookahead == 'e') ADVANCE(25);
      END_STATE();
    case 17:
      if (lookahead == 'u') ADVANCE(26);
      END_STATE();
    case 18:
      ACCEPT_TOKEN(anon_sym_div);
      END_STATE();
    case 19:
      if (lookahead == 'e') ADVANCE(27);
      END_STATE();
    case 20:
      if (lookahead == 's') ADVANCE(28);
      END_STATE();
    case 21:
      if (lookahead == 's') ADVANCE(29);
      END_STATE();
    case 22:
      ACCEPT_TOKEN(anon_sym_let);
      END_STATE();
    case 23:
      ACCEPT_TOKEN(anon_sym_mod);
      END_STATE();
    case 24:
      ACCEPT_TOKEN(anon_sym_not);
      END_STATE();
    case 25:
      if (lookahead == 'n') ADVANCE(30);
      END_STATE();
    case 26:
      if (lookahead == 'e') ADVANCE(31);
      END_STATE();
    case 27:
      ACCEPT_TOKEN(anon_sym_else);
      END_STATE();
    case 28:
      if (lookahead == 'e') ADVANCE(32);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(anon_sym_less);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(anon_sym_then);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(anon_sym_true);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_false);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 0},
  [2] = {.lex_state = 25},
  [3] = {.lex_state = 25},
  [4] = {.lex_state = 25},
  [5] = {.lex_state = 25},
  [6] = {.lex_state = 25},
  [7] = {.lex_state = 25},
  [8] = {.lex_state = 25},
  [9] = {.lex_state = 25},
  [10] = {.lex_state = 25},
  [11] = {.lex_state = 25},
  [12] = {.lex_state = 25},
  [13] = {.lex_state = 25},
  [14] = {.lex_state = 25},
  [15] = {.lex_state = 25},
  [16] = {.lex_state = 25},
  [17] = {.lex_state = 25},
  [18] = {.lex_state = 25},
  [19] = {.lex_state = 25},
  [20] = {.lex_state = 25},
  [21] = {.lex_state = 25},
  [22] = {.lex_state = 25},
  [23] = {.lex_state = 25},
  [24] = {.lex_state = 25},
  [25] = {.lex_state = 1},
  [26] = {.lex_state = 25},
  [27] = {.lex_state = 25},
  [28] = {.lex_state = 25},
  [29] = {.lex_state = 25},
  [30] = {.lex_state = 25},
  [31] = {.lex_state = 1},
  [32] = {.lex_state = 1},
  [33] = {.lex_state = 1},
  [34] = {.lex_state = 1},
  [35] = {.lex_state = 1},
  [36] = {.lex_state = 1},
  [37] = {.lex_state = 1},
  [38] = {.lex_state = 25},
  [39] = {.lex_state = 25},
  [40] = {.lex_state = 0},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 0},
  [43] = {.lex_state = 0},
  [44] = {.lex_state = 0},
  [45] = {.lex_state = 0},
  [46] = {.lex_state = 0},
  [47] = {.lex_state = 25},
  [48] = {.lex_state = 25},
  [49] = {.lex_state = 0},
  [50] = {.lex_state = 0},
  [51] = {.lex_state = 25},
  [52] = {.lex_state = 0},
  [53] = {.lex_state = 0},
  [54] = {.lex_state = 0},
  [55] = {.lex_state = 3},
  [56] = {.lex_state = 0},
  [57] = {.lex_state = 3},
  [58] = {.lex_state = 4},
  [59] = {.lex_state = 5},
  [60] = {.lex_state = 0},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 3},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_identifier] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_COLON] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_PLUS] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_not] = ACTIONS(1),
    [anon_sym_BANG] = ACTIONS(1),
    [anon_sym_less] = ACTIONS(1),
    [anon_sym_STAR] = ACTIONS(1),
    [anon_sym_SLASH] = ACTIONS(1),
    [anon_sym_mod] = ACTIONS(1),
    [anon_sym_div] = ACTIONS(1),
    [anon_sym_let] = ACTIONS(1),
    [anon_sym_COLON_EQ] = ACTIONS(1),
    [anon_sym_if] = ACTIONS(1),
    [anon_sym_then] = ACTIONS(1),
    [anon_sym_else] = ACTIONS(1),
    [anon_sym_true] = ACTIONS(1),
    [anon_sym_false] = ACTIONS(1),
    [anon_sym_SQUOTE] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
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
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
  [1] = {
    [sym_source_file] = STATE(56),
    [sym__item] = STATE(53),
    [sym_indexing] = STATE(53),
    [sym_let_decl] = STATE(53),
    [aux_sym_source_file_repeat1] = STATE(45),
    [ts_builtin_sym_end] = ACTIONS(5),
    [anon_sym_LBRACE] = ACTIONS(7),
    [anon_sym_let] = ACTIONS(9),
    [sym_line_comment] = ACTIONS(3),
    [sym_block_comment] = ACTIONS(3),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 14,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(25), 1,
      sym_number_literal,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(13), 2,
      anon_sym_RBRACE,
      anon_sym_COLON,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(39), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [53] = 14,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(25), 1,
      sym_number_literal,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(33), 2,
      anon_sym_RBRACE,
      anon_sym_COLON,
    STATE(39), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [106] = 14,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(35), 1,
      sym_identifier,
    ACTIONS(37), 1,
      sym_number_literal,
    STATE(51), 1,
      sym__sexpr_list,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(38), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [158] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(39), 1,
      sym_identifier,
    ACTIONS(41), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(27), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [207] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(43), 1,
      sym_identifier,
    ACTIONS(45), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(41), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [256] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(47), 1,
      sym_identifier,
    ACTIONS(49), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(28), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [305] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(51), 1,
      sym_identifier,
    ACTIONS(53), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [354] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(55), 1,
      sym_identifier,
    ACTIONS(57), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(42), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [403] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(59), 1,
      sym_identifier,
    ACTIONS(61), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(21), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [452] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(63), 1,
      sym_identifier,
    ACTIONS(65), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(22), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [501] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(67), 1,
      sym_identifier,
    ACTIONS(69), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(44), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [550] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(71), 1,
      sym_identifier,
    ACTIONS(73), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(23), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [599] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(75), 1,
      sym_identifier,
    ACTIONS(77), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(43), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [648] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(79), 1,
      sym_identifier,
    ACTIONS(81), 1,
      sym_number_literal,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(40), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [697] = 13,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(11), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_not,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(25), 1,
      sym_number_literal,
    ACTIONS(29), 1,
      anon_sym_SQUOTE,
    ACTIONS(31), 1,
      anon_sym_DQUOTE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(17), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(27), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(39), 7,
      sym_indexing,
      sym__expr,
      sym_unary_operator,
      sym_infix_operator,
      sym_if_then_else,
      sym_boolean_literal,
      sym_string_literal,
  [746] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(85), 7,
      anon_sym_less,
      anon_sym_SLASH,
      anon_sym_mod,
      anon_sym_div,
      anon_sym_then,
      anon_sym_else,
      sym_identifier,
    ACTIONS(83), 9,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_STAR,
  [771] = 3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(89), 7,
      anon_sym_less,
      anon_sym_SLASH,
      anon_sym_mod,
      anon_sym_div,
      anon_sym_then,
      anon_sym_else,
      sym_identifier,
    ACTIONS(87), 9,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_STAR,
  [796] = 3,
    ACTIONS(93), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(91), 14,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
      anon_sym_then,
      anon_sym_else,
  [820] = 3,
    ACTIONS(97), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(95), 14,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
      anon_sym_then,
      anon_sym_else,
  [844] = 3,
    ACTIONS(101), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(99), 14,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
      anon_sym_then,
      anon_sym_else,
  [868] = 5,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
    ACTIONS(99), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_then,
      anon_sym_else,
  [896] = 4,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
    ACTIONS(95), 11,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
      anon_sym_then,
      anon_sym_else,
  [922] = 3,
    ACTIONS(111), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(109), 14,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
      anon_sym_then,
      anon_sym_else,
  [946] = 10,
    ACTIONS(115), 1,
      sym_string_characters,
    ACTIONS(121), 1,
      anon_sym_BSLASH,
    ACTIONS(124), 1,
      anon_sym_BSLASHx,
    ACTIONS(127), 1,
      anon_sym_BSLASHu,
    ACTIONS(130), 1,
      anon_sym_BSLASHU,
    STATE(25), 1,
      aux_sym__string_content,
    STATE(37), 1,
      sym_escape_sequence,
    ACTIONS(113), 2,
      anon_sym_SQUOTE,
      anon_sym_DQUOTE,
    ACTIONS(133), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(118), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [984] = 3,
    ACTIONS(111), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(109), 14,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
      anon_sym_then,
      anon_sym_else,
  [1008] = 6,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(137), 1,
      anon_sym_else,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
    ACTIONS(135), 7,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_then,
  [1038] = 5,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
    ACTIONS(139), 8,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_then,
      anon_sym_else,
  [1066] = 3,
    ACTIONS(143), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(141), 14,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
      anon_sym_then,
      anon_sym_else,
  [1090] = 3,
    ACTIONS(147), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(145), 14,
      ts_builtin_sym_end,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
      anon_sym_then,
      anon_sym_else,
  [1114] = 10,
    ACTIONS(149), 1,
      anon_sym_DQUOTE,
    ACTIONS(151), 1,
      sym_string_characters,
    ACTIONS(155), 1,
      anon_sym_BSLASH,
    ACTIONS(157), 1,
      anon_sym_BSLASHx,
    ACTIONS(159), 1,
      anon_sym_BSLASHu,
    ACTIONS(161), 1,
      anon_sym_BSLASHU,
    STATE(33), 1,
      aux_sym__string_content,
    STATE(37), 1,
      sym_escape_sequence,
    ACTIONS(133), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(153), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [1151] = 10,
    ACTIONS(151), 1,
      sym_string_characters,
    ACTIONS(155), 1,
      anon_sym_BSLASH,
    ACTIONS(157), 1,
      anon_sym_BSLASHx,
    ACTIONS(159), 1,
      anon_sym_BSLASHu,
    ACTIONS(161), 1,
      anon_sym_BSLASHU,
    ACTIONS(163), 1,
      anon_sym_SQUOTE,
    STATE(25), 1,
      aux_sym__string_content,
    STATE(37), 1,
      sym_escape_sequence,
    ACTIONS(133), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(153), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [1188] = 10,
    ACTIONS(151), 1,
      sym_string_characters,
    ACTIONS(155), 1,
      anon_sym_BSLASH,
    ACTIONS(157), 1,
      anon_sym_BSLASHx,
    ACTIONS(159), 1,
      anon_sym_BSLASHu,
    ACTIONS(161), 1,
      anon_sym_BSLASHU,
    ACTIONS(165), 1,
      anon_sym_DQUOTE,
    STATE(25), 1,
      aux_sym__string_content,
    STATE(37), 1,
      sym_escape_sequence,
    ACTIONS(133), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(153), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [1225] = 10,
    ACTIONS(149), 1,
      anon_sym_SQUOTE,
    ACTIONS(151), 1,
      sym_string_characters,
    ACTIONS(155), 1,
      anon_sym_BSLASH,
    ACTIONS(157), 1,
      anon_sym_BSLASHx,
    ACTIONS(159), 1,
      anon_sym_BSLASHu,
    ACTIONS(161), 1,
      anon_sym_BSLASHU,
    STATE(32), 1,
      aux_sym__string_content,
    STATE(37), 1,
      sym_escape_sequence,
    ACTIONS(133), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(153), 6,
      anon_sym_BSLASH_SQUOTE,
      anon_sym_BSLASH_DQUOTE,
      anon_sym_BSLASH_BSLASH,
      anon_sym_BSLASHr,
      anon_sym_BSLASHn,
      anon_sym_BSLASHt,
  [1262] = 3,
    ACTIONS(169), 1,
      sym_string_characters,
    ACTIONS(133), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(167), 12,
      anon_sym_SQUOTE,
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
  [1284] = 3,
    ACTIONS(173), 1,
      sym_string_characters,
    ACTIONS(133), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(171), 12,
      anon_sym_SQUOTE,
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
  [1306] = 3,
    ACTIONS(177), 1,
      sym_string_characters,
    ACTIONS(133), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(175), 12,
      anon_sym_SQUOTE,
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
  [1328] = 7,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(181), 1,
      anon_sym_COMMA,
    STATE(47), 1,
      aux_sym__sexpr_list_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(179), 2,
      anon_sym_RBRACE,
      anon_sym_COLON,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
  [1356] = 5,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
    ACTIONS(183), 3,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
  [1379] = 5,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(185), 2,
      ts_builtin_sym_end,
      anon_sym_SEMI,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
  [1401] = 5,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(187), 2,
      ts_builtin_sym_end,
      anon_sym_SEMI,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
  [1423] = 5,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(189), 1,
      anon_sym_RPAREN,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
  [1444] = 5,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(191), 1,
      anon_sym_RBRACE,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
  [1465] = 5,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(193), 1,
      anon_sym_then,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(103), 3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_less,
    ACTIONS(105), 3,
      anon_sym_STAR,
      anon_sym_mod,
      anon_sym_div,
  [1486] = 6,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(9), 1,
      anon_sym_let,
    ACTIONS(195), 1,
      ts_builtin_sym_end,
    STATE(46), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(52), 3,
      sym__item,
      sym_indexing,
      sym_let_decl,
  [1508] = 6,
    ACTIONS(197), 1,
      ts_builtin_sym_end,
    ACTIONS(199), 1,
      anon_sym_LBRACE,
    ACTIONS(202), 1,
      anon_sym_let,
    STATE(46), 1,
      aux_sym_source_file_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    STATE(54), 3,
      sym__item,
      sym_indexing,
      sym_let_decl,
  [1530] = 4,
    ACTIONS(205), 1,
      anon_sym_COMMA,
    STATE(48), 1,
      aux_sym__sexpr_list_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(13), 2,
      anon_sym_RBRACE,
      anon_sym_COLON,
  [1545] = 4,
    ACTIONS(207), 1,
      anon_sym_COMMA,
    STATE(48), 1,
      aux_sym__sexpr_list_repeat1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(183), 2,
      anon_sym_RBRACE,
      anon_sym_COLON,
  [1560] = 2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
    ACTIONS(210), 3,
      ts_builtin_sym_end,
      anon_sym_LBRACE,
      anon_sym_let,
  [1570] = 4,
    ACTIONS(7), 1,
      anon_sym_LBRACE,
    ACTIONS(212), 1,
      sym_identifier,
    STATE(61), 1,
      sym_indexing,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1584] = 3,
    ACTIONS(214), 1,
      anon_sym_RBRACE,
    ACTIONS(216), 1,
      anon_sym_COLON,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1595] = 3,
    ACTIONS(218), 1,
      ts_builtin_sym_end,
    ACTIONS(220), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1606] = 3,
    ACTIONS(220), 1,
      anon_sym_SEMI,
    ACTIONS(222), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1617] = 2,
    ACTIONS(220), 1,
      anon_sym_SEMI,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1625] = 2,
    ACTIONS(224), 1,
      anon_sym_COLON_EQ,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1633] = 2,
    ACTIONS(226), 1,
      ts_builtin_sym_end,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1641] = 2,
    ACTIONS(228), 1,
      aux_sym_escape_sequence_token4,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1649] = 2,
    ACTIONS(228), 1,
      aux_sym_escape_sequence_token3,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1657] = 2,
    ACTIONS(228), 1,
      aux_sym_escape_sequence_token2,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1665] = 2,
    ACTIONS(228), 1,
      aux_sym_escape_sequence_token1,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1673] = 2,
    ACTIONS(230), 1,
      sym_identifier,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
  [1681] = 2,
    ACTIONS(232), 1,
      anon_sym_COLON_EQ,
    ACTIONS(3), 2,
      sym_line_comment,
      sym_block_comment,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 53,
  [SMALL_STATE(4)] = 106,
  [SMALL_STATE(5)] = 158,
  [SMALL_STATE(6)] = 207,
  [SMALL_STATE(7)] = 256,
  [SMALL_STATE(8)] = 305,
  [SMALL_STATE(9)] = 354,
  [SMALL_STATE(10)] = 403,
  [SMALL_STATE(11)] = 452,
  [SMALL_STATE(12)] = 501,
  [SMALL_STATE(13)] = 550,
  [SMALL_STATE(14)] = 599,
  [SMALL_STATE(15)] = 648,
  [SMALL_STATE(16)] = 697,
  [SMALL_STATE(17)] = 746,
  [SMALL_STATE(18)] = 771,
  [SMALL_STATE(19)] = 796,
  [SMALL_STATE(20)] = 820,
  [SMALL_STATE(21)] = 844,
  [SMALL_STATE(22)] = 868,
  [SMALL_STATE(23)] = 896,
  [SMALL_STATE(24)] = 922,
  [SMALL_STATE(25)] = 946,
  [SMALL_STATE(26)] = 984,
  [SMALL_STATE(27)] = 1008,
  [SMALL_STATE(28)] = 1038,
  [SMALL_STATE(29)] = 1066,
  [SMALL_STATE(30)] = 1090,
  [SMALL_STATE(31)] = 1114,
  [SMALL_STATE(32)] = 1151,
  [SMALL_STATE(33)] = 1188,
  [SMALL_STATE(34)] = 1225,
  [SMALL_STATE(35)] = 1262,
  [SMALL_STATE(36)] = 1284,
  [SMALL_STATE(37)] = 1306,
  [SMALL_STATE(38)] = 1328,
  [SMALL_STATE(39)] = 1356,
  [SMALL_STATE(40)] = 1379,
  [SMALL_STATE(41)] = 1401,
  [SMALL_STATE(42)] = 1423,
  [SMALL_STATE(43)] = 1444,
  [SMALL_STATE(44)] = 1465,
  [SMALL_STATE(45)] = 1486,
  [SMALL_STATE(46)] = 1508,
  [SMALL_STATE(47)] = 1530,
  [SMALL_STATE(48)] = 1545,
  [SMALL_STATE(49)] = 1560,
  [SMALL_STATE(50)] = 1570,
  [SMALL_STATE(51)] = 1584,
  [SMALL_STATE(52)] = 1595,
  [SMALL_STATE(53)] = 1606,
  [SMALL_STATE(54)] = 1617,
  [SMALL_STATE(55)] = 1625,
  [SMALL_STATE(56)] = 1633,
  [SMALL_STATE(57)] = 1641,
  [SMALL_STATE(58)] = 1649,
  [SMALL_STATE(59)] = 1657,
  [SMALL_STATE(60)] = 1665,
  [SMALL_STATE(61)] = 1673,
  [SMALL_STATE(62)] = 1681,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(39),
  [13] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__sexpr_list, 2),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [19] = {.entry = {.count = 1, .reusable = false}}, SHIFT(11),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [23] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [27] = {.entry = {.count = 1, .reusable = false}}, SHIFT(30),
  [29] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [33] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__sexpr_list, 3),
  [35] = {.entry = {.count = 1, .reusable = false}}, SHIFT(38),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(27),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [43] = {.entry = {.count = 1, .reusable = false}}, SHIFT(41),
  [45] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(28),
  [49] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [51] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [53] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [55] = {.entry = {.count = 1, .reusable = false}}, SHIFT(42),
  [57] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [59] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [61] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [63] = {.entry = {.count = 1, .reusable = false}}, SHIFT(22),
  [65] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [67] = {.entry = {.count = 1, .reusable = false}}, SHIFT(44),
  [69] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [71] = {.entry = {.count = 1, .reusable = false}}, SHIFT(23),
  [73] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [75] = {.entry = {.count = 1, .reusable = false}}, SHIFT(43),
  [77] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [79] = {.entry = {.count = 1, .reusable = false}}, SHIFT(40),
  [81] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [83] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_indexing, 5),
  [85] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_indexing, 5),
  [87] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_indexing, 3),
  [89] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_indexing, 3),
  [91] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 2),
  [93] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 2),
  [95] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_infix_operator, 3, .production_id = 11),
  [97] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_infix_operator, 3, .production_id = 11),
  [99] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_unary_operator, 2, .production_id = 5),
  [101] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_unary_operator, 2, .production_id = 5),
  [103] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [105] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [107] = {.entry = {.count = 1, .reusable = false}}, SHIFT(8),
  [109] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 3, .production_id = 9),
  [111] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 3, .production_id = 9),
  [113] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 10),
  [115] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__string_content, 2, .production_id = 10), SHIFT_REPEAT(37),
  [118] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 10), SHIFT_REPEAT(35),
  [121] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 10), SHIFT_REPEAT(60),
  [124] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 10), SHIFT_REPEAT(59),
  [127] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 10), SHIFT_REPEAT(58),
  [130] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__string_content, 2, .production_id = 10), SHIFT_REPEAT(57),
  [133] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [135] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_then_else, 4, .production_id = 13),
  [137] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [139] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_then_else, 6, .production_id = 15),
  [141] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__expr, 3),
  [143] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__expr, 3),
  [145] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean_literal, 1),
  [147] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean_literal, 1),
  [149] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [151] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [153] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [155] = {.entry = {.count = 1, .reusable = false}}, SHIFT(60),
  [157] = {.entry = {.count = 1, .reusable = false}}, SHIFT(59),
  [159] = {.entry = {.count = 1, .reusable = false}}, SHIFT(58),
  [161] = {.entry = {.count = 1, .reusable = false}}, SHIFT(57),
  [163] = {.entry = {.count = 1, .reusable = false}}, SHIFT(26),
  [165] = {.entry = {.count = 1, .reusable = false}}, SHIFT(24),
  [167] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_escape_sequence, 1, .production_id = 7),
  [169] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_escape_sequence, 1, .production_id = 7),
  [171] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_escape_sequence, 2, .production_id = 8),
  [173] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_escape_sequence, 2, .production_id = 8),
  [175] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__string_content, 1, .production_id = 6),
  [177] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__string_content, 1, .production_id = 6),
  [179] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__sexpr_list, 1),
  [181] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [183] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__sexpr_list_repeat1, 2),
  [185] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_let_decl, 5, .production_id = 14),
  [187] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_let_decl, 4, .production_id = 12),
  [189] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [191] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [193] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [195] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, .production_id = 2),
  [197] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 4),
  [199] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 4), SHIFT_REPEAT(4),
  [202] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 4), SHIFT_REPEAT(50),
  [205] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [207] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__sexpr_list_repeat1, 2), SHIFT_REPEAT(16),
  [210] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, .production_id = 1),
  [212] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [214] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [216] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [218] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 2, .production_id = 3),
  [220] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [222] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, .production_id = 1),
  [224] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [226] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [228] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [230] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [232] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef _WIN32
#define extern __declspec(dllexport)
#endif

extern const TSLanguage *tree_sitter_ampl(void) {
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
