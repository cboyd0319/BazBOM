import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext): void {
  console.log('BazBOM extension is now active');

  // Get configuration
  const config = vscode.workspace.getConfiguration('bazbom');
  const lspPath = config.get<string>('lspPath', 'bazbom-lsp');
  const enableRealTimeScanning = config.get<boolean>('enableRealTimeScanning', true);

  if (!enableRealTimeScanning) {
    console.log('Real-time scanning is disabled');
    return;
  }

  // Configure the language server
  const serverOptions: ServerOptions = {
    command: lspPath,
    args: [],
    transport: TransportKind.stdio
  };

  // Configure the language client
  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'xml', pattern: '**/pom.xml' },
      { scheme: 'file', language: 'groovy', pattern: '**/build.gradle' },
      { scheme: 'file', language: 'kotlin', pattern: '**/build.gradle.kts' },
      { scheme: 'file', pattern: '**/BUILD' },
      { scheme: 'file', pattern: '**/BUILD.bazel' }
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher(
        '**/{pom.xml,build.gradle,build.gradle.kts,BUILD,BUILD.bazel}'
      )
    },
    outputChannel: vscode.window.createOutputChannel('BazBOM Language Server')
  };

  // Create and start the language client
  client = new LanguageClient(
    'bazbom',
    'BazBOM Security Scanner',
    serverOptions,
    clientOptions
  );

  // Start the client
  client.start().then(() => {
    console.log('BazBOM Language Server started');
    vscode.window.showInformationMessage('BazBOM Security Scanner is now active');
  }).catch((error) => {
    console.error('Failed to start BazBOM Language Server:', error);
    vscode.window.showErrorMessage(
      `Failed to start BazBOM Language Server: ${error.message}. ` +
      'Make sure bazbom-lsp is installed and in your PATH.'
    );
  });

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand('bazbom.scan', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showWarningMessage('No active editor');
        return;
      }

      vscode.window.showInformationMessage('Scanning with BazBOM...');
      
      // Save the file first to trigger scan
      await editor.document.save();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('bazbom.syncAdvisories', async () => {
      vscode.window.showInformationMessage('Syncing BazBOM advisory database...');
      
      const terminal = vscode.window.createTerminal('BazBOM Sync');
      terminal.show();
      terminal.sendText('bazbom db sync');
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
