use markdown::{
    mdast::{self, Link, Node},
    unist::{Point, Position},
};
use regex::Regex;

struct Extracted {
    content: String,
    start_position: usize,
    line_number: usize,
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

pub fn make_wiki_links(node: &mut Node) {
    let mut links = Vec::new();
    if let Some(children) = node.children() {
        for child in children {
            if let Node::Text(t) = child {
                if t.value.contains("[[") && t.value.contains("]]") {
                    let t_position = t.position.as_ref().unwrap().clone();
                    let extracted = extract_links(&t.value);

                    for i in extracted {
                        let link_text_ast = mdast::Text {
                            value: i.content.clone(),
                            position: Some(Position {
                                start: Point {
                                    line: t_position.start.line + i.line_number,
                                    column: i.start_position,
                                    offset: i.start_position,
                                },
                                end: Point {
                                    line: t_position.start.line + i.line_number,
                                    column: i.start_position + i.content.len(),
                                    offset: i.start_position + i.content.len(),
                                },
                            }),
                        };

                        let link_ast = Link {
                            children: vec![Node::Text(link_text_ast)],
                            position: Some(Position {
                                start: Point {
                                    line: t_position.start.line + i.line_number,
                                    column: i.start_position - 2,
                                    offset: i.start_position - 2,
                                },
                                end: Point {
                                    line: t_position.start.line + i.line_number,
                                    column: i.start_position + i.content.len() + 2,
                                    offset: i.start_position + i.content.len() + 2,
                                },
                            }),
                            url: i.content.clone(),
                            title: Some(i.content),
                        };

                        links.push(Node::Link(link_ast))
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
            make_wiki_links(child);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_wiki_links() {
        let md = "Wikilink [[link]] in paragraph";
        let mut ast = markdown::to_mdast(md, &markdown::ParseOptions::gfm()).unwrap();
        // dbg!(&ast);
        make_wiki_links(&mut ast);
        // dbg!(&ast);
    }

    #[test]
    fn test_make_wiki_links2() {
        let md = "Another line with [[link1]]
Wikilink [[link]] in paragraph ";
        let mut ast = markdown::to_mdast(md, &markdown::ParseOptions::gfm()).unwrap();
        dbg!(&ast);
        make_wiki_links(&mut ast);
        dbg!(&ast);
    }
}
