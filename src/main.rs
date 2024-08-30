use std::{fs::OpenOptions, path::PathBuf, time::SystemTime};

use anyhow::Result;
use clap::{ArgAction, Parser};
use log::LevelFilter;
use lsp_server::Connection;
use lsp_types::{
    CodeActionProviderCapability, HoverProviderCapability, InitializeParams, OneOf, RenameOptions,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};
use md_lsp::{server::Server, state::State};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Log to FILE instead of stderr
    #[clap(short, long, name = "FILE")]
    logfile: Option<PathBuf>,
    /// Increase log message verbosity
    #[clap(short, long, action = ArgAction::Count)]
    verbosity: u8,
    /// No message output to stderr
    #[clap(short, long)]
    quiet: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    init_logging(&args);

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

// Only log to stderr
fn init_logging(args: &Args) {
    let log_level = if !args.quiet {
        match args.verbosity {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    } else {
        LevelFilter::Off
    };

    let logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}]: {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stderr());

    let logger = match &args.logfile {
        Some(log_file) => logger.chain(
            OpenOptions::new()
                //.write(true)
                .create(true)
                .append(true)
                //.truncate(true)
                .open(log_file)
                .expect("Failed to open the log file"),
        ),
        None => logger,
    };

    logger.apply().expect("Failed to initialize logging");
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
