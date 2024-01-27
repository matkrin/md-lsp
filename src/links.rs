use std::path::PathBuf;

use lsp_types::{Url, WorkspaceFolder};
use markdown::{
    mdast::{Link, Node, Text},
    unist::{Point, Position},
};
use regex::Regex;

use crate::state::State;

#[derive(Debug)]
pub struct ResolvedLink<'a> {
    pub heading: Option<&'a str>,
    pub uri: Url,
}

impl<'a> ResolvedLink<'a> {
    fn new(heading: Option<&'a str>, uri: Url) -> Self {
        Self { heading, uri }
    }

    // fn from_request(
    //     root_path: PathBuf,
    //     linked_file: &'a str,
    //     heading_text: Option<&'a str>,
    // ) -> Option<ResolvedLink<'a>> {
    //     let file = if linked_file.ends_with(".md") {
    //         linked_file.to_string()
    //     } else {
    //         format!("{}.md", linked_file)
    //     };
    //     let full_path = root_path.join(file);
    //     Url::from_file_path(full_path)
    //         .ok()
    //         .map(|u| ResolvedLink::new(heading_text, u))
    // }

    pub fn from_state(
        linked_file: &'a str,
        heading_text: Option<&'a str>,
        state: &State,
    ) -> Option<ResolvedLink<'a>> {
        log::info!("LINKED FILE  : {}", linked_file);
        let file = if linked_file.ends_with(".md") {
            PathBuf::from(linked_file)
        } else {
            PathBuf::from(format!("{}.md", linked_file))
        };
        for url in state.md_files.keys() {
            if url.to_file_path().unwrap().file_name().unwrap() == file.file_name().unwrap() {
                return Some(ResolvedLink::new(heading_text, url.clone()));
            }
        }
        None
    }
}

pub fn resolve_link<'a>(link: &'a Link, state: &State) -> Option<ResolvedLink<'a>> {
    match link.url.split_once('#') {
        Some(("", _)) => None,
        // link with referece to heading `...#...`
        Some((file, heading_text)) => ResolvedLink::from_state(file, Some(heading_text), state),
        // link without referece to heading
        None => ResolvedLink::from_state(&link.url, None, state),
    }
}

struct Extracted {
    content: String,
    start_position: usize,
    line_number: usize,
}

impl Extracted {
    fn link_text_node(&self, start_line: usize) -> Node {
        let link_text = Text {
            value: self.content.clone(),
            position: Some(Position {
                start: Point {
                    line: start_line + self.line_number,
                    column: self.start_position,
                    offset: self.start_position,
                },
                end: Point {
                    line: start_line + self.line_number,
                    column: self.start_position + self.content.len(),
                    offset: self.start_position + self.content.len(),
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
            title: Some(self.content.clone()),
        };
        Node::Link(link)
    }
}

fn extract_wiki_links(input: &str) -> Vec<Extracted> {
    let re = Regex::new(r"\[\[([\s\S]*?)\]\]").unwrap();

    input
        .lines()
        .enumerate()
        .flat_map(|(line_number, line)| {
            re.captures_iter(line).filter_map(move |captures| {
                captures.get(1).map(|content| Extracted {
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
