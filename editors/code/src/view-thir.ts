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

type ThirStage = "lowered" | "transformed"

const views = {
	lowered: {
		uri: Uri.parse("shackle-ls-thir://viewLoweredThir/lowered-thir.mzn"),
		request: new RequestType<TextDocumentPositionParams, string, void>(
			"shackle-ls/viewLoweredThir"
		),
	},
	transformed: {
		uri: Uri.parse(
			"shackle-ls-thir://viewTransformedThir/transformed-thir.mzn"
		),
		request: new RequestType<TextDocumentPositionParams, string, void>(
			"shackle-ls/viewTransformedThir"
		),
	},
}

class ThirProvider implements TextDocumentContentProvider, Disposable {
	readonly eventEmitter = new EventEmitter<Uri>()

	private client: LanguageClient
	private subscriptions: Disposable[] = []

	constructor(client: LanguageClient) {
		this.client = client

		this.subscriptions.push(
			workspace.registerTextDocumentContentProvider("shackle-ls-thir", this)
		)

		workspace.onDidChangeTextDocument(
			(e) => {
				if (e.document === window.activeTextEditor?.document) {
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
	}

	dispose() {
		for (const disposable of this.subscriptions) {
			disposable.dispose()
		}
		this.eventEmitter.dispose()
		provider = null
	}

	private invalidate() {
		if (isMiniZinc(window.activeTextEditor)) {
			for (const view of Object.values(views)) {
				this.eventEmitter.fire(view.uri)
			}
		}
	}

	async activate(stage: ThirStage) {
		const uri = views[stage].uri
		const document = await workspace.openTextDocument(uri)
		this.eventEmitter.fire(uri)
		await window.showTextDocument(document, {
			viewColumn: ViewColumn.Two,
			preserveFocus: true,
		})
	}

	provideTextDocumentContent(
		uri: Uri,
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
		const view = Object.values(views).find(
			(view) => view.uri.toString() === uri.toString()
		)
		return view ? this.client.sendRequest(view.request, params, ct) : ""
	}

	get onDidChange(): Event<Uri> {
		return this.eventEmitter.event
	}
}

let provider: ThirProvider | null = null
export async function handleThirViewCommand(
	client: LanguageClient,
	context: ExtensionContext,
	stage: ThirStage
) {
	if (!provider) {
		provider = new ThirProvider(client)
		context.subscriptions.push(provider)
	}
	await provider.activate(stage)
}
