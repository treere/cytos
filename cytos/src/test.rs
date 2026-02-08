use std::collections::HashMap;

use crate::{MetadataProvider, NodeMetadata, ParamId, Prop, PropInspector, Result, Stepper};

/// An empty transformer implementation for testing purposes.
///
/// This struct implements both `Stepper` and `Transformer` traits but always returns errors
/// for operations, making it useful for testing error handling in the system.
#[derive(Default)]
pub struct Empty {}

impl Stepper for Empty {
    fn step(&mut self) -> Result<()> {
        todo!()
    }
}

impl PropInspector for Empty {
    fn get_prop(&self, _val: ParamId) -> Option<&dyn crate::props::GenericPropInterface> {
        None
    }

    fn get_prop_mut(
        &mut self,
        _val: ParamId,
    ) -> Option<&mut dyn crate::props::GenericPropInterface> {
        None
    }

    fn metadata(&self) -> &NodeMetadata {
        use std::sync::OnceLock;
        static METADATA: OnceLock<NodeMetadata> = OnceLock::new();
        METADATA.get_or_init(|| <Self as MetadataProvider>::metadata())
    }
}

impl MetadataProvider for Empty {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            name: "Empty".to_string(),
            description: "Test empty node".to_string(),
            input_ids: vec![],
            output_ids: vec![],
            params: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct Constant {
    input: Prop<i32>,
    output: Prop<i32>,
}

impl Stepper for Constant {
    fn step(&mut self) -> Result<()> {
        *self.output = *self.input;
        Ok(())
    }
}

impl PropInspector for Constant {
    fn get_prop(&self, val: ParamId) -> Option<&dyn crate::props::GenericPropInterface> {
        match val {
            ParamId(0) => Some(&self.input),
            ParamId(1) => Some(&self.output),
            _ => None,
        }
    }

    fn get_prop_mut(
        &mut self,
        val: ParamId,
    ) -> Option<&mut dyn crate::props::GenericPropInterface> {
        match val {
            ParamId(0) => Some(&mut self.input),
            ParamId(1) => Some(&mut self.output),
            _ => None,
        }
    }

    fn metadata(&self) -> &NodeMetadata {
        use std::sync::OnceLock;
        static METADATA: OnceLock<NodeMetadata> = OnceLock::new();
        METADATA.get_or_init(|| <Self as MetadataProvider>::metadata())
    }
}

impl MetadataProvider for Constant {
    fn metadata() -> NodeMetadata {
        use crate::{ParamDirection, ParamInfo};
        NodeMetadata {
            name: "Constant".to_string(),
            description: "Test constant node".to_string(),
            input_ids: vec![ParamId(0)],
            output_ids: vec![ParamId(1)],
            params: HashMap::from([
                (
                    ParamId(0),
                    ParamInfo {
                        name: "input".to_string(),
                        description: "Input value".to_string(),
                        direction: ParamDirection::Input,
                        type_name: "Prop<i32>".to_string(),
                    },
                ),
                (
                    ParamId(1),
                    ParamInfo {
                        name: "output".to_string(),
                        description: "Output value".to_string(),
                        direction: ParamDirection::Output,
                        type_name: "Prop<i32>".to_string(),
                    },
                ),
            ]),
        }
    }
}
