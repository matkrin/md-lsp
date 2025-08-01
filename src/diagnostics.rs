use std::hash::Hash;

use itertools::Itertools;
use lsp_types::{Position, Range, Url};
use markdown::mdast::{Link, Node};
use regex::Regex;

use crate::{
    ast::TraverseNode,
    definition::range_from_position,
    links::{resolve_link, ResolvedLink},
    state::State,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrokenLinkKind {
    /// Invalid Link
    IvalidSyntax,
    /// Link to non-existent heading
    HeadingNotFound,
    /// Link to non-existent heading in other file
    ExternalHeadingNotFound,
    /// Link to non-existent file
    FileNotFound,
    /// Link reference to non-existent link definition
    LinkRefNotFound,
    /// Footnote reference to non-existent footnote definition
    FootnoteRefNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenLink {
    pub kind: BrokenLinkKind,
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

impl BrokenLink {
    pub fn error_code(&self) -> usize {
        match self.kind {
            BrokenLinkKind::IvalidSyntax => 0,
            BrokenLinkKind::HeadingNotFound => 1,
            BrokenLinkKind::ExternalHeadingNotFound => 2,
            BrokenLinkKind::FileNotFound => 3,
            BrokenLinkKind::LinkRefNotFound => 4,
            BrokenLinkKind::FootnoteRefNotFound => 5,
        }
    }
}

pub fn check_links(ast: &Node, req_uri: &Url, state: &State) -> Vec<BrokenLink> {
    let mut v = Vec::new();

    if let Some(children) = ast.children() {
        for child in children {
            match child {
                Node::Link(link) if link.url.starts_with('#') => {
                    if let Some(broken_link) = handle_broken_heading_link(link, req_uri, state) {
                        v.push(broken_link)
                    }
                }
                Node::Link(link) => v.extend(handle_broken_link(state, link)),
                Node::Text(t) => {
                    // Link that gets not parsed because not valid
                    if t.value.contains("](") {
                        v.extend(handle_invalid_link(req_uri, state))
                    }
                    // LinkRef
                    if t.value.contains("][") {
                        v.extend(handle_broken_ref(req_uri, state))
                    }
                    // FootnoteRef
                    if t.value.contains("[^") {
                        v.extend(handle_broken_footnote_ref(req_uri, state))
                    }
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
        ResolvedLink::File { file_uri, .. } => {
            if let Some(pos) = &link.position {
                let file_path = file_uri.to_file_path().unwrap();
                let file_name = file_path.file_name().and_then(|f| f.to_str());
                if link.url.contains('#') {
                    if let Some(f) = file_name {
                        broken_links.push(BrokenLink {
                            kind: BrokenLinkKind::ExternalHeadingNotFound,
                            range: range_from_position(pos),
                            message: format!(
                                "Link to non-existent heading `{}` in file `{}`",
                                link.url, f
                            ),
                        })
                    }
                }
            };
        }
        ResolvedLink::Unresolved => {
            if let Some(pos) = &link.position {
                broken_links.push(BrokenLink {
                    kind: BrokenLinkKind::FileNotFound,
                    range: range_from_position(pos),
                    message: format!("Link to non-existent file `{}`", link.url),
                })
            };
        }
        _ => {}
    };
    broken_links
}

fn handle_broken_heading_link(link: &Link, req_uri: &Url, state: &State) -> Option<BrokenLink> {
    let found = state
        .ast_for_uri(req_uri)
        .and_then(|ast| ast.find_heading_for_link(link));
    if let (Some(pos), None) = (&link.position, found) {
        Some(BrokenLink {
            kind: BrokenLinkKind::HeadingNotFound,
            range: range_from_position(pos),
            message: format!("Link to non-existent heading `{}`", &link.url),
        })
    } else {
        None
    }
}

fn handle_broken_ref(req_uri: &Url, state: &State) -> Vec<BrokenLink> {
    let ast = state.ast_for_uri(req_uri).unwrap();
    let mut ranges = Vec::new();
    let re = Regex::new(r"\[([^]]+)\]\[([^]]+)\]").unwrap();
    find_broken_link_ref(ast, &re, &mut ranges);
    ranges
        .iter()
        .map(|broken_link_ref| BrokenLink {
            kind: BrokenLinkKind::LinkRefNotFound,
            range: broken_link_ref.range,
            message: format!(
                "Link reference to non-existent link definition `{}`",
                broken_link_ref.text
            ),
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
        .map(|broken_link_ref| BrokenLink {
            kind: BrokenLinkKind::FootnoteRefNotFound,
            range: broken_link_ref.range,
            message: format!(
                "Footnote reference to non-existent footnote definition `{}`",
                broken_link_ref.text
            ),
        })
        .collect()
}

fn handle_invalid_link(req_uri: &Url, state: &State) -> Vec<BrokenLink> {
    let ast = state.ast_for_uri(req_uri).unwrap();
    let mut ranges = Vec::new();
    let re = Regex::new(r"\[[^\]]+]\(([^)]+)\)").unwrap();
    find_broken_link_ref(ast, &re, &mut ranges);
    ranges
        .iter()
        .map(|broken_link_ref| BrokenLink {
            kind: BrokenLinkKind::IvalidSyntax,
            range: broken_link_ref.range,
            message: format!("Invalid Link `{}`", broken_link_ref.text),
        })
        .collect()
}

struct BrokenLinkRef<'a> {
    range: Range,
    text: &'a str,
}

fn find_broken_link_ref<'a>(node: &'a Node, re: &Regex, positions: &mut Vec<BrokenLinkRef<'a>>) {
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
                let broken_link_text = captures.get(0).map(|m| m.as_str());

                if let (Some(s), Some(e), Some(text), Some(pos)) =
                    (start, end, broken_link_text, &text.position)
                {
                    positions.push(BrokenLinkRef {
                        range: Range {
                            start: Position {
                                line: (pos.start.line + i - 1) as u32,
                                character: (pos.start.column + s - 1) as u32,
                            },
                            end: Position {
                                line: (pos.start.line + i - 1) as u32,
                                character: (pos.start.column + e - 1) as u32,
                            },
                        },
                        text,
                    });
                }
            }
        }
    }
}
