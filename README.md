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
* [.] formatting:
    - [x] entire buffer
    - [ ] ranged

* [ ] rename
    - [ ] Heading
    - [ ] LinkReference
    - [ ] Definition
    - [ ] FootnoteReference
    - [ ] FootnoteDefinition

* [ ] completion
* [ ] code actions


[^heading]: `# Heading`
[^link]: `[...](...)`
[^link-ref]: `[...][...]`
[^footnote-ref]: `[^...]`
[^wikilink]: `[[...]]`
[^definition]: `[...]: ...`
[^footnote-def]: `[^...]: ...`
