use std::hash::Hash;

use itertools::Itertools;
use lsp_types::{Position, Range, Url};
use markdown::mdast::{FootnoteReference, Link, LinkReference, Node};
use regex::Regex;

use crate::{
    ast::{find_definition_for_identifier, find_foot_definition_for_identifier},
    definition::range_from_position,
    links::resolve_link,
    state::State,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenLink {
    pub range: Range,
    pub message: String,
}

impl Hash for BrokenLink {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.range.start.line.hash(state);
        self.range.start.character.hash(state);
        self.range.end.line.hash(state);
        self.range.end.character.hash(state);
        self.message.hash(state);
    }
}

pub fn check_links(ast: &Node, req_uri: &Url, state: &State) -> Vec<BrokenLink> {
    let mut v = Vec::new();

    if let Some(children) = ast.children() {
        for child in children {
            match child {
                Node::Link(link)
                    if !link.url.starts_with('#') && !link.url.starts_with("https") =>
                {
                    v.extend(handle_broken_link(state, link))
                }
                // LinkRef
                Node::Text(t) if t.value.contains("][") => {
                    v.extend(handle_broken_ref(req_uri, state))
                }
                // FootnoteRef
                Node::Text(t) if t.value.contains("[^") => {
                    v.extend(handle_broken_footnote_ref(req_uri, state))
                }
                _ => {
                    v.append(&mut check_links(child, req_uri, state));
                }
            }
        }
    }
    v.into_iter().unique().collect()
}

fn handle_broken_link(state: &State, link: &Link) -> Vec<BrokenLink> {
    let resolved_link = resolve_link(link, state);
    let mut broken_links = Vec::new();
    match resolved_link {
        Some(_) => {}
        None => {
            if let Some(pos) = &link.position {
                let range = range_from_position(pos);
                broken_links.push(BrokenLink {
                    range,
                    message: "Link to non-existent file".to_string(),
                })
            };
        }
    };
    broken_links
}

fn handle_broken_ref(req_uri: &Url, state: &State) -> Vec<BrokenLink> {
    let ast = state.ast_for_uri(req_uri).unwrap();
    let mut ranges = Vec::new();
    let re = Regex::new(r"\[([^]]+)\]\[([^]]+)\]").unwrap();
    find_broken_link_ref(ast, &re, &mut ranges);
    ranges
        .iter()
        .map(|r| BrokenLink {
            range: *r,
            message: "Link reference to non-existent link definition".to_string(),
        })
        .collect::<Vec<_>>()
}

fn handle_broken_footnote_ref(req_uri: &Url, state: &State) -> Vec<BrokenLink> {
    let ast = state.ast_for_uri(req_uri).unwrap();
    let mut ranges = Vec::new();
    let re = Regex::new(r"\[(\^\S+)\]").unwrap();
    find_broken_link_ref(ast, &re, &mut ranges);
    ranges
        .iter()
        .map(|r| BrokenLink {
            range: *r,
            message: "Footnote reference to non-existent footnote definition".to_string(),
        })
        .collect()
}

fn find_broken_link_ref(node: &Node, re: &Regex, positions: &mut Vec<Range>) {
    if let Some(children) = node.children() {
        for child in children {
            find_broken_link_ref(child, re, positions)
        }
    }

    if let Node::Text(text) = node {
        for (i, line) in text.value.lines().enumerate() {
            for captures in re.captures_iter(line) {
                let start = captures.get(0).map(|m| m.start());
                let end = captures.get(0).map(|m| m.end());
                if let (Some(s), Some(e), Some(pos)) = (start, end, &text.position) {
                    positions.push(Range {
                        start: Position {
                            line: (pos.start.line + i - 1) as u32,
                            character: (pos.start.column + s - 1) as u32,
                        },
                        end: Position {
                            line: (pos.start.line + i - 1) as u32,
                            character: (pos.start.column + e - 1) as u32,
                        },
                    });
                }
            }
        }
    }
}
