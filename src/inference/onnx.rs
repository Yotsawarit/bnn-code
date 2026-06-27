use anyhow::{Context, Result};
use ndarray::Array2;
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;

pub struct OnnxEngine {
    session: Session,
}

impl OnnxEngine {
    pub fn new(model_path: &Path) -> Result<Self> {
        tracing::info!("Loading ONNX model from: {:?}", model_path);

        let session = Session::builder()
            .map_err(|e| anyhow::anyhow!("ONNX builder error: {:?}", e))?
            .with_intra_threads(4)
            .map_err(|e| anyhow::anyhow!("Failed to set intra threads: {:?}", e))?
            .commit_from_file(model_path)
            .map_err(|e| anyhow::anyhow!("Failed to load model from {:?}: {:?}", model_path, e))?;

        Ok(Self { session })
    }

    pub fn run(
        &mut self,
        input_ids: Array2<i64>,
        attention_mask: Array2<i64>,
    ) -> Result<Array2<f32>> {
        // Create tensors from ndarray data
        let input_tensor = Tensor::from_array(input_ids)
            .context("Failed to create input_ids tensor")?;
        let mask_tensor = Tensor::from_array(attention_mask)
            .context("Failed to create attention_mask tensor")?;

        // Run inference with named inputs
        let outputs = self
            .session
            .run(ort::inputs! {
                "input_ids" => input_tensor,
                "attention_mask" => mask_tensor,
            })
            .map_err(|e| anyhow::anyhow!("ONNX inference failed: {:?}", e))?;

        // Extract the first output as an f32 tensor view
        let output_view = outputs[0]
            .try_extract_array::<f32>()
            .map_err(|e| anyhow::anyhow!("Failed to extract output tensor: {:?}", e))?;

        // Convert ArrayViewD to owned Array2
        let output_array = output_view.to_owned();
        // Reshape to 2D if needed (infer dimensions)
        let shape: Vec<usize> = output_array.shape().to_vec();
        if shape.len() == 1 {
            // Reshape to (1, N) for single-dimension output
            let n = shape[0];
            let vec: Vec<f32> = output_array.iter().cloned().collect();
            Ok(Array2::from_shape_vec((1, n), vec)
                .map_err(|e| anyhow::anyhow!("Shape error: {:?}", e))?)
        } else if shape.len() == 2 {
            let vec: Vec<f32> = output_array.iter().cloned().collect();
            Ok(Array2::from_shape_vec((shape[0], shape[1]), vec)
                .map_err(|e| anyhow::anyhow!("Shape error: {:?}", e))?)
        } else if shape.len() == 3 {
            // For 3D output like (1, seq_len, vocab), merge batch and seq
            let batch = shape[0];
            let seq = shape[1];
            let vocab = shape[2];
            let vec: Vec<f32> = output_array.iter().cloned().collect();
            Ok(Array2::from_shape_vec((batch * seq, vocab), vec)
                .map_err(|e| anyhow::anyhow!("Shape error: {:?}", e))?)
        } else {
            // Flatten to 2D
            let total: usize = shape.iter().product();
            let vec: Vec<f32> = output_array.iter().cloned().collect();
            Ok(Array2::from_shape_vec((1, total), vec)
                .map_err(|e| anyhow::anyhow!("Shape error: {:?}", e))?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "requires ONNX Runtime system library"]
    #[test]
    fn test_onnx_engine_new_missing_model() {
        let result = OnnxEngine::new(Path::new("/nonexistent/model.onnx"));
        assert!(result.is_err());
    }

    #[ignore = "requires ONNX Runtime system library"]
    #[test]
    fn test_onnx_engine_new_invalid_path() {
        let result = OnnxEngine::new(Path::new(""));
        assert!(result.is_err());
    }

    // Note: Full integration tests requiring an actual ONNX model
    // are in tests/integration_test.rs
}
