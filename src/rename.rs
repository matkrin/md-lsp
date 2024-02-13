use lsp_types::{Position, PrepareRenameResponse, Range};
use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, LinkReference, Node, Text,
};

use crate::traverse_ast;

pub fn rename(node: &Node) -> Option<PrepareRenameResponse> {
    match node {
        Node::Heading(Heading { position, .. }) => {
            log::info!("RENAME NODE: {:?}", node);
            return Some(PrepareRenameResponse::DefaultBehavior {
                default_behavior: true,
            });
        }
        Node::LinkReference(LinkReference {
            position,
            children,
            identifier,
            ..
        }) => {
            let text = get_link_ref_text(children)?;
            if let (Some(link_ref_pos), Some(text_pos)) = (position, &text.position) {
                let len_text_value = text.value.len();
                let pos_start = Position {
                    line: (link_ref_pos.start.line - 1) as u32,
                    character: (link_ref_pos.start.column + len_text_value + 2) as u32,
                };
                let pos_end = Position {
                    line: (link_ref_pos.end.line - 1) as u32,
                    character: (link_ref_pos.end.column - 2) as u32,
                };
                return Some(PrepareRenameResponse::Range(Range {
                    start: pos_start,
                    end: pos_end,
                }));
            }
        }
        Node::Definition(Definition {
            position,
            identifier,
            ..
        }) => {
            log::info!("RENAME NODE: {:?}", node);
            if let Some(def_pos) = position {
                let pos_start = Position {
                    line: (def_pos.start.line - 1) as u32,
                    character: (def_pos.start.column) as u32,
                };
                let pos_end = Position {
                    line: (def_pos.end.line - 1) as u32,
                    character: (def_pos.start.column + identifier.len()) as u32,
                };
                return Some(PrepareRenameResponse::Range(Range {
                    start: pos_start,
                    end: pos_end,
                }));
            }
        }
        Node::FootnoteReference(FootnoteReference { position, identifier, .. }) => {
            log::info!("RENAME NODE: {:?}", node);

            if let Some(foot_ref_pos) = position {
                let pos_start = Position {
                    line: (foot_ref_pos.start.line - 1) as u32,
                    character: (foot_ref_pos.start.column + 1) as u32,
                };
                let pos_end = Position {
                    line: (foot_ref_pos.end.line - 1) as u32,
                    character: (foot_ref_pos.start.column + identifier.len() + 1) as u32,
                };
                return Some(PrepareRenameResponse::Range(Range {
                    start: pos_start,
                    end: pos_end,
                }));
            }
            return Some(PrepareRenameResponse::DefaultBehavior {
                default_behavior: true,
            });
        }
        Node::FootnoteDefinition(FootnoteDefinition { position, .. }) => {
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

fn get_link_ref_text(children: &Vec<Node>) -> Option<&Text> {
    for child in children {
        if let Node::Text(t) = child {
            return Some(t);
        }
    }
    None
}
