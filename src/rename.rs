use lsp_types::{Position, PrepareRenameResponse};
use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, LinkReference, Node,
};

use crate::traverse_ast;

pub fn rename(node: &Node) -> Option<PrepareRenameResponse> {
    match node {
        Node::Heading(Heading { position, .. })
        | Node::LinkReference(LinkReference { position, .. })
        | Node::Definition(Definition { position, .. })
        | Node::FootnoteReference(FootnoteReference { position, .. })
        | Node::FootnoteDefinition(FootnoteDefinition { position, .. }) => {
            log::info!("RENAME NODE: {:?}", node);
            return Some(PrepareRenameResponse::DefaultBehavior {
                default_behavior: true,
            });
        }
        _ => {}
    }

    traverse_ast!(node, rename)
}

pub fn find_renameable_for_position<'a>(node: &'a Node, req_pos: &Position) -> Option<&'a Node> {
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
