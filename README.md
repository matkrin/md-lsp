# md-lsp

## TODO

* [x] hover:
    - [x] Headings[^heading] 
    - [x] Link[^link]
    - [x] LinkReference[^link-ref]
    - [x] FootnoteReference[^footnote-ref]
    - [x] Wikilinks (as Link)[^wikilink], no support for `[[...|...]]` yet

* [x] parse Wikilinks

* [x] go to definition:
    - [x] Link (for Headings)
    - [x] LinkReference
    - [x] FootnoteReference
    - [x] Wikilinks (as Link)

* [x] find references:
    - [x] Headings (find references on heading also returns refs to file)
    - [x] Definition[^definition]
    - [x] FootnoteDefinition[^footnote-def]

* [x] diagnostics (messages could be improved)
    - [x] Links to other document
    - [x] Links to heading in other document
    - [x] Links to heading in same file
    - [x] LinkRefernces
    - [x] FootnoteRefernces

* [x] document symbols
* [x] workspace symbols
* [x] formatting:
    - [x] entire buffer
    - [x] ranged

* [x] rename
    - [x] Heading
    - [x] LinkReference
    - [x] Definition
    - [x] FootnoteReference
    - [x] FootnoteDefinition

* [x] code actions
    - [x] creating table of contents
    - [x] updating table of contents
    - [ ] Tables:
        - [ ] add row
        - [ ] delete row, more or less unnecessary
        - [ ] add column
        - [ ] delete column
 
* [x] completion
    - [x] for LinkReference with list of Definitions (triggered with `[`)
    - [x] for FootnoteReference with list of FootnoteDefinition (triggered
        with `^`)
    - [x] Links (needs refactoring)
    - [x] Wikilinks (needs refactoring)

[^heading]: Heading: `# ...`
[^link]: Link: `[...](...)`
[^link-ref]: LinkReference: `[...][...]`
[^footnote-ref]: FootnoteReference: `[^...]`
[^wikilink]: Wikilink: `[[...]]`
[^definition]: Definition: `[...]: ...`
[^footnote-def]: FootnoteDefinition: `[^...]: ...`
