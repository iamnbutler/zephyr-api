# Zephyr API

> **Note**
> The uh... actually api portion is still a work in progress.

A little api for serving syntax trees.

Will be used to power the Zephyr figma plugin for creating accurate syntax highlighted code blocks with individually selectable nodes.

You can hit the api with a string, and it will return a syntax tree in json format.

Currently only supports rust.

---

The API portion of this is still WIP.

But you can highlight static strings for now by replacing STATIC_CODE_TO_HIGHLIGHT and running a `cargo run` in the root folder.

The syntax tree will be output to `dist/output.json`.

## TODO
- [x] Create a rust syntax tree from a string
- [x] Create a json representation of the syntax tree
- [ ] Create a web api to serve the syntax tree
- [ ] Allow specifying the language
- [ ] Allow specifying a static theme (instead of using tokens)

## Polish
- [ ] Look into injections to correctly highlight macro invocations, etc

Thanks @nathansobo and @maxbrunsfeld for helping me get this off the ground 🙏🏽
