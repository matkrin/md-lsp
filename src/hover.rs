use markdown::mdast::{self, Node};

use crate::ast::find_heading_for_url;

pub struct State {
    pub md_buffer: String,
}

pub fn hov_handle_heading_links(ast: &Node, link: &mdast::Link, state: &State) -> Option<String> {
    let linked_heading = find_heading_for_url(ast, &link.url);

    match linked_heading {
        Some(heading) => {
            let linked_heading_end = heading.position.as_ref().unwrap().end.line;
            let depth = heading.depth;
            let next_heading = find_next_heading(ast, linked_heading_end, depth);
            let start_line = heading.position.as_ref().unwrap().start.line;
            let end_line = next_heading.map(|h| h.position.as_ref().unwrap().start.line);
            let buffer_lines = state.md_buffer.lines().collect::<Vec<_>>();

            let message = if let Some(el) = end_line {
                buffer_lines[(start_line - 1)..(el - 1)]
                    .iter()
                    .map(|x| x.to_string() + "\n")
                    .collect::<String>()
            } else {
                buffer_lines[(start_line - 1)..]
                    .iter()
                    .map(|x| x.to_string() + "\n")
                    .collect::<String>()
            };
            Some(message)
        }
        None => None,
    }
}

pub fn hov_handle_link_reference(ast: &Node, link_ref: &mdast::LinkReference) -> Option<String> {
    let def = find_def_for_link_ref(ast, link_ref);

    def.map(|d| format!("[{}]: {}", d.identifier, d.url))
}

pub fn hov_handle_footnote_reference(
    ast: &Node,
    footnote_ref: &mdast::FootnoteReference,
) -> Option<String> {
    let def_node = find_def_for_footnote_ref(ast, footnote_ref);

    match def_node {
        Some(dn) => {
            let footnote_identifier = get_footnote_identifier(dn);
            let footnote_text = get_footnote_def_text(dn);

            if let (Some(fni), Some(fnt)) = (footnote_identifier, footnote_text) {
                Some(format!("[^{}]: {}", fni, fnt))
            } else {
                None
            }
        }
        None => None,
    }
}

fn find_def_for_link_ref<'a>(
    node: &'a Node,
    link_ref: &mdast::LinkReference,
) -> Option<&'a mdast::Definition> {
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

fn find_def_for_footnote_ref<'a>(
    node: &'a Node,
    foot_ref: &mdast::FootnoteReference,
) -> Option<&'a Node> {
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

fn find_next_heading(node: &Node, end_line: usize, depth: u8) -> Option<&mdast::Heading> {
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
