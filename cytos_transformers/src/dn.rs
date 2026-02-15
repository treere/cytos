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

/// Sigmoid backward pass: dL/dx = dL/dy * σ(x) * (1 - σ(x))
#[derive(CytosNode, Default)]
pub struct SigmoidBackward {
    /// Output from forward pass σ(x)
    #[cytos(input)]
    x: Prop<Vec<f32>>,
    /// Gradient from downstream dL/dy
    #[cytos(input)]
    gy: Prop<Vec<f32>>,
    /// Gradient to upstream dL/dx
    #[cytos(output)]
    gx: Prop<Vec<f32>>,
}

impl Stepper for SigmoidBackward {
    fn step(&mut self) -> Result<()> {
        let x = &*self.x;
        let gy = &*self.gy;

        let gx: Vec<f32> = x
            .iter()
            .zip(gy.iter())
            .map(|(&x_val, &g_val)| {
                let sig = 1.0 / (1.0 + (-x_val).exp());
                g_val * sig * (1.0 - sig)
            })
            .collect();

        *self.gx = gx;
        Ok(())
    }
}

/// Linear backward pass: computes gradients for weights, bias, and input
#[derive(CytosNode, Default)]
pub struct LinearBackward {
    /// Input from forward pass x
    #[cytos(input)]
    x: Prop<Vec<f32>>,
    /// Weights from forward pass W
    #[cytos(input)]
    w: Prop<Vec<f32>>,
    /// Gradient from downstream dL/dy
    #[cytos(input)]
    gy: Prop<Vec<f32>>,
    /// Gradient to upstream dL/dx
    #[cytos(output)]
    gx: Prop<Vec<f32>>,
    /// Gradient for weights dL/dW
    #[cytos(output)]
    gw: Prop<Vec<f32>>,
    /// Gradient for bias dL/db
    #[cytos(output)]
    gb: Prop<Vec<f32>>,
}

impl Stepper for LinearBackward {
    fn step(&mut self) -> Result<()> {
        let x = &*self.x;
        let w = &*self.w;
        let gy = &*self.gy;

        let input_size = x.len();
        let output_size = gy.len();

        if w.len() != input_size * output_size {
            return Err(format!(
                "expected {} weights, got {}",
                input_size * output_size,
                w.len()
            )
            .into());
        }

        // dL/dx = dL/dy @ W
        let mut gx = vec![0.0f32; input_size];
        for i in 0..input_size {
            let mut sum = 0.0;
            for j in 0..output_size {
                sum += gy[j] * w[j * input_size + i];
            }
            gx[i] = sum;
        }

        // dL/dW = dL/dy.T @ x
        let mut gw = vec![0.0f32; input_size * output_size];
        for j in 0..output_size {
            for i in 0..input_size {
                gw[j * input_size + i] = gy[j] * x[i];
            }
        }

        // dL/db = dL/dy
        let gb = gy.clone();

        *self.gx = gx;
        *self.gw = gw;
        *self.gb = gb;
        Ok(())
    }
}

/// Mean Squared Error loss: L = (1/n) * Σ(`y_pred` - `y_true`)²
#[derive(CytosNode, Default)]
pub struct Mse {
    /// Predicted values
    #[cytos(input)]
    pred: Prop<Vec<f32>>,
    /// Target values
    #[cytos(input)]
    target: Prop<Vec<f32>>,
    /// Loss value
    #[cytos(output)]
    loss: Prop<f32>,
    /// Gradient w.r.t prediction `dL/dy_pred`
    #[cytos(output)]
    gpred: Prop<Vec<f32>>,
}

impl Stepper for Mse {
    fn step(&mut self) -> Result<()> {
        let pred = &*self.pred;
        let target = &*self.target;

        if pred.len() != target.len() {
            return Err(format!(
                "prediction and target must have same length, got {} and {}",
                pred.len(),
                target.len()
            )
            .into());
        }

        let n = pred.len() as f32;
        let mut sum = 0.0;
        let mut gpred = vec![0.0f32; pred.len()];

        for i in 0..pred.len() {
            let diff = pred[i] - target[i];
            sum += diff * diff;
            gpred[i] = 2.0 / n * diff;
        }

        *self.loss = sum / n;
        *self.gpred = gpred;
        Ok(())
    }
}

/// Stochastic Gradient Descent weight update: W = W - lr * dL/dW
#[derive(CytosNode, Default)]
pub struct Sgd {
    /// Current weights
    #[cytos(input)]
    w: Prop<Vec<f32>>,
    /// Current bias
    #[cytos(input)]
    b: Prop<Vec<f32>>,
    /// Gradient for weights
    #[cytos(input)]
    gw: Prop<Vec<f32>>,
    /// Gradient for bias
    #[cytos(input)]
    gb: Prop<Vec<f32>>,
    /// Learning rate
    #[cytos(input)]
    lr: Prop<f32>,
    /// Updated weights
    #[cytos(output)]
    uw: Prop<Vec<f32>>,
    /// Updated bias
    #[cytos(output)]
    ub: Prop<Vec<f32>>,
}

impl Stepper for Sgd {
    fn step(&mut self) -> Result<()> {
        let w = &*self.w;
        let b = &*self.b;
        let gw = &*self.gw;
        let gb = &*self.gb;
        let lr = *self.lr;

        if w.len() != gw.len() {
            return Err(format!(
                "weights and gw must have same length, got {} and {}",
                w.len(),
                gw.len()
            )
            .into());
        }

        if b.len() != gb.len() {
            return Err(format!(
                "bias and gb must have same length, got {} and {}",
                b.len(),
                gb.len()
            )
            .into());
        }

        let uw: Vec<f32> = w
            .iter()
            .zip(gw.iter())
            .map(|(&wt, &g)| lr.mul_add(-g, wt))
            .collect();

        let ub: Vec<f32> = b
            .iter()
            .zip(gb.iter())
            .map(|(&bt, &g)| lr.mul_add(-g, bt))
            .collect();

        *self.uw = uw;
        *self.ub = ub;
        Ok(())
    }
}

/// Generates random vectors from a normal (Gaussian) distribution
#[derive(CytosNode, Default)]
pub struct NormalDistribution {
    /// Mean of the distribution
    #[cytos(input)]
    mean: Prop<f32>,
    /// Standard deviation of the distribution
    #[cytos(input)]
    std: Prop<f32>,
    /// Size of the output vector
    #[cytos(input)]
    size: Prop<usize>,
    /// Output vector (random sample)
    #[cytos(output)]
    output: Prop<Vec<f32>>,
}

impl Stepper for NormalDistribution {
    fn step(&mut self) -> Result<()> {
        let mean = *self.mean;
        let std = *self.std;
        let size = *self.size;

        let mut rng = rand::thread_rng();
        let mut output = Vec::with_capacity(size);

        for _ in 0..size {
            let u1: f32 = rand::Rng::r#gen(&mut rng);
            let u2: f32 = rand::Rng::r#gen(&mut rng);
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
            output.push(std.mul_add(z, mean));
        }

        *self.output = output;
        Ok(())
    }
}

pub fn load_registry(registry: &mut cytos::loader::DynamicLoadingRegistryWrapper) {
    registry.add("Linear", Linear::default);
    registry.add("Sigmoid", Sigmoid::default);
    registry.add("SigmoidBackward", SigmoidBackward::default);
    registry.add("LinearBackward", LinearBackward::default);
    registry.add("Mse", Mse::default);
    registry.add("Sgd", Sgd::default);
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

    #[test]
    fn test_mse() {
        let mut mse = Mse::default();

        *mse.pred = vec![1.0, 2.0, 3.0];
        *mse.target = vec![1.0, 2.0, 3.0];

        mse.step().unwrap();

        assert!((*mse.loss - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_sgd() {
        let mut sgd = Sgd::default();

        *sgd.w = vec![1.0, 2.0, 3.0, 4.0];
        *sgd.b = vec![0.5, 0.5];
        *sgd.gw = vec![0.1, 0.1, 0.1, 0.1];
        *sgd.gb = vec![0.1, 0.1];
        *sgd.lr = 0.1;

        sgd.step().unwrap();

        let uw = &*sgd.uw;
        let ub = &*sgd.ub;

        assert_eq!(uw.len(), 4);
        assert!((uw[0] - 0.99).abs() < 1e-6);
        assert!((ub[0] - 0.49).abs() < 1e-6);
    }
}
