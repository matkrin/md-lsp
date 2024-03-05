use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Response};
use lsp_types::{
    CodeActionParams, CompletionParams, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentFormattingParams, DocumentRangeFormattingParams, DocumentSymbolParams,
    GotoDefinitionParams, HoverParams, PublishDiagnosticsParams, ReferenceParams, RenameParams,
    TextDocumentPositionParams, Url, WorkspaceEdit,
};

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

    fn send(&self, response: lsp_server::Response) -> Result<()> {
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
                        "textDocument/definition" => self.handle_defintion(req, &mut state)?,
                        "textDocument/hover" => self.handle_hover(req, &mut state)?,
                        "textDocument/references" => self.handle_references(req, &mut state)?,
                        "textDocument/documentSymbol" => {
                            self.handle_document_symbol(req, &mut state)?
                        }
                        "workspace/symbol" => self.handle_workspace_symbol(req, &mut state)?,
                        "textDocument/formatting" => self.handle_formatting(req, &mut state)?,
                        "textDocument/rangeFormatting" => {
                            self.handle_range_formatting(req, &mut state)?
                        }
                        "textDocument/prepareRename" => self.handle_prepare_rename(req, &state)?,
                        "textDocument/rename" => self.handle_rename(req, &state)?,
                        "textDocument/diagnostic" => {
                            log::info!("DIAGNOSTIC REQUEST: {:?}", req);
                        }
                        "textDocument/codeAction" => self.handle_code_action(req, &state)?,
                        "textDocument/completion" => self.handle_completion(req, &state)?,
                        _ => {
                            log::info!("OTHER REQUEST: {:?}", req);
                        }
                    }
                }
                Message::Response(resp) => {
                    log::info!("GOT RESPONSE: {resp:?}");
                }
                Message::Notification(not) => match not.method.as_ref() {
                    "textDocument/didOpen" => self.handle_did_open(not, &mut state)?,
                    "textDocument/didChange" => self.handle_did_change(not, &mut state)?,
                    "textDocument/didClose" => self.handle_did_close(not)?,
                    "workspace/didChangeWatchedFiles" => {
                        self.handle_did_change_watched_files(not)?
                    }
                    _ => {
                        log::info!("OTHER NOTIFICATION: {:?}", not)
                    }
                },
            }
        }
        Ok(())
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
        let change_event = params.content_changes.into_iter().last().unwrap();
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
                    code: None,
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
        let result = definition(&params, state).and_then(|def| serde_json::to_value(def).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover
    fn handle_hover(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: HoverParams = serde_json::from_value(req.params)?;
        let result = hover(&params, state).and_then(|hov| serde_json::to_value(hov).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_references
    fn handle_references(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: ReferenceParams = serde_json::from_value(req.params)?;
        let result = references(&params, state).and_then(|refs| serde_json::to_value(refs).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentSymbol
    fn handle_document_symbol(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: DocumentSymbolParams = serde_json::from_value(req.params)?;
        let result =
            document_symbols(&params, state).and_then(|res| serde_json::to_value(res).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_symbol
    fn handle_workspace_symbol(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let result = workspace_symbols(state).and_then(|res| serde_json::to_value(res).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_formatting
    fn handle_formatting(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: DocumentFormattingParams = serde_json::from_value(req.params)?;
        let result = formatting(&params, state).and_then(|it| serde_json::to_value(it).ok());
        let resp = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(resp)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_rangeFormatting
    fn handle_range_formatting(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: DocumentRangeFormattingParams = serde_json::from_value(req.params)?;
        let result = range_formatting(&params, state).and_then(|it| serde_json::to_value(it).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_prepareRename
    fn handle_prepare_rename(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: TextDocumentPositionParams = serde_json::from_value(req.params)?;
        let result = prepare_rename(&params, state).and_then(|it| serde_json::to_value(it).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_rename
    fn handle_rename(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: RenameParams = serde_json::from_value(req.params)?;

        let result = rename(&params, state)
            .map(|changes| WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            })
            .and_then(|it| serde_json::to_value(it).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }

    fn handle_did_change_watched_files(&self, not: lsp_server::Notification) -> Result<()> {
        log::info!("HANDLE DID CHANGE WATCHED FILE: {:?}", not);
        Ok(())
    }

    fn handle_code_action(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: CodeActionParams = serde_json::from_value(req.params)?;
        let result = code_actions(&params, state).and_then(|it| serde_json::to_value(it).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }

    fn handle_completion(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: CompletionParams = serde_json::from_value(req.params)?;
        let result = completion(params, state)
            .and_then(|completion_list| serde_json::to_value(completion_list).ok());
        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.send(response)
    }
}
