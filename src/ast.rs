use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, Html, Link, LinkReference, Node,
    Text,
};

/// Recursive AST traversal
#[macro_export]
macro_rules! traverse_ast {
    ($node: expr, $func: expr $(, $args: expr)*) => {
        if let Some(children) = $node.children() {
            for child in children {
                if let Some(result) = $func(child, $($args),*) {
                    return Some(result);
                }
            }
            None
        } else {
            None
        }
    };
}

pub struct AstIterator<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> AstIterator<'a> {
    pub fn new(root: &'a Node) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for AstIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            if let Some(children) = node.children() {
                self.stack.extend(children.iter().rev());
            }
            Some(node)
        } else {
            None
        }
    }
}

pub trait TraverseNode {
    fn find_linkable_for_position(&self, line: u32, character: u32) -> Option<&Node>;
    fn find_definition_for_position(&self, line: u32, character: u32) -> Option<&Node>;
    fn find_heading_for_link(&self, link: &Link) -> Option<&Heading>;
    fn find_heading_for_link_identifier(&self, link: &str) -> Option<&Heading>;
    fn find_definition_for_identifier(&self, identifier: &str) -> Option<&Definition>;
}

impl TraverseNode for Node {
    /// Finds a linkable node at the given position
    fn find_linkable_for_position(&self, line: u32, character: u32) -> Option<&Node> {
        AstIterator::new(self).find(|node| match node {
            Node::Heading(Heading { position, .. })
            | Node::Link(Link { position, .. })
            | Node::LinkReference(LinkReference { position, .. })
            | Node::FootnoteReference(FootnoteReference { position, .. }) => {
                if let Some(pos) = position {
                    (line + 1) as usize >= pos.start.line
                        && (line + 1) as usize <= pos.end.line
                        && (character + 1) as usize >= pos.start.column
                        && (character + 1) as usize <= pos.end.column
                } else {
                    false
                }
            }
            _ => false,
        })
    }
    fn find_definition_for_position(&self, line: u32, character: u32) -> Option<&Node> {
        AstIterator::new(self).find(|node| match node {
            Node::Heading(Heading { position, .. })
            | Node::Definition(Definition { position, .. })
            | Node::FootnoteDefinition(FootnoteDefinition { position, .. }) => {
                if let Some(pos) = position {
                    (line + 1) as usize >= pos.start.line
                        && (line + 1) as usize <= pos.end.line
                        && (character + 1) as usize >= pos.start.column
                    // && (character + 1) as usize <= pos.end.column
                } else {
                    false
                }
            }
            _ => false,
        })
    }

    fn find_heading_for_link(&self, link: &Link) -> Option<&Heading> {
        let target = link.url.replace('#', "");
        let normalized_target = target.to_lowercase().replace(' ', "-");

        AstIterator::new(self).find_map(|node| {
            let Node::Heading(heading) = node else {
                return None;
            };
            let Node::Text(Text { value, .. }) = heading.children.first()? else {
                return None;
            };

            if value == &target || value.to_lowercase().replace(' ', "-") == normalized_target {
                Some(heading)
            } else {
                None
            }
        })
    }

    fn find_heading_for_link_identifier(&self, link_identifier: &str) -> Option<&Heading> {
        let target = link_identifier.replace('#', "");
        let normalized_target = target.to_lowercase().replace(' ', "-");

        AstIterator::new(self).find_map(|node| {
            let Node::Heading(heading) = node else {
                return None;
            };
            let Node::Text(Text { value, .. }) = heading.children.first()? else {
                return None;
            };

            if value == &target || value.to_lowercase().replace(' ', "-") == normalized_target {
                Some(heading)
            } else {
                None
            }
        })
    }
    fn find_definition_for_identifier(&self, identifier: &str) -> Option<&Definition> {
        AstIterator::new(self).find_map(|node| {
            if let Node::Definition(def) = node {
                if def.identifier == identifier {
                    Some(def)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

pub fn find_definition_for_identifier<'a>(
    node: &'a Node,
    identifier: &str,
) -> Option<&'a Definition> {
    if let Node::Definition(def) = node {
        if def.identifier == identifier {
            return Some(def);
        }
    }

    traverse_ast!(node, find_definition_for_identifier, identifier)
}

pub fn find_foot_definition_for_identifier<'a>(
    node: &'a Node,
    identifier: &str,
) -> Option<&'a FootnoteDefinition> {
    if let Node::FootnoteDefinition(def) = node {
        if def.identifier == identifier {
            return Some(def);
        }
    }

    traverse_ast!(node, find_foot_definition_for_identifier, identifier)
}

pub fn find_link_references_for_identifier<'a>(
    node: &'a Node,
    identifier: &str,
) -> Vec<&'a LinkReference> {
    let mut link_refs = Vec::new();
    match node {
        Node::LinkReference(lref) => {
            if lref.identifier == identifier {
                link_refs.push(lref)
            }
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    link_refs.extend(find_link_references_for_identifier(child, identifier))
                }
            }
        }
    }
    link_refs
}

pub fn find_footnote_references_for_identifier<'a>(
    node: &'a Node,
    identifier: &str,
) -> Vec<&'a FootnoteReference> {
    let mut footnote_refs = Vec::new();
    match node {
        Node::FootnoteReference(fn_ref) => {
            if fn_ref.identifier == identifier {
                footnote_refs.push(fn_ref)
            }
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    footnote_refs.extend(find_footnote_references_for_identifier(child, identifier))
                }
            }
        }
    }
    footnote_refs
}

pub fn find_headings(node: &Node) -> Vec<&Heading> {
    let mut headings = Vec::new();
    match node {
        Node::Heading(heading) => {
            headings.push(heading);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    headings.extend(find_headings(child))
                }
            }
        }
    }
    headings
}

pub fn find_links(node: &Node) -> Vec<&Link> {
    let mut links = Vec::new();
    match node {
        Node::Link(link) => {
            links.push(link);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    links.extend(find_links(child))
                }
            }
        }
    }
    links
}

pub fn find_defintions(node: &Node) -> Vec<&Definition> {
    let mut definitions = Vec::new();
    match node {
        Node::Definition(def) => {
            definitions.push(def);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    definitions.extend(find_defintions(child))
                }
            }
        }
    }
    definitions
}

pub fn find_link_references(node: &Node) -> Vec<&LinkReference> {
    let mut link_refs = Vec::new();
    match node {
        Node::LinkReference(link_ref) => {
            link_refs.push(link_ref);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    link_refs.extend(find_link_references(child))
                }
            }
        }
    }
    link_refs
}

pub fn find_footnote_definitions(node: &Node) -> Vec<&FootnoteDefinition> {
    let mut footnote_defs = Vec::new();
    match node {
        Node::FootnoteDefinition(footnote_def) => {
            footnote_defs.push(footnote_def);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    footnote_defs.extend(find_footnote_definitions(child))
                }
            }
        }
    }
    footnote_defs
}

pub fn find_footnote_references(node: &Node) -> Vec<&FootnoteReference> {
    let mut footnote_refs = Vec::new();
    match node {
        Node::FootnoteReference(footnote_ref) => {
            footnote_refs.push(footnote_ref);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    footnote_refs.extend(find_footnote_references(child))
                }
            }
        }
    }
    footnote_refs
}

pub fn find_html_nodes(node: &Node) -> Vec<&Html> {
    let mut htmls = Vec::new();
    match node {
        Node::Html(html) => {
            htmls.push(html);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    htmls.extend(find_html_nodes(child))
                }
            }
        }
    }
    htmls
}

pub fn get_heading_text(heading: &Heading) -> Option<&str> {
    for child in &heading.children {
        if let Node::Text(Text { value, .. }) = child {
            return Some(value);
        };
    }
    None
}

pub fn find_next_heading(node: &Node, end_line: usize, depth: u8) -> Option<&Heading> {
    if let Node::Heading(heading) = node {
        if let Some(pos) = &heading.position {
            if end_line < pos.start.line && depth == heading.depth {
                return Some(heading);
            }
        }
    }

    traverse_ast!(node, find_next_heading, end_line, depth)
}

pub fn find_def_for_link_ref<'a>(
    node: &'a Node,
    link_ref: &LinkReference,
) -> Option<&'a Definition> {
    if let Node::Definition(def) = node {
        if link_ref.identifier == def.identifier {
            return Some(def);
        }
    }

    traverse_ast!(node, find_def_for_link_ref, link_ref)
}

pub fn find_footnote_def_for_footnote_ref<'a>(
    node: &'a Node,
    foot_ref: &FootnoteReference,
) -> Option<&'a FootnoteDefinition> {
    if let Node::FootnoteDefinition(def) = node {
        if foot_ref.identifier == def.identifier {
            return Some(def);
        }
    }

    traverse_ast!(node, find_footnote_def_for_footnote_ref, foot_ref)
}

#[cfg(test)]
mod tests {
    use super::TraverseNode;
    use super::*;

    use markdown::mdast::{Node, ReferenceKind};
    use markdown::unist::{Point, Position};

    use crate::links::parse_wiki_links;

    fn ast() -> Node {
        let markdown = std::fs::read_to_string(r#"testdata/test file c.md"#)
            .expect("test file does not exists");
        let mut ast = markdown::to_mdast(&markdown, &markdown::ParseOptions::gfm())
            .expect("markdown can't be parsed");
        parse_wiki_links(&mut ast);
        ast
    }

    #[test]
    fn test_find_linkable_for_position() {
        let ast = ast();
        let line_number = 31;
        // let linkable = find_linkable_for_position(&ast, line_number, 0);
        // let linkable_2 = find_linkable_for_position(&ast, line_number, 11);
        let linkable = ast.find_linkable_for_position(line_number, 0);
        let linkable_2 = ast.find_linkable_for_position(line_number, 11);
        insta::assert_debug_snapshot!(linkable);
        insta::assert_debug_snapshot!(linkable_2);
    }

    #[test]
    fn test_find_definition_for_position() {
        let ast = ast();
        let line_number = 31;
        let linkable = ast.find_definition_for_position(line_number, 0);
        let linkable_2 = ast.find_definition_for_position(line_number, 11);
        insta::assert_debug_snapshot!(linkable);
        insta::assert_debug_snapshot!(linkable_2);
    }

    #[test]
    fn test_find_heading_for_link() {
        let ast = ast();
        let line_number = 6;
        let link = Link {
            children: vec![Node::Text(Text {
                value: "Heading 1".to_string(),
                position: Some(Position {
                    start: Point {
                        line: line_number,
                        column: 4,
                        offset: 152,
                    },
                    end: Point {
                        line: line_number,
                        column: 13,
                        offset: 161,
                    },
                }),
            })],
            position: Some(Position {
                start: Point {
                    line: line_number,
                    column: 3,
                    offset: 151,
                },
                end: Point {
                    line: line_number,
                    column: 26,
                    offset: 174,
                },
            }),
            url: "#heading-1".to_string(),
            title: None,
        };
        let found_heading = ast.find_heading_for_link(&link);
        insta::assert_debug_snapshot!(found_heading);
    }

    #[test]
    fn test_find_heading_for_link_identifier() {
        let ast = ast();
        let found_heading = ast.find_heading_for_link_identifier("#heading-1");
        insta::assert_debug_snapshot!(found_heading);
    }

    #[test]
    fn test_find_definition_for_identifier() {
        let ast = ast();
        let found_definition_1 = find_definition_for_identifier(&ast, "google-link");
        let found_definition_2 = find_definition_for_identifier(&ast, "duckduckgo");
        insta::assert_debug_snapshot!(found_definition_1);
        insta::assert_debug_snapshot!(found_definition_2);
    }

    #[test]
    fn test_find_foot_definition_for_identifier() {
        let ast = ast();
        let found_footnote_definition = find_foot_definition_for_identifier(&ast, "1");
        insta::assert_debug_snapshot!(found_footnote_definition);
    }

    #[test]
    fn test_find_link_references_for_identifier() {
        let ast = ast();
        let found_link_refs = find_link_references_for_identifier(&ast, "google-link");
        insta::assert_debug_snapshot!(found_link_refs);
    }
    #[test]
    fn test_find_footnote_references_for_identifier() {
        let ast = ast();
        let found_footnote_refs = find_footnote_references_for_identifier(&ast, "multi");
        insta::assert_debug_snapshot!(found_footnote_refs);
    }

    #[test]
    fn test_find_headings() {
        let ast = ast();
        let headings = find_headings(&ast);
        insta::assert_debug_snapshot!(headings);
    }

    #[test]
    fn test_find_links() {
        let ast = ast();
        let links = find_links(&ast);
        insta::assert_debug_snapshot!(links);
    }

    #[test]
    fn test_find_defintions() {
        let ast = ast();
        let defs = find_defintions(&ast);
        insta::assert_debug_snapshot!(defs);
    }

    #[test]
    fn test_find_link_references() {
        let ast = ast();
        let link_refs = find_link_references(&ast);
        insta::assert_debug_snapshot!(link_refs);
    }

    #[test]
    fn test_find_footnote_definitions() {
        let ast = ast();
        let footnote_defs = find_footnote_definitions(&ast);
        insta::assert_debug_snapshot!(footnote_defs);
    }

    #[test]
    fn test_find_footnote_references() {
        let ast = ast();
        let footnote_refs = find_footnote_references(&ast);
        insta::assert_debug_snapshot!(footnote_refs);
    }

    #[test]
    fn test_find_html_nodes() {
        let ast = ast();
        let htmls = find_html_nodes(&ast);
        insta::assert_debug_snapshot!(htmls);
    }

    #[test]
    fn test_get_heading_text() {
        let heading = Heading {
            children: vec![Node::Text(Text {
                value: "Heading 1".to_string(),
                position: Some(Position {
                    start: Point {
                        line: 30,
                        column: 3,
                        offset: 791,
                    },
                    end: Point {
                        line: 30,
                        column: 12,
                        offset: 800,
                    },
                }),
            })],
            position: Some(Position {
                start: Point {
                    line: 30,
                    column: 1,
                    offset: 789,
                },
                end: Point {
                    line: 30,
                    column: 12,
                    offset: 800,
                },
            }),
            depth: 1,
        };
        assert_eq!(get_heading_text(&heading), Some("Heading 1"));
    }

    #[test]
    fn test_find_next_heading() {
        let ast = ast();
        let next_heading_1 = find_next_heading(&ast, 30, 3);
        let next_heading_2 = find_next_heading(&ast, 30, 5);
        insta::assert_debug_snapshot!(next_heading_1);
        assert_eq!(next_heading_2, None);
    }

    #[test]
    fn test_find_def_for_link_ref() {
        let ast = ast();
        let line_number = 151;
        let link_ref = LinkReference {
            children: vec![Node::Text(Text {
                value: "google".to_string(),
                position: Some(Position {
                    start: Point {
                        line: line_number,
                        column: 31,
                        offset: 2715,
                    },
                    end: Point {
                        line: line_number,
                        column: 37,
                        offset: 2721,
                    },
                }),
            })],
            position: Some(Position {
                start: Point {
                    line: line_number,
                    column: 30,
                    offset: 2714,
                },
                end: Point {
                    line: line_number,
                    column: 51,
                    offset: 2735,
                },
            }),
            reference_kind: ReferenceKind::Full,
            identifier: "google-link".to_string(),
            label: Some("google-link".to_string()),
        };
        let found_definition = find_def_for_link_ref(&ast, &link_ref);
        insta::assert_debug_snapshot!(found_definition);
    }

    #[test]
    fn test_find_footnote_def_for_footnote_ref() {
        let ast = ast();
        let line_number = 105;
        let footnote_ref = FootnoteReference {
            position: Some(Position {
                start: Point {
                    line: line_number,
                    column: 26,
                    offset: 1820,
                },
                end: Point {
                    line: line_number,
                    column: 30,
                    offset: 1824,
                },
            }),
            identifier: "1".to_string(),
            label: Some("1".to_string()),
        };
        let found_footnote_ref = find_footnote_def_for_footnote_ref(&ast, &footnote_ref);
        insta::assert_debug_snapshot!(found_footnote_ref);
    }
}
