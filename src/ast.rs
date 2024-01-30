use lsp_types::{Position, Range};
use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, Link, LinkReference, Node, Text,
};
use regex::Regex;

use crate::definition::range_from_position;

/// Recursive AST traversal
#[macro_export]
macro_rules! traverse_ast {
    ($node: expr, $func: expr $(, $args: expr)*) => {
        if let Some(children) = $node.children() {
            for child in children {
                if let Some(result) = $func(child, $($args),*) {
                    return Some(result);
                }
            }
            None
        } else {
            None
        }
    };
}

pub fn find_link_for_position(node: &Node, line: u32, character: u32) -> Option<&Node> {
    match node {
        Node::Link(Link { position, .. })
        | Node::LinkReference(LinkReference { position, .. })
        | Node::FootnoteReference(FootnoteReference { position, .. }) => {
            if let Some(pos) = position {
                if (line + 1) as usize >= pos.start.line
                    && (line + 1) as usize <= pos.end.line
                    && ((character + 1) as usize) >= pos.start.column
                    && ((character + 1) as usize) <= pos.end.column
                {
                    return Some(node);
                }
            }
        }
        _ => {}
    };

    traverse_ast!(node, find_link_for_position, line, character)
}

pub fn find_definition_for_position(node: &Node, line: u32, character: u32) -> Option<&Node> {
    match node {
        Node::Heading(Heading { position, .. })
        | Node::Definition(Definition { position, .. })
        | Node::FootnoteDefinition(FootnoteDefinition { position, .. }) => {
                log::info!("FIND before: {:?}", node);
            if let Some(pos) = position {
                    log::info!("POS: {:?}", pos );
                    log::info!("LINE: {:?}", line );
                    log::info!("CHARACTER: {:?}", character );
                if (line + 1) as usize >= pos.start.line
                    && (line + 1) as usize <= pos.end.line
                    && ((character + 1) as usize) >= pos.start.column
                    // && ((character + 1) as usize) <= pos.end.column
                {
                    log::info!("FIND: {:?}", node);
                    return Some(node);
                }
            }
        }
        _ => {}
    };

    traverse_ast!(node, find_definition_for_position, line, character)
}

pub fn find_heading_for_url<'a>(node: &'a Node, link_url: &str) -> Option<&'a Heading> {
    if let Node::Heading(heading) = node {
        if let Some(Node::Text(Text { value, .. })) = heading.children.first() {
            if value == &link_url.replace('#', "") {
                return Some(heading);
            }
        }
    };

    traverse_ast!(node, find_heading_for_url, link_url)
}

pub fn find_definition_for_identifier<'a>(
    node: &'a Node,
    identifier: &str,
) -> Option<&'a Definition> {
    if let Node::Definition(def) = node {
        if def.identifier == identifier {
            return Some(def);
        }
    }

    traverse_ast!(node, find_definition_for_identifier, identifier)
}

pub fn find_foot_definition_for_identifier<'a>(
    node: &'a Node,
    identifier: &str,
) -> Option<&'a FootnoteDefinition> {
    if let Node::FootnoteDefinition(def) = node {
        if def.identifier == identifier {
            return Some(def);
        }
    }

    traverse_ast!(node, find_foot_definition_for_identifier, identifier)
}

pub fn find_link_references_for_identifier(
    node: &Node,
    identifier: &str,
    link_refs: &mut Vec<Range>,
) {
    if let Some(children) = node.children() {
        for child in children {
            if let Node::LinkReference(lref) = child {
                if lref.identifier == identifier {
                    if let Some(pos) = &lref.position {
                        link_refs.push(range_from_position(pos))
                    }
                }
            } else {
                find_link_references_for_identifier(child, identifier, link_refs)
            }
        }
    };
}

pub fn find_footnote_references_for_identifier(
    node: &Node,
    identifier: &str,
    footnote_refs: &mut Vec<Range>,
) {
    // log::info!("FFFFFFF");
    log::info!("FOOTNOTE IDENTIFIER Before: {:?}", identifier);
    // log::info!("FOOTNOTE IDENTIFIER : {:?}", fn_ref.identifier);
    if let Some(children) = node.children() {
        for child in children {
            if let Node::FootnoteReference(fn_ref) = child {

                log::info!("FOOTNOTE IDENTIFIER : {:?}", identifier);
                log::info!("FOOTNOTE IDENTIFIER : {:?}", fn_ref.identifier);
                if fn_ref.identifier == identifier {
                    if let Some(pos) = &fn_ref.position {
                        footnote_refs.push(range_from_position(pos))
                    }
                }
            } else {
                find_footnote_references_for_identifier(child, identifier, footnote_refs)
            }
        }
    };
}
