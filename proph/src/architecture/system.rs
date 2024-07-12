use crate::loader::{Registry, SystemRepr};

use super::{
    runner::{Command, Response, Runner},
    GraphId, Result,
};

#[derive(Default)]
pub struct System {
    runners: Vec<(GraphId, Runner)>,
}

impl System {
    pub fn command(&mut self, graph: GraphId, command: Command) -> Result<Response> {
        let v = self
            .runners
            .iter_mut()
            .find(|x| x.0 == graph)
            .ok_or("not found")?;
        v.1.command(command)
    }

    pub fn from_repr(repr: SystemRepr, loader: &Registry) -> Result<Self> {
        let v = repr
            .graphs
            .into_iter()
            .map(|x| Runner::from_repr(x, loader.clone()))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { runners: v })
    }

    pub fn keys(&self) -> impl Iterator<Item = &GraphId> {
        self.runners.iter().map(|(v, _)| v)
    }
}
