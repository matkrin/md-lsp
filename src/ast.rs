use markdown::mdast::{
    Definition, FootnoteDefinition, FootnoteReference, Heading, Html, Link, LinkReference, Node,
    Text,
};

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
    fn ast_iter(&self) -> impl Iterator<Item = &Node>;
    // fn find_nodes<T, F>(&self, extractor: F) -> Vec<&T>
    // where
    //     F: Fn(&Node) -> Option<&T>;
    fn find_linkable_for_position(&self, line: u32, character: u32) -> Option<&Node>;
    fn find_definition_for_position(&self, line: u32, character: u32) -> Option<&Node>;
    fn find_heading_for_link(&self, link: &Link) -> Option<&Heading>;
    fn find_heading_for_link_identifier(&self, link: &str) -> Option<&Heading>;
    fn find_definition_for_identifier(&self, identifier: &str) -> Option<&Definition>;
    fn find_foot_definition_for_identifier(&self, identifier: &str) -> Option<&FootnoteDefinition>;
    fn find_link_references_for_identifier(&self, identifier: &str) -> Vec<&LinkReference>;
    fn find_footnote_references_for_identifier(&self, identifier: &str) -> Vec<&FootnoteReference>;
    fn find_headings(&self) -> Vec<&Heading>;
    fn find_links(&self) -> Vec<&Link>;
    fn find_defintions(&self) -> Vec<&Definition>;
    fn find_link_references(&self) -> Vec<&LinkReference>;
    fn find_footnote_definitions(&self) -> Vec<&FootnoteDefinition>;
    fn find_footnote_references(&self) -> Vec<&FootnoteReference>;
    fn find_html_nodes(&self) -> Vec<&Html>;
    fn find_next_heading(&self, end_line: usize, depth: u8) -> Option<&Heading>;
    fn find_def_for_link_ref(&self, link_ref: &LinkReference) -> Option<&Definition>;
    fn find_footnote_def_for_footnote_ref(
        &self,
        foot_ref: &FootnoteReference,
    ) -> Option<&FootnoteDefinition>;
}

impl TraverseNode for Node {
    fn ast_iter(&self) -> impl Iterator<Item = &Node> {
        AstIterator::new(self)
    }

    // fn find_nodes<T, F>(&self, extractor: F) -> Vec<&T>
    // where
    //     F: Fn(&Node) -> Option<&T>,
    // {
    //     self.ast_iter().filter_map(extractor).collect()
    // }

    fn find_linkable_for_position(&self, line: u32, character: u32) -> Option<&Node> {
        self.ast_iter().find(|node| match node {
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
        self.ast_iter().find(|node| match node {
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

        self.ast_iter().find_map(|node| {
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

        self.ast_iter().find_map(|node| {
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
        self.ast_iter().find_map(|node| match node {
            Node::Definition(def) if def.identifier == identifier => Some(def),
            _ => None,
        })
    }

    fn find_foot_definition_for_identifier(&self, identifier: &str) -> Option<&FootnoteDefinition> {
        self.ast_iter().find_map(|node| match node {
            Node::FootnoteDefinition(def) if def.identifier == identifier => Some(def),
            _ => None,
        })
    }

    fn find_link_references_for_identifier(&self, identifier: &str) -> Vec<&LinkReference> {
        self.ast_iter()
            .filter_map(|node| match node {
                Node::LinkReference(lref) if lref.identifier == identifier => Some(lref),
                _ => None,
            })
            .collect()
    }

    fn find_footnote_references_for_identifier(&self, identifier: &str) -> Vec<&FootnoteReference> {
        self.ast_iter()
            .filter_map(|node| match node {
                Node::FootnoteReference(fn_ref) if fn_ref.identifier == identifier => Some(fn_ref),
                _ => None,
            })
            .collect()
    }

    fn find_headings(&self) -> Vec<&Heading> {
        self.ast_iter()
            .filter_map(|node| match node {
                Node::Heading(heading) => Some(heading),
                _ => None,
            })
            .collect()
    }

    fn find_links(&self) -> Vec<&Link> {
        self.ast_iter()
            .filter_map(|node| match node {
                Node::Link(link) => Some(link),
                _ => None,
            })
            .collect()
    }

    fn find_defintions(&self) -> Vec<&Definition> {
        self.ast_iter()
            .filter_map(|node| match node {
                Node::Definition(def) => Some(def),
                _ => None,
            })
            .collect()
    }

    fn find_link_references(&self) -> Vec<&LinkReference> {
        self.ast_iter()
            .filter_map(|node| match node {
                Node::LinkReference(link_ref) => Some(link_ref),
                _ => None,
            })
            .collect()
    }

    fn find_footnote_definitions(&self) -> Vec<&FootnoteDefinition> {
        self.ast_iter()
            .filter_map(|node| match node {
                Node::FootnoteDefinition(footnote_def) => Some(footnote_def),
                _ => None,
            })
            .collect()
    }

    fn find_footnote_references(&self) -> Vec<&FootnoteReference> {
        self.ast_iter()
            .filter_map(|node| match node {
                Node::FootnoteReference(footnote_ref) => Some(footnote_ref),
                _ => None,
            })
            .collect()
    }

    fn find_html_nodes(&self) -> Vec<&Html> {
        self.ast_iter()
            .filter_map(|node| match node {
                Node::Html(html) => Some(html),
                _ => None,
            })
            .collect()
    }

    fn find_next_heading(&self, end_line: usize, depth: u8) -> Option<&Heading> {
        self.ast_iter().find_map(|node| match node {
            Node::Heading(heading) => match &heading.position {
                Some(pos) if end_line < pos.start.line && depth == heading.depth => Some(heading),
                _ => None,
            },
            _ => None,
        })
    }

    fn find_def_for_link_ref(&self, link_ref: &LinkReference) -> Option<&Definition> {
        self.ast_iter().find_map(|node| match node {
            Node::Definition(def) if def.identifier == link_ref.identifier => Some(def),
            _ => None,
        })
    }

    fn find_footnote_def_for_footnote_ref(
        &self,
        foot_ref: &FootnoteReference,
    ) -> Option<&FootnoteDefinition> {
        self.ast_iter().find_map(|node| match node {
            Node::FootnoteDefinition(def) if def.identifier == foot_ref.identifier => Some(def),
            _ => None,
        })
    }
}

pub fn get_heading_text(heading: &Heading) -> Option<&str> {
    for child in &heading.children {
        if let Node::Text(Text { value, .. }) = child {
            return Some(value);
        };
    }
    None
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
        let found_definition_1 = ast.find_definition_for_identifier("google-link");
        let found_definition_2 = ast.find_definition_for_identifier("duckduckgo");
        insta::assert_debug_snapshot!(found_definition_1);
        insta::assert_debug_snapshot!(found_definition_2);
    }

    #[test]
    fn test_find_foot_definition_for_identifier() {
        let ast = ast();
        let found_footnote_definition = ast.find_foot_definition_for_identifier("1");
        insta::assert_debug_snapshot!(found_footnote_definition);
    }

    #[test]
    fn test_find_link_references_for_identifier() {
        let ast = ast();
        let found_link_refs = ast.find_link_references_for_identifier("google-link");
        insta::assert_debug_snapshot!(found_link_refs);
    }
    #[test]
    fn test_find_footnote_references_for_identifier() {
        let ast = ast();
        let found_footnote_refs = ast.find_footnote_references_for_identifier("multi");
        insta::assert_debug_snapshot!(found_footnote_refs);
    }

    #[test]
    fn test_find_headings() {
        let ast = ast();
        let headings = ast.find_headings();
        insta::assert_debug_snapshot!(headings);
    }

    #[test]
    fn test_find_links() {
        let ast = ast();
        let links = ast.find_links();
        insta::assert_debug_snapshot!(links);
    }

    #[test]
    fn test_find_defintions() {
        let ast = ast();
        let defs = ast.find_defintions();
        insta::assert_debug_snapshot!(defs);
    }

    #[test]
    fn test_find_link_references() {
        let ast = ast();
        let link_refs = ast.find_link_references();
        insta::assert_debug_snapshot!(link_refs);
    }

    #[test]
    fn test_find_footnote_definitions() {
        let ast = ast();
        let footnote_defs = ast.find_footnote_definitions();
        insta::assert_debug_snapshot!(footnote_defs);
    }

    #[test]
    fn test_find_footnote_references() {
        let ast = ast();
        let footnote_refs = ast.find_footnote_references();
        insta::assert_debug_snapshot!(footnote_refs);
    }

    #[test]
    fn test_find_html_nodes() {
        let ast = ast();
        let htmls = ast.find_html_nodes();
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
        let next_heading_1 = ast.find_next_heading(30, 3);
        let next_heading_2 = ast.find_next_heading(30, 5);
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
        let found_definition = ast.find_def_for_link_ref(&link_ref);
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
        let found_footnote_ref = ast.find_footnote_def_for_footnote_ref(&footnote_ref);
        insta::assert_debug_snapshot!(found_footnote_ref);
    }
}
