use lsp_types::{
    GotoDefinitionParams, GotoDefinitionResponse, Location, Position as LspPosition, Range, Url,
};
use markdown::{
    mdast::{FootnoteReference, Link, LinkReference, Node},
    unist,
};

use crate::{
    ast::{find_definition_for_identifier, find_foot_definition_for_identifier, TraverseNode},
    links::{resolve_link, ResolvedLink},
    state::State,
};

pub fn definition(params: &GotoDefinitionParams, state: &State) -> Option<GotoDefinitionResponse> {
    let position_params = &params.text_document_position_params;
    let req_uri = &position_params.text_document.uri;
    let LspPosition { line, character } = position_params.position;
    let req_ast = state.ast_for_uri(req_uri).unwrap();
    let node = req_ast.find_linkable_for_position(line, character);

    let location = match node? {
        Node::Link(link) => handle_link_to_heading(link, state),
        Node::LinkReference(link_ref) => handle_link_ref(req_uri, link_ref, state),
        Node::FootnoteReference(foot_ref) => handle_link_footnote(req_uri, foot_ref, state),
        _ => None,
    };
    location.map(GotoDefinitionResponse::Scalar)
}

fn handle_link_to_heading(link: &Link, state: &State) -> Option<Location> {
    match resolve_link(link, state) {
        ResolvedLink::File { file_uri, .. } => Some(Location {
            uri: file_uri.clone(),
            range: range_zero(),
        }),
        ResolvedLink::InternalHeading {
            file_uri, heading, ..
        }
        | ResolvedLink::ExternalHeading {
            file_uri, heading, ..
        } => Some(Location {
            uri: file_uri.clone(),
            range: range_from_position(heading.position.as_ref()?),
        }),
        _ => None,
    }
}

fn handle_link_ref(req_uri: &Url, link_ref: &LinkReference, state: &State) -> Option<Location> {
    let req_ast = state.ast_for_uri(req_uri).unwrap();
    find_definition_for_identifier(req_ast, &link_ref.identifier).and_then(|def| {
        def.position.as_ref().map(|pos| Location {
            uri: req_uri.clone(),
            range: range_from_position(pos),
        })
    })
}

fn handle_link_footnote(
    req_uri: &Url,
    foot_ref: &FootnoteReference,
    state: &State,
) -> Option<Location> {
    let req_ast = state.ast_for_uri(req_uri).unwrap();
    find_foot_definition_for_identifier(req_ast, &foot_ref.identifier).and_then(|foot_def| {
        foot_def.position.as_ref().map(|pos| Location {
            uri: req_uri.clone(),
            range: range_from_position(pos),
        })
    })
}

/// Takes markdown::unist::Position`, returned `Position in `Range` from `lsp_types::Position`
pub fn range_from_position(position: &unist::Position) -> Range {
    Range {
        start: LspPosition {
            line: (position.start.line - 1) as u32,
            character: (position.start.column - 1) as u32,
        },
        end: LspPosition {
            line: (position.end.line - 1) as u32,
            character: (position.end.column - 1) as u32,
        },
    }
}

fn range_zero() -> Range {
    Range {
        start: LspPosition {
            line: 0,
            character: 0,
        },
        end: LspPosition {
            line: 0,
            character: 0,
        },
    }
}
