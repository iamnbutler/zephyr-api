# Zephyr API

A little api for serving syntax trees.

Will be used to power the Zephyr figma plugin for creating accurate syntax highlighted code blocks with individually selectable nodes.

You can hit the api with a string, and it will return a syntax tree in json format.

Currently only supports rust.

## Try Me Out

You can send a POST request to `zephyr-api.vercel.app/api/highlight` like this:

```json
{
  "code": "fn process_string(code: &str) -> Vec<Node> {\\n    let mut parser = Parser::new();\\n}"
}
```

You should get back a result like this:

```json
[
  [
    {
      "node_type": "keyword",
      "element_text": "fn"
    },
    {
      "node_type": "function.definition",
      "element_text": "process_string"
    },
    // more lines ...
  ],
]
```

---

## TODO
- [x] Create a rust syntax tree from a string
- [x] Create a json representation of the syntax tree
- [x] Create a web api to serve the syntax tree
- [ ] Allow specifying the language
- [ ] Allow specifying a static theme (instead of using tokens)

## Polish
- [ ] Look into injections to correctly highlight macro invocations, etc

Thanks @nathansobo and @maxbrunsfeld for helping me get this off the ground 🙏🏽
