litcode
===
A collection of literate programming source files, with a tool `make_html` to generate
an html file from source code, using [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark/)
for markdown and [syntect](https://github.com/trishume/syntect) for source highlighting.

# Run

The html files can be generated with:
```
cargo run --bin make_html
```

The source files can be run with
```
cargo run --bin litcode
```
or run the tests with
```
cargo test
```

# License

This project is licensed under the [MIT License](LICENSE)
