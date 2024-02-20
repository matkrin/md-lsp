use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, CompletionParams, Url};

use crate::{
    ast::{find_defintions, find_link_references},
    state::State,
};

pub fn completion(params: CompletionParams, state: &State) -> Option<CompletionList> {
    let req_uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    let context = params.context?;
    let trigger_kind = context.trigger_kind;
    let trigger_character = context.trigger_character;
    match trigger_character.as_deref() {
        Some("[") => link_ref_completion(&req_uri, state),
        _ => None,
    }
}

fn link_ref_completion(req_uri: &Url, state: &State) -> Option<CompletionList> {
    let ast = state.ast_for_uri(req_uri)?;
    let link_refs = find_link_references(ast);
    let definitions = find_defintions(ast);
    let def_completion_items = definitions
        .into_iter()
        .map(|def| CompletionItem {
            label: def.identifier.clone(),
            label_details: None,
            kind: Some(CompletionItemKind::TEXT),
            detail: Some(def.url.clone()),
            documentation: None,
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: None,
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            data: None,
            tags: None,
        })
        .collect();
    Some(CompletionList {
        is_incomplete: true,
        items: def_completion_items,
    })
}
