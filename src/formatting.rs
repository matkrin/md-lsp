use anyhow::Result;
use lsp_types::{DocumentFormattingParams, DocumentRangeFormattingParams, Position, Range, TextEdit};

use crate::state::State;

pub fn formatting(params: &DocumentFormattingParams, state: &State) -> Option<Vec<TextEdit>> {
    let req_uri = &params.text_document.uri;

    state.buffer_for_uri(req_uri).map(|doc| {
        let lines = doc.lines().count();
        let last_char = doc.lines().last().map(|it| it.chars().count());
        let formatted = format_md(doc);

        if let (Some(last_char), Ok(Some(form))) = (last_char, formatted) {
            let range = range_for_full_doc(lines as u32, last_char as u32);
            Some(vec![TextEdit {
                range,
                new_text: form,
            }])
        } else {
            None
        }
    })?
}

pub fn range_formatting(params: &DocumentRangeFormattingParams, state: &State) -> Option<Vec<TextEdit>> {
    let req_uri = &params.text_document.uri;
    let req_range = &params.range;

    state.buffer_range_for_uri(req_uri, req_range).map(|text| {
        log::info!("TEXT TO FORMAT: {:?}", text);
        let formatted = format_md(&text);
        if let Ok(Some(form)) = formatted {
            let range = *req_range;
            Some(vec![TextEdit {
                range,
                new_text: form,
            }])
        } else {
            None
        }
    })?
}

fn format_md(md: &str) -> Result<Option<String>> {
    let global_config = dprint_core::configuration::GlobalConfiguration {
        line_width: Some(80),
        use_tabs: Some(true),
        indent_width: Some(4),
        new_line_kind: Some(dprint_core::configuration::NewLineKind::Auto),
    };
    let config = dprint_plugin_markdown::configuration::ConfigurationBuilder::new()
        .global_config(global_config)
        .text_wrap(dprint_plugin_markdown::configuration::TextWrap::Always)
        .build();
    dprint_plugin_markdown::format_text(md, &config, |_, _, _| Ok(None))
}

fn range_for_full_doc(num_lines: u32, last_char: u32) -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: num_lines,
            character: last_char,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_for_full_doc() {
        let res = range_for_full_doc(10, 10);
        assert_eq!(res.start.line, 0);
        assert_eq!(res.start.character, 0);
        assert_eq!(res.end.line, 10);
        assert_eq!(res.end.character, 10);
    }
}
