# Working Links

All headings and links should be hoverable and show no diagnostics. Find
references should work for all headings and jump to definition should work for
all headings.

## As autocompleted

[[/testdata/test_file_a#Working Links]]

[](/testdata/test_file_a.md#working-links)

## To internal headings

[[#As autocompleted]]

[](#to-internal-headings)

## To another file

[[/README]]

[[/README#Features]]

[Link to README](/README.md)

[Link to README, heading feature](/README.md#features)

[[/testdata/test-file-b#Working Footnotes]]
[[/testdata/test file c#Lists]]

[Link to test file b, working footnotes](/testdata/test-file-b.md#working-footnotes)

[Link to test file c, code](/testdata/test-file-c.md#code)

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

[Link to test file c, code](/testdata/test file c.md#code)
