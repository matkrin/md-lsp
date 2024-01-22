use markdown::{
    mdast::{FootnoteReference, Heading, Link, LinkReference, Node, Text},
    unist::{Point, Position},
};
use regex::Regex;

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

fn extract_links(input: &str) -> Vec<Extracted> {
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
                    let extracted = extract_links(&t.value);

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

pub fn find_heading_for_url<'a>(node: &'a Node, link_url: &str) -> Option<&'a Heading> {
    if let Node::Heading(heading) = node {
        if let Some(Node::Text(Text { value, .. })) = heading.children.first() {
            if value == &link_url.replace('#', "") {
                return Some(heading);
            }
        }
    };

    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = find_heading_for_url(child, link_url) {
                return Some(n);
            }
        }
    }
    None
}

pub fn find_link_for_position(node: &Node, line: u32, character: u32) -> Option<&Node> {
    match node {
        Node::Link(Link { position, .. })
        | Node::LinkReference(LinkReference { position, .. })
        | Node::FootnoteReference(FootnoteReference { position, .. }) => {
            if let Some(pos) = position {
                if (line + 1) as usize >= pos.start.line
                    && (line + 1) as usize <= pos.end.line
                    && ((character + 1) as usize) >= pos.start.column
                    && ((character + 1) as usize) <= pos.end.column
                {
                    return Some(node);
                }
            }
        }
        _ => {}
    };

    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = find_link_for_position(child, line, character) {
                return Some(n);
            }
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_wiki_links() {
        let md = "Wikilink [[link]] in paragraph";
        let mut ast = markdown::to_mdast(md, &markdown::ParseOptions::gfm()).unwrap();
        // dbg!(&ast);
        parse_wiki_links(&mut ast);
        // dbg!(&ast);
    }

    #[test]
    fn test_make_wiki_links2() {
        let md = "Another line with [[link1]]
Wikilink [[link]] in paragraph ";
        let mut ast = markdown::to_mdast(md, &markdown::ParseOptions::gfm()).unwrap();
        dbg!(&ast);
        parse_wiki_links(&mut ast);
        dbg!(&ast);
    }
}
