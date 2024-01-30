use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Response};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
    Location, MarkupContent, MarkupKind, Position, PublishDiagnosticsParams, Url,
};
use markdown::mdast::Node;

use crate::ast::{find_definition_for_position, find_link_for_position};
use crate::definition::{
    def_handle_link_footnote, def_handle_link_ref, def_handle_link_to_heading,
};
use crate::diagnostics::check_links;
use crate::hover::{hov_handle_footnote_reference, hov_handle_link, hov_handle_link_reference};
use crate::references::{handle_definition, handle_footnote_definition, handle_heading};
use crate::state::State;

pub struct Server {
    connection: Connection,
}

impl Server {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn run(&self, mut state: State) -> Result<()> {
        for msg in &self.connection.receiver {
            log::info!("GOT MSG: {msg:?}");
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    log::info!("GOT REQUEST: {req:?}");
                    match req.method.as_ref() {
                        "textDocument/definition" => self.handle_defintion(req, &mut state)?,
                        "textDocument/hover" => self.handle_hover(req, &mut state)?,
                        "textDocument/references" => self.handle_references(req, &mut state)?,
                        "textDocument/diagnostic" => {
                            log::info!("DIAGNOSTIC REQUEST: {:?}", req);
                        }
                        _ => {
                            log::info!("OTHER REQUEST: {:?}", req);
                        }
                    }
                }
                Message::Response(resp) => {
                    log::info!("GOT RESPONSE: {resp:?}");
                }
                Message::Notification(not) => {
                    log::info!("GOT NOTIFICATION: {not:?}");
                    match not.method.as_ref() {
                        "textDocument/didOpen" => self.handle_did_open(not, &mut state)?,
                        "textDocument/didChange" => self.handle_did_change(not, &mut state)?,
                        "textDocument/didClose" => self.handle_did_close(not)?,
                        _ => {
                            log::info!("OTHER NOTIFICATION: {:?}", not)
                        }
                    }
                }
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
    pub fn handle_diagnostic(&self, uri: &Url, state: &State) -> Result<()> {
        let ast = state.ast_for_uri(uri).unwrap();
        // for link, reflink, footnote check if their definitions exist
        let diagnostics = check_links(ast, uri, state)
            .into_iter()
            .map(|x| {
                Diagnostic {
                    range: x.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("md-lsp".to_string()),
                    message: x.message,
                    related_information: None, // might be interesting
                    tags: None,
                    data: None,
                }
            })
            .collect::<Vec<_>>();

        let diagnostic_params = PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version: None,
        };

        let diagnostic_params = serde_json::to_value(diagnostic_params).unwrap();

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
        log::info!("GOT gotoDefinition REQUEST #{}: {:?}", req.id, params);
        let position_params = params.text_document_position_params;
        let req_uri = position_params.text_document.uri;
        let Position { line, character } = position_params.position;
        let req_ast = state.ast_for_uri(&req_uri).unwrap();
        let node = find_link_for_position(req_ast, line, character);
        log::info!("GOTO FOUND NODE : {:?}", node);

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

        let message = match node {
            Some(n) => match n {
                Node::Link(link) => hov_handle_link(&req_uri, link, state),
                Node::LinkReference(link_ref) => hov_handle_link_reference(req_ast, link_ref),
                Node::FootnoteReference(foot_ref) => {
                    hov_handle_footnote_reference(req_ast, foot_ref)
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

        // log::info!("AST : {:?}", req_ast);
        // log::info!("NODE : {:?}", node);
        let found_links = match node {
            Some(n) => match n {
                Node::Heading(h) => Some(handle_heading(h, state)),
                Node::Definition(d) => Some(handle_definition(req_ast, &req_uri, d)),
                Node::FootnoteDefinition(f) => {
                    Some(handle_footnote_definition(req_ast, &req_uri, f))
                }
                _ => None,
            },
            None => None,
        };

        let result = found_links.map(|found_links| {
            found_links
                .into_iter()
                .map(|found_link| Location {
                    uri: found_link.file_url,
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
}
