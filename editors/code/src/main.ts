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

function bundledMiniZincStdlib(context: ExtensionContext): string | undefined {
	const stdlib = path.join(
		context.extensionPath,
		"vendor",
		"minizinc",
		"share",
		"minizinc"
	)
	return fs.existsSync(stdlib) ? stdlib : undefined
}

export async function activate(context: ExtensionContext) {
	const configuration = workspace.getConfiguration("shackleLanguageServer")
	const configuredCommand = configuration.get<string>("executable")
	const configuredEnvironment =
		configuration.get<Record<string, string>>("environment") ?? {}
	const bundledStdlib = bundledMiniZincStdlib(context)

	const run: Executable = {
		command:
			configuredCommand && configuredCommand.trim().length > 0
				? configuredCommand
				: (bundledExecutable(context) ?? "shackle-ls"),
		options: {
			env: {
				...process.env,
				...(bundledStdlib ? { MZN_STDLIB_DIR: bundledStdlib } : {}),
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
		"shackleLanguageServer",
		"MiniZinc language server",
		serverOptions,
		clientOptions
	)

	await client.start()
	context.subscriptions.push(
		commands.registerCommand("shackleLanguageServer.viewCst", () =>
			handleCstViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("shackleLanguageServer.viewAst", () =>
			handleAstViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("shackleLanguageServer.viewFormatIr", () =>
			handleFormatIrViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("shackleLanguageServer.viewHir", () =>
			handleHirViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("shackleLanguageServer.viewScope", () =>
			handleScopeViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("shackleLanguageServer.viewPrettyPrint", () =>
			handlePrettyPrintViewCommand(client)
		)
	)
	context.subscriptions.push(
		commands.registerCommand("shackleLanguageServer.viewMir", () =>
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
