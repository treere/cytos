#![allow(unused)]

use std::{collections::HashMap, time::Duration};

use cytos::{
    GraphId, MetadataProvider, NodeMetadata, Stepper, Transformer,
    loader::Registry,
    repr::{GraphRepr, InternalNodeRepr, NodeRepr, OnError, SystemRepr},
};

#[derive(Default)]
struct Add;

impl Stepper for Add {
    fn step(&mut self) -> cytos::Result<()> {
        Ok(())
    }
}

impl Transformer for Add {
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

impl MetadataProvider for Add {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            name: "Add".to_string(),
            description: "Example add node".to_string(),
            params: std::collections::HashMap::new(),
        }
    }
}

fn main() {
    let mut registry = Registry::default();
    registry.add("add", || Add::default());

    let nodes_count = 1000;

    let graph_repr = GraphRepr {
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
    };

    let g: GraphId = "g".to_owned().into();
    let mut system = SystemRepr {
        graphs: HashMap::from([(g, graph_repr)]),
        requests: vec![],
    }
    .to_system(&registry)
    .unwrap();

    system.graph(g).unwrap().start();
    std::thread::sleep(Duration::from_secs(60));
    system.graph(g).unwrap().stop();
}
