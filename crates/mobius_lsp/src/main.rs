//! `mobius-lsp` — stdio language server. Full-document sync; on every
//! open/change the document is reparsed (sub-millisecond at `.mobius`
//! sizes — no incremental machinery needed) and parse diagnostics are
//! published.

use lsp_server::{Connection, ErrorCode, Message, Notification, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    PublishDiagnostics,
};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    Uri,
};

fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let (connection, io_threads) = Connection::stdio();

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
    };
    connection.initialize(serde_json::to_value(capabilities)?)?;

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                // Nothing else is advertised; answer rather than hang.
                let resp = Response::new_err(
                    req.id,
                    ErrorCode::MethodNotFound as i32,
                    format!("mobius-lsp does not handle `{}`", req.method),
                );
                connection.sender.send(resp.into())?;
            }
            Message::Notification(note) => match note.method.as_str() {
                DidOpenTextDocument::METHOD => {
                    let params: DidOpenTextDocumentParams = serde_json::from_value(note.params)?;
                    publish(&connection, params.text_document.uri, &params.text_document.text)?;
                }
                DidChangeTextDocument::METHOD => {
                    let params: DidChangeTextDocumentParams = serde_json::from_value(note.params)?;
                    // FULL sync: the last change carries the whole document.
                    if let Some(change) = params.content_changes.into_iter().next_back() {
                        publish(&connection, params.text_document.uri, &change.text)?;
                    }
                }
                DidCloseTextDocument::METHOD => {
                    let params: DidCloseTextDocumentParams = serde_json::from_value(note.params)?;
                    // Clear stale squiggles for closed documents.
                    publish(&connection, params.text_document.uri, "")?;
                }
                _ => {}
            },
            Message::Response(_) => {}
        }
    }

    io_threads.join()?;
    Ok(())
}

fn publish(
    connection: &Connection,
    uri: Uri,
    text: &str,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics: mobius_lsp::diagnostics(text),
        version: None,
    };
    let note = Notification::new(PublishDiagnostics::METHOD.to_owned(), params);
    connection.sender.send(note.into())?;
    Ok(())
}
