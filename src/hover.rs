use lsp_types::{
    Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Position, Range, Url,
};
use markdown::mdast::{FootnoteReference, Heading, Link, LinkReference, Node};

use crate::{
    ast::{
        find_def_for_link_ref, find_footnote_def_for_footnote_ref, find_heading_for_link,
        find_link_for_position, find_next_heading,
    },
    definition::range_from_position,
    links::{resolve_link, ResolvedLink},
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

// pub fn get_target_heading_uri<'a>(
//     req_uri: &Url,
//     link: &'a Link,
//     state: &'a State,
// ) -> (Url, Option<&'a str>) {
//     match &state.workspace_folder() {
//         Some(_) => match resolve_link(link, state) {
//             // link to other file
//             Some(rl) => (rl.uri, rl.heading),
//             // link to internal heading
//             None => (req_uri.clone(), Some(&link.url)),
//         },
//         _ => (req_uri.clone(), Some(&link.url)),
//     }
// }

fn handle_link(req_uri: &Url, link: &Link, state: &State) -> Option<String> {
    match resolve_link(link, state) {
        ResolvedLink::File { file_uri, .. } => handle_link_other_file(file_uri, state),
        ResolvedLink::InternalHeading {
            file_uri, heading, ..
        }
        | ResolvedLink::ExternalHeading {
            file_uri, heading, ..
        } => handle_link_heading(file_uri, heading, state),
        _ => None,
    }
}

fn handle_link_heading(target_uri: &Url, heading: &Heading, state: &State) -> Option<String> {
    let target_ast = state.ast_for_uri(target_uri)?;

    log::info!("LINK TO target_uri: {:?}", &target_uri);
    log::info!("LINK TO Heading: {:?}", &heading);
    let linked_heading_pos = heading.position.as_ref()?;
    let depth = heading.depth;
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
        None => {
            let range = Range {
                start: Position {
                    line: (linked_heading_pos.start.line - 1) as u32,
                    character: (linked_heading_pos.start.column - 1) as u32,
                },
                end: Position {
                    line: 99999,
                    character: 99999,
                },
            };
            state.buffer_range_for_uri(target_uri, &range)
        }
    }
}

fn handle_link_other_file(target_uri: &Url, state: &State) -> Option<String> {
    log::info!("LINK TO OTHER FILE FUNC TARGET: {:?}", target_uri);
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
