use markdown::mdast::{Definition, FootnoteReference, Heading, Link, LinkReference, Node};

use crate::ast::find_heading_for_url;

pub struct State {
    pub md_buffer: String,
}

pub fn hov_handle_heading_links(ast: &Node, link: &Link, state: &State) -> Option<String> {
    let linked_heading = find_heading_for_url(ast, &link.url)?;
    linked_heading.position.as_ref().map(|pos| {
        let next_heading = find_next_heading(ast, pos.end.line, linked_heading.depth);
        let start_line = pos.start.line;
        let end_line = next_heading.and_then(|h| h.position.as_ref().map(|p| p.start.line));
        let buffer_lines = state.md_buffer.lines().collect::<Vec<_>>();

        let message = if let Some(el) = end_line {
            buffer_lines[(start_line - 1)..(el - 1)].iter()
        } else {
            buffer_lines[(start_line - 1)..].iter()
        };

        message.map(|x| x.to_string() + "\n").collect::<String>()
    })
}

pub fn hov_handle_link_reference(ast: &Node, link_ref: &LinkReference) -> Option<String> {
    let def = find_def_for_link_ref(ast, link_ref);

    def.map(|d| format!("[{}]: {}", d.identifier, d.url))
}

pub fn hov_handle_footnote_reference(
    ast: &Node,
    footnote_ref: &FootnoteReference,
) -> Option<String> {
    let def_node = find_def_for_footnote_ref(ast, footnote_ref)?;
    let footnote_identifier = get_footnote_identifier(def_node)?;
    let footnote_text = get_footnote_def_text(def_node)?;
    Some(format!("[^{}]: {}", footnote_identifier, footnote_text))
}

fn find_def_for_link_ref<'a>(node: &'a Node, link_ref: &LinkReference) -> Option<&'a Definition> {
    if let Node::Definition(def) = node {
        if link_ref.identifier == def.identifier {
            return Some(def);
        }
    }

    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = find_def_for_link_ref(child, link_ref) {
                return Some(n);
            }
        }
    }
    None
}

fn find_def_for_footnote_ref<'a>(node: &'a Node, foot_ref: &FootnoteReference) -> Option<&'a Node> {
    if let Node::FootnoteDefinition(def) = node {
        if foot_ref.identifier == def.identifier {
            return Some(node);
        }
    }

    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = find_def_for_footnote_ref(child, foot_ref) {
                return Some(n);
            }
        }
    }
    None
}

fn get_footnote_identifier(node: &Node) -> Option<String> {
    if let Node::FootnoteDefinition(def) = node {
        return Some(def.identifier.clone());
    }
    None
}

fn get_footnote_def_text(node: &Node) -> Option<String> {
    if let Node::Text(t) = node {
        return Some(t.value.clone());
    }
    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = get_footnote_def_text(child) {
                return Some(n);
            }
        }
    }
    None
}

fn find_next_heading(node: &Node, end_line: usize, depth: u8) -> Option<&Heading> {
    if let Node::Heading(heading) = node {
        if let Some(pos) = &heading.position {
            if end_line < pos.start.line && depth == heading.depth {
                return Some(heading);
            }
        }
    }

    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = find_next_heading(child, end_line, depth) {
                return Some(n);
            }
        }
    }
    None
}
