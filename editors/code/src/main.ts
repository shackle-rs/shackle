import {
  ExtensionContext,
  workspace,
  commands,
  TextDocumentContentProvider,
  Uri,
  EventEmitter,
  CancellationToken,
  ProviderResult,
  window,
  TextEditor,
  TextDocumentChangeEvent,
  Event,
  ViewColumn,
} from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
  RequestType,
  TextDocumentPositionParams,
} from "vscode-languageclient/node";
import { handleHirViewCommand } from "./view-hir";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const command = workspace
    .getConfiguration("shackleLanguageServer")
    .get<string>("executable");

  const run = {
    command,
    transport: TransportKind.ipc,
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

  client.start();
  client.onReady().then(() => {
    context.subscriptions.push(
      commands.registerCommand("shackleLanguageServer.viewHir", () =>
        handleHirViewCommand(client)
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
