use std::fs::File;

use anyhow::Result;
use lsp_types::request::HoverRequest;
use lsp_types::{
    request::GotoDefinition, GotoDefinitionResponse, InitializeParams, ServerCapabilities,
};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    GotoDefinitionParams, Hover, HoverParams, HoverProviderCapability, MarkupContent, MarkupKind,
    OneOf, Position, TextDocumentSyncCapability, TextDocumentSyncKind,
};

use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};
use markdown::mdast::{self, Node};

fn main() -> Result<()> {
    // Note that  we must have our logging only write out to stderr.

    let log_file = File::options()
        .create(true)
        .append(true)
        .open("./log.log")
        .expect("Couldn't open log file");
    structured_logger::Builder::with_level("TRACE")
        .with_default_writer(structured_logger::json::new_writer(log_file))
        .init();
    log::info!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = match connection.initialize(server_capabilities) {
        Ok(it) => it,
        Err(e) => {
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    log::trace!("shutting down server");
    Ok(())
}

fn main_loop(connection: Connection, params: serde_json::Value) -> Result<()> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    log::info!("starting example main loop");
    let server = Server {
        connection,
        md_buffer: "".to_string(),
    };
    server.run()
}

struct Server {
    connection: Connection,
    md_buffer: String,
}

struct State {
    md_buffer: String,
}

impl Server {
    fn run(&self) -> Result<()> {
        let mut state = State {
            md_buffer: "".to_string(),
        };
        for msg in &self.connection.receiver {
            log::info!("GOT MSG: {msg:?}");
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    log::info!("GOT REQUEST: {req:?}");
                    match req.method.as_ref() {
                        "textDocument/definition" => {
                            self.handle_defintion(req)?;
                        }
                        "textDocument/hover" => self.handle_hover(req, &state)?,
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
                        "textDocument/didOpen" => {
                            self.handle_did_open(not, &mut state)?;
                        }
                        "textDocument/didChange" => {
                            self.handle_did_change(not, &mut state)?;
                        }
                        "textDocument/didClose" => {
                            self.handle_did_close(not)?;
                        }
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
        log::info!("GOT didOpen NOTIFICATION : {:?}", params);
        state.md_buffer = params.text_document.text;
        let ast = markdown::to_mdast(&state.md_buffer, &markdown::ParseOptions::gfm());
        // log::info!("MARKDOWN AST: {:?}", ast);

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_didChange
    fn handle_did_change(&self, not: lsp_server::Notification, state: &mut State) -> Result<()> {
        let params: DidChangeTextDocumentParams = serde_json::from_value(not.params)?;
        log::info!("GOT didChange NOT : {:?}", params);
        // This needs to be changed when partial updating
        let change_event = params.content_changes.iter().last().unwrap();
        state.md_buffer = change_event.text.clone();

        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_didClose
    fn handle_did_close(&self, not: lsp_server::Notification) -> Result<()> {
        let params: DidCloseTextDocumentParams = serde_json::from_value(not.params)?;
        log::info!("GOT didClose NOT : {:?}", params);

        Ok(())
    }

    fn handle_defintion(&self, req: lsp_server::Request) -> Result<()> {
        let params: GotoDefinitionParams = serde_json::from_value(req.params)?;
        log::info!("GOT gotoDefinition REQUEST #{}: {:?}", req.id, params);
        let result = Some(GotoDefinitionResponse::Array(Vec::new()));
        let result = serde_json::to_value(result).unwrap();
        let resp = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };
        self.connection.sender.send(Message::Response(resp))?;
        Ok(())
    }

    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover
    fn handle_hover(&self, req: lsp_server::Request, state: &State) -> Result<()> {
        let params: HoverParams = serde_json::from_value(req.params)?;
        let position_params = params.text_document_position_params;
        let _uri = position_params.text_document.uri;
        let Position { line, character } = position_params.position;
        let ast = markdown::to_mdast(&state.md_buffer, &markdown::ParseOptions::gfm()).unwrap();
        let node = find_node_for_position(&ast, line, character);

        log::info!("AST : {:?}", ast);
        log::info!("POSITION LINE: {}, CHARACTER: {}", line, character);
        log::info!("FOUND NODE : {:?}", node);

        if node.is_none() {
            self.connection.sender.send(Message::Response(Response {
                id: req.id,
                result: None,
                error: None,
            }))?;
            return Ok(());
        }


        let mut msg = String::new();
        if let Some(n) = node {
            match n {
                Node::Link(link) => msg = link.url.clone(),
                // Node::LinkReference => {}
                // Node::FootnoteReference => {}
                _ => msg = "not Link".to_string(),
            }
        }
        let result = Some(Hover {
            contents: lsp_types::HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: msg,
            }),
            range: None,
        });
        let result = serde_json::to_value(result).unwrap();
        let response = Response {
            id: req.id,
            result: Some(result),
            error: None,
        };
        self.connection.sender.send(Message::Response(response))?;

        Ok(())
    }
}

fn find_node_for_position(node: &Node, line: u32, character: u32) -> Option<&Node> {
    if let Some(children) = node.children() {
        for child in children {
            match child {
                Node::Link(mdast::Link { position, .. })
                | Node::LinkReference(mdast::LinkReference { position, .. })
                | Node::FootnoteReference(mdast::FootnoteReference { position, .. }) => {
                    if let Some(pos) = position {
                        if (line + 1) as usize >= pos.start.line
                            && (line + 1) as usize <= pos.end.line
                            && ((character + 1) as usize) >= pos.start.column
                            && ((character + 1) as usize) <= pos.end.column
                        {
                            log::info!("SHOULD RETURN CHILD: {:?}", &child);
                            return Some(child);
                        }
                    }
                }
                _ => {
                    let n = find_node_for_position(child, line, character);
                    if n.is_some() {
                        return n;
                    }
                }
            }
        }
    }
    None
}
