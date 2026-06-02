#![allow(unused)]

use std::{collections::HashMap, time::Duration};

use cytos::{
    GraphId, MetadataProvider, NodeMetadata, PropInspector, Stepper,
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
        METADATA.get_or_init(<Self as MetadataProvider>::metadata)
    }
}

impl MetadataProvider for Add {
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

    let graph_repr = GraphRepr {
        links: vec![],
        nodes: (0..nodes_count)
            .map(|id| InternalNodeRepr {
                name: format!("{}", id).into(),
                on_error: OnError::default(),
                node: NodeRepr {
                    typ: "add".to_string(),
                    ..Default::default()
                },
            })
            .collect(),
    };

    let g: GraphId = "g".to_owned().into();
    let mut system = SystemRepr {
        graphs: HashMap::from([(g, graph_repr)]),
        requests: vec![],
        ..Default::default()
    }
    .to_system(&registry)
    .unwrap();

    system.graph(g).unwrap().start();
    std::thread::sleep(Duration::from_secs(60));
    system.graph(g).unwrap().stop();
}
