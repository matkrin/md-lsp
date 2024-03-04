use lsp_types::{Location, Position as LspPosition, ReferenceParams, Url};
use markdown::mdast::{Definition, FootnoteDefinition, Heading, Link, Node};

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

pub fn handle_heading(heading: &Heading, state: &State) -> Vec<Location> {
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

pub fn handle_definition(
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

pub fn handle_footnote_definition(
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
    // req_uri: &'a Url,
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
        // heading_refs.append(&mut find_links_in(
        //     &md_file.ast,
        //     req_heading,
        //     // req_uri,
        //     // url,
        //     state,
        // ));
    }
    heading_refs
}

#[derive(Debug)]
pub enum FoundLink<'a> {
    /// Link to a file
    File { link: &'a Link, uri: &'a Url },
    /// Link to internal heading
    InternalHeading {
        link: &'a Link,
        uri: &'a Url,
        heading_text: &'a str,
    },
    /// Link to external heading
    ExternalHeading {
        link: &'a Link,
        uri: &'a Url,
        heading_text: &'a str,
    },
}

// fn find_links_in<'a>(
//     node: &'a Node,
//     req_heading: &Heading,
//     // req_uri: &Url,
//     // target_uri: &Url,
//     state: &'a State,
// ) -> Vec<&'a ResolvedLink<'a>> {
//     // let file_path = req_uri.to_file_path().unwrap();
//     // let file_name = file_path.file_stem();
//     let mut found_links = Vec::new();
//     // if let Some(f_name) = file_name {
//     match node {
//         Node::Link(link) => {
//             let resolved_link = resolve_link(link, state);
//             match resolved_link {
//                 ResolvedLink::InternalHeading { heading, .. }
//                 | ResolvedLink::ExternalHeading { heading, .. } => {
//                     if req_heading == heading {
//                         found_links.push(&resolved_link);
//                     }
//                 }
//                 _ => {}
//             }
//         }
// // Link to a file
// Node::Link(link) if link.url.as_str() == f_name => {
//     found_links.push(FoundLink::File {
//         link,
//         uri: target_uri,
//     });
// }
// // Link to heading
// Node::Link(link) => match link.url.split_once('#') {
//     // Link to internal heading
//     Some(("", ht))
//         if heading_text == ht
//             || heading_text.to_lowercase().replace(' ', "-") == ht =>
//     {
//         found_links.push(FoundLink::InternalHeading {
//             link,
//             uri: target_uri,
//             heading_text: ht,
//         });
//     }
//     // Link to heading in other file
//     Some((file_name, ht))
//         if (heading_text == ht
//             || heading_text.to_lowercase().replace(' ', "-") == ht)
//             && f_name == file_name =>
//     {
//         found_links.push(FoundLink::ExternalHeading {
//             link,
//             uri: target_uri,
//             heading_text: ht,
//         });
//     }
//     _ => {}
// },
// _ => {
//     if let Some(children) = node.children() {
//         for child in children {
//             found_links.extend(find_links_in(
//                 child,
//                 req_heading,
//                 // req_uri,
//                 // target_uri,
//                 state,
//             ))
//         }
//     }
// }
// }
// found_links
// }
