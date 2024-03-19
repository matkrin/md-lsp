# TODO

* [>] diagnostics (messages could be improved)
    - [x] Links to other document
    - [x] Links to heading in other document
    - [x] Links to heading in same file
    - [x] LinkReferences without Definition
    - [x] FootnoteReferences without FootnoteDefinition
    - [ ] Warning/Info for unused Definition
    - [ ] Warning/Info for unused FootnoteDefinition

* [>] code actions
    - [x] creating table of contents
    - [x] updating table of contents
    - [ ] Tables:
        - [ ] add column, left/ right
        - [ ] delete column
    - [ ] On Wikilink: replace with canonical link
    - [ ] Build HTML? maybe

* [x] parse Wikilinks, no support for `[[...|...]]` yet

* [x] hover:
    - [x] Headings[^heading]
    - [x] Link[^link]
    - [x] LinkReference[^link-ref]
    - [x] FootnoteReference[^footnote-ref]
    - [x] Wikilinks[^wikilink]

* [x] go to definition:
    - [x] Link (for Headings)
    - [x] LinkReference
    - [x] FootnoteReference
    - [x] Wikilinks (as Link)

* [x] find references:
    - [x] Headings (find references on heading also returns refs to file)
    - [x] Definition[^definition]
    - [x] FootnoteDefinition[^footnote-def]

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

* [x] completion
    - [x] for LinkReference with list of Definitions (triggered with `[`)
    - [x] for FootnoteReference with list of FootnoteDefinition (triggered
        with `[^`)
    - [x] Links (triggered with `](`)
    - [x] Wikilinks (triggered with `[[`)

[^heading]: Heading: `# ...`
[^link]: Link: `[...](...)`
[^link-ref]: LinkReference: `[...][...]`
[^footnote-ref]: FootnoteReference: `[^...]`
[^wikilink]: Wikilink: `[[...]]`
[^definition]: Definition: `[...]: ...`
[^footnote-def]: FootnoteDefinition: `[^...]: ...`
