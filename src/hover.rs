use markdown::mdast::{self, Node};

pub struct State {
    pub md_buffer: String,
}

pub fn handle_heading_links(ast: &Node, link: &mdast::Link, state: &State) -> String {
    let linked_heading = find_heading_for_url(ast, &link.url);
    let mut msg = String::new();
    if let Some(heading) = linked_heading {
        let linked_heading_end = heading.position.as_ref().unwrap().end.line;
        let depth = heading.depth;
        let next_heading = find_next_heading(ast, linked_heading_end, depth);
        let start_line = heading.position.as_ref().unwrap().start.line;
        let end_line = next_heading.map(|h| h.position.as_ref().unwrap().start.line);
        let buffer_lines = state.md_buffer.lines().collect::<Vec<_>>();
        msg = if let Some(el) = end_line {
            buffer_lines[(start_line - 1)..(el - 1)]
                .iter()
                .map(|x| x.to_string() + "\n")
                .collect::<String>()
        } else {
            buffer_lines[(start_line - 1)..]
                .iter()
                .map(|x| x.to_string() + "\n")
                .collect::<String>()
        };
    };
    msg
}

pub fn handle_footnote_reference(ast: &Node, footnote_ref: &mdast::FootnoteReference) -> String {
    let def_node = find_def_for_footnote_ref(ast, footnote_ref);
    let mut msg = String::new();
    if let Some(dn) = def_node {
        let footnote_identifier = get_footnote_identifier(dn);
        let footnote_text = get_footnote_def_text(dn);
        if let (Some(fni), Some(fnt)) = (footnote_identifier, footnote_text) {
            msg = format!("[^{}]: {}", fni, fnt);
        }
    }
    msg
}

pub fn find_def_for_link_ref<'a>(
    node: &'a Node,
    link_ref: &mdast::LinkReference,
) -> Option<&'a mdast::Definition> {
    if let Node::Definition(def) = node {
        if link_ref.identifier == def.identifier {
            return Some(def);
        }
    }

    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = find_def_for_link_ref(child, link_ref) {
                return Some(n);
            }
        }
    }
    None
}

pub fn find_def_for_footnote_ref<'a>(
    node: &'a Node,
    foot_ref: &mdast::FootnoteReference,
) -> Option<&'a Node> {
    if let Node::FootnoteDefinition(def) = node {
        if foot_ref.identifier == def.identifier {
            return Some(node);
        }
    }

    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = find_def_for_footnote_ref(child, foot_ref) {
                return Some(n);
            }
        }
    }
    None
}

pub fn get_footnote_identifier(node: &Node) -> Option<String> {
    if let Node::FootnoteDefinition(def) = node {
        return Some(def.identifier.clone());
    }
    None
}

pub fn get_footnote_def_text(node: &Node) -> Option<String> {
    if let Node::Text(t) = node {
        return Some(t.value.clone());
    }
    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = get_footnote_def_text(child) {
                return Some(n);
            }
        }
    }
    None
}

pub fn find_link_for_position(node: &Node, line: u32, character: u32) -> Option<&Node> {
    match node {
        Node::Link(mdast::Link { position, .. })
        | Node::LinkReference(mdast::LinkReference { position, .. })
        | Node::FootnoteReference(mdast::FootnoteReference { position, .. }) => {
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

pub fn find_heading_for_url<'a>(node: &'a Node, link_url: &str) -> Option<&'a mdast::Heading> {
    if let Node::Heading(heading) = node {
        if let Some(Node::Text(mdast::Text { value, .. })) = heading.children.first() {
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

pub fn find_next_heading(node: &Node, end_line: usize, depth: u8) -> Option<&mdast::Heading> {
    if let Node::Heading(heading) = node {
        log::info!("POTENTIAL NEXT HEADING : {:?}", heading);
        log::info!("END LINE : {:?}", end_line);
        if let Some(pos) = &heading.position {
            if end_line < pos.start.line && depth == heading.depth {
                return Some(heading);
            }
        }
    }

    // recurse through children
    if let Some(children) = node.children() {
        for child in children {
            if let Some(n) = find_next_heading(child, end_line, depth) {
                return Some(n);
            }
        }
    }
    None
}
