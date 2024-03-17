# Working Links

All headings and links should be hoverable and show no diagnostics. Find
references should work for all headings and jump to definition should work for
all headings.

## As autocompleted

[[/testdata/test_file_a#Working Links]]

[working links](/testdata/test_file_a.md#working-links)

[to test file c](/testdata/test%20file%20c.md#some-markdown-features-(github-flavored))

[to test file c](/testdata/test%20file%20c.md#heading-1)

## To internal headings

[[#As autocompleted]]

[](#to-internal-headings)

## To another file

[[/README]]

[[/README#Features]]

[Link to README](/README.md)

[Link to README, heading feature](/README.md#features)

[[/testdata/test-file-b#Working Footnotes]] [[/testdata/test file c#Lists]]

[Link to test file b, working footnotes](/testdata/test-file-b.md#working-footnotes)

[with escaping](/testdata/test%20file%20c.md#code)

# Broken links

These links should all show diagnostics.

## To non-existent heading

[[#Non existent]]

[Try but](#non-existent)

## To non-existent file

[[abc]] [](abc.md)

## Wrong format

[[testdata/test_file_a#Working-Links]]

[](testdata/test_file_a#Working Links)

## Should be url-encoded:

[Link to test file c, code](/testdata/test file c.md#code)

[Link to test file c, code](/testdata/test-file-c.md#code)
