use lsp_types::Url;
use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, Link, LinkReference, Node,
};

use crate::{
    ast::find_heading_for_url, definition::range_from_position, links::resolve_link, state::State,
    traverse_ast,
};

pub fn get_target_heading_uri<'a>(
    req_uri: &Url,
    link: &'a Link,
    state: &'a State,
) -> (Url, Option<&'a str>) {
    match &state.workspace_folder() {
        Some(_) => match resolve_link(link, state) {
            Some(rl) => (rl.uri, rl.heading),
            None => (req_uri.clone(), Some(&link.url)),
        },
        _ => (req_uri.clone(), Some(&link.url)),
    }
}

pub fn hov_handle_link(req_uri: &Url, link: &Link, state: &State) -> Option<String> {
    let (target_uri, heading_text) = get_target_heading_uri(req_uri, link, state);
    match heading_text {
        Some(ht) => handle_link_heading(&target_uri, ht, state),
        None => handle_link_other_file(&target_uri, state),
    }
}

fn handle_link_heading(target_uri: &Url, heading_text: &str, state: &State) -> Option<String> {
    let target_ast = state.ast_for_uri(target_uri)?;
    let target_buffer = state.buffer_for_uri(target_uri)?;

    let linked_heading = find_heading_for_url(target_ast, heading_text)?;
    linked_heading.position.as_ref().map(|pos| {
        let next_heading = find_next_heading(target_ast, pos.end.line, linked_heading.depth);
        let start_line = pos.start.line;
        let end_line = next_heading.and_then(|h| h.position.as_ref().map(|p| p.start.line));
        let buffer_lines = target_buffer.lines().collect::<Vec<_>>();

        let message = if let Some(el) = end_line {
            buffer_lines[(start_line - 1)..(el - 1)].iter()
        } else {
            buffer_lines[(start_line - 1)..].iter()
        };

        message.map(|x| x.to_string() + "\n").collect::<String>()
    })
}

fn handle_link_other_file(target_uri: &Url, state: &State) -> Option<String> {
    state.buffer_for_uri(target_uri).map(ToString::to_string)
}

pub fn hov_handle_link_reference(
    req_uri: &Url,
    link_ref: &LinkReference,
    state: &State,
) -> Option<String> {
    let ast = state.ast_for_uri(req_uri)?;
    let def = find_def_for_link_ref(ast, link_ref)?;
    def.position.as_ref().and_then(|pos| {
        let range = range_from_position(pos);
        state.buffer_range_for_uri(req_uri, &range)
    })
}

pub fn hov_handle_footnote_reference(
    req_uri: &Url,
    footnote_ref: &FootnoteReference,
    state: &State,
) -> Option<String> {
    let ast = state.ast_for_uri(req_uri)?;
    let footnote_def_node = find_footnote_def_for_footnote_ref(ast, footnote_ref)?;
    footnote_def_node.position.as_ref().and_then(|pos| {
        let range = range_from_position(pos);
        state.buffer_range_for_uri(req_uri, &range)
    })
}

fn find_def_for_link_ref<'a>(node: &'a Node, link_ref: &LinkReference) -> Option<&'a Definition> {
    if let Node::Definition(def) = node {
        if link_ref.identifier == def.identifier {
            return Some(def);
        }
    }

    traverse_ast!(node, find_def_for_link_ref, link_ref)
}

fn find_footnote_def_for_footnote_ref<'a>(
    node: &'a Node,
    foot_ref: &FootnoteReference,
) -> Option<&'a FootnoteDefinition> {
    if let Node::FootnoteDefinition(def) = node {
        if foot_ref.identifier == def.identifier {
            return Some(def);
        }
    }

    traverse_ast!(node, find_footnote_def_for_footnote_ref, foot_ref)
}

pub fn find_next_heading(node: &Node, end_line: usize, depth: u8) -> Option<&Heading> {
    if let Node::Heading(heading) = node {
        if let Some(pos) = &heading.position {
            if end_line < pos.start.line && depth == heading.depth {
                return Some(heading);
            }
        }
    }

    traverse_ast!(node, find_next_heading, end_line, depth)
}
