# Cytos: Lightweight Processing Pipelines

Cytos is a Rust library that provides a lightweight foundation for building processing pipelines. It's designed to have minimal overhead, making it suitable for high-performance applications. Cytos allows you to create multiple graphs that can run concurrently in different threads, enabling efficient and scalable processing.

## Overview

Cytos provides a set of APIs and data structures for building and executing processing pipelines. The library is designed to be highly customizable, allowing you to define your own node and edge types, as well as custom processing logic.

## Key Features

* **Low-Overhead Architecture**: Cytos is designed to have minimal overhead, making it suitable for high-performance applications.
* **Multi-Graph Support**: Cytos allows you to create multiple graphs that can run concurrently in different threads.
* **Customizable Nodes and Edges**: Define your own node and edge types to suit your specific use case.
* **Custom Processing Logic**: Implement custom processing logic for each node and edge.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cytos = "0.1"
```

## Quick Start

```rust
use cytos::System;

// Create a processing system
let mut system = System::new();

// Create a graph for your pipeline
let graph_id = system.create_graph().unwrap();

// Add processing nodes and connect them...
```

See the [examples/](examples/) directory for complete examples.

## Architecture

Cytos uses a dataflow model where:

- **Systems** manage execution environments
- **Graphs** define processing pipelines
- **Nodes** contain the actual processing logic
- **Edges** route data between nodes

Graphs can run concurrently for scalable processing.

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details.
