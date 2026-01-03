# Cytos Derive

Procedural macros for the Cytos dataflow framework. This crate provides the `CytosNode` derive macro, which automatically implements the `Transformer` trait for structs used as nodes in Cytos graphs.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cytos_derive = { path = "../cytos_derive" }
```

Then annotate your structs:

```rust
use cytos_derive::CytosNode;

#[derive(CytosNode)]
struct MyNode {
    #[cytos(input)]
    input1: cytos::props::GenericProp,
    #[cytos(output)]
    output1: cytos::props::GenericProp,
}
```

The macro generates implementations for parameter linking, loading, assignment, and dumping based on the field attributes.

## License

This project is part of the Cytos framework.