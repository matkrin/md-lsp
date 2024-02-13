use lsp_types::{Position, PrepareRenameResponse, Range};
use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, LinkReference, Node, Text,
};

use crate::traverse_ast;

pub fn prepare_rename(node: &Node) -> Option<PrepareRenameResponse> {
    match node {
        Node::Heading(_) => {
            log::info!("RENAME NODE: {:?}", node);
            Some(PrepareRenameResponse::DefaultBehavior {
                default_behavior: true,
            })
        }
        Node::LinkReference(LinkReference {
            position, children, ..
        }) => {
            let text = get_link_ref_text(children)?;
            position.as_ref().map(|link_ref_pos| {
                let start_line = link_ref_pos.start.line - 1;
                let start_char = link_ref_pos.start.column + text.value.len() + 2;
                let end_line = link_ref_pos.end.line - 1;
                let end_char = link_ref_pos.end.column - 2;
                let range = rename_range(start_line, end_line, start_char, end_char);
                PrepareRenameResponse::Range(range)
            })
        }
        Node::Definition(Definition {
            position,
            identifier,
            ..
        }) => {
            log::info!("RENAME NODE: {:?}", node);
            position.as_ref().map(|def_pos| {
                let start_line = def_pos.start.line - 1;
                let start_char = def_pos.start.column;
                let end_line = def_pos.end.line - 1;
                let end_char = def_pos.start.column + identifier.len();
                let range = rename_range(start_line, end_line, start_char, end_char);
                PrepareRenameResponse::Range(range)
            })
        }
        Node::FootnoteReference(FootnoteReference {
            position,
            identifier,
            ..
        }) => {
            log::info!("RENAME NODE: {:?}", node);

            position.as_ref().map(|foot_ref_pos| {
                let start_line = foot_ref_pos.start.line - 1;
                let start_char = foot_ref_pos.start.column + 1;
                let end_line = foot_ref_pos.end.line - 1;
                let end_char = foot_ref_pos.start.column + identifier.len() + 1;
                let range = rename_range(start_line, end_line, start_char, end_char);
                PrepareRenameResponse::Range(range)
            })
        }
        Node::FootnoteDefinition(FootnoteDefinition {
            position,
            identifier,
            ..
        }) => {
            log::info!("RENAME NODE: {:?}", node);

            position.as_ref().map(|foot_def_pos| {
                let start_line = foot_def_pos.start.line - 1;
                let start_char = foot_def_pos.start.column + 1;
                let end_line = foot_def_pos.start.line - 1;
                let end_char = foot_def_pos.start.column + identifier.len() + 1;
                let range = rename_range(start_line, end_line, start_char, end_char);
                PrepareRenameResponse::Range(range)
            })
        }
        _ => None,
    }
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
