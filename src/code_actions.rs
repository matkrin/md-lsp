use std::collections::HashMap;

use lsp_types::{CodeAction, Position, Range, TextEdit, Url, WorkspaceEdit};
use markdown::mdast::{Heading, Html};

use crate::{
    ast::{find_headings, find_html_nodes},
    rename::get_text_child,
    state::State,
};

const TOC_START: &str = "<!--toc:start-->";
const TOC_END: &str = "<!--toc:end-->";

// first edit should be enough new lines that content pushed down, second edit actual content
pub fn code_actions(req_uri: &Url, state: &State) -> Option<Vec<CodeAction>> {
    let mut code_actions = Vec::new();
    let ast = state.ast_for_uri(req_uri)?;
    let headings = find_headings(ast);
    let toc_tags: Vec<&Html> = find_html_nodes(ast)
        .into_iter()
        .filter(|html| html.value == TOC_START || html.value == TOC_END)
        .collect();

    if toc_tags.is_empty() {
        code_actions.push(create_toc(&headings, req_uri));
    } else {
        code_actions.push(update_toc(&headings, &toc_tags, req_uri));
    }

    code_actions.into_iter().collect()
}

fn toc(headings: &[&Heading]) -> String {
    let toc = headings.iter().fold(String::new(), |mut acc, heading| {
        match get_text_child(&heading.children) {
            Some(text) => {
                let link_text = text.value.clone();
                let url_text = format!("#{}", text.value.to_lowercase().replace(' ', "-"));
                let indent = (0..heading.depth - 1).fold(String::new(), |mut acc, _| {
                    acc.push_str("  ");
                    acc
                });
                let toc_entry = format!("{}- [{}]({})\n", indent, link_text, url_text);
                acc.push_str(&toc_entry);
                acc
            }
            _ => acc,
        }
    });
    format!("{}\n{}{}\n\n", TOC_START, toc, TOC_END)
}

fn create_toc(headings: &[&Heading], req_uri: &Url) -> Option<CodeAction> {
    let first_heading_line = headings.first()?.position.as_ref()?.start.line;
    let toc = toc(headings);
    let text_edit = TextEdit {
        range: Range {
            start: Position {
                line: (first_heading_line + 1) as u32,
                character: 0,
            },
            end: Position {
                line: (first_heading_line + 1) as u32,
                character: 0,
            },
        },
        new_text: toc,
    };
    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    changes.entry(req_uri.clone()).or_default().push(text_edit);
    let workspace_edit = WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    };
    Some(CodeAction {
        title: "Create Table of Contents".to_string(),
        kind: None,
        diagnostics: None,
        edit: Some(workspace_edit),
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    })
}

fn update_toc(headings: &[&Heading], toc_tags: &[&Html], req_uri: &Url) -> Option<CodeAction> {
    // let first_heading_line = headings.first()?.position.as_ref()?.start.line;
    let toc = toc(headings);
    let toc_start_pos = toc_tags
        .iter()
        .find(|x| x.value == TOC_START)
        .and_then(|it| it.position.as_ref())?;
    let toc_end_pos = toc_tags
        .iter()
        .find(|x| x.value == TOC_END)
        .and_then(|it| it.position.as_ref())?;
    let text_edit = TextEdit {
        range: Range {
            start: Position {
                line: (toc_start_pos.start.line - 1) as u32,
                character: 0,
            },
            end: Position {
                line: (toc_end_pos.start.line + 1) as u32,
                character: 0,
            },
        },
        new_text: toc,
    };
    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    changes.entry(req_uri.clone()).or_default().push(text_edit);
    let workspace_edit = WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    };
    Some(CodeAction {
        title: "Update Table of Contents".to_string(),
        kind: None,
        diagnostics: None,
        edit: Some(workspace_edit),
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    })
}
