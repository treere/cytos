#![allow(unused)]

use std::collections::HashMap;

use cytos::{
    NodeMetadata, Stepper, Transformer,
    loader::Registry,
    repr::{GraphRepr, InternalNodeRepr, NodeRepr, OnError},
};

#[derive(Default)]
struct Add;

impl Stepper for Add {
    fn step(&mut self) -> cytos::Result<()> {
        Ok(())
    }
}

impl Transformer for Add {
    fn input_names(&self) -> Vec<cytos::ParamId> {
        todo!()
    }

    fn output_names(&self) -> Vec<cytos::ParamId> {
        todo!()
    }

    fn get_prop(&self, val: cytos::ParamId) -> Option<&dyn cytos::props::GenericPropInterface> {
        todo!()
    }

    fn get_prop_mut(
        &mut self,
        val: cytos::ParamId,
    ) -> Option<&mut dyn cytos::props::GenericPropInterface> {
        todo!()
    }
}

impl cytos::MetadataProvider for Add {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            name: "Add".to_string(),
            description: "Example add node".to_string(),
            params: HashMap::new(),
        }
    }
}

fn main() {
    let mut registry = Registry::default();
    registry.add("add", || Add::default());

    let nodes_count = 1000;

    let mut graph = GraphRepr {
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
    .unwrap();

    graph.initialize().unwrap();

    loop {
        graph.step().unwrap();
    }
}
