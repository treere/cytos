use std::f64::consts::PI;
fn norm_coeff(n: usize) -> f64 {
    if n == 0 {
        1.0 / 2.0_f64.sqrt()
    } else {
        1.0
    }
}

pub struct IDCT {
    coeffs: [f64; 64],
}

impl Default for IDCT {
    fn default() -> Self {
        let precision = 8;
        let mut coeffs = [0.0; 64];
        for u in 0..precision {
            for x in 0..precision {
                coeffs[u * 8 + x] =
                    norm_coeff(u) * (((2.0 * x as f64 + 1.0) * u as f64 * PI) / 16.0).cos();
            }
        }
        Self { coeffs }
    }
}
impl IDCT {
    pub fn perform_idct(&self, input: &[i32; 64], output: &mut [i32; 64]) {
        for x in 0..8 {
            for y in 0..8 {
                let mut local_sum = 0.0;
                for u in 0..8 {
                    for v in 0..8 {
                        local_sum += input[v * 8 + u] as f64
                            * self.coeffs[u * 8 + x]
                            * self.coeffs[v * 8 + y];
                    }
                }
                output[y * 8 + x] = local_sum as i32 / 4 - 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name() {
        let idct = IDCT::default();
        let input = [
            -261, 50, 2, -3, 0, 0, 0, 0, //
            14, 6, -6, 0, -5, 0, 0, 0, //
            3, -15, 3, 5, 0, 0, 0, 0, //
            0, 3, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, //
        ];
        let output = [
            -24, -23, -26, -30, -32, -33, -36, -39, //
            -24, -23, -25, -29, -33, -35, -37, -40, //
            -24, -23, -24, -29, -34, -38, -41, -42, //
            -23, -23, -24, -29, -36, -41, -44, -44, //
            -23, -24, -26, -31, -37, -43, -45, -45, //
            -24, -26, -29, -32, -38, -42, -44, -43, //
            -26, -30, -33, -35, -37, -40, -40, -39, //
            -28, -32, -36, -36, -37, -38, -38, -36, //
        ];

        let mut out = [0; 64];
        idct.perform_idct(&input, &mut out);

        assert_eq!(out, output);
    }
}
