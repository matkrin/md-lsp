use markdown::mdast::{self, Node};

pub fn find_node_for_position(node: &Node, line: u32, character: u32) -> Option<&Node> {
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
            if let Some(n) = find_node_for_position(child, line, character) {
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

pub fn find_next_heading(node: &Node, end_line: usize, depth: u8) -> Option<& mdast::Heading> {
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
