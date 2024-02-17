use std::collections::HashMap;

use lsp_types::{Position, PrepareRenameResponse, Range, TextEdit, Url};
use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, LinkReference, Node, Text,
};

use crate::{references::get_heading_refs, state::State, traverse_ast};

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
                let heading_refs = get_heading_refs(req_uri, heading, state);
                let changes: HashMap<Url, Vec<TextEdit>> =
                    heading_refs
                        .into_iter()
                        .fold(HashMap::new(), |mut acc, found_ref| {
                            let text_edit = TextEdit {
                                range: found_ref.range,
                                new_text: new_name.to_string(),
                            };
                            acc.entry(found_ref.file_url.clone()).or_default().push(text_edit);
                            acc
                        });
                Some(changes)
            }
            Node::LinkReference(LinkReference { position, .. }) => None,
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
        Node::Heading(Heading { children, .. }) => {
            let text = get_text_child(children)?;
            text.position.as_ref().map(|pos| {
                let start_line = pos.start.line - 1;
                let start_char = pos.start.column - 1;
                let end_line = pos.end.line - 1;
                let end_char = pos.end.column;
                rename_range(start_line, end_line, start_char, end_char)
            })
        }
        Node::LinkReference(LinkReference {
            position, children, ..
        }) => {
            let text = get_text_child(children)?;
            position.as_ref().map(|link_ref_pos| {
                let start_line = link_ref_pos.start.line - 1;
                let start_char = link_ref_pos.start.column + text.value.len() + 2;
                let end_line = link_ref_pos.end.line - 1;
                let end_char = link_ref_pos.end.column - 2;
                rename_range(start_line, end_line, start_char, end_char)
            })
        }
        Node::Definition(Definition {
            position,
            identifier,
            ..
        }) => position.as_ref().map(|def_pos| {
            let start_line = def_pos.start.line - 1;
            let start_char = def_pos.start.column;
            let end_line = def_pos.end.line - 1;
            let end_char = def_pos.start.column + identifier.len();
            rename_range(start_line, end_line, start_char, end_char)
        }),
        Node::FootnoteReference(FootnoteReference {
            position,
            identifier,
            ..
        }) => position.as_ref().map(|foot_ref_pos| {
            let start_line = foot_ref_pos.start.line - 1;
            let start_char = foot_ref_pos.start.column + 1;
            let end_line = foot_ref_pos.end.line - 1;
            let end_char = foot_ref_pos.start.column + identifier.len() + 1;
            rename_range(start_line, end_line, start_char, end_char)
        }),
        Node::FootnoteDefinition(FootnoteDefinition {
            position,
            identifier,
            ..
        }) => position.as_ref().map(|foot_def_pos| {
            let start_line = foot_def_pos.start.line - 1;
            let start_char = foot_def_pos.start.column + 1;
            let end_line = foot_def_pos.start.line - 1;
            let end_char = foot_def_pos.start.column + identifier.len() + 1;
            rename_range(start_line, end_line, start_char, end_char)
        }),
        _ => None,
    }
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
