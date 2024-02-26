use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Response};
use lsp_types::{
    CodeActionParams, CompletionParams, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentFormattingParams, DocumentRangeFormattingParams, DocumentSymbolParams,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams, Location, MarkupContent,
    MarkupKind, Position, PublishDiagnosticsParams, RenameParams, TextDocumentPositionParams, Url,
    WorkspaceEdit,
};
use markdown::mdast::Node;

use crate::ast::{find_definition_for_position, find_link_for_position};
use crate::code_actions::code_actions;
use crate::completion::completion;
use crate::definition::{
    def_handle_link_footnote, def_handle_link_ref, def_handle_link_to_heading,
};
use crate::diagnostics::check_links;
use crate::formatting::{formatting, range_formatting};
use crate::hover::{hov_handle_footnote_reference, hov_handle_link, hov_handle_link_reference};
use crate::references::{handle_definition, handle_footnote_definition, handle_heading};
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
        let position_params = params.text_document_position_params;
        let req_uri = position_params.text_document.uri;
        let Position { line, character } = position_params.position;
        let req_ast = state.ast_for_uri(&req_uri).unwrap();
        let node = find_link_for_position(req_ast, line, character);

        let (target_uri, range) = match node {
            Some(n) => match n {
                Node::Link(link) => def_handle_link_to_heading(&req_uri, link, state),
                Node::LinkReference(link_ref) => (
                    req_uri.clone(),
                    def_handle_link_ref(req_ast, &link_ref.identifier),
                ),
                Node::FootnoteReference(foot_ref) => (
                    req_uri.clone(),
                    def_handle_link_footnote(req_ast, &foot_ref.identifier),
                ),
                _ => (req_uri.clone(), None),
            },
            None => (req_uri.clone(), None),
        };

        let result = match range {
            Some(r) => {
                let location = Location {
                    uri: target_uri,
                    range: r,
                };
                let result = Some(GotoDefinitionResponse::Scalar(location));
                serde_json::to_value(result).ok()
            }
            None => None,
        };

        let resp = Response {
            id: req.id,
            result,
            error: None,
        };
        self.connection.sender.send(Message::Response(resp))?;

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover
    fn handle_hover(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: HoverParams = serde_json::from_value(req.params)?;
        let position_params = params.text_document_position_params;
        let req_uri = position_params.text_document.uri;
        let Position { line, character } = position_params.position;

        let req_ast = state.ast_for_uri(&req_uri).unwrap();
        let node = find_link_for_position(req_ast, line, character);
        log::info!("HOVERR NODE : {:?}", node);

        let message = match node {
            Some(n) => match n {
                Node::Link(link) => hov_handle_link(&req_uri, link, state),
                Node::LinkReference(link_ref) => {
                    hov_handle_link_reference(&req_uri, link_ref, state)
                }
                Node::FootnoteReference(foot_ref) => {
                    hov_handle_footnote_reference(&req_uri, foot_ref, state)
                }
                _ => None,
            },
            None => None,
        };

        let result = match message {
            Some(msg) => {
                let result = Some(Hover {
                    contents: lsp_types::HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: msg,
                    }),
                    range: None,
                });
                serde_json::to_value(result).ok()
            }
            None => None,
        };

        let response = Response {
            id: req.id,
            result,
            error: None,
        };
        self.connection.sender.send(Message::Response(response))?;

        Ok(())
    }
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_references
    fn handle_references(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: HoverParams = serde_json::from_value(req.params)?;
        let position_params = params.text_document_position_params;
        let req_uri = position_params.text_document.uri;
        let Position { line, character } = position_params.position;

        let req_ast = state.ast_for_uri(&req_uri).unwrap();
        let node = find_definition_for_position(req_ast, line, character);

        let found_links = match node {
            Some(n) => match n {
                Node::Heading(h) => Some(handle_heading(h, &req_uri, state)),
                Node::Definition(d) => handle_definition(req_ast, &req_uri, d),
                Node::FootnoteDefinition(f) => handle_footnote_definition(req_ast, &req_uri, f),
                _ => None,
            },
            None => None,
        };

        let result = found_links.map(|found_links| {
            found_links
                .into_iter()
                .map(|found_link| Location {
                    uri: found_link.file_url.clone(),
                    range: found_link.range,
                })
                .collect::<Vec<Location>>()
        });

        let result = serde_json::to_value(result)?;

        let resp = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };

        self.connection.sender.send(Message::Response(resp))?;

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentSymbol
    fn handle_document_symbol(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: DocumentSymbolParams = serde_json::from_value(req.params)?;
        let req_uri = params.text_document.uri;
        let req_ast = state.ast_for_uri(&req_uri).unwrap();

        let result = document_symbols(req_ast).and_then(|res| serde_json::to_value(res).ok());

        let resp = Response {
            id: req.id,
            result,
            error: None,
        };

        self.connection.sender.send(Message::Response(resp))?;

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_symbol
    fn handle_workspace_symbol(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let result = workspace_symbols(state).and_then(|res| serde_json::to_value(res).ok());

        let resp = Response {
            id: req.id,
            result,
            error: None,
        };

        self.connection.sender.send(Message::Response(resp))?;

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_formatting
    fn handle_formatting(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: DocumentFormattingParams = serde_json::from_value(req.params)?;
        let req_uri = params.text_document.uri;

        let result = formatting(&req_uri, state).and_then(|it| serde_json::to_value(it).ok());
        let resp = Response {
            id: req.id,
            result,
            error: None,
        };

        self.connection.sender.send(Message::Response(resp))?;

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_rangeFormatting
    fn handle_range_formatting(&self, req: lsp_server::Request, state: &mut State) -> Result<()> {
        let params: DocumentRangeFormattingParams = serde_json::from_value(req.params)?;
        let req_uri = params.text_document.uri;
        let req_range = params.range;
        let result = range_formatting(&req_uri, &req_range, state)
            .and_then(|it| serde_json::to_value(it).ok());
        let resp = Response {
            id: req.id,
            result,
            error: None,
        };

        self.connection.sender.send(Message::Response(resp))?;

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_prepareRename
    fn handle_prepare_rename(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: TextDocumentPositionParams = serde_json::from_value(req.params)?;
        let req_uri = params.text_document.uri;
        let req_position = params.position;
        let result = prepare_rename(&req_uri, &req_position, state)
            .and_then(|it| serde_json::to_value(it).ok());

        let resp = Response {
            id: req.id,
            result,
            error: None,
        };

        self.connection.sender.send(Message::Response(resp))?;

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_rename
    fn handle_rename(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: RenameParams = serde_json::from_value(req.params)?;
        let req_uri = params.text_document_position.text_document.uri;
        let new_name = params.new_name;
        let req_pos = params.text_document_position.position;

        let result = rename(&new_name, &req_uri, &req_pos, state)
            .map(|changes| WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            })
            .and_then(|it| serde_json::to_value(it).ok());
        let resp = Response {
            id: req.id,
            result,
            error: None,
        };

        self.connection.sender.send(Message::Response(resp))?;
        Ok(())
    }

    fn handle_did_change_watched_files(&self, not: lsp_server::Notification) -> Result<()> {
        log::info!("HANDLE DID CHANGE WATCHED FILE: {:?}", not);
        Ok(())
    }

    fn handle_code_action(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        log::info!("CODE ACTION : {:?}", req);
        let params: CodeActionParams = serde_json::from_value(req.params)?;
        log::info!("PARAMS : {:?}", params);
        let req_uri = params.text_document.uri;
        let result = code_actions(&req_uri, state).and_then(|it| serde_json::to_value(it).ok());
        let resp = Response {
            id: req.id,
            result,
            error: None,
        };
        self.connection.sender.send(Message::Response(resp))?;

        Ok(())
        // CODE ACTION : Request {
        // id: RequestId(I32(2)),
        // method: "textDocument/codeAction",
        // params: Object {
        //      "context": Object {"diagnostics": Array [], "triggerKind": Number(1)},
        //      "range": Object {"end": Object {"character":  Number(0), "line": Number(0)}, "start": Object {"character": Number(0), "line": Number(0)}},
        //      "textDocument": Object {"uri": String("file:///home/matthias/github/md-lsp/test.md")}}
        // }
        //  PARAMS : CodeActionParams {
        //  text_document: TextDocumentIdentifier { uri: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/home/matthias/github/md-lsp/test.md", query: None, fragment: None } },
        //  range: Range { start: Position { line: 0, character: 0 }, end: Position { line: 0, character: 0 } },
        //  context: CodeActionContext { diagnostics: [], only: None, trigger_kind: Some(Invoked) }, work_done_progress_params: WorkDoneProgressParams { work_done_token: None }, partial_result_params: PartialResultParams { partial_result_token: None } }
    }

    fn handle_completion(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: CompletionParams = serde_json::from_value(req.params)?;
        log::info!("COMPLETION PARAMS : {:?}", params);
        let result = completion(params, state)
            .and_then(|completion_list| serde_json::to_value(completion_list).ok());
        let resp = Response {
            id: req.id,
            result,
            error: None,
        };
        self.connection.sender.send(Message::Response(resp))?;
        Ok(())
    }
}
