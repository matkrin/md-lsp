use std::fs::File;

use anyhow::Result;
use lsp_server::Connection;
use lsp_types::{
    CodeActionProviderCapability, HoverProviderCapability, InitializeParams, OneOf, RenameOptions,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};
use md_lsp::{server::Server, state::State};

fn main() -> Result<()> {
    // Note that  we must have our logging only write out to stderr.

    let mut log_file_path = std::env::temp_dir();
    log_file_path.push("md-lsp.log");
    let log_file = File::options()
        .create(true)
        .append(true)
        .open(&log_file_path)
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
        references_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        workspace_symbol_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        document_range_formatting_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        })),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![
                "[".to_string(),
                "^".to_string(),
                "(".to_string(),
                "#".to_string(),
                "|".to_string(),
            ]),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
            all_commit_characters: None,
            completion_item: None,
        }),
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
    log::info!("Starting main loop");
    let params: InitializeParams = serde_json::from_value(params)?;
    log::info!("INIT PARAMS: {:#?}", &params);
    let work_space_folders = params.workspace_folders;

    // else is single file mode, I guess
    let mut state = State::new();
    let server = Server::new(connection);

    if let Some(wsf) = work_space_folders {
        state.index_md_files(&wsf);
        state.set_workspace_folder(wsf[0].clone());
        for uri in state.md_files.keys() {
            server.handle_diagnostic(uri, &state)?;
        }
    }

    server.run(state)
}
