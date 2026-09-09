# Change Log

All notable changes to the MiniZinc Visual Studio Code Extension will be documented in this file.

## [0.3.2] - 2026-09-10

- Fix errors and warnings not being reported at all for files inside the directory the language
  server was started in.
- Fix go to definition, find references and rename returning document URIs the editor rejects.
- Report positions using the encoding negotiated with the editor rather than byte offsets, which
  misplaced ranges and corrupted renames on lines containing non-ASCII characters.
- Apply a rename as one change per file, so undoing it takes a single step rather than one per
  occurrence.
- Fix includes of neighbouring files failing to resolve.
- Report a missing or incomplete MiniZinc standard library, rather than only the unresolved
  builtins that it causes.
- Distinguish warnings from errors, which were previously all shown as errors.
- Provide the parameters that signature help highlights, and select the overload being called.
- Clear the diagnostics for a file when it is closed.
- Fix hovering past the end of a line reporting an error instead of doing nothing.
- Answer requests the language server does not implement, and stop exiting on malformed request
  parameters or a workspace with no folders.

## [0.3.1] - 2026-09-07

- Bump version to fix marketplace publishing issue.

## [0.3.0] - 2026-09-07

- Fix bug in invalid overloading checks causing false positive duplicate definition errors.
- Enable signature help and hover info to correctrly find doc comments in more cases.

## [0.2.0] - 2026-08-05

- Support named argument calls and default parameter values
- Improve printing of function signatures on hover/function help

## [0.1.2] - 2026-07-14

- Use the newly developed language server to provide richer information, with the caveat that
  diagnostics may not be fully compatible with current MiniZinc as the language server targets
  a newer edition of MiniZinc.

## [0.1.1] – 2023-02-20

- Update the syntax definition to match MiniZinc version 2.7.

## [0.1.0] – 2018-02-15

- Initial release includes syntax highlighting and snippets
