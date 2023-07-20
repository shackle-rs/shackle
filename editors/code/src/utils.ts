import { TextEditor } from "vscode"

export function isMiniZinc(editor: TextEditor) {
	return (
		editor &&
		editor.document.languageId === "minizinc" &&
		editor.document.uri.scheme === "file"
	)
}
