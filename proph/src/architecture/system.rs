use crate::loader::{Registry, SystemRepr};

use super::{runner::Runner, GraphId};

struct System {
    _runners: Vec<(GraphId,Runner)>,
}

impl System {
    pub fn new(repr: SystemRepr, reg: Registry) -> Self {
        Self {
            _runners: repr
                .graphs
                .into_iter()
                .map(|g| Runner::new(g, reg.clone()))
                .collect(),
        }
    }
}
