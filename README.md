# Cliquers

Manage filesequences with a common numeric component in Rust.

## Usage

```rust
use cliquers

let files = vec![
    "/show/sequence/shot/task/main_v001/render.1001.exr",
    "/show/sequence/shot/task/main_v001/render.1002.exr",
    "/show/sequence/shot/task/main_v001/render.1003.exr",
    "/show/sequence/shot/task/main_v001/render.1004.exr",
    "/show/sequence/shot/task/main_v001/render.1005.exr",
];
let collections = cliquers.assemble(&files, None);
let c = &collections[0];

// access structure of file sequence
assert_eq!(c.head, "/show/sequence/shot/task/main_v001/render.");
assert_eq!(c.tail, ".exr");
assert_eq!(c.padding, 4);
assert_eq!(c.indexes, vec![1001, 1002, 1003, 1004, 1005]);
assert_eq!(
    c.format(None),
    "/show/sequence/shot/task/main_v001/render.%04d.exr [1001-1005]"
);
assert_eq!(
    c.format(Some("{head}####{tail}")),
    "/show/sequence/shot/task/main_v001/render.####.exr"
);

// iterate over files of filesequence
let iter = c.into_iter();
assert_eq!(
    iter.next(),
    Some("/show/sequence/shot/task/main_v001/render.1001.exr".to_string())
);
```

## Documentation

## Origin

This library is a direct port of the fantastic Python module [Clique](https://gitlab.com/4degrees/clique).

This is still a work in progress, so is not a complete port... yet!
