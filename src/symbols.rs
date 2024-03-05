use lsp_types::{DocumentSymbol, DocumentSymbolParams, Location, SymbolKind, WorkspaceSymbol};

use crate::{
    ast::{find_headings, get_heading_text},
    definition::range_from_position,
    state::State,
};

pub fn document_symbols(params: &DocumentSymbolParams, state: &State) -> Option<Vec<DocumentSymbol>> {
    let req_uri = &params.text_document.uri;
    let req_ast = state.ast_for_uri(req_uri)?;
    let headings = find_headings(req_ast);

    headings
        .into_iter()
        .map(|heading| {
            get_heading_text(heading).and_then(|heading_text| {
                heading.position.as_ref().map(|pos| {
                    let range = range_from_position(pos);
                    let name = add_pounds(heading_text, heading.depth);
                    #[allow(deprecated)]  // TODO: don't know how else
                    DocumentSymbol {
                        name,
                        detail: None,
                        kind: SymbolKind::STRING,
                        tags: None,
                        deprecated: None,
                        range,
                        selection_range: range,
                        children: None,
                    }
                })
            })
        })
        .collect()
}

pub fn workspace_symbols(state: &State) -> Option<Vec<WorkspaceSymbol>> {
    state
        .md_files
        .iter()
        .flat_map(|(url, md_file)| {
            let headings = find_headings(&md_file.ast);
            headings.into_iter().map(|heading| {
                get_heading_text(heading).and_then(|heading_text| {
                    heading.position.as_ref().map(|pos| {
                        let range = range_from_position(pos);
                        let name = add_pounds(heading_text, heading.depth);
                        let location = Location {
                            uri: url.clone(),
                            range,
                        };
                        WorkspaceSymbol {
                            name,
                            kind: SymbolKind::STRING,
                            tags: None,
                            container_name: None,
                            location: lsp_types::OneOf::Left(location),
                            data: None,
                        }
                    })
                })
            })
        })
        .collect()
}

fn add_pounds(heading_text: &str, depth: u8) -> String {
    let pounds = (0..depth).map(|_| '#').collect::<String>();
    format!("{pounds} {heading_text}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_pounds() {
        let res = add_pounds("hello", 1);
        assert_eq!(res, "# hello");
        let res = add_pounds("test", 3);
        assert_eq!(res, "### test");
    }
}
