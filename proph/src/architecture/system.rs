use crate::loader::{Registry, SystemRepr};

use super::{runner::{Command, Response, Runner}, GraphId, Result};

struct System {
    runners: Vec<(GraphId, Runner)>,
}

impl System {
    pub fn new(repr: SystemRepr, reg: Registry) -> Self {
        Self {
            runners: repr
                .graphs
                .into_iter()
                .map(|g| Runner::from_repr(g, reg.clone()))
                .collect(),
        }
    }

    pub fn command(&mut self, graph: GraphId, command: Command) -> Result<Response> {
        let v = self.runners.iter_mut().find(|x| x.0 == graph).ok_or("not found")?;
        v.1.command(command)
    }
}
