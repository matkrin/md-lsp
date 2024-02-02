use lsp_types::{DocumentSymbol, SymbolKind};
use markdown::mdast::Node;

use crate::{
    ast::{find_headings, get_heading_text},
    definition::range_from_position,
};

pub fn document_symbols(req_ast: &Node) -> Option<Vec<DocumentSymbol>> {
    let mut headings = Vec::new();
    find_headings(req_ast, &mut headings);
    log::info!("HEADINGS: {:?}", headings);

    headings
        .into_iter()
        .map(|heading| {
            get_heading_text(heading).and_then(|heading_text| {
                heading.position.as_ref().map(|pos| {
                    let range = range_from_position(pos);
                    let name = add_pounds(heading_text, heading.depth);
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

fn add_pounds(heading_text: &str, depth: u8) -> String {
    let pounds = (0..depth).map(|_| '#').collect::<String>();
    format!("{pounds} {heading_text}")
}
