use std::path::PathBuf;

use lsp_types::Url;
use markdown::{
    mdast::{Heading, Link, Node, Text},
    unist::{Point, Position},
};
use regex::Regex;

use crate::{
    ast::{find_heading_for_link, find_heading_for_link_identifier},
    state::{relative_path, State},
};

#[derive(Debug)]
pub enum MdLink<'a> {
    NormalLink(&'a Link),
    WikiLink(&'a Link),
}

impl<'a, 'b> MdLink<'a>
where
    'b: 'a,
{
    fn new(link: &'b Link) -> Self {
        match &link.title {
            Some(title) if title == "wikilink" => MdLink::WikiLink(link),
            _ => MdLink::NormalLink(link),
        }
    }
}

#[derive(Debug)]
pub enum ResolvedLink<'a> {
    File {
        link: MdLink<'a>,
        file_uri: &'a Url,
    },
    InternalHeading {
        link: MdLink<'a>,
        file_uri: &'a Url,
        heading: &'a Heading,
    },
    ExternalHeading {
        link: MdLink<'a>,
        file_uri: &'a Url,
        heading: &'a Heading,
    },
    Http,
    Unresolved,
}

// #[derive(Debug)]
// pub struct ResolvedLink<'a> {
//     pub heading: Option<&'a str>,
//     pub uri: Url,
// }
//
// impl<'a> ResolvedLink<'a> {
//     fn new(heading: Option<&'a str>, uri: Url) -> Self {
//         Self { heading, uri }
//     }
//
//     pub fn from_state(
//         linked_file: &'a str,
//         heading_text: Option<&'a str>,
//         state: &State,
//     ) -> Option<ResolvedLink<'a>> {
//         let file = if linked_file.ends_with(".md") {
//             PathBuf::from(linked_file)
//         } else {
//             PathBuf::from(format!("{}.md", linked_file))
//         };
//         for url in state.md_files.keys() {
//             if url.to_file_path().unwrap().file_name().unwrap() == file.file_name().unwrap() {
//                 return Some(ResolvedLink::new(heading_text, url.clone()));
//             }
//         }
//         None
//     }
// }

pub fn resolve_link<'a>(link: &'a Link, state: &'a State) -> ResolvedLink<'a> {
    let md_link = MdLink::new(link);

    if link.url.starts_with("http") {
        return ResolvedLink::Http;
    }

    match link.url.split_once('#') {
        // internal link to heading `#...`
        Some(("", heading_ref_text)) => {
            log::info!("INTERNAL LINK: {:?}", &link);
            for (url, md_file) in state.md_files.iter() {
                if let Some(heading) = find_heading_for_link(&md_file.ast, &link) {
                    return ResolvedLink::InternalHeading {
                        link: md_link,
                        file_uri: &url,
                        heading,
                    };
                }
            }
            ResolvedLink::Unresolved
        }
        // link with referece to heading `...#...`
        Some((file_ref_text, heading_ref_text)) => {
            log::info!("LINK WITH REF TO HEADING: {:?}", &link);
            // allow both: with suffix and without
            let file = if file_ref_text.ends_with(".md") {
                file_ref_text.into()
            } else {
                format!("{}.md", file_ref_text)
            };

            log::info!("LINK WITH REF TO HEADING file: {:?}", &file);
            for (url, relative_path) in state.get_file_list() {
                log::info!("rel path, file: {:?}, {:?}", &relative_path, file);
                if relative_path == file {
                    log::info!("YES");
                    log::info!("URL  : {:?}", &url);
                    // as we get url from state it must be in there
                    let ast = state.ast_for_uri(url).unwrap();
                    if let Some(heading) = find_heading_for_link_identifier(ast, heading_ref_text) {
                        log::info!("BEFORE RETURN: {:?}", &md_link);
                        return ResolvedLink::ExternalHeading {
                            link: md_link,
                            file_uri: url,
                            heading,
                        };
                    }
                    return ResolvedLink::File { link: md_link, file_uri: url }
                }
            }
            log::info!("Unresolved: {:?}", &md_link);
            ResolvedLink::Unresolved
        }
        // link without referece to heading `...`
        None => {
            log::info!("LINK WITHOUT REF TO HEADING: {:?}", &link);
            let file = if link.url.ends_with(".md") {
                link.url.to_string()
            } else {
                format!("{}.md", link.url)
            };
            log::info!("FILE: {:?}", &file);
            for (url, relative_path) in state.get_file_list() {
                if relative_path == file {
                    return ResolvedLink::File {
                        link: md_link,
                        file_uri: url,
                    };
                }
            }
            ResolvedLink::Unresolved
        }
    }
}

struct ExtractedWikiLink {
    content: String,
    start_position: usize,
    line_number: usize,
}

impl ExtractedWikiLink {
    fn link_text_node(&self, start_line: usize) -> Node {
        let value = "".to_string(); // TODO parse Wikilinks with `|` (everything before is value)
        let value_len = value.len();
        let link_text = Text {
            value,
            position: Some(Position {
                start: Point {
                    line: start_line + self.line_number,
                    column: self.start_position,
                    offset: self.start_position,
                },
                end: Point {
                    line: start_line + self.line_number,
                    column: self.start_position + value_len,
                    offset: self.start_position + value_len,
                },
            }),
        };
        Node::Text(link_text)
    }

    fn link_node(&self, start_line: usize) -> Node {
        let link_text_node = self.link_text_node(start_line);

        let link = Link {
            children: vec![link_text_node],
            position: Some(Position {
                start: Point {
                    line: start_line + self.line_number,
                    column: self.start_position - 2,
                    offset: self.start_position - 2,
                },
                end: Point {
                    line: start_line + self.line_number,
                    column: self.start_position + self.content.len() + 2,
                    offset: self.start_position + self.content.len() + 2,
                },
            }),
            url: self.content.clone(),
            title: Some("wikilink".to_string()),
        };
        Node::Link(link)
    }
}

fn extract_wiki_links(input: &str) -> Vec<ExtractedWikiLink> {
    let re = Regex::new(r"\[\[([\s\S]*?)\]\]").unwrap();

    input
        .lines()
        .enumerate()
        .flat_map(|(line_number, line)| {
            re.captures_iter(line).filter_map(move |captures| {
                captures.get(1).map(|content| ExtractedWikiLink {
                    content: content.as_str().to_string(),
                    start_position: content.start() + 1,
                    line_number,
                })
            })
        })
        .collect()
}

pub fn parse_wiki_links(node: &mut Node) {
    let mut links = Vec::new();
    if let Some(children) = node.children() {
        for child in children {
            if let Node::Text(t) = child {
                if t.value.contains("[[") && t.value.contains("]]") {
                    let t_position = t.position.as_ref().unwrap().clone();
                    let extracted = extract_wiki_links(&t.value);

                    for i in extracted {
                        let link_ast = i.link_node(t_position.start.line);
                        links.push(link_ast);
                    }
                }
            }
        }
    }
    if let Some(children) = node.children_mut() {
        children.append(&mut links);
    }
    // recurse through children
    if let Some(children) = node.children_mut() {
        for child in children {
            parse_wiki_links(child);
        }
    }
}
