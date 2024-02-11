use lsp_types::{Position, Range, TextEdit, Url};

use crate::state::State;

pub fn formatting(req_uri: &Url, state: &State) -> Option<Vec<TextEdit>> {
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
    state.buffer_for_uri(req_uri).map(|doc| {
        let lines = doc.lines().count();
        let last_char = doc.lines().last().map(|it| it.chars().count());
        let formatted = dprint_plugin_markdown::format_text(doc, &config, |_, _, _| Ok(None));

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
