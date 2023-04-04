# Zephyr Server

A little api for serving syntax trees.

You can hit the api with a string, and it will return a syntax tree in json format.

Currently only supports rust.

## TODO
- [ ] Create a rust syntax tree from a string
- [ ] Create a json representation of the syntax tree
- [ ] Create a web api to serve the syntax tree
- [ ] Allow specifying the language
- [ ] Allow specifying a static theme (instead of using tokens)

## Polish
- [ ] Look into injections to correctly highlight macro invocations, etc
