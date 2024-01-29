use lsp_types::{Range, Url};
use markdown::mdast::{Definition, FootnoteDefinition, Heading, Link, Node, Text};

use crate::{
    ast::{find_footnote_references_for_identifier, find_link_references_for_identifier}, definition::range_from_position, hover::get_target_heading_uri, state::State, traverse_ast
};

pub struct FoundRef {
    pub file_url: Url,
    pub range:  Range,
}

pub fn handle_heading(
    heading: & Heading,
    state: & State,
) -> Vec<FoundRef> {
    let mut heading_refs = Vec::new();
    get_heading_refs(&mut heading_refs, heading, state);
    heading_refs
}

pub fn handle_definition(req_ast: & Node, req_uri: & Url, definition: &Definition) -> Vec<FoundRef> {
    let mut link_ranges = Vec::new();
    find_link_references_for_identifier(req_ast, &definition.identifier, &mut link_ranges);
    link_ranges.into_iter().map(|lr| FoundRef {file_url: req_uri.clone(), range: lr}).collect()
}

pub fn handle_footnote_definition(
    req_ast: & Node,
    req_uri: & Url,
    fn_definition: &FootnoteDefinition,
) -> Vec<FoundRef> {
    let mut footnote_ranges = Vec::new();
    find_footnote_references_for_identifier(req_ast, &fn_definition.identifier, &mut footnote_ranges);
    footnote_ranges.into_iter().map(|fnr| FoundRef {file_url: req_uri.clone(), range: fnr}).collect()
}

pub fn get_heading_refs(
    heading_refs: & mut Vec<FoundRef>,
    heading: & Heading,
    state: & State,
) {
    if let Some(Node::Text(Text { value, .. })) = heading.children.first() {
        for (url, md_file) in state.md_files.iter() {
            find_links_in(&md_file.ast, value, url, heading_refs)
        }
    }
}

fn find_links_in(ast: &Node, heading_text: &str, url: & Url, heading_refs: &mut Vec<FoundRef>) {
    if let Some(children) = ast.children() {
        for child in children {
            match child {
                Node::Link(link) => {
                    if let Some((_, ht)) = link.url.split_once('#') {
                        if heading_text == ht {
                            if let Some(pos) = &link.position {
                                heading_refs.push(FoundRef {
                                    file_url: url.clone(),
                                    range: range_from_position(pos),
                                })
                            }
                        }
                    }
                }
                _ => find_links_in(child, heading_text, url, heading_refs),
            }
        }
    }
}
