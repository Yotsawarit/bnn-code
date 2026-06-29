import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as path from 'path';

let outputChannel: vscode.OutputChannel;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
    console.log('🧠 BNN Code extension activated');

    // Create output channel
    outputChannel = vscode.window.createOutputChannel('BNN Code');

    // Create status bar item
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = '$(brain) BNN';
    statusBarItem.tooltip = 'BNN Code - AI Coding Assistant';
    statusBarItem.command = 'bnn.query';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Register commands
    const commands = [
        vscode.commands.registerCommand('bnn.explain', () => runBnn('explain')),
        vscode.commands.registerCommand('bnn.refactor', () => runBnn('refactor')),
        vscode.commands.registerCommand('bnn.test', () => runBnn('test')),
        vscode.commands.registerCommand('bnn.fix', () => runBnn('fix')),
        vscode.commands.registerCommand('bnn.commit', () => runBnn('commit')),
        vscode.commands.registerCommand('bnn.review', () => runBnn('review')),
        vscode.commands.registerCommand('bnn.document', () => runBnn('document')),
        vscode.commands.registerCommand('bnn.query', () => runBnnQuery()),
        vscode.commands.registerCommand('bnn.terminal', () => openBnnTerminal()),
    ];

    context.subscriptions.push(...commands);
}

export function deactivate() {
    console.log('🧠 BNN Code extension deactivated');
}

async function runBnn(command: string) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const selection = editor.selection;
    const text = editor.document.getText(selection);

    if (!text && command !== 'fix' && command !== 'commit' && command !== 'review') {
        vscode.window.showErrorMessage('No text selected');
        return;
    }

    const config = vscode.workspace.getConfiguration('bnn');
    const binaryPath = config.get<string>('binaryPath', 'bnn');
    const codebasePath = config.get<string>('codebasePath', '.');

    // Build command
    let args: string[] = [];

    switch (command) {
        case 'explain':
            args = ['explain', editor.document.fileName];
            break;
        case 'refactor':
            args = ['refactor', editor.document.fileName];
            break;
        case 'test':
            args = ['test', editor.document.fileName];
            break;
        case 'fix':
            // Pass file if there is a selection, otherwise scan codebase
            if (text) {
                args = ['fix', editor.document.fileName];
            } else {
                args = ['fix'];
            }
            break;
        case 'commit':
            args = ['commit'];
            break;
        case 'review':
            // Pass file if there is a selection, otherwise review diff
            if (text) {
                args = ['review', editor.document.fileName];
            } else {
                args = ['review'];
            }
            break;
        case 'document':
            args = ['document', editor.document.fileName];
            break;
    }

    args.push('--path', codebasePath);

    // Show progress
    await vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            title: `BNN: ${command}`,
            cancellable: false,
        },
        async (progress) => {
            progress.report({ message: 'Processing...' });

            try {
                const result = await executeBnn(binaryPath, args);

                if (config.get<boolean>('showOutput', true)) {
                    outputChannel.clear();
                    outputChannel.appendLine(`🧠 BNN Code - ${command}\n`);
                    outputChannel.appendLine(result);
                    outputChannel.show();
                }

                // Show in editor if it contains code blocks
                if (result.includes('```')) {
                    const codeMatch = result.match(/```[\s\S]*?\n([\s\S]*?)\n```/);
                    if (codeMatch) {
                        const code = codeMatch[1];
                        const doc = await vscode.workspace.openTextDocument({
                            content: code,
                            language: editor.document.languageId,
                        });
                        await vscode.window.showTextDocument(doc, vscode.ViewColumn.Beside);
                    }
                }

                vscode.window.showInformationMessage(`✅ BNN ${command} completed`);
            } catch (error) {
                vscode.window.showErrorMessage(`❌ BNN error: ${error}`);
            }
        }
    );
}

async function runBnnQuery() {
    const query = await vscode.window.showInputBox({
        prompt: 'Ask BNN Code',
        placeHolder: 'e.g., explain the authentication flow',
        ignoreFocusOut: true,
    });

    if (!query) {
        return;
    }

    const config = vscode.workspace.getConfiguration('bnn');
    const binaryPath = config.get<string>('binaryPath', 'bnn');
    const codebasePath = config.get<string>('codebasePath', '.');

    await vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            title: 'BNN: Query',
            cancellable: false,
        },
        async (progress) => {
            progress.report({ message: 'Thinking...' });

            try {
                const result = await executeBnn(binaryPath, [query, '--path', codebasePath]);

                if (config.get<boolean>('showOutput', true)) {
                    outputChannel.clear();
                    outputChannel.appendLine(`🧠 BNN Code - Query\n`);
                    outputChannel.appendLine(`Question: ${query}\n`);
                    outputChannel.appendLine(result);
                    outputChannel.show();
                }

                vscode.window.showInformationMessage('✅ BNN query completed');
            } catch (error) {
                vscode.window.showErrorMessage(`❌ BNN error: ${error}`);
            }
        }
    );
}

function openBnnTerminal() {
    const terminal = vscode.window.createTerminal({
        name: 'BNN Code',
        shellPath: process.platform === 'win32' ? 'cmd.exe' : '/bin/bash',
        shellArgs: process.platform === 'win32' ? [] : ['-c', 'bnn'],
    });
    terminal.show();
}

function executeBnn(binaryPath: string, args: string[]): Promise<string> {
    return new Promise((resolve, reject) => {
        const cwd = vscode.workspace.rootPath || process.cwd();
        const proc = cp.spawn(binaryPath, args, { cwd });

        let stdout = '';
        let stderr = '';

        proc.stdout.on('data', (data: Buffer) => {
            stdout += data.toString();
        });

        proc.stderr.on('data', (data: Buffer) => {
            stderr += data.toString();
        });

        proc.on('close', (code: number | null) => {
            if (code === 0) {
                resolve(stdout);
            } else {
                reject(stderr || `Process exited with code ${code}`);
            }
        });

        proc.on('error', (err: Error) => {
            reject(err.message);
        });

        // Timeout after 60 seconds
        setTimeout(() => {
            proc.kill();
            reject('Process timed out after 60 seconds');
        }, 60000);
    });
}
