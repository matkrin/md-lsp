use std::collections::HashMap;

use lsp_types::{Position, PrepareRenameResponse, Range, TextEdit, Url};
use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, LinkReference, Node, Text,
};

use crate::{
    ast::find_definition_for_identifier,
    definition,
    references::{get_heading_refs, FoundLink},
    state::State,
    traverse_ast,
};

pub fn prepare_rename(
    req_uri: &Url,
    req_pos: &Position,
    state: &State,
) -> Option<PrepareRenameResponse> {
    state
        .ast_for_uri(req_uri)
        .and_then(|ast| find_renameable_for_position(ast, req_pos))
        .and_then(|node| prepare_rename_range(node).map(PrepareRenameResponse::Range))
}

pub fn rename(
    new_name: &str,
    req_uri: &Url,
    req_pos: &Position,
    state: &State,
) -> Option<HashMap<Url, Vec<TextEdit>>> {
    let node = state
        .ast_for_uri(req_uri)
        .and_then(|ast| find_renameable_for_position(ast, req_pos));

    if let Some(node) = node {
        match node {
            Node::Heading(heading) => {
                let mut ref_changes = rename_heading_refs(new_name, req_uri, heading, state);
                if let Some(range) = heading_rename_range(heading) {
                    let heading_change = TextEdit {
                        range,
                        new_text: new_name.to_string(),
                    };
                    ref_changes
                        .entry(req_uri.clone())
                        .or_default()
                        .push(heading_change);
                }
                Some(ref_changes)
            }
            Node::LinkReference(link_ref) => {
                let mut def_changes = rename_definition(new_name, req_uri, link_ref, state);
                if let Some(range) = link_ref_rename_range(link_ref) {
                    let link_ref_change = TextEdit {
                        range,
                        new_text: new_name.to_string(),
                    };
                    def_changes
                        .entry(req_uri.clone())
                        .or_default()
                        .push(link_ref_change);
                }
                Some(def_changes)
            }
            Node::Definition(Definition { position, .. }) => None,
            Node::FootnoteReference(FootnoteReference { position, .. }) => None,
            Node::FootnoteDefinition(FootnoteDefinition { position, .. }) => None,
            _ => unreachable!(),
        }
    } else {
        None
    }
}

fn prepare_rename_range(node: &Node) -> Option<Range> {
    match node {
        Node::Heading(heading) => heading_rename_range(heading),
        Node::LinkReference(link_ref) => link_ref_rename_range(link_ref),
        Node::Definition(definition) => definition_rename_range(definition),
        Node::FootnoteReference(footnote_ref) => footnote_ref_rename_range(footnote_ref),
        Node::FootnoteDefinition(footnote_def) => footnote_def_rename_range(footnote_def),
        _ => None,
    }
}

fn heading_rename_range(heading: &Heading) -> Option<Range> {
    let text = get_text_child(&heading.children)?;
    text.position.as_ref().map(|pos| {
        let start_line = pos.start.line - 1;
        let start_char = pos.start.column - 1;
        let end_line = pos.end.line - 1;
        let end_char = pos.end.column;
        rename_range(start_line, end_line, start_char, end_char)
    })
}

fn link_ref_rename_range(link_ref: &LinkReference) -> Option<Range> {
    let text = get_text_child(&link_ref.children)?;
    link_ref.position.as_ref().map(|link_ref_pos| {
        let start_line = link_ref_pos.start.line - 1;
        let start_char = link_ref_pos.start.column + text.value.len() + 2;
        let end_line = link_ref_pos.end.line - 1;
        let end_char = link_ref_pos.end.column - 2;
        rename_range(start_line, end_line, start_char, end_char)
    })
}

fn definition_rename_range(def: &Definition) -> Option<Range> {
    def.position.as_ref().map(|def_pos| {
        let start_line = def_pos.start.line - 1;
        let start_char = def_pos.start.column;
        let end_line = def_pos.end.line - 1;
        let end_char = def_pos.start.column + def.identifier.len();
        rename_range(start_line, end_line, start_char, end_char)
    })
}

fn footnote_ref_rename_range(footnote_ref: &FootnoteReference) -> Option<Range> {
    footnote_ref.position.as_ref().map(|foot_ref_pos| {
        let start_line = foot_ref_pos.start.line - 1;
        let start_char = foot_ref_pos.start.column + 1;
        let end_line = foot_ref_pos.end.line - 1;
        let end_char = foot_ref_pos.start.column + footnote_ref.identifier.len() + 1;
        rename_range(start_line, end_line, start_char, end_char)
    })
}

fn footnote_def_rename_range(footnote_def: &FootnoteDefinition) -> Option<Range> {
    footnote_def.position.as_ref().map(|foot_def_pos| {
        let start_line = foot_def_pos.start.line - 1;
        let start_char = foot_def_pos.start.column + 1;
        let end_line = foot_def_pos.start.line - 1;
        let end_char = foot_def_pos.start.column + footnote_def.identifier.len() + 1;
        rename_range(start_line, end_line, start_char, end_char)
    })
}

fn find_renameable_for_position<'a>(node: &'a Node, req_pos: &Position) -> Option<&'a Node> {
    match node {
        Node::Heading(Heading { position, .. })
        | Node::LinkReference(LinkReference { position, .. })
        | Node::Definition(Definition { position, .. })
        | Node::FootnoteReference(FootnoteReference { position, .. })
        | Node::FootnoteDefinition(FootnoteDefinition { position, .. }) => {
            if let Some(pos) = position {
                if (req_pos.line + 1) as usize >= pos.start.line
                    && (req_pos.line + 1) as usize <= pos.end.line
                    && (req_pos.character + 1) as usize >= pos.start.column
                // && (req_pos.character + 1) as usize <= pos.end.column
                {
                    return Some(node);
                }
            };
        }
        _ => {}
    }

    traverse_ast!(node, find_renameable_for_position, req_pos)
}

fn get_text_child(children: &Vec<Node>) -> Option<&Text> {
    for child in children {
        if let Node::Text(t) = child {
            return Some(t);
        }
    }
    None
}

fn rename_range(start_line: usize, end_line: usize, start_char: usize, end_char: usize) -> Range {
    let start = Position {
        line: start_line as u32,
        character: start_char as u32,
    };
    let end = Position {
        line: end_line as u32,
        character: end_char as u32,
    };
    Range { start, end }
}

/// Renaming of references to headings, these are contained in links
fn rename_heading_refs(
    new_name: &str,
    req_uri: &Url,
    heading: &Heading,
    state: &State,
) -> HashMap<Url, Vec<TextEdit>> {
    get_heading_refs(req_uri, heading, state).into_iter().fold(
        HashMap::new(),
        |mut acc, found_ref| match found_ref {
            FoundLink::InternalHeading {
                link,
                uri,
                heading_text,
            }
            | FoundLink::ExternalHeading {
                link,
                uri,
                heading_text,
            } => {
                if let Some(pos) = &link.position {
                    let start_line = pos.start.line - 1;
                    let mut start_char = pos.end.column - 2 - heading_text.len();
                    let end_line = pos.end.line - 1;
                    let mut end_char = pos.end.column - 2;
                    if let Some("wikilink") = link.title.as_deref() {
                        start_char -= 1;
                        end_char -= 1;
                    }
                    let range = rename_range(start_line, end_line, start_char, end_char);
                    let text_edit = TextEdit {
                        range,
                        new_text: new_name.to_string(),
                    };
                    acc.entry(uri.clone()).or_default().push(text_edit);
                }
                acc
            }
            _ => acc,
        },
    )
}

// Definition { position: Some(123:1-123:34 (1519-1552)), url: "https://google.com", title: None, identifier: "google-link", label: Some("google-link") }
/// There is always one definition, which is in same file as the request, I asume
fn rename_definition(
    new_name: &str,
    req_uri: &Url,
    link_ref: &LinkReference,
    state: &State,
) -> HashMap<Url, Vec<TextEdit>> {
    let mut definition_changes = HashMap::new();
    if let Some(ast) = state.ast_for_uri(req_uri) {
        if let Some(definition) = find_definition_for_identifier(ast, &link_ref.identifier) {
            if let Some(pos) = &definition.position {
                let start_line = pos.start.line - 1;
                let start_char = pos.start.column;
                let end_line = pos.end.line - 1;
                let end_char = pos.start.column + link_ref.identifier.len();
                let range = rename_range(start_line, end_line, start_char, end_char);
                let text_edit = TextEdit {
                    range,
                    new_text: new_name.to_string(),
                };
                definition_changes
                    .entry(req_uri.clone())
                    .or_insert(vec![text_edit]);
            }
        }
    }
    definition_changes
}
