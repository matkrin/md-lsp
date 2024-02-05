use lsp_types::{Range, Url};
use markdown::mdast::{Definition, FootnoteDefinition, Heading, Node, Text};

use crate::ast::find_link_references_for_identifier;
use crate::{
    ast::find_footnote_references_for_identifier, definition::range_from_position, state::State,
};

pub struct FoundRef {
    pub file_url: Url,
    pub range: Range,
}

pub fn handle_heading(heading: &Heading, req_uri: &Url, state: &State) -> Vec<FoundRef> {
    let mut heading_refs = Vec::new();
    get_heading_refs(&mut heading_refs, req_uri, heading, state);
    heading_refs
}

pub fn handle_definition(req_ast: &Node, req_uri: &Url, definition: &Definition) -> Vec<FoundRef> {
    let mut link_ranges = Vec::new();
    find_link_references_for_identifier(req_ast, &definition.identifier, &mut link_ranges);
    link_ranges
        .into_iter()
        .map(|lr| FoundRef {
            file_url: req_uri.clone(),
            range: lr,
        })
        .collect()
}

pub fn handle_footnote_definition(
    req_ast: &Node,
    req_uri: &Url,
    fn_definition: &FootnoteDefinition,
) -> Vec<FoundRef> {
    let mut footnote_ranges = Vec::new();
    find_footnote_references_for_identifier(
        req_ast,
        &fn_definition.identifier,
        &mut footnote_ranges,
    );
    footnote_ranges
        .into_iter()
        .map(|fnr| FoundRef {
            file_url: req_uri.clone(),
            range: fnr,
        })
        .collect()
}

pub fn get_heading_refs(
    heading_refs: &mut Vec<FoundRef>,
    req_uri: &Url,
    heading: &Heading,
    state: &State,
) {
    if let Some(Node::Text(Text { value, .. })) = heading.children.first() {
        for (url, md_file) in state.md_files.iter() {
            find_links_in(&md_file.ast, value, req_uri, url, heading_refs)
        }
    }
}

fn find_links_in(
    ast: &Node,
    heading_text: &str,
    req_uri: &Url,
    target_url: &Url,
    heading_refs: &mut Vec<FoundRef>,
) {
    let f_path = req_uri.to_file_path().unwrap();
    let f_name = f_path.file_stem().unwrap();
    if let Some(children) = ast.children() {
        for child in children {
            match child {
                Node::Link(link) => match (&link.position, link.url.split_once('#')) {
                    (Some(pos), Some(("", ht))) => {
                        if heading_text == ht {
                            heading_refs.push(FoundRef {
                                file_url: req_uri.clone(),
                                range: range_from_position(pos),
                            })
                        }
                    },
                    (Some(pos), Some((file_name, ht))) => {
                        if heading_text == ht && f_name == file_name {
                            heading_refs.push(FoundRef {
                                file_url: target_url.clone(),
                                range: range_from_position(pos),
                            })
                        }
                    }
                    _ => (),
                },
                _ => find_links_in(child, heading_text, req_uri, target_url, heading_refs),
            }
        }
    }
}
