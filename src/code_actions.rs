use std::collections::HashMap;

use lsp_types::{CodeAction, Position, Range, TextEdit, Url, WorkspaceEdit};

use crate::{
    ast::{find_headings, find_html_nodes},
    rename::get_text_child,
    state::State,
};

const TOC_START: &str = "<!--toc:start-->";
const TOC_END: &str = "<!--toc:end-->";

// first edit should be enough new lines that content pushed down, second edit actual content
pub fn code_actions(req_uri: &Url, state: &State) -> Option<Vec<CodeAction>> {
    let ast = state.ast_for_uri(req_uri)?;
    let headings = find_headings(ast);
    let first_heading_line = headings.first()?.position.as_ref()?.start.line;
    let contains_toc = find_html_nodes(ast)
        .into_iter()
        .map(|html| html.value.clone())
        .any(|val| val == TOC_START);

    let toc = headings
        .into_iter()
        .fold(String::new(), |mut acc, heading| {
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
    let toc = format!("{}\n{}{}\n\n", TOC_START, toc, TOC_END);
    let title = "Create Table of Contents".to_string();
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
    let code_action = CodeAction {
        title,
        kind: None,
        diagnostics: None,
        edit: Some(workspace_edit),
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    };
    // }
    // todo!()
    Some(vec![code_action])
}
