use lsp_types::{Range, Url};
use markdown::mdast::{Definition, FootnoteDefinition, Heading, Link, Node, Text};

use crate::ast::find_link_references_for_identifier;
use crate::{
    ast::find_footnote_references_for_identifier, definition::range_from_position, state::State,
};

#[derive(Debug)]
pub struct FoundRef<'a> {
    pub file_url: &'a Url,
    pub range: Range,
}

impl<'a, 'b> FoundRef<'a>
where
    'b: 'a,
{
    fn from_found_link(found_link: FoundLink<'b>) -> Option<Self> {
        match found_link {
            FoundLink::File { link, uri }
            | FoundLink::InternalHeading { link, uri }
            | FoundLink::ExternalHeading { link, uri } => link.position.as_ref().map(|pos| Self {
                file_url: uri,
                range: range_from_position(pos),
            }),
        }
    }
}

pub fn handle_heading<'a>(
    heading: &Heading,
    req_uri: &'a Url,
    state: &'a State,
) -> Vec<FoundRef<'a>> {
    get_heading_refs(req_uri, heading, state)
}

pub fn handle_definition<'a>(
    req_ast: &Node,
    req_uri: &'a Url,
    definition: &Definition,
) -> Vec<FoundRef<'a>> {
    let mut link_ranges = Vec::new();
    find_link_references_for_identifier(req_ast, &definition.identifier, &mut link_ranges);
    link_ranges
        .into_iter()
        .map(|lr| FoundRef {
            file_url: req_uri,
            range: lr,
        })
        .collect()
}

pub fn handle_footnote_definition<'a>(
    req_ast: &Node,
    req_uri: &'a Url,
    fn_definition: &FootnoteDefinition,
) -> Vec<FoundRef<'a>> {
    let mut footnote_ranges = Vec::new();
    find_footnote_references_for_identifier(
        req_ast,
        &fn_definition.identifier,
        &mut footnote_ranges,
    );
    footnote_ranges
        .into_iter()
        .map(|fnr| FoundRef {
            file_url: req_uri,
            range: fnr,
        })
        .collect()
}

pub fn get_heading_refs<'a>(
    req_uri: &'a Url,
    heading: &Heading,
    state: &'a State,
) -> Vec<FoundRef<'a>> {
    let mut heading_refs = Vec::new();
    if let Some(Node::Text(Text { value, .. })) = heading.children.first() {
        for (url, md_file) in state.md_files.iter() {
            let refs = find_links_in(&md_file.ast, value, req_uri, url);
            log::info!("REFS: {:?}", refs);
            let refs = refs.into_iter().filter_map(FoundRef::from_found_link);
            heading_refs.extend(refs);
        }
    }
    log::info!("HEADING REFS: {:?}", heading_refs);
    heading_refs
}

#[derive(Debug)]
enum FoundLink<'a> {
    File { link: &'a Link, uri: &'a Url },
    InternalHeading { link: &'a Link, uri: &'a Url },
    ExternalHeading { link: &'a Link, uri: &'a Url },
}

fn find_links_in<'a>(node: &'a Node, heading_text: &str, req_uri: &'a Url, target_uri: &'a Url) -> Vec<FoundLink<'a>> {
    let file_path = req_uri.to_file_path().unwrap();
    let file_name = file_path.file_stem();
    let mut heading_refs = Vec::new();
    if let Some(f_name) = file_name {
        match node {
            // Link to a file
            Node::Link(link) if link.url.as_str() == f_name => {
                log::info!("LINK TO FILE {:?}", link);
                heading_refs.push(FoundLink::File { link, uri: target_uri });
            }
            // Link to heading
            Node::Link(link) => match link.url.split_once('#') {
                // Link to internal heading
                Some(("", ht)) if heading_text == ht => {
                    log::info!("LINK TO INTERNAL HEADING {:?}", link);
                    log::info!("FILE NAME: {:?}", file_name);
                    heading_refs.push(FoundLink::InternalHeading { link, uri: target_uri });
                }
                // Link to heading in other file
                Some((file_name, ht)) => {
                    log::info!("HT : {:?}", ht);
                    log::info!("HEADING TEXT : {:?}", heading_text);
                    log::info!("FILE NAME : {:?}", file_name);
                    log::info!("F NAME : {:?}", f_name);
                    log::info!("---------------------");
                    if heading_text == ht && f_name == file_name {
                        {
                            log::info!("LINK TO EXTERNAL HEADING {:?}", link);
                            heading_refs.push(FoundLink::ExternalHeading { link, uri: target_uri });
                        }
                    }
                }
                _ => {}
            },
            _ => {
                if let Some(children) = node.children() {
                    for child in children {
                        heading_refs.extend(find_links_in(child, heading_text, req_uri, target_uri))
                    }
                }
            }
        }
    }
    heading_refs
}
