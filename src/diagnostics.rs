use lsp_types::{Position, Range, Url};
use markdown::mdast::{FootnoteReference, LinkReference, Node};
use regex::Regex;

use crate::{
    ast::{find_definition_for_identifier, find_foot_definition_for_identifier},
    definition::range_from_position,
    state::State,
};

pub fn check_links(ast: &Node, req_uri: &Url, state: &State) -> Vec<(Range, String)> {
    let mut v = Vec::new();

    if let Some(children) = ast.children() {
        for child in children {
            match child {
                Node::Link(link) => {}
                // Node::LinkReference(link_ref) => {
                //     let res = handle_link_ref(req_uri, link_ref, state);
                //     v.extend(res);
                // }
                // Node::FootnoteReference(footnote_ref) => {
                //     v.extend(handle_footnote_ref(ast, footnote_ref));
                // }
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
    v
}

fn handle_broken_ref(req_uri: &Url, state: &State) -> Vec<(Range, String)> {
    let ast = state.ast_for_uri(req_uri).unwrap();
    let mut ranges = Vec::new();
    let re = Regex::new(r"\[([^]]+)\]\[([^]]+)\]").unwrap();
    find_broken_link_ref(ast, &re, &mut ranges);
    log::info!("RANGES {:?}", ranges);
    ranges
        .iter()
        .map(|r| (*r, "BROKEN".to_string()))
        .collect::<Vec<_>>()
}

fn handle_broken_footnote_ref(req_uri: &Url, state: &State) -> Vec<(Range, String)> {
    let ast = state.ast_for_uri(req_uri).unwrap();
    let mut ranges = Vec::new();
    let re = Regex::new(r"\[(\^\S+)\]").unwrap();
    find_broken_link_ref(ast, &re, &mut ranges);
    log::info!("RANGES {:?}", ranges);
    ranges
        .iter()
        .map(|r| (*r, "BROKEN FOOTNOTE".to_string()))
        .collect::<Vec<_>>()
}

fn handle_link_ref(
    req_uri: &Url,
    link_ref: &LinkReference,
    state: &State,
) -> Option<(Range, String)> {
    let ast = state.ast_for_uri(req_uri).unwrap();
    log::info!("ID  {:?}", &link_ref.identifier);
    log::info!(
        "ABDS  {:?}",
        find_definition_for_identifier(ast, &link_ref.identifier)
    );
    match find_definition_for_identifier(ast, &link_ref.identifier) {
        Some(_) => None,
        None => {
            if let Some(pos) = link_ref.position.as_ref() {
                let range = range_from_position(pos);
                let msg = "Link definition not found".to_string();
                Some((range, msg))
            } else {
                None
            }
        }
    }
}

fn handle_footnote_ref(ast: &Node, footnote_ref: &FootnoteReference) -> Option<(Range, String)> {
    match find_foot_definition_for_identifier(ast, &footnote_ref.identifier) {
        Some(_) => None,
        None => {
            if let Some(pos) = footnote_ref.position.as_ref() {
                let range = range_from_position(pos);
                let msg = "FootnoteReference not found".to_string();
                Some((range, msg))
            } else {
                None
            }
        }
    }
}

fn find_broken_link_ref(node: &Node, re: &Regex, positions: &mut Vec<Range>) {
    if let Some(children) = node.children() {
        for child in children {
            find_broken_link_ref(child, re, positions)
        }
    }

    if let Node::Text(text) = node {
        for captures in re.captures_iter(&text.value) {
            let start = captures.get(0).map(|m| m.start());
            let end = captures.get(0).map(|m| m.end());
            if let (Some(s), Some(e), Some(pos)) = (start, end, &text.position) {
                let t_start_line = pos.start.line - 2;
                let lines_up_to_start = text.value[..s].lines().count();
                log::info!("text.value : {:?}", text.value);
                log::info!("lines_up_to_start : {}", lines_up_to_start);

                let chars_up_to_start = if lines_up_to_start > 0 {
                    text.value[..s]
                        .lines()
                        .take(lines_up_to_start - 1)
                        .map(|line| line.len())
                        .sum::<usize>()
                } else {
                    0
                };

                let lines_up_to_end = text.value[..e].lines().count();
                log::info!("lines_up_to_end : {}", lines_up_to_end);

                let chars_up_to_end = if lines_up_to_end > 0 {
                    text.value[..e]
                        .lines()
                        .take(lines_up_to_end - 1)
                        .map(|line| line.len())
                        .sum::<usize>()
                } else {
                    0
                };

                let line = t_start_line + text.value[..s].lines().count();
                let start_character = pos.start.column + s - chars_up_to_start;
                let end_character = pos.end.column + e - chars_up_to_end;
                log::info!("S : {}", s);
                log::info!("E : {}", e);
                log::info!("chars_up_to_start : {}", chars_up_to_start);
                log::info!("chars_up_to_end : {}", chars_up_to_end);
                log::info!("start_character : {}", start_character);
                log::info!("end_character : {}", end_character);

                positions.push(Range {
                    start: Position {
                        line: line as u32,
                        character: start_character as u32,
                    },
                    end: Position {
                        line: line as u32,
                        character: end_character as u32,
                    },
                });
            }
        }
    }
}
