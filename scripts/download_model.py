#!/usr/bin/env python3
"""
📥 BNN Model Downloader + ONNX Converter
=======================================
Downloads a model from HuggingFace (default: a causal code LM) and converts
it to ONNX for fast onnxruntime inference.

Usage:
    python scripts/download_model.py                    # Download default model
    python scripts/download_model.py --model gpt2       # Alternative
    python scripts/download_model.py --quantize         # Apply quantization
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

import torch

MODELS_DIR = Path(__file__).resolve().parent.parent / "models"
DEFAULT_MODEL = "codeparrot/codeparrot-small"  # GPT2-style causal LM trained on Python code
QUANTIZED_MODEL = "philschmid/codebert-base-quantized-dynamic"

REQUIRED_PKGS = ["torch", "transformers", "onnx", "onnxruntime", "onnxscript", "psutil"]


def check_deps():
    """Install required Python packages if missing."""
    missing = []
    for pkg in REQUIRED_PKGS:
        try:
            __import__(pkg.replace("-", "_"))
        except ImportError:
            missing.append(pkg)

    if missing:
        print(f"📦 Installing missing packages: {', '.join(missing)}")
        subprocess.check_call(
            [sys.executable, "-m", "pip", "install", *missing, "-q"]
        )


def safe_name(model_name: str) -> str:
    """Filesystem-safe directory name for a model id."""
    return model_name.replace("/", "--")


class _LogitsWrapper(torch.nn.Module):
    """Wrap an HF LM so ONNX export only sees the `logits` output.

    The default dynamo exporter fails on the model's `past_key_values`
    cache (DynamicCache), which we don't need for single-pass inference.
    """

    def __init__(self, model):
        super().__init__()
        self.model = model

    def forward(self, input_ids):
        return self.model(input_ids=input_ids).logits


def export_to_onnx(model, out_dir: Path, quantize: bool = False):
    """Export a (possibly quantized) LM to model.onnx with only `logits` output."""
    import torch

    if quantize:
        model = torch.quantization.quantize_dynamic(
            model, {torch.nn.Linear}, dtype=torch.qint8
        )
    model.eval()
    wrapped = _LogitsWrapper(model)

    dummy_input = torch.randint(0, 100, (1, 128), dtype=torch.long)
    torch.onnx.export(
        wrapped,
        dummy_input,
        out_dir / "model.onnx",
        input_names=["input_ids"],
        output_names=["logits"],
        dynamic_axes={
            "input_ids": {0: "batch_size", 1: "sequence_length"},
            "logits": {0: "batch_size", 1: "sequence_length"},
        },
        opset_version=14,
        do_constant_folding=True,
    )
    return model


def list_local_models():
    """Return local model directories sorted by most-recently modified first."""
    if not MODELS_DIR.exists():
        return []
    dirs = [d for d in MODELS_DIR.iterdir() if d.is_dir()]
    return sorted(dirs, key=lambda d: d.stat().st_mtime, reverse=True)


def latest_local_model():
    """Return the most recently modified local model directory, or None."""
    models = list_local_models()
    return models[0] if models else None


def download_model(model_name: str, quantize: bool = False, out_dir: Path = None):
    """Download model from HuggingFace and export to ONNX."""
    import torch
    import transformers
    import onnx

    out_dir = out_dir or (MODELS_DIR / safe_name(model_name))
    print(f"🧠 Downloading model: {model_name}")
    print(f"📁 Output dir: {out_dir}")
    out_dir.mkdir(parents=True, exist_ok=True)

    # Load tokenizer and model
    print("  Loading tokenizer...")
    tokenizer = transformers.AutoTokenizer.from_pretrained(model_name)

    print("  Loading model...")
    model = transformers.AutoModelForCausalLM.from_pretrained(model_name)

    # Save tokenizer + model weights locally so they can be reused in `local` mode
    tokenizer.save_pretrained(out_dir)
    model.save_pretrained(out_dir)
    print(f"  ✓ Tokenizer + weights saved to {out_dir}")

    # Export to ONNX (logits only)
    print("  Exporting to ONNX...")
    model = export_to_onnx(model, out_dir, quantize=quantize)
    onnx_path = out_dir / "model.onnx"

    # Verify ONNX model
    onnx_model = onnx.load(onnx_path)
    onnx.checker.check_model(onnx_model)

    # Save model metadata
    metadata = {
        "model": model_name,
        "quantized": quantize,
        "parameters": sum(p.numel() for p in model.parameters()),
        "onnx_opset": 14,
        "input_dim": 128,
        "vocab_size": tokenizer.vocab_size,
    }

    with open(out_dir / "metadata.json", "w") as f:
        json.dump(metadata, f, indent=2)

    print(f"\n✅ Model exported successfully!")
    print(f"  ├── model.onnx       ({onnx_path.stat().st_size / 1024 / 1024:.1f} MB)")
    print(f"  ├── tokenizer files  ({out_dir})")
    print(f"  ├── metadata.json")
    print(f"  └── params: {metadata['parameters']:,}")


def local_pipeline(out_dir: Path, quantize: bool = False):
    """Use a model already on disk (no HuggingFace download) and (re)export ONNX."""
    import torch
    import transformers
    import onnx

    print(f"📂 Using local model from: {out_dir}")
    if not out_dir.exists():
        print(f"❌ Local model dir not found: {out_dir}")
        return

    tokenizer = transformers.AutoTokenizer.from_pretrained(out_dir)
    weights = out_dir / "pytorch_model.bin"
    safetensors = out_dir / "model.safetensors"
    config = out_dir / "config.json"

    if (weights.exists() or safetensors.exists()) and config.exists():
        print("  Loading local weights...")
        model = transformers.AutoModelForCausalLM.from_pretrained(out_dir)
        export_to_onnx(model, out_dir, quantize=quantize)
        onnx_path = out_dir / "model.onnx"
        onnx_model = onnx.load(onnx_path)
        onnx.checker.check_model(onnx_model)
        print(f"  ✓ Re-exported ONNX ({onnx_path.stat().st_size / 1024 / 1024:.1f} MB)")
    else:
        print("  No local weights found — verifying existing model.onnx only.")

    verify(out_dir)


def verify(model_dir: Path = None):
    """Test ONNX inference with onnxruntime."""
    import onnxruntime as ort

    model_dir = model_dir or MODELS_DIR
    onnx_path = model_dir / "model.onnx"
    tokenizer_path = model_dir / "tokenizer.json"

    if not onnx_path.exists():
        print("❌ model.onnx not found. Run download first.")
        return

    print("\n🔍 Verifying ONNX inference...")

    # Load tokenizer
    from transformers import AutoTokenizer
    tokenizer = AutoTokenizer.from_pretrained(model_dir)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    # Create ONNX session
    session = ort.InferenceSession(str(onnx_path))

    # Test input
    text = "def hello_world():"
    inputs = tokenizer(text, return_tensors="np", padding="max_length", max_length=128)

    # Run inference
    outputs = session.run(
        ["logits"],
        {"input_ids": inputs["input_ids"].astype("int64")},
    )

    print(f"  Input: '{text}'")
    print(f"  Output shape: {outputs[0].shape}")
    print(f"  Output logits: [{outputs[0][0][0][0]:.2f}, ...]")
    print("✅ ONNX inference verified!")


def main():
    parser = argparse.ArgumentParser(description="Download BNN model + export to ONNX")
    parser.add_argument(
        "--model",
        default=DEFAULT_MODEL,
        help=f"HF model name (default: {DEFAULT_MODEL})",
    )
    parser.add_argument(
        "--quantize",
        action="store_true",
        help="Apply dynamic quantization before export",
    )
    parser.add_argument(
        "--verify-only",
        action="store_true",
        help="Only verify existing ONNX model",
    )
    parser.add_argument(
        "tokens",
        nargs="*",
        choices=["local", "last"],
        help="local = use on-disk model (no HF download); last = use most recent local model dir",
    )
    args = parser.parse_args()

    check_deps()

    use_local = "local" in args.tokens

    if args.verify_only and not use_local:
        target = MODELS_DIR / safe_name(args.model)
        if not (target / "model.onnx").exists():
            target = latest_local_model()
        verify(target)
        return

    if use_local:
        if "last" in args.tokens:
            out_dir = latest_local_model()
            if out_dir is None:
                print("❌ No local models found. Run a download first.")
                return
            print(f"📌 Selected latest local model: {out_dir}")
        else:
            out_dir = MODELS_DIR / safe_name(args.model)
        local_pipeline(out_dir, quantize=args.quantize)
        return

    out_dir = MODELS_DIR / safe_name(args.model)
    download_model(args.model, quantize=args.quantize, out_dir=out_dir)
    verify(out_dir)


if __name__ == "__main__":
    main()
