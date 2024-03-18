# Some Markdown features (github flavored)

<!--toc:start-->
- [Some Markdown features (github flavored)](#some-markdown-features-(github-flavored))
- [Heading 1](#heading-1)
  - [Heading 2](#heading-2)
    - [Heading 3](#heading-3)
    - [Subscript](#subscript)
    - [Superscript](#superscript)
    - [Lists](#lists)
      - [Unordered](#unordered)
      - [Ordered](#ordered)
      - [Nested](#nested)
    - [Task List](#task-list)
    - [Code](#code)
      - [Inline](#inline)
      - [Block](#block)
    - [Quotes](#quotes)
    - [Links](#links)
    - [Images](#images)
    - [Footnotes](#footnotes)
    - [Tables](#tables)
    - [Colors](#colors)
    - [Alerts](#alerts)
    - [Comments](#comments)
    - [LinkRefernce and Definition](#linkrefernce-and-definition)
<!--toc:end-->

This file just exist for testing autocompletion and linking to it.

# Heading 1
## Heading 2
### Heading 3

These can go up to six

**Bold Text**
__Also bold__
*italic*
_also italic_
~~strikethrough~~

### Subscript

H<sub>2</sub>O

### Superscript

<sup>a</sup> + <sup>b</sup> = <sup>c</sup>

### Lists

#### Unordered

- Foo
- Bar
- Baz

#### Ordered

1. Foo
2. Bar

#### Nested

1. ordered
   - unordered
     - unordered

### Task List

- [x] Butter
- [x] Flour
- [ ] Eggs

### Code

#### Inline

Inline `code` look like `this`.

#### Block

```python
import numpy as np

two = np.sqrt(4)
```

### Quotes

> Well begun is half done.

### Links

You can find this repository [here](https://github.com/matkrin/md-lsp)

### Images

![Alt text](path/to/image.png) Can also point to online images resource:
![Screenshot of a comment on a GitHub issue showing an image, added in the Markdown, of an Octocat smiling and raising a tentacle.](https://myoctocat.com/assets/images/base-octocat.svg)

### Footnotes

Here is a simple footnote[^1].

A footnote can also have multiple lines[^multi].


[^1]: My reference.

[^multi]: To add line breaks within a footnote, prefix new lines with 2 spaces. This
  is a second line.


### Tables

| Item | Quantity |
| ---- | -------: |
| Foo  |        5 |
| Bar  |       99 |
| Baz  |  1000000 |

### Colors

White in hex: `#ffffff`; black as rgb: `rgb(0, 0, 0)`

### Alerts

> [!NOTE]
> Useful information that users should know, even when skimming content.

> [!TIP]
> Helpful advice for doing things better or more easily.

> [!IMPORTANT]
> Key information users need to know to achieve their goal.

> [!WARNING]
> Urgent info that needs immediate user attention to avoid problems.

> [!CAUTION]
> Advises about risks or negative outcomes of certain actions.

### Comments

<!-- This is a comment and will not be rendered -->

### LinkRefernce and Definition

Search it on [duckduckgo] or [google][google-link].

[duckduckgo]: https://duckduckgo.com

[google-link]: https://google.com
