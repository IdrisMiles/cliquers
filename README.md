# Cliquers

![crates.io](https://img.shields.io/crates/v/$CRATE.svg)

Manage filesequences with a common numeric component in Rust.

## Usage

```rust
use cliquers

let files = vec![
    "/shot/task/main/render.1001.exr",
    "/shot/task/main/render.1002.exr",
    "/shot/task/main/render.1003.exr",
    "/shot/task/main/render.1004.exr",
    "/shot/task/main/render.1005.exr",
];
let (collections, remainders) = cliquers.assemble(&files, None);
let c = &collections[0];

// access structure of file sequence
assert_eq!(c.head, "/shot/task/main/render.");
assert_eq!(c.tail, ".exr");
assert_eq!(c.padding, 4);
assert_eq!(c.indexes, vec![1001, 1002, 1003, 1004, 1005]);
assert_eq!(c.format(None), "/shot/task/main/render.%04d.exr [1001-1005]");
assert_eq!(c.format(Some("{head}####{tail}")), "/shot/task/main/render.####.exr");

// iterate over files of filesequence
let mut iter = c.into_iter();
assert_eq!(iter.next(), Some("/shot/task/main/render.1001.exr".to_string()));
```

## Documentation

## Origin

This library is a direct port of the fantastic Python module [Clique](https://gitlab.com/4degrees/clique).

This is still a work in progress, so is not a complete port... yet!
