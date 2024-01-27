use lsp_types::Range;
use markdown::mdast::{LinkReference, Node};

use crate::{
    ast::find_definition_for_identifier, definition::range_from_position, state::State,
    traverse_ast,
};

pub fn check_links(ast: &Node, state: &State) -> Option<(Range, String)> {
    match ast {
        Node::Link(link) => {},
        Node::LinkReference(link_ref) => {
            return handle_link_ref(&ast, &link_ref);
        }
        Node::FootnoteReference(footnote_ref) => {},
        _ => {},
    };

    traverse_ast!(ast, check_links, state)
}

fn handle_link_ref(ast: &Node, link_ref: &LinkReference) -> Option<(Range, String)> {
    match find_definition_for_identifier(ast, &link_ref.identifier) {
        Some(_) => None,
        None => {
            if let Some(pos) = link_ref.position.as_ref() {
                let range = range_from_position(pos);
                let msg = "Link definition not found".to_string();
                Some((range, msg))
            } else {
                None
            }
        }
    }
}
