import { ExtensionContext, workspace, commands } from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { handleAstViewCommand } from "./view-ast";
import { handleCstViewCommand } from "./view-cst";
import { handleHirViewCommand } from "./view-hir";
import { handlePrettyPrintViewCommand } from "./view-pretty-print";
import { handleScopeViewCommand } from "./view-scope";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const command = workspace
    .getConfiguration("shackleLanguageServer")
    .get<string>("executable");

  const run = {
    command,
    transport: TransportKind.stdio,
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "minizinc" }],
  };

  client = new LanguageClient(
    "shackleLanguageServer",
    "MiniZinc language server",
    serverOptions,
    clientOptions
  );

  client.start().then(() => {
    context.subscriptions.push(
      commands.registerCommand("shackleLanguageServer.viewCst", () =>
        handleCstViewCommand(client)
      )
    );
    context.subscriptions.push(
      commands.registerCommand("shackleLanguageServer.viewAst", () =>
        handleAstViewCommand(client)
      )
    );
    context.subscriptions.push(
      commands.registerCommand("shackleLanguageServer.viewHir", () =>
        handleHirViewCommand(client)
      )
    );
    context.subscriptions.push(
      commands.registerCommand("shackleLanguageServer.viewScope", () =>
        handleScopeViewCommand(client)
      )
    );
    context.subscriptions.push(
      commands.registerCommand("shackleLanguageServer.viewPrettyPrint", () =>
        handlePrettyPrintViewCommand(client)
      )
    );
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
