import * as fs from "fs"
import * as path from "path"
import { ExtensionContext, workspace, commands } from "vscode"

import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
} from "vscode-languageclient/node"
import { handleAstViewCommand } from "./view-ast"
import { handleCstViewCommand } from "./view-cst"
import { handleHirViewCommand } from "./view-hir"
import { handlePrettyPrintViewCommand } from "./view-pretty-print"
import { handleScopeViewCommand } from "./view-scope"
import { handleFormatIrViewCommand } from "./view-format-ir"
import { handleMirViewCommand } from "./view-mir"

let client: LanguageClient

function bundledExecutable(context: ExtensionContext): string | undefined {
	const executableName =
		process.platform === "win32" ? "shackle-ls.exe" : "shackle-ls"
	const executable = path.join(context.extensionPath, "bin", executableName)
	return fs.existsSync(executable) ? executable : undefined
}

export async function activate(context: ExtensionContext) {
	const configuration = workspace.getConfiguration("minizincLanguageServer")
	const configuredCommand = configuration.get<string>("executable")
	const configuredEnvironment =
		configuration.get<Record<string, string>>("environment") ?? {}

	const run: Executable = {
		command:
			configuredCommand && configuredCommand.trim().length > 0
				? configuredCommand
				: (bundledExecutable(context) ?? "shackle-ls"),
		options: {
			env: {
				...process.env,
				...configuredEnvironment,
			},
		},
	}
	const serverOptions: ServerOptions = {
		run,
		debug: run,
	}

	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: "file", language: "minizinc" }],
	}

	client = new LanguageClient(
		"minizincLanguageServer",
		"MiniZinc language server",
		serverOptions,
		clientOptions
	)

	await client.start()
	context.subscriptions.push(
		commands.registerCommand("minizincLanguageServer.viewCst", () =>
			handleCstViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("minizincLanguageServer.viewAst", () =>
			handleAstViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("minizincLanguageServer.viewFormatIr", () =>
			handleFormatIrViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("minizincLanguageServer.viewHir", () =>
			handleHirViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("minizincLanguageServer.viewScope", () =>
			handleScopeViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("minizincLanguageServer.viewPrettyPrint", () =>
			handlePrettyPrintViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("minizincLanguageServer.viewMir", () =>
			handleMirViewCommand(client)
		)
	)
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined
	}
	return client.stop()
}
