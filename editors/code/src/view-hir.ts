import {
	Disposable,
	Event,
	EventEmitter,
	ExtensionContext,
	ProviderResult,
	TextDocumentContentProvider,
	Uri,
	ViewColumn,
	window,
	workspace,
} from "vscode"
import {
	CancellationToken,
	RequestType,
	TextDocumentPositionParams,
} from "vscode-languageclient"
import { LanguageClient } from "vscode-languageclient/node"
import { isMiniZinc } from "./utils"

const viewHirRequest = new RequestType<
	TextDocumentPositionParams,
	string,
	void
>("shackle-ls/viewHir")

class HirProvider implements TextDocumentContentProvider {
	readonly eventEmitter = new EventEmitter<Uri>()
	readonly uri: Uri = Uri.parse("shackle-ls://viewHir/hir.txt")

	private client: LanguageClient
	private subscriptions: Disposable[] = []

	constructor(client: LanguageClient, onFinish: () => void) {
		this.client = client

		this.subscriptions.push(
			workspace.registerTextDocumentContentProvider("shackle-ls", this)
		)

		workspace.onDidChangeTextDocument(
			(e) => {
				if (e.document === window.activeTextEditor.document) {
					this.invalidate()
				}
			},
			this,
			this.subscriptions
		)

		window.onDidChangeActiveTextEditor(
			() => this.invalidate(),
			this,
			this.subscriptions
		)

		window.onDidChangeTextEditorSelection(
			(e) => {
				if (e.textEditor === window.activeTextEditor) {
					this.invalidate()
				}
			},
			this,
			this.subscriptions
		)

		workspace.onDidCloseTextDocument(
			(e) => {
				if (e.uri.toString() == this.uri.toString()) {
					for (const disposable of this.subscriptions) {
						disposable.dispose()
					}
					onFinish()
				}
			},
			this,
			this.subscriptions
		)
	}

	private invalidate() {
		if (isMiniZinc(window.activeTextEditor)) {
			this.eventEmitter.fire(this.uri)
		}
	}

	async activate() {
		const document = await workspace.openTextDocument(this.uri)
		this.eventEmitter.fire(this.uri)
		await window.showTextDocument(document, {
			viewColumn: ViewColumn.Two,
			preserveFocus: true,
		})
	}

	provideTextDocumentContent(
		_uri: Uri,
		ct: CancellationToken
	): ProviderResult<string> {
		const editor = window.activeTextEditor
		if (!isMiniZinc(editor)) {
			return ""
		}
		const params = {
			textDocument: this.client.code2ProtocolConverter.asTextDocumentIdentifier(
				editor.document
			),
			position: this.client.code2ProtocolConverter.asPosition(
				editor.selection.active
			),
		}
		return this.client.sendRequest(viewHirRequest, params, ct)
	}

	get onDidChange(): Event<Uri> {
		return this.eventEmitter.event
	}
}

let provider: HirProvider | null = null
export async function handleHirViewCommand(client: LanguageClient) {
	if (!provider) {
		provider = new HirProvider(client, () => {
			provider = null
		})
	}
	await provider.activate()
}
