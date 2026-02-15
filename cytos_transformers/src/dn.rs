use cytos::{Prop, Result, Stepper};
use cytos_derive::CytosNode;

/// Fully connected linear layer: output = input @ weights.T + bias
#[derive(CytosNode, Default)]
pub struct Linear {
    /// Input vector
    #[cytos(input)]
    input: Prop<Vec<f32>>,
    /// Output vector
    #[cytos(output)]
    output: Prop<Vec<f32>>,
    /// Weight matrix (flat, row-major: `output_size` × `input_size`)
    #[cytos(input)]
    weights: Prop<Vec<f32>>,
    /// Bias vector (size = `output_size`)
    #[cytos(input)]
    bias: Prop<Vec<f32>>,
    /// Input size
    #[cytos(input)]
    inputsize: Prop<usize>,
    /// Output size
    #[cytos(input)]
    outputsize: Prop<usize>,
}

impl Stepper for Linear {
    fn step(&mut self) -> Result<()> {
        let input = &*self.input;
        let weights = &*self.weights;
        let bias = &*self.bias;
        let input_size = *self.inputsize;
        let output_size = *self.outputsize;

        if input.len() != input_size {
            return Err(format!("expected input size {}, got {}", input_size, input.len()).into());
        }
        if weights.len() != input_size * output_size {
            return Err(format!(
                "expected {} weights, got {}",
                input_size * output_size,
                weights.len()
            )
            .into());
        }
        if bias.len() != output_size {
            return Err(format!("expected {} bias, got {}", output_size, bias.len()).into());
        }

        let mut output = vec![0.0f32; output_size];
        for j in 0..output_size {
            let mut sum = bias[j];
            for i in 0..input_size {
                sum += input[i] * weights[j * input_size + i];
            }
            output[j] = sum;
        }

        *self.output = output;
        Ok(())
    }
}

/// Sigmoid activation: 1 / (1 + exp(-x))
#[derive(CytosNode, Default)]
pub struct Sigmoid {
    /// Input vector
    #[cytos(input)]
    input: Prop<Vec<f32>>,
    /// Output vector
    #[cytos(output)]
    output: Prop<Vec<f32>>,
}

impl Stepper for Sigmoid {
    fn step(&mut self) -> Result<()> {
        let input = &*self.input;
        let output: Vec<f32> = input.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect();
        *self.output = output;
        Ok(())
    }
}

pub fn load_registry(registry: &mut cytos::loader::DynamicLoadingRegistryWrapper) {
    registry.add("Linear", Linear::default);
    registry.add("Sigmoid", Sigmoid::default);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let mut linear = Linear::default();

        *linear.input = vec![1.0, 2.0, 3.0];
        *linear.weights = vec![
            1.0, 0.0, 0.0, // row 0
            0.0, 1.0, 0.0, // row 1
        ];
        *linear.bias = vec![0.5, 0.5];
        *linear.inputsize = 3;
        *linear.outputsize = 2;

        linear.step().unwrap();

        let output = &*linear.output;
        assert_eq!(output.len(), 2);
        assert!((output[0] - 1.5).abs() < 1e-6); // 1*1 + 2*0 + 3*0 + 0.5
        assert!((output[1] - 2.5).abs() < 1e-6); // 1*0 + 2*1 + 3*0 + 0.5
    }

    #[test]
    fn test_sigmoid() {
        let mut sigmoid = Sigmoid::default();

        *sigmoid.input = vec![0.0, 1.0, -1.0];

        sigmoid.step().unwrap();

        let output = &*sigmoid.output;
        assert_eq!(output.len(), 3);
        assert!((output[0] - 0.5).abs() < 1e-6); // sigmoid(0) = 0.5
        assert!((output[1] - 0.731058).abs() < 1e-6); // sigmoid(1) ≈ 0.731
        assert!((output[2] - 0.268941).abs() < 1e-6); // sigmoid(-1) ≈ 0.269
    }
}
