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

pub fn find_linkable_for_position(node: &Node, line: u32, character: u32) -> Option<&Node> {
    // log::info!("NODE: {:?}", node);
    match node {
        Node::Heading(Heading { position, .. })
        | Node::Link(Link { position, .. })
        | Node::LinkReference(LinkReference { position, .. })
        | Node::FootnoteReference(FootnoteReference { position, .. }) => {
            if let Some(pos) = position {
                if (line + 1) as usize >= pos.start.line
                    && (line + 1) as usize <= pos.end.line
                    && (character + 1) as usize >= pos.start.column
                    && (character + 1) as usize <= pos.end.column
                {
                    return Some(node);
                }
            }
        }
        _ => {}
    };

    traverse_ast!(node, find_linkable_for_position, line, character)
}

pub fn find_definition_for_position(node: &Node, line: u32, character: u32) -> Option<&Node> {
    match node {
        Node::Heading(Heading { position, .. })
        | Node::Definition(Definition { position, .. })
        | Node::FootnoteDefinition(FootnoteDefinition { position, .. }) => {
            if let Some(pos) = position {
                if (line + 1) as usize >= pos.start.line
                    && (line + 1) as usize <= pos.end.line
                    && (character + 1) as usize >= pos.start.column
                // && (character + 1) as usize <= pos.end.column
                {
                    return Some(node);
                }
            }
        }
        _ => {}
    };

    traverse_ast!(node, find_definition_for_position, line, character)
}

pub fn find_heading_for_link<'a>(node: &'a Node, link: &Link) -> Option<&'a Heading> {
    if let Node::Heading(heading) = node {
        if let Some(Node::Text(Text { value, .. })) = heading.children.first() {
            if value == &link.url.replace('#', "")
                || value.to_lowercase().replace(' ', "-") == link.url.replace('#', "")
            {
                log::info!("IN FIND H: {:?}", &heading);
                return Some(heading);
            }
        }
    };

    traverse_ast!(node, find_heading_for_link, link)
}

pub fn find_heading_for_link_identifier<'a>(
    node: &'a Node,
    link_identifier: &str,
) -> Option<&'a Heading> {
    if let Node::Heading(heading) = node {
        if let Some(Node::Text(Text { value, .. })) = heading.children.first() {
            if value == &link_identifier.replace('#', "")
                || value.to_lowercase().replace(' ', "-") == link_identifier.replace('#', "")
            {
                log::info!("IN FIND H: {:?}", &heading);
                return Some(heading);
            }
        }
    };

    traverse_ast!(node, find_heading_for_link_identifier, link_identifier)
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

pub fn find_foonote_references(node: &Node) -> Vec<&FootnoteReference> {
    let mut footnote_refs = Vec::new();
    match node {
        Node::FootnoteReference(footnote_ref) => {
            footnote_refs.push(footnote_ref);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    footnote_refs.extend(find_foonote_references(child))
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
