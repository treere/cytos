#![allow(unused)]

use std::collections::HashMap;

use cytos::{
    NodeMetadata, PropInspector, Stepper,
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

impl PropInspector for Add {
    fn get_prop(&self, _val: cytos::ParamId) -> Option<&dyn cytos::props::GenericPropInterface> {
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
        METADATA.get_or_init(<Self as cytos::MetadataProvider>::metadata)
    }
}

impl cytos::MetadataProvider for Add {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            name: "Add".to_string(),
            description: "Example add node".to_string(),
            params: vec![],
        }
    }
}

fn main() {
    let mut registry = Registry::default();
    registry.add("add", Add::default);

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
