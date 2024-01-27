use lsp_types::{Position, Range, Url};
use markdown::{mdast::{Node, Link}, unist};

use crate::{ast::{
    find_definition_for_identifier, find_foot_definition_for_identifier, find_heading_for_url,
}, state::State, hover::get_target_uri};

pub fn def_handle_link_to_heading(req_uri: &Url, link: &Link, state: &State) -> (Url, Option<Range>) {

    let (target_uri, heading_text) = get_target_uri(req_uri, link, state);
    match heading_text {
        Some(ht) => (target_uri.clone(), handle_link_heading(&target_uri, ht, state)),
        None => (target_uri, Some(range_zero())),
    }
    // find_heading_for_url(req_ast, link_url).and_then(|heading| {
    //     heading.position.as_ref().map(range_from_position)
    // })
}

fn handle_link_heading(target_uri: &Url, heading_text: &str, state: &State) -> Option<Range> {
    let target_ast = state.ast_for_uri(target_uri)?;
    find_heading_for_url(target_ast, heading_text).and_then(|heading| {
        heading.position.as_ref().map(range_from_position)
    })
}

pub fn def_handle_link_ref(req_ast: &Node, identifier: &str) -> Option<Range> {
    find_definition_for_identifier(req_ast, identifier).and_then(|def| {
        def.position.as_ref().map(range_from_position)
    })
}

pub fn def_handle_link_footnote(req_ast: &Node, identifier: &str) -> Option<Range> {
    find_foot_definition_for_identifier(req_ast, identifier).and_then(|foot_def| {
        foot_def.position.as_ref().map(range_from_position)
    })
}

/// Takes `Position` from `markdown::unist`, `Position` in the returned `Range` from `lsp_types`
fn range_from_position(position: &unist::Position) -> Range {
    Range {
        start: Position {
            line: (position.start.line - 1) as u32,
            character: (position.start.column - 1) as u32,
        },
        end: Position {
            line: (position.end.line - 1) as u32,
            character: (position.end.column - 1) as u32,
        },
    }
}

fn range_zero() -> Range {
    Range {
        start: Position { line: 0, character: 0 },
        end: Position {line: 0, character: 0},
    }
}
