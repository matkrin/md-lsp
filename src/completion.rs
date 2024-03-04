use std::path::{Path, PathBuf};

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionParams, Position, Range, Url,
};
use markdown::mdast::{FootnoteDefinition, Heading, Node, Text};

use crate::{
    ast::{find_defintions, find_footnote_definitions, find_headings, find_next_heading, get_heading_text},
    state::State,
};

pub fn completion(params: CompletionParams, state: &State) -> Option<CompletionList> {
    let req_uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    let peek_behind = state.peek_behind_position(&req_uri, &position);
    log::info!("PEEK BEHIND: {:?}", peek_behind);
    let context = params.context?;
    // let trigger_kind = context.trigger_kind;
    let trigger_character = context.trigger_character;
    match trigger_character.as_deref() {
        Some("(") if peek_behind == Some(']') => link_completion(state),
        Some("[") if peek_behind == Some('[') => wikilink_completion(state),
        Some("^") if peek_behind == Some('[') => footnote_ref_completion(&req_uri, state),
        Some("[") => link_ref_completion(&req_uri, state),
        _ => None,
    }
}

fn link_completion( state: &State) -> Option<CompletionList> {
    let root_uri = PathBuf::from(&state.workspace_folder()?.name);
    let completion_items: Option<Vec<CompletionItem>> = state
        .md_files
        .iter()
        .flat_map(|(url, md_file)| {
            let headings = find_headings(&md_file.ast);
            headings.into_iter().map(|heading| {
                let file_path = url.to_file_path().ok()?;
                let relative_path = relative_path(&root_uri, &file_path)?;
                let heading_text = get_heading_text(heading)?;
                let range = link_detail_range(&md_file.ast, heading)?;
                let detail = state.buffer_range_for_uri(url, &range)?;
                Some(CompletionItem {
                    label: format!(
                        "{}#{}",
                        relative_path,
                        heading_text.to_lowercase().replace(' ', "-")
                    ),
                    kind: Some(CompletionItemKind::TEXT),
                    detail: Some(detail),
                    ..CompletionItem::default()
                })
            })
        })
        .collect();
    completion_items.map(|comp_items| CompletionList {
        is_incomplete: true,
        items: comp_items,
    })
}

fn wikilink_completion(state: &State) -> Option<CompletionList> {
    let root_uri = PathBuf::from(&state.workspace_folder()?.name);
    let completion_items: Option<Vec<CompletionItem>> = state
        .md_files
        .iter()
        .flat_map(|(url, md_file)| {
            let headings = find_headings(&md_file.ast);
            headings.into_iter().map(|heading| {
                let file_path = url.to_file_path().ok()?;
                let relative_path = relative_path(&root_uri, &file_path)?;
                let path = relative_path.split_once('.')?.0;
                let heading_text = get_heading_text(heading)?;
                let range = link_detail_range(&md_file.ast, heading)?;
                let detail = state.buffer_range_for_uri(url, &range)?;
                Some(CompletionItem {
                    label: format!("{}#{}", path, heading_text),
                    kind: Some(CompletionItemKind::TEXT),
                    detail: Some(detail),
                    ..CompletionItem::default()
                })
            })
        })
        .collect();
    completion_items.map(|comp_items| CompletionList {
        is_incomplete: true,
        items: comp_items,
    })
}

fn link_detail_range(ast: &Node, heading: &Heading) -> Option<Range> {
    let heading_pos = heading.position.as_ref()?;
    match find_next_heading(ast, heading_pos.end.line, heading.depth) {
        Some(next_heading) => Some(Range {
            start: Position {
                line: (heading_pos.start.line - 1) as u32,
                character: (heading_pos.start.column - 1) as u32,
            },
            end: Position {
                line: next_heading.position.as_ref()?.start.line as u32,
                character: next_heading.position.as_ref()?.end.line as u32,
            },
        }),
        None => Some(Range {
            start: Position {
                line: (heading_pos.start.line - 1) as u32,
                character: (heading_pos.start.column - 1) as u32,
            },
            end: Position {
                line: 999999,
                character: 999999,
            },
        }),
    }
}

fn link_ref_completion(req_uri: &Url, state: &State) -> Option<CompletionList> {
    let ast = state.ast_for_uri(req_uri)?;
    let definitions = find_defintions(ast);
    let def_completion_items = definitions
        .into_iter()
        .map(|def| CompletionItem {
            label: def.identifier.clone(),
            kind: Some(CompletionItemKind::TEXT),
            detail: Some(def.url.clone()),
            ..CompletionItem::default()
        })
        .collect();
    Some(CompletionList {
        is_incomplete: true,
        items: def_completion_items,
    })
}

fn footnote_ref_completion(req_uri: &Url, state: &State) -> Option<CompletionList> {
    let ast = state.ast_for_uri(req_uri)?;
    let footnote_defs = find_footnote_definitions(ast);
    let completion_items: Option<Vec<CompletionItem>> = footnote_defs
        .into_iter()
        .map(|footnote_def| {
            get_footnote_def_text(footnote_def).map(|text| CompletionItem {
                label: footnote_def.identifier.clone(),
                label_details: None,
                kind: Some(CompletionItemKind::TEXT),
                detail: Some(text.value.clone()),
                ..CompletionItem::default()
            })
        })
        .collect();
    completion_items.map(|items| CompletionList {
        is_incomplete: true,
        items,
    })
}

fn get_footnote_def_text(footnote_def: &FootnoteDefinition) -> Option<&Text> {
    for child in &footnote_def.children {
        if let Node::Paragraph(paragraph) = child {
            for text in &paragraph.children {
                if let Node::Text(text) = text {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn relative_path(from: &Path, to: &Path) -> Option<String> {
    if let Ok(rel) = to.strip_prefix(from) {
        Some(rel.to_string_lossy().into_owned())
    } else {
        None
    }
}
