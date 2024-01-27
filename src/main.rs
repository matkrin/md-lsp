use std::fs::File;

use anyhow::Result;
use lsp_server::Connection;
use lsp_types::{ HoverProviderCapability, InitializeParams,  ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, OneOf, };
use md_lsp::{state::State, server::Server};


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
    let work_space_folders = _params.workspace_folders;
    log::info!("starting example main loop");

    // else is single file mode, I guess
    let mut state = State::new();

    if let Some(wsf) = work_space_folders {
        state.index_md_files(&wsf);
        state.set_workspace_folder(wsf[0].clone());
    }

    let server = Server { connection };
    server.run(state)
}
