// Client for mobius-lsp: spawns the server binary over stdio and lets
// vscode-languageclient handle the protocol. The server publishes parse
// diagnostics; everything else (highlighting, brackets) stays declarative.

const os = require("os");
const fs = require("fs");
const path = require("path");
const { workspace, window } = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");

let client;

function serverPath() {
  const configured = workspace.getConfiguration("mobius").get("lsp.path");
  if (configured) {
    return configured;
  }
  const cargoBin = path.join(os.homedir(), ".cargo", "bin", "mobius-lsp");
  if (fs.existsSync(cargoBin)) {
    return cargoBin;
  }
  return "mobius-lsp"; // hope it's on PATH
}

exports.activate = function activate(context) {
  const command = serverPath();
  client = new LanguageClient(
    "mobius",
    "mobius-lang",
    { command },
    { documentSelector: [{ scheme: "file", language: "mobius" }] }
  );
  client.start().catch((err) => {
    window.showWarningMessage(
      `mobius-lsp failed to start (${command}): ${err.message}. ` +
        "Install it with `cargo install --path crates/mobius_lsp` or set mobius.lsp.path."
    );
  });
  context.subscriptions.push({ dispose: () => client && client.stop() });
};

exports.deactivate = function deactivate() {
  return client ? client.stop() : undefined;
};
