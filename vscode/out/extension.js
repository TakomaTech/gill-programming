"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = require("vscode");
function activate(context) {
    const welcomeCommand = vscode.commands.registerCommand('gill.openWelcome', () => {
        vscode.window.showInformationMessage('Welcome to Gill language support! Use .gill files to write Gill programs.');
    });
    const hoverProvider = vscode.languages.registerHoverProvider('gill', {
        provideHover(document, position) {
            const wordRange = document.getWordRangeAtPosition(position, /[A-Za-z0-9_.]+/);
            if (!wordRange) {
                return undefined;
            }
            const word = document.getText(wordRange);
            if (word === 'func') {
                return new vscode.Hover('`func` declares a new function in Gill.');
            }
            if (word === 'print.console') {
                return new vscode.Hover('`print.console(...)` writes a string to the console.');
            }
            return undefined;
        }
    });
    context.subscriptions.push(welcomeCommand, hoverProvider);
}
function deactivate() { }
//# sourceMappingURL=extension.js.map