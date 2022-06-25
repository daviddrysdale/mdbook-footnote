# mdbook-footnote

A preprocessor for [mdbook](https://github.com/rust-lang/mdBook) to support the inclusion of footnotes.

It turns this:

```text
Normal text{{footnote: Or is it?}} in body.
```

into:

> Normal text<sup><a name="to-footnote-1"><a href="#footnote-1">1</a></a></sup> in body.

with the footnotes accumulated at the bottom of the page, following an `<hr/>`.

## Installation

To use, install the tool

```sh
cargo install mdbook-footnote
```

and add it as a preprocessor in `book.toml`:

```toml
[preprocessor.footnote]
```

<p><hr/>
<p><a name="footnote-1"><a href="#to-footnote-1">1</a></a>: Or is it?</p>
