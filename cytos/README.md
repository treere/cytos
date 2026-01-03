# Cytos

The core library for building lightweight processing pipelines in Rust. Cytos provides a set of APIs and data structures for creating and executing processing pipelines with minimal overhead. It supports multiple concurrent graphs, customizable nodes and edges, and dynamic loading of transformers.

## Features

- **Low-Overhead Architecture**: Designed for high-performance applications
- **Multi-Graph Support**: Run multiple graphs concurrently in different threads
- **Customizable Nodes and Edges**: Define your own types to suit specific needs
- **Dynamic Loading**: Load transformers at runtime using libloading

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cytos = { path = "../cytos" }
```

## License

This project is part of the Cytos framework.