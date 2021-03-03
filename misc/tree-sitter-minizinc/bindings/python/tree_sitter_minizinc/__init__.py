#!/usr/bin/env python3

from tree_sitter import Language, Parser
from pathlib import Path

import pkg_resources

LANGUAGE = Language(next(Path(__file__).parent.glob("binding.*.so")), "minizinc")
HIGHLIGHT_QUERY = LANGUAGE.query(
    pkg_resources.resource_string(__name__, "queries/highlights.scm")
)


try:
    from pygments.lexer import Lexer
    from pygments import token

    class TreeSitterLexer(Lexer):
        ts_alias = {
            "comment": token.Comment,
            "type.builtin": token.Name.Builtin,
            "punctuation.delimiter": token.Punctuation,
            "function": token.Name.Function,
            "keyword": token.Keyword,
            "operator": token.Operator,
            "punctuation.bracket": token.Punctuation,
            "number": token.Number,
            "string": token.String,
            "escape": token.String.Escape,
            "constant.builtin": token.Generic,
            "variable": token.Name.Variable,
        }

        def __init__(self, **options):
            self.parser = Parser()
            self.parser.set_language(self.language)
            super().__init__(**options)

        def get_tokens_unprocessed(self, text):
            to_bytes = bytes(text, "utf8")
            tree = self.parser.parse(to_bytes)
            captures = self.highlight_query.captures(tree.root_node)

            last_pos = 0
            for node, annotation in captures:
                if last_pos > node.start_byte:
                    # Double match - only use the first capture
                    continue
                if last_pos != node.start_byte:
                    yield last_pos, token.Generic, to_bytes[
                        last_pos : node.start_byte
                    ].decode()
                yield node.start_byte, self.ts_alias[annotation], to_bytes[
                    node.start_byte : node.end_byte
                ].decode()
                last_pos = node.end_byte

            if last_pos != tree.root_node.end_byte:
                yield last_pos, token.Generic, to_bytes[
                    last_pos : tree.root_node.end_byte
                ].decode()

    class MiniZincLexer(TreeSitterLexer):
        name = "MiniZinc"
        aliases = ["fzn", "dzn", "mzn", "minizinc"]
        filenames = ["*.mzn", "*.fzn", "*.dzn"]

        language = LANGUAGE
        highlight_query = HIGHLIGHT_QUERY


except ImportError:
    pass
