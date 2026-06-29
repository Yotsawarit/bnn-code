use anyhow::{bail, Context, Result};
use ndarray::Array2;
use ort::session::Session;
use ort::value::Tensor;
use std::fs;
use std::io::Read;
use std::path::Path;

const MIN_MODEL_BYTES: u64 = 64;
const ONNX_MAGIC_BYTE: u8 = 0x08;

pub fn cpu_supports_sse42() -> bool {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    { return is_x86_feature_detected!("sse4.2"); }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    { return true; }
}

pub fn validate_model_file(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!(
            "Model file not found: {}\n\
             Run: bash scripts/download_model.sh",
            path.display()
        );
    }

    let size = fs::metadata(path)
        .with_context(|| format!("Cannot read metadata: {}", path.display()))?
        .len();

    if size < MIN_MODEL_BYTES {
        bail!(
            "Model file too small ({} bytes): {}\n\
             Looks like a partial download. Delete and re-run: bash scripts/download_model.sh",
            size, path.display()
        );
    }

    let mut file = fs::File::open(path)
        .with_context(|| format!("Cannot open: {}", path.display()))?;
    let mut first_byte = [0u8; 1];
    file.read_exact(&mut first_byte)
        .with_context(|| format!("Cannot read: {}", path.display()))?;

    if first_byte[0] != ONNX_MAGIC_BYTE {
        bail!(
            "Not a valid ONNX model (bad magic byte 0x{:02X}): {}\n\
             Re-download: bash scripts/download_model.sh",
            first_byte[0], path.display()
        );
    }

    Ok(())
}

#[derive(Debug)]
pub struct OnnxEngine {
    session: Session,
}

impl OnnxEngine {
    pub fn new(model_path: &Path) -> Result<Self> {
        // 1. SSE4.2 check
        if !cpu_supports_sse42() {
            bail!(
                "CPU does not support SSE4.2 (required for BNN inference).\n\
                 Check: grep sse4_2 /proc/cpuinfo"
            );
        }

        // 2. Validate file before touching ONNX Runtime
        validate_model_file(model_path)
            .with_context(|| format!("Model validation failed: {}", model_path.display()))?;

        tracing::info!("Loading ONNX model from: {}", model_path.display());

        // 3. Build session — ort::Error<SessionBuilder> ไม่ implement Send+Sync
        //    จึงต้องใช้ map_err แทน .context()
        let builder = Session::builder()
            .map_err(|e| anyhow::anyhow!("Failed to create ONNX session builder: {e}"))?;

        let mut builder = builder
            .with_intra_threads(4)
            .map_err(|e| anyhow::anyhow!("Failed to set intra-op threads: {e}"))?;

        let session = builder
            .commit_from_file(model_path)
            .map_err(|e| anyhow::anyhow!(
                "ONNX Runtime could not load model {}: {e}\n\
                 The model may use an unsupported opset version.",
                model_path.display()
            ))?;

        tracing::info!("ONNX model loaded successfully");
        Ok(Self { session })
    }

    pub fn run(
        &mut self,
        input_ids: Array2<i64>,
        attention_mask: Array2<i64>,
    ) -> Result<Array2<f32>> {
        let input_tensor = Tensor::from_array(input_ids)
            .context("Failed to create input_ids tensor")?;
        let mask_tensor = Tensor::from_array(attention_mask)
            .context("Failed to create attention_mask tensor")?;

        let outputs = self
            .session
            .run(ort::inputs! {
                "input_ids" => input_tensor,
                "attention_mask" => mask_tensor,
            })
            .context("ONNX inference forward pass failed")?;

        let output_view = outputs[0]
            .try_extract_array::<f32>()
            .context("Failed to extract f32 output tensor")?;

        let output_array = output_view.to_owned();
        let shape: Vec<usize> = output_array.shape().to_vec();
        let vec: Vec<f32> = output_array.iter().cloned().collect();

        match shape.len() {
            1 => Array2::from_shape_vec((1, shape[0]), vec)
                .context("Failed to reshape 1D output to (1, N)"),
            2 => Array2::from_shape_vec((shape[0], shape[1]), vec)
                .context("Failed to reshape 2D output"),
            3 => Array2::from_shape_vec((shape[0] * shape[1], shape[2]), vec)
                .context("Failed to reshape 3D output (batch, seq, vocab)"),
            _ => {
                let total: usize = shape.iter().product();
                Array2::from_shape_vec((1, total), vec)
                    .context("Failed to flatten N-D output to 2D")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn missing_file_gives_actionable_error() {
        let err = validate_model_file(Path::new("/no/such/model.onnx")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not found"), "got: {msg}");
        assert!(msg.contains("download_model.sh"), "got: {msg}");
    }

    #[test]
    fn partial_download_gives_clear_error() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"small").unwrap();
        let err = validate_model_file(tmp.path()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("too small"), "got: {msg}");
        assert!(msg.contains("partial"), "got: {msg}");
    }

    #[test]
    fn wrong_magic_gives_clear_error() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(&[0xFF_u8; 128]).unwrap();
        let err = validate_model_file(tmp.path()).unwrap_err();
        assert!(err.to_string().contains("Not a valid ONNX"), "got: {err}");
    }

    #[test]
    fn valid_magic_passes_validation() {
        let mut tmp = NamedTempFile::new().unwrap();
        let mut data = vec![ONNX_MAGIC_BYTE];
        data.extend_from_slice(&[0x01_u8; 127]);
        tmp.write_all(&data).unwrap();
        assert!(validate_model_file(tmp.path()).is_ok());
    }

    #[test]
    fn cpu_check_does_not_panic() {
        let _ = cpu_supports_sse42();
    }

    #[ignore = "requires ONNX Runtime system library"]
    #[test]
    fn test_onnx_engine_new_missing_model() {
        let result = OnnxEngine::new(Path::new("/nonexistent/model.onnx"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[ignore = "requires ONNX Runtime system library"]
    #[test]
    fn test_onnx_engine_new_invalid_path() {
        let result = OnnxEngine::new(Path::new(""));
        assert!(result.is_err());
    }
}
