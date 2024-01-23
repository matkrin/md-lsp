use lsp_types::{Position, Range};
use markdown::{mdast::Node, unist};

use crate::ast::{
    find_definition_for_identifier, find_foot_definition_for_identifier, find_heading_for_url,
};

pub fn def_handle_link_to_heading(ast: &Node, link_url: &str) -> Option<Range> {
    find_heading_for_url(ast, link_url).and_then(|heading| {
        heading.position.as_ref().map(range_from_position)
    })
}

pub fn def_handle_link_ref(ast: &Node, identifier: &str) -> Option<Range> {
    find_definition_for_identifier(ast, identifier).and_then(|def| {
        def.position.as_ref().map(range_from_position)
    })
}

pub fn def_handle_link_footnote(ast: &Node, identifier: &str) -> Option<Range> {
    find_foot_definition_for_identifier(ast, identifier).and_then(|foot_def| {
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
