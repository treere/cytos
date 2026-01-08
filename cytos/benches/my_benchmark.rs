#![allow(unused)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use cytos::{
    MetadataProvider, NodeMetadata, Stepper, Transformer,
    graph::Graph,
    loader::Registry,
    repr::{GraphRepr, InternalNodeRepr, NodeRepr, OnError},
};
use std::{collections::HashMap, hint::black_box};

fn create_graph(nodes_count: u64) -> Graph {
    #[derive(Default)]
    struct Add;

    impl Stepper for Add {
        fn step(&mut self) -> cytos::Result<()> {
            Ok(())
        }
    }

    impl Transformer for Add {
        fn link(
            &mut self,
            name: cytos::ParamId,
            val: cytos::props::GenericProp,
        ) -> cytos::Result<()> {
            todo!()
        }

        fn load(&mut self, name: cytos::ParamId, val: cytos::Value) -> cytos::Result<()> {
            todo!()
        }

        fn assign(&mut self, name: cytos::ParamId, val: cytos::Value) -> cytos::Result<()> {
            todo!()
        }

        fn dump(&self, name: cytos::ParamId) -> cytos::Result<cytos::Value> {
            todo!()
        }

        fn load_owned(
            &mut self,
            name: cytos::ParamId,
            val: cytos::GenericOwnedProp,
        ) -> cytos::Result<()> {
            todo!()
        }

        fn assign_owned(
            &mut self,
            name: cytos::ParamId,
            val: cytos::GenericOwnedProp,
        ) -> cytos::Result<()> {
            todo!()
        }

        fn dump_owned(&self, name: cytos::ParamId) -> cytos::Result<cytos::GenericOwnedProp> {
            todo!()
        }

        fn output(&self, val: cytos::ParamId) -> Option<cytos::props::GenericProp> {
            todo!()
        }

        fn input(&self, val: cytos::ParamId) -> Option<cytos::props::GenericProp> {
            todo!()
        }

        fn input_names(&self) -> Vec<cytos::ParamId> {
            todo!()
        }

        fn output_names(&self) -> Vec<cytos::ParamId> {
            todo!()
        }
    }

    impl MetadataProvider for Add {
        fn metadata() -> NodeMetadata {
            NodeMetadata {
                name: "Add".to_string(),
                description: "Benchmark add node".to_string(),
                params: HashMap::new(),
            }
        }
    }

    let mut registry = Registry::default();
    registry.add("add", || Add::default());

    GraphRepr {
        links: vec![],
        nodes: (0..nodes_count)
            .map(|id| InternalNodeRepr {
                name: format!("{}", id).into(),
                on_error: OnError::default(),
                node: NodeRepr {
                    typ: "add".to_string(),
                    props: HashMap::default(),
                },
            })
            .collect(),
    }
    .into_graph(&registry)
    .unwrap()
}

fn graph_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph size");
    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut graph = create_graph(size);
            graph.initialize();
            b.iter(|| black_box(graph.step()))
        });
    }
    group.finish();
}

criterion_group!(benches, graph_size);
criterion_main!(benches);
