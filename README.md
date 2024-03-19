# md-lsp

Markdown language server with support for [github flavored markdown][gfm].

[gfm]: https://github.github.com/gfm/

## Features

- **Hover**:
  - Heading: shows outline of _Headings_ with the current one marked
  - Link: shows preview of destination file / _Heading_ in destination file
  - LinkReference: shows its _Definition_
  - FootnoteReference: shows its _Definition_
  - Wikilink: shows preview of destination file / heading in destination file

- **Go to definition**:
  - Link: go to destination file / _Heading_ in destination file
  - LinkReference: go to its _Definition_
  - FootnoteReference: go to its _Definition_
  - Wikilink: go to destination file / _Heading_ in destination file

- **Find references**:
  - Heading: find all _Links_ that reference this _Heading_
  - Definition: find all _LinkReferences_ that reference this _Definition_
  - FootnoteDefinition: find all _FootnoteReferences_ that reference this
    _FootnoteDefinition_

- **Diagnostics**:
  - Links to other document
  - Links to _Heading_ in other document
  - Links to _Heading_ in same file
  - LinkReferences
  - FootnoteRefernces

- **Document symbols**: shows all _Headings_ in a document

- **Workspace symbols**: shows all _Headings_ of the documents in the workspace

- **Formatting**:
  - entire file
  - only selection

- **Rename**:
  - Heading: updates all _LinkReferences_ that reference the _Heading_
  - LinkReference: updates its _Definition_
  - Definition: updates all _LinkReferences_ that reference the _Definition_
  - FootnoteReference: update its _FootnoteDefinition_
  - FootnoteDefinition: updates all _FootnoteReferences_ that reference the
    _FootnoteDefinition_

- **Code actions**:
  - create table of contents
  - update table of contents

- **Autocompletion**:
  - Link: shows list of _Headings_ in current file / other file in workspace
    with _Headings_
  - LinkReference: shows list of _Definitions_
  - FootnoteReference: shows list of _FootnoteDefinitions_
  - Wikilink: shows list of _Headings_ in current file / other file in workspace
    with _Headings_

## Installation

After cloning the repository, install with

```bash
$ cargo install --path .
```

## Setup

### Neovim

With lspconfig:

```lua
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

configs.md_lsp = {
    default_config = {
        name = "md-lsp",
        cmd = { "md-lsp" },
        filetypes = { "markdown" },
        root_dir = lspconfig.util.root_pattern('.git'),
        single_file_support = true,
    },
}

lspconfig.md_lsp.setup({})
```

### Helix

In languages.toml:

```toml
[language-server.md-lsp]
command = "md-lsp"
roots = [".git"]

[[language]]
name = "markdown"
language-servers = [{ name = "md-lsp" }]
```
