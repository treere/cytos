#![allow(unused)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use cytos::{
    MetadataProvider, NodeMetadata, PropInspector, Stepper,
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

    impl PropInspector for Add {
        fn get_prop(
            &self,
            _val: cytos::ParamId,
        ) -> Option<&dyn cytos::props::GenericPropInterface> {
            todo!()
        }

        fn get_prop_mut(
            &mut self,
            _val: cytos::ParamId,
        ) -> Option<&mut dyn cytos::props::GenericPropInterface> {
            todo!()
        }

        fn metadata(&self) -> &NodeMetadata {
            use std::sync::OnceLock;
            static METADATA: OnceLock<NodeMetadata> = OnceLock::new();
            METADATA.get_or_init(<Self as MetadataProvider>::metadata)
        }
    }

    impl MetadataProvider for Add {
        fn metadata() -> NodeMetadata {
            NodeMetadata {
                name: "Add".to_string(),
                description: "Benchmark add node".to_string(),
                params: vec![],
            }
        }
    }

    let mut registry = Registry::default();
    registry.add("add", Add::default);

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
        group.throughput(Throughput::Elements(*size));
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
