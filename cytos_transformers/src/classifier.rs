use cytos::{Prop, Stepper};
use cytos_derive::CytosNode;
use ort::{
    memory::Allocator,
    session::{run_options::OutputSelector, RunOptions, Session},
    value::Tensor,
};

use crate::decoder::Image;

#[derive(CytosNode, Default)]
pub struct BinaryClassifier {
    #[input]
    filename: Prop<String>,

    #[input]
    image: Prop<Image>,

    #[output]
    prediction: Prop<f32>,

    session: Option<Session>,
}

impl Stepper for BinaryClassifier {
    fn step(&mut self) -> cytos::Result<()> {
        if let Some(session) = &self.session {
            let width = self.image.image.width();
            let height = self.image.image.height();

            let input = &*self
                .image
                .image
                .iter()
                .map(|x| *x as f32 / 255.0)
                .collect::<Vec<f32>>();

            let input = Tensor::from_array(([1usize, width as usize, height as usize], input))?;

            let output = session.run(ort::inputs![input]?)?;
            let output = output[0].try_extract_tensor::<f32>()?.t().into_owned();
            dbg!(output);
        }
        Ok(())
    }

    fn initialize(&mut self) -> cytos::Result<()> {
        let session = Session::builder()?.commit_from_file(&*self.filename)?;

        let input = session.inputs.iter().next().ok_or("one input only")?;
        let input = Tensor::<f32>::new(
            &Allocator::default(),
            input
                .input_type
                .tensor_dimensions()
                .ok_or("missing tensor dimention")?
                .clone(),
        )?;

        let output = session.outputs.iter().next().ok_or("one output only")?;
        let output_tensor = Tensor::<f32>::new(
            &Allocator::default(),
            output
                .output_type
                .tensor_dimensions()
                .ok_or("missing tensor dimention")?
                .clone(),
        )?;

        let options = RunOptions::new().unwrap().with_outputs(
            // Disable all outputs...
            OutputSelector::no_default()
                // except for the first one...
                .with(&output.name)
                // and since this is a 2x upsampler model, pre-allocate the output to be twice as large.
                .preallocate(&output.name, output_tensor),
        );

        self.session = Some(session);

        Ok(())
    }

    fn terminate(&mut self) -> cytos::Result<()> {
        Ok(())
    }
}
