use lsp_types::{Position, Range};
use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, Link, LinkReference, Node, Text,
};

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
    log::info!("NODE: {:?}", node);
    match node {
        Node::Link(Link { position, .. })
        | Node::LinkReference(LinkReference { position, .. })
        | Node::FootnoteReference(FootnoteReference { position, .. }) => {
            if let Some(pos) = position {
                if (line + 1) as usize >= pos.start.line
                    && (line + 1) as usize <= pos.end.line
                    && (character + 1) as usize >= pos.start.column
                    && (character + 1) as usize <= pos.end.column
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
            if let Some(pos) = position {
                if (line + 1) as usize >= pos.start.line
                    && (line + 1) as usize <= pos.end.line
                    && (character + 1) as usize >= pos.start.column
                // && (character + 1) as usize <= pos.end.column
                {
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

pub fn find_link_references_for_identifier<'a>(
    node: &'a Node,
    identifier: &str,
) -> Vec<&'a LinkReference> {
    let mut link_refs = Vec::new();
    match node {
        Node::LinkReference(lref) => {
            if lref.identifier == identifier {
                link_refs.push(lref)
            }
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    link_refs.extend(find_link_references_for_identifier(child, identifier))
                }
            }
        }
    }
    link_refs
}

pub fn find_footnote_references_for_identifier<'a>(
    node: &'a Node,
    identifier: &str,
) -> Vec<&'a FootnoteReference>{
    let mut footnote_refs = Vec::new();
    match node {
         Node::FootnoteReference(fn_ref) => {
            if fn_ref.identifier == identifier {
                footnote_refs.push(fn_ref)
            }
        },
        _ => {
                if let Some(children) = node.children() {
                    for child in children {
                        footnote_refs.extend(find_footnote_references_for_identifier(child, identifier))
                    }
                }
            }
        }
    footnote_refs
}

pub fn find_headings<'a>(node: &'a Node, headings: &mut Vec<&'a Heading>) {
    if let Some(children) = node.children() {
        for child in children {
            if let Node::Heading(heading) = child {
                headings.push(heading)
            } else {
                find_headings(child, headings)
            }
        }
    }
}

pub fn get_heading_text(heading: &Heading) -> Option<&str> {
    for child in &heading.children {
        if let Node::Text(Text { value, .. }) = child {
            return Some(value);
        };
    }
    None
}
