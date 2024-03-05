# md-lsp

Markdown language server

## Features

* Hover:
    - Headings[^heading]: shows outline of *Heading*s with the current one marked 
    - Link[^link]: shows preview of destination file / *Heading* in destination file
    - LinkReference[^link-ref]: shows its *Definition*
    - FootnoteReference[^footnote-ref]: shows its *Definition*
    - Wikilinks[^wikilink]: shows preview of destination file / heading in destination file

* Go to definition:
    - Link[^link]: go to destination file / *Heading* in destination file
    - LinkReference[^link-ref]: go to its *Definition*
    - FootnoteReference[^footnote-ref]: go to its *Definition*
    - Wikilinks[^wikilink]: go to destination file / *Heading* in destination file

* Find references:
    - Headings[^heading]: find all *Link*s that reference this *Heading*
    - Definition[^definition]: find all *LinkReferences* that reference this *Definition*
    - FootnoteDefinition[^footnote-def]: find all *FootnoteReference*s that reference this *FootnoteDefinition*

* Diagnostics:
    - Links[^link] to other document
    - Links to *Heading* in other document
    - Links to *Heading* in same file
    - LinkReferences[^link-ref]
    - FootnoteRefernces[^footnote-ref]

* Document symbols: shows all *Heading*s in a document

* Workspace symbols: shows all *Heading*s of the documents in the workspace

* Formatting:
    - entire file
    - only selection

* Rename
    - Heading[^heading]: updates all *LinkReferences* that reference the *Heading*
    - LinkReference[^link-ref]: updates its *Definition*
    - Definition[^definition]: updates all *LinkReference*s that reference the *Definition*
    - FootnoteReference[^footnote-ref]: update its *FootnoteDefinition*
    - FootnoteDefinition[^footnote-def]: updates all *FootnoteReference*s that reference the *FootnoteDefinition*

* Code actions:
    - create table of contents
    - update table of contents
 
* Autocompletion:
    - LinkReference[^link-ref]: shows list of *Definition*s
    - FootnoteReference[^footnote-ref]: shows list of *FootnoteDefinition*s
    - Links[^link]: shows list of *Heading*s in current file / other file in workspace with *Heading*s
    - Wikilinks[^wikilink]: shows list of *Heading*s in current file / other file in workspace with *Heading*s


## TODO

- improve diagnostics

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
    - [ ] Build HTML
    - [ ] On Wikilink: replace with canonical link
 
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
