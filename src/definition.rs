use lsp_types::{Position, Range};
use markdown::mdast::Node;

use crate::ast::find_heading_for_url;

pub fn def_handle_link_to_heading(ast: &Node, link_url: &str) -> Option<Range> {
    let linked_heading = find_heading_for_url(ast, link_url);
    if let Some(heading) = linked_heading {
        heading.position.as_ref().map(|pos| Range {
            start: Position {
                line: (pos.start.line - 1) as u32,
                character: (pos.start.column - 1) as u32,
            },
            end: Position {
                line: (pos.end.line - 1) as u32,
                character: (pos.end.column - 1) as u32,
            },
        })
    } else {
        None
    }
}
