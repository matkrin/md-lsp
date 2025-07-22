use anyhow::Result;
use lsp_server::{Connection, Message, Notification, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument, Exit,
    Notification as _,
};
use lsp_types::request::{
    CodeActionRequest, Completion, DocumentDiagnosticRequest, DocumentSymbolRequest, Formatting,
    GotoDefinition, HoverRequest, PrepareRenameRequest, RangeFormatting, References, Rename,
    Request, Shutdown, WorkspaceSymbolRequest,
};
use lsp_types::{
    CodeActionParams, CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentFormattingParams, DocumentRangeFormattingParams, DocumentSymbolParams,
    GotoDefinitionParams, HoverParams, NumberOrString, PublishDiagnosticsParams, ReferenceParams,
    RenameParams, TextDocumentPositionParams, Url, WorkspaceEdit,
};
use serde::Serialize;

use crate::code_actions::code_actions;
use crate::completion::completion;
use crate::definition::definition;
use crate::diagnostics::check_links;
use crate::formatting::{formatting, range_formatting};
use crate::hover::hover;
use crate::references::references;
use crate::rename::{prepare_rename, rename};
use crate::state::State;
use crate::symbols::{document_symbols, workspace_symbols};

pub struct Server {
    connection: Connection,
}

impl Server {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    fn send_result<S: Serialize>(&self, req_id: RequestId, result: S) -> Result<()> {
        let response = Response::new_ok(req_id, serde_json::to_value(result).ok());
        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    pub fn run(&self, mut state: State) -> Result<()> {
        for msg in &self.connection.receiver {
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    match req.method.as_ref() {
                        GotoDefinition::METHOD => self.handle_defintion(req, &mut state)?,
                        HoverRequest::METHOD => self.handle_hover(req, &mut state)?,
                        References::METHOD => self.handle_references(req, &mut state)?,
                        DocumentSymbolRequest::METHOD => {
                            self.handle_document_symbol(req, &mut state)?
                        }
                        WorkspaceSymbolRequest::METHOD => {
                            self.handle_workspace_symbol(req, &mut state)?
                        }
                        Formatting::METHOD => self.handle_formatting(req, &mut state)?,
                        RangeFormatting::METHOD => self.handle_range_formatting(req, &mut state)?,
                        PrepareRenameRequest::METHOD => self.handle_prepare_rename(req, &state)?,
                        Rename::METHOD => self.handle_rename(req, &state)?,
                        DocumentDiagnosticRequest::METHOD => {
                            log::info!("DIAGNOSTIC REQUEST: {:?}", req);
                        }
                        CodeActionRequest::METHOD => self.handle_code_action(req, &state)?,
                        Completion::METHOD => self.handle_completion(req, &state)?,
                        Shutdown::METHOD => self.handle_shutdown(req)?,
                        _ => {
                            log::info!("OTHER REQUEST: {:?}", req);
                        }
                    }
                }
                Message::Response(resp) => {
                    log::info!("GOT RESPONSE: {resp:?}");
                }
                Message::Notification(not) => match not.method.as_ref() {
                    DidOpenTextDocument::METHOD => self.handle_did_open(not, &mut state)?,
                    DidChangeTextDocument::METHOD => self.handle_did_change(not, &mut state)?,
                    DidCloseTextDocument::METHOD => self.handle_did_close(not)?,
                    DidChangeWatchedFiles::METHOD => self.handle_did_change_watched_files(not)?,
                    Exit::METHOD => self.handle_exit(not),
                    _ => {
                        log::info!("OTHER NOTIFICATION: {:?}", not)
                    }
                },
            }
        }
        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#shutdown
    fn handle_shutdown(&self, req: lsp_server::Request) -> Result<()> {
        log::info!("SHUTDOWN REQ: {:?}", &req);
        let response = Response {
            id: req.id,
            result: None,
            error: None,
        };
        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#exit
    fn handle_exit(&self, not: lsp_server::Notification) {
        log::info!("EXIT NOT: {:?}", &not);
        std::process::exit(0);
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_didOpen
    fn handle_did_open(&self, not: lsp_server::Notification, state: &mut State) -> Result<()> {
        let params: DidOpenTextDocumentParams = serde_json::from_value(not.params)?;
        let uri = params.text_document.uri;
        state.set_buffer(&uri, params.text_document.text);

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_didChange
    fn handle_did_change(&self, not: lsp_server::Notification, state: &mut State) -> Result<()> {
        let params: DidChangeTextDocumentParams = serde_json::from_value(not.params)?;
        let change_event = params.content_changes.into_iter().next_back().unwrap();
        let uri = params.text_document.uri;
        state.set_buffer(&uri, change_event.text);
        self.handle_diagnostic(&uri, state)?;

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_didClose
    fn handle_did_close(&self, not: lsp_server::Notification) -> Result<()> {
        let _params: DidCloseTextDocumentParams = serde_json::from_value(not.params)?;

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics
    pub fn handle_diagnostic(&self, req_uri: &Url, state: &State) -> Result<()> {
        let ast = state.ast_for_uri(req_uri).unwrap();
        // for link, reflink, footnote check if their definitions exist
        let diagnostics = check_links(ast, req_uri, state)
            .into_iter()
            .map(|it| {
                Diagnostic {
                    range: it.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::Number(
                        it.error_code()
                            .try_into()
                            .expect("error code value to large for i32"),
                    )),
                    code_description: None,
                    source: Some("md-lsp".to_string()),
                    message: it.message,
                    related_information: None, // might be interesting
                    tags: None,
                    data: None,
                }
            })
            .collect::<Vec<_>>();

        let diagnostic_params = PublishDiagnosticsParams {
            uri: req_uri.clone(),
            diagnostics,
            version: None,
        };

        let diagnostic_params = serde_json::to_value(diagnostic_params)?;

        let resp = Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params: diagnostic_params,
        };
        self.connection.sender.send(Message::Notification(resp))?;
        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_definition
    fn handle_defintion(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: GotoDefinitionParams = serde_json::from_value(req.params)?;
        let result = definition(&params, state);
        self.send_result(req.id, result)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover
    fn handle_hover(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: HoverParams = serde_json::from_value(req.params)?;
        let result = hover(&params, state);
        self.send_result(req.id, result)
    }
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_references
    fn handle_references(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: ReferenceParams = serde_json::from_value(req.params)?;
        let result = references(&params, state);
        self.send_result(req.id, result)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentSymbol
    fn handle_document_symbol(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: DocumentSymbolParams = serde_json::from_value(req.params)?;
        let result = document_symbols(&params, state);
        self.send_result(req.id, result)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_symbol
    fn handle_workspace_symbol(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let result = workspace_symbols(state);
        self.send_result(req.id, result)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_formatting
    fn handle_formatting(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: DocumentFormattingParams = serde_json::from_value(req.params)?;
        let result = formatting(&params, state);
        self.send_result(req.id, result)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_rangeFormatting
    fn handle_range_formatting(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: DocumentRangeFormattingParams = serde_json::from_value(req.params)?;
        let result = range_formatting(&params, state);
        self.send_result(req.id, result)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_prepareRename
    fn handle_prepare_rename(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: TextDocumentPositionParams = serde_json::from_value(req.params)?;
        let result = prepare_rename(&params, state);
        self.send_result(req.id, result)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_rename
    fn handle_rename(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: RenameParams = serde_json::from_value(req.params)?;

        let result = rename(&params, state).map(|changes| WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        });
        self.send_result(req.id, result)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_didChangeWatchedFiles
    fn handle_did_change_watched_files(&self, not: lsp_server::Notification) -> Result<()> {
        log::info!("HANDLE DID CHANGE WATCHED FILE: {:?}", not);
        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeAction
    fn handle_code_action(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: CodeActionParams = serde_json::from_value(req.params)?;
        let result = code_actions(&params, state);
        self.send_result(req.id, result)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion
    fn handle_completion(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: CompletionParams = serde_json::from_value(req.params)?;
        let result = completion(params, state).map(CompletionResponse::List);
        self.send_result(req.id, result)
    }
}
