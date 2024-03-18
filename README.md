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


## Installaion

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
