//! A small multilayer perceptron for inference, in plain Rust so it runs identically natively and
//! in WebAssembly. Weights come from `train/` (PyTorch) in a simple binary format:
//!
//! `b"FHNN"`, u32 layer count, then per layer: u32 inputs, u32 outputs, outputs×inputs f32
//! weights (row-major), outputs f32 biases. All little-endian. ReLU between layers.

pub struct Mlp {
    layers: Vec<Layer>,
}

struct Layer {
    inputs: usize,
    outputs: usize,
    weights: Vec<f32>,
    bias: Vec<f32>,
}

impl Mlp {
    pub fn from_bytes(bytes: &[u8]) -> Result<Mlp, String> {
        let mut at = 0usize;
        let mut take = |n: usize| -> Result<&[u8], String> {
            let slice = bytes.get(at..at + n).ok_or("model file is truncated")?;
            at += n;
            Ok(slice)
        };
        if take(4)? != b"FHNN" {
            return Err("not a model file".into());
        }
        let u32_at = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap()) as usize;
        let count = u32_at(take(4)?);
        let mut layers = Vec::with_capacity(count);
        for _ in 0..count {
            let inputs = u32_at(take(4)?);
            let outputs = u32_at(take(4)?);
            let floats = |b: &[u8]| b.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect::<Vec<f32>>();
            let weights = floats(take(inputs * outputs * 4)?);
            let bias = floats(take(outputs * 4)?);
            if let Some(prev) = layers.last().map(|l: &Layer| l.outputs) {
                if prev != inputs {
                    return Err("layer sizes don't chain".into());
                }
            }
            layers.push(Layer { inputs, outputs, weights, bias });
        }
        Ok(Mlp { layers })
    }

    pub fn inputs(&self) -> usize {
        self.layers.first().map_or(0, |l| l.inputs)
    }

    pub fn outputs(&self) -> usize {
        self.layers.last().map_or(0, |l| l.outputs)
    }

    /// The raw outputs (logits) for input `x`.
    pub fn forward(&self, x: &[f32]) -> Vec<f32> {
        let mut current = x.to_vec();
        for (i, layer) in self.layers.iter().enumerate() {
            let mut next = layer.bias.clone();
            for (o, out) in next.iter_mut().enumerate() {
                let row = &layer.weights[o * layer.inputs..(o + 1) * layer.inputs];
                *out += row.iter().zip(&current).map(|(w, v)| w * v).sum::<f32>();
            }
            if i + 1 < self.layers.len() {
                next.iter_mut().for_each(|v| *v = v.max(0.0));
            }
            current = next;
        }
        current
    }
}

/// Softmax over a slice of logits.
pub fn softmax(logits: &[f32]) -> Vec<f64> {
    let max = logits.iter().copied().fold(f32::MIN, f32::max);
    let exps: Vec<f64> = logits.iter().map(|&l| ((l - max) as f64).exp()).collect();
    let sum: f64 = exps.iter().sum();
    exps.into_iter().map(|e| e / sum).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(layers: &[(usize, usize, Vec<f32>, Vec<f32>)]) -> Vec<u8> {
        let mut bytes = b"FHNN".to_vec();
        bytes.extend((layers.len() as u32).to_le_bytes());
        for (i, o, w, b) in layers {
            bytes.extend((*i as u32).to_le_bytes());
            bytes.extend((*o as u32).to_le_bytes());
            w.iter().chain(b).for_each(|f| bytes.extend(f.to_le_bytes()));
        }
        bytes
    }

    #[test]
    fn forward_matches_hand_computation() {
        // 2 → 2 (ReLU) → 1.
        let bytes = model(&[(2, 2, vec![1.0, -1.0, 0.5, 0.5], vec![0.0, 1.0]), (2, 1, vec![2.0, 3.0], vec![-1.0])]);
        let mlp = Mlp::from_bytes(&bytes).unwrap();
        // x = [3, 1]: hidden = [relu(3 - 1), relu(1.5 + 0.5 + 1)] = [2, 3]; out = 4 + 9 - 1 = 12.
        assert_eq!(mlp.forward(&[3.0, 1.0]), vec![12.0]);
        assert!(Mlp::from_bytes(b"nope").is_err());
        let p = softmax(&[0.0, 0.0]);
        assert!((p[0] - 0.5).abs() < 1e-12);
    }
}
