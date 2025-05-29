use lsp_types::Url;
use markdown::{
    mdast::{Heading, Link, Node, Text},
    unist::{Point, Position as AstPosition},
};
use regex::Regex;

use crate::{
    ast::{get_heading_text, TraverseNode},
    state::State,
};

use percent_encoding::{AsciiSet, CONTROLS};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"');

pub fn url_encode(s: &str) -> String {
    percent_encoding::percent_encode(s.as_bytes(), FRAGMENT).to_string()
}

pub fn url_decode(s: &str) -> String {
    percent_encoding::percent_decode(s.as_bytes())
        .decode_utf8_lossy()
        .to_string()
}

#[derive(Debug)]
pub enum MdLink<'a> {
    NormalLink(&'a Link),
    WikiLink(&'a Link),
}

impl<'a> MdLink<'a> {
    fn new(link: &'a Link) -> Self {
        match &link.title {
            Some(title) if title == "wikilink" => MdLink::WikiLink(link),
            _ => MdLink::NormalLink(link),
        }
    }

    pub fn position(&self) -> Option<&AstPosition> {
        match self {
            MdLink::NormalLink(link) | MdLink::WikiLink(link) => link.position.as_ref(),
        }
    }

    pub fn title(&self) -> Option<&str> {
        match self {
            MdLink::NormalLink(link) | MdLink::WikiLink(link) => link.title.as_deref(),
        }
    }

    pub fn is_wikilink(&self) -> bool {
        match self {
            MdLink::NormalLink(_) => false,
            MdLink::WikiLink(_) => true,
        }
    }
}

#[derive(Debug)]
pub enum ResolvedLink<'a> {
    File {
        link: MdLink<'a>,
        /// The Uri of the file, the Link links to
        file_uri: &'a Url,
    },
    InternalHeading {
        link: MdLink<'a>,
        /// The Uri of the file, the Link links to
        file_uri: &'a Url,
        /// The Heading, the Link links to
        heading: &'a Heading,
    },
    ExternalHeading {
        link: MdLink<'a>,
        /// The Uri of the file, the Link links to
        file_uri: &'a Url,
        /// The Heading, the Link links to
        heading: &'a Heading,
    },
    Http,
    Unresolved,
}

impl ResolvedLink<'_> {
    /// The position of the Link
    pub fn link_position(&self) -> Option<&AstPosition> {
        match self {
            ResolvedLink::File { link, .. }
            | ResolvedLink::InternalHeading { link, .. }
            | ResolvedLink::ExternalHeading { link, .. } => link.position(),
            _ => None,
        }
    }

    /// The Url of the file, the Link links to
    pub fn file_uri(&self) -> Option<&Url> {
        match self {
            ResolvedLink::File { file_uri, .. }
            | ResolvedLink::InternalHeading { file_uri, .. }
            | ResolvedLink::ExternalHeading { file_uri, .. } => Some(file_uri),
            _ => None,
        }
    }

    /// If Link links to a heading, return the text of this heading
    pub fn heading_text(&self) -> Option<&str> {
        match self {
            ResolvedLink::InternalHeading { heading, .. }
            | ResolvedLink::ExternalHeading { heading, .. } => get_heading_text(heading),
            _ => None,
        }
    }
}

pub fn resolve_link<'a>(link: &'a Link, state: &'a State) -> ResolvedLink<'a> {
    let md_link = MdLink::new(link);

    if link.url.starts_with("http") {
        return ResolvedLink::Http;
    }

    match link.url.split_once('#') {
        Some(("", _)) => {
            for (url, md_file) in state.md_files.iter() {
                if let Some(heading) = &md_file.ast.find_heading_for_link(link) {
                    return ResolvedLink::InternalHeading {
                        link: md_link,
                        file_uri: url,
                        heading,
                    };
                }
            }
            ResolvedLink::Unresolved
        }
        // link with referece to heading `...#...`
        Some((file_ref_text, heading_ref_text)) => {
            let file = match md_link {
                MdLink::NormalLink(_) => url_decode(file_ref_text),
                MdLink::WikiLink(_) => {
                    // allow both for wikilinks: with suffix and without
                    if file_ref_text.ends_with(".md") {
                        file_ref_text.into()
                    } else {
                        format!("{}.md", file_ref_text)
                    }
                }
            };

            for (url, relative_path) in state.get_file_list() {
                if relative_path == file {
                    // as we get url from state it must be in there
                    let ast = state.ast_for_uri(url).unwrap();
                    if let Some(heading) = ast.find_heading_for_link_identifier(heading_ref_text) {
                        return ResolvedLink::ExternalHeading {
                            link: md_link,
                            file_uri: url,
                            heading,
                        };
                    }
                    return ResolvedLink::File {
                        link: md_link,
                        file_uri: url,
                    };
                }
            }
            ResolvedLink::Unresolved
        }
        None => {
            let file = if link.url.ends_with(".md") {
                link.url.to_string()
            } else {
                format!("{}.md", link.url)
            };
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

#[derive(Debug, PartialEq, Eq)]
struct ExtractedWikiLink {
    content: String,
    start_position: usize,
    line_number: usize,
}

impl ExtractedWikiLink {
    fn link_text_node(&self, start_line: usize) -> Node {
        let value = "".to_string(); // TODO parse Wikilinks with `|` (everything after is value)
        let value_len = value.len();
        let link_text = Text {
            value,
            position: Some(AstPosition {
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
            position: Some(AstPosition {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wiki_links() {
        let link_content_1 = "link content 1".to_string();
        let link_content_2 = "link content 2".to_string();
        let input = format!(
            "[[{}]]\n\n# Heading \n\n[[{}]]",
            link_content_1, link_content_2
        );
        let extracted = extract_wiki_links(&input);
        let expected_1 = ExtractedWikiLink {
            content: link_content_1,
            start_position: 3,
            line_number: 0,
        };
        let expected_2 = ExtractedWikiLink {
            content: link_content_2,
            start_position: 3,
            line_number: 4,
        };
        assert_eq!(extracted[0], expected_1);
        assert_eq!(extracted[1], expected_2);
    }
}
