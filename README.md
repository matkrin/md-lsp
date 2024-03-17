# md-lsp

Markdown language server

## Features

* Hover:
    - **Heading**: shows outline of *Headings* with the current one marked 
    - **Link**: shows preview of destination file / *Heading* in destination file
    - **LinkReference**: shows its *Definition*
    - **FootnoteReference**: shows its *Definition*
    - **Wikilink**: shows preview of destination file / heading in destination file

* Go to definition:
    - **Link**: go to destination file / *Heading* in destination file
    - **LinkReference**: go to its *Definition*
    - **FootnoteReference**: go to its *Definition*
    - **Wikilink**: go to destination file / *Heading* in destination file

* Find references:
    - **Heading**: find all *Links* that reference this *Heading*
    - **Definition**: find all *LinkReferences* that reference this *Definition*
    - **FootnoteDefinition**: find all *FootnoteReferences* that reference this *FootnoteDefinition*

* Diagnostics:
    - **Links** to other document
    - **Links** to *Heading* in other document
    - **Links** to *Heading* in same file
    - **LinkReferences**
    - **FootnoteRefernces**

* Document symbols: shows all *Headings* in a document

* Workspace symbols: shows all *Headings* of the documents in the workspace

* Formatting:
    - entire file
    - only selection

* Rename
    - **Heading**: updates all *LinkReferences* that reference the *Heading*
    - **LinkReference**: updates its *Definition*
    - **Definition**: updates all *LinkReferences* that reference the *Definition*
    - **FootnoteReference**: update its *FootnoteDefinition*
    - **FootnoteDefinition**: updates all *FootnoteReferences* that reference the *FootnoteDefinition*

* Code actions:
    - create table of contents
    - update table of contents
 
* Autocompletion:
    - **Link**: shows list of *Headings* in current file / other file in workspace with *Headings*
    - **LinkReference**: shows list of *Definitions*
    - **FootnoteReference**: shows list of *FootnoteDefinitions*
    - **Wikilink**: shows list of *Headings* in current file / other file in workspace with *Headings*


## TODO

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
