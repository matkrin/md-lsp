use lsp_types::{
    Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Position, Range, Url,
};
use markdown::mdast::{FootnoteReference, Link, LinkReference, Node};

use crate::{
    ast::{
        find_def_for_link_ref, find_footnote_def_for_footnote_ref, find_heading_for_url,
        find_link_for_position, find_next_heading,
    },
    definition::range_from_position,
    links::resolve_link,
    state::State,
};

pub fn hover(params: &HoverParams, state: &State) -> Option<Hover> {
    let position_params = &params.text_document_position_params;
    let req_uri = &position_params.text_document.uri;
    let Position { line, character } = position_params.position;

    let req_ast = state.ast_for_uri(req_uri)?;
    let node = find_link_for_position(req_ast, line, character)?;
    log::info!("HOVERRRRRR NODE : {:#?}", node);

    let message = match node {
        Node::Link(link) => handle_link(req_uri, link, state),
        Node::LinkReference(link_ref) => handle_link_reference(req_uri, link_ref, state),
        Node::FootnoteReference(foot_ref) => handle_footnote_reference(req_uri, foot_ref, state),
        _ => None,
    };
    message.map(|msg| Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: msg,
        }),
        range: None,
    })
}

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

fn handle_link(req_uri: &Url, link: &Link, state: &State) -> Option<String> {
    let (target_uri, heading_text) = get_target_heading_uri(req_uri, link, state);
    match heading_text {
        Some(ht) => handle_link_heading(&target_uri, ht, state),
        None => handle_link_other_file(&target_uri, state),
    }
}

fn handle_link_heading(target_uri: &Url, heading_text: &str, state: &State) -> Option<String> {
    let target_ast = state.ast_for_uri(target_uri)?;

    let linked_heading = find_heading_for_url(target_ast, heading_text)?;
    let linked_heading_pos = linked_heading.position.as_ref()?;
    let depth = linked_heading.depth;
    match find_next_heading(target_ast, linked_heading_pos.end.line, depth) {
        Some(next_heading) => {
            let next_heading_pos = next_heading.position.as_ref()?;
            let range = Range {
                start: Position {
                    line: (linked_heading_pos.start.line - 1) as u32,
                    character: (linked_heading_pos.start.column - 1) as u32,
                },
                end: Position {
                    line: (next_heading_pos.start.line - 1) as u32,
                    character: (next_heading_pos.end.column - 1) as u32,
                },
            };
            state.buffer_range_for_uri(target_uri, &range)
        }
        None => state.buffer_for_uri(target_uri).map(ToString::to_string),
    }
}

fn handle_link_other_file(target_uri: &Url, state: &State) -> Option<String> {
    state.buffer_for_uri(target_uri).map(ToString::to_string)
}

fn handle_link_reference(req_uri: &Url, link_ref: &LinkReference, state: &State) -> Option<String> {
    let ast = state.ast_for_uri(req_uri)?;
    let def = find_def_for_link_ref(ast, link_ref)?;
    def.position.as_ref().and_then(|pos| {
        let range = range_from_position(pos);
        state.buffer_range_for_uri(req_uri, &range)
    })
}

fn handle_footnote_reference(
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
