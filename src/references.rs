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
            | FoundLink::InternalHeading { link, uri, .. }
            | FoundLink::ExternalHeading { link, uri, .. } => {
                link.position.as_ref().map(|pos| Self {
                    file_url: uri,
                    range: range_from_position(pos),
                })
            }
        }
    }
}

pub fn handle_heading<'a>(
    heading: &Heading,
    req_uri: &'a Url,
    state: &'a State,
) -> Vec<FoundRef<'a>> {
    get_heading_refs(req_uri, heading, state)
        .into_iter()
        .filter_map(FoundRef::from_found_link)
        .collect()
}

pub fn handle_definition<'a>(
    req_ast: &Node,
    req_uri: &'a Url,
    definition: &Definition,
) -> Option<Vec<FoundRef<'a>>> {
    let link_refs = find_link_references_for_identifier(req_ast, &definition.identifier);
    link_refs
        .into_iter()
        .map(|link_ref| {
            link_ref.position.as_ref().map(|pos| FoundRef {
                file_url: req_uri,
                range: range_from_position(pos),
            })
        })
        .collect()
}

pub fn handle_footnote_definition<'a>(
    req_ast: &Node,
    req_uri: &'a Url,
    fn_definition: &FootnoteDefinition,
) -> Option<Vec<FoundRef<'a>>> {
    let footnote_refs = find_footnote_references_for_identifier(req_ast, &fn_definition.identifier);
    footnote_refs
        .into_iter()
        .map(|footnote_ref| {
            footnote_ref.position.as_ref().map(|pos| FoundRef {
                file_url: req_uri,
                range: range_from_position(pos),
            })
        })
        .collect()
}

pub fn get_heading_refs<'a>(
    req_uri: &'a Url,
    heading: &Heading,
    state: &'a State,
) -> Vec<FoundLink<'a>> {
    let mut heading_refs = Vec::new();
    if let Some(Node::Text(Text { value, .. })) = heading.children.first() {
        for (url, md_file) in state.md_files.iter() {
            heading_refs.append(&mut find_links_in(&md_file.ast, value, req_uri, url));
        }
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

fn find_links_in<'a>(
    node: &'a Node,
    heading_text: &str,
    req_uri: &'a Url,
    target_uri: &'a Url,
) -> Vec<FoundLink<'a>> {
    let file_path = req_uri.to_file_path().unwrap();
    let file_name = file_path.file_stem();
    let mut found_links = Vec::new();
    if let Some(f_name) = file_name {
        match node {
            // Link to a file
            Node::Link(link) if link.url.as_str() == f_name => {
                found_links.push(FoundLink::File {
                    link,
                    uri: target_uri,
                });
            }
            // Link to heading
            Node::Link(link) => match link.url.split_once('#') {
                // Link to internal heading
                Some(("", ht))
                    if heading_text == ht
                        || heading_text.to_lowercase().replace(' ', "-") == ht =>
                {
                    found_links.push(FoundLink::InternalHeading {
                        link,
                        uri: target_uri,
                        heading_text: ht,
                    });
                }
                // Link to heading in other file
                Some((file_name, ht))
                    if (heading_text == ht
                        || heading_text.to_lowercase().replace(' ', "-") == ht)
                        && f_name == file_name =>
                {
                    found_links.push(FoundLink::ExternalHeading {
                        link,
                        uri: target_uri,
                        heading_text: ht,
                    });
                }
                _ => {}
            },
            _ => {
                if let Some(children) = node.children() {
                    for child in children {
                        found_links.extend(find_links_in(child, heading_text, req_uri, target_uri))
                    }
                }
            }
        }
    }
    found_links
}
