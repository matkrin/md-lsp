use lsp_types::{Location, Position as LspPosition, ReferenceParams, Url};
use markdown::mdast::{Definition, FootnoteDefinition, Heading, Node};

use crate::ast::{find_definition_for_position, find_link_references_for_identifier, find_links};
use crate::links::{resolve_link, ResolvedLink};
use crate::{
    ast::find_footnote_references_for_identifier, definition::range_from_position, state::State,
};

pub fn references(params: &ReferenceParams, state: &State) -> Option<Vec<Location>> {
    let text_document_params = &params.text_document_position;
    let req_uri = &text_document_params.text_document.uri;
    let LspPosition { line, character } = text_document_params.position;

    let req_ast = state.ast_for_uri(req_uri).unwrap();
    let node = find_definition_for_position(req_ast, line, character);

    match node {
        Some(n) => match n {
            Node::Heading(h) => Some(handle_heading(h, state)),
            Node::Definition(d) => handle_definition(req_ast, req_uri, d),
            Node::FootnoteDefinition(f) => handle_footnote_definition(req_ast, req_uri, f),
            _ => None,
        },
        None => None,
    }
}

fn handle_heading(heading: &Heading, state: &State) -> Vec<Location> {
    get_heading_refs(heading, state)
        .into_iter()
        .filter_map(|(link_uri, resolved_link)| {
            let pos = resolved_link.link_position()?;
            let range = range_from_position(pos);
            Some(Location {
                uri: link_uri.clone(),
                range,
            })
        })
        .collect()
}

fn handle_definition(
    req_ast: &Node,
    req_uri: &Url,
    definition: &Definition,
) -> Option<Vec<Location>> {
    let link_refs = find_link_references_for_identifier(req_ast, &definition.identifier);
    link_refs
        .into_iter()
        .map(|link_ref| {
            link_ref.position.as_ref().map(|pos| Location {
                uri: req_uri.clone(),
                range: range_from_position(pos),
            })
        })
        .collect()
}

fn handle_footnote_definition(
    req_ast: &Node,
    req_uri: &Url,
    fn_definition: &FootnoteDefinition,
) -> Option<Vec<Location>> {
    let footnote_refs = find_footnote_references_for_identifier(req_ast, &fn_definition.identifier);
    footnote_refs
        .into_iter()
        .map(|footnote_ref| {
            footnote_ref.position.as_ref().map(|pos| Location {
                uri: req_uri.clone(),
                range: range_from_position(pos),
            })
        })
        .collect()
}

pub fn get_heading_refs<'a>(
    req_heading: &Heading,
    state: &'a State,
) -> Vec<(&'a Url, ResolvedLink<'a>)> {
    let mut heading_refs = Vec::new();
    for (url, md_file) in state.md_files.iter() {
        let mut resolved_links = find_links(&md_file.ast)
            .into_iter()
            .filter_map(|link| {
                let resolved_link = resolve_link(link, state);
                match resolved_link {
                    ResolvedLink::InternalHeading { heading, .. }
                    | ResolvedLink::ExternalHeading { heading, .. } => {
                        if req_heading == heading {
                            Some((url, resolved_link))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        heading_refs.append(&mut resolved_links);
    }
    heading_refs
}
