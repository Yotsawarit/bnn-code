#!/usr/bin/env python3
"""
📥 BNN Model Downloader + ONNX Converter
========================================
Downloads a quantized CodeBERTa model from HuggingFace and converts to ONNX.

Usage:
    python scripts/download_model.py                    # Download default model
    python scripts/download_model.py --model microsoft/codebert-base  # Alternative
    python scripts/download_model.py --quantize         # Apply quantization
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

MODELS_DIR = Path(__file__).resolve().parent.parent / "models"
DEFAULT_MODEL = "huggingface/CodeBERTa-small-v1"  # 6-layer, 84M params → ONNX ~50MB
QUANTIZED_MODEL = "philschmid/codebert-base-quantized-dynamic"

REQUIRED_PKGS = ["torch", "transformers", "onnx", "onnxruntime", "psutil"]


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


def download_model(model_name: str, quantize: bool = False):
    """Download model from HuggingFace and export to ONNX."""
    import torch
    import transformers
    import onnx

    print(f"🧠 Downloading model: {model_name}")
    print(f"📁 Output dir: {MODELS_DIR}")
    MODELS_DIR.mkdir(parents=True, exist_ok=True)

    # Load tokenizer and model
    print("  Loading tokenizer...")
    tokenizer = transformers.AutoTokenizer.from_pretrained(model_name)

    print("  Loading model...")
    model = transformers.AutoModelForMaskedLM.from_pretrained(model_name)

    if quantize:
        print("  Applying dynamic quantization...")
        model = torch.quantization.quantize_dynamic(
            model, {torch.nn.Linear}, dtype=torch.qint8
        )

    model.eval()

    # Save tokenizer
    tokenizer_path = MODELS_DIR / "tokenizer.json"
    if not tokenizer_path.exists():
        tokenizer.save_pretrained(MODELS_DIR)
        print(f"  ✓ Tokenizer saved to {MODELS_DIR}")

    # Export to ONNX
    print("  Exporting to ONNX...")
    onnx_path = MODELS_DIR / "model.onnx"

    # Create dummy input
    dummy_input = torch.randint(0, 100, (1, 128), dtype=torch.long)

    torch.onnx.export(
        model,
        dummy_input,
        onnx_path,
        input_names=["input_ids"],
        output_names=["logits"],
        dynamic_axes={
            "input_ids": {0: "batch_size", 1: "sequence_length"},
            "logits": {0: "batch_size", 1: "sequence_length"},
        },
        opset_version=14,
        do_constant_folding=True,
    )

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

    with open(MODELS_DIR / "metadata.json", "w") as f:
        json.dump(metadata, f, indent=2)

    print(f"\n✅ Model exported successfully!")
    print(f"  ├── model.onnx       ({onnx_path.stat().st_size / 1024 / 1024:.1f} MB)")
    print(f"  ├── tokenizer.json   ({(MODELS_DIR / 'tokenizer.json').stat().st_size / 1024:.1f} KB)")
    print(f"  ├── metadata.json")
    print(f"  └── params: {metadata['parameters']:,}")


def verify():
    """Test ONNX inference with onnxruntime."""
    import onnxruntime as ort

    onnx_path = MODELS_DIR / "model.onnx"
    tokenizer_path = MODELS_DIR / "tokenizer.json"

    if not onnx_path.exists():
        print("❌ model.onnx not found. Run download first.")
        return

    print("\n🔍 Verifying ONNX inference...")

    # Load tokenizer
    from transformers import AutoTokenizer
    tokenizer = AutoTokenizer.from_pretrained(MODELS_DIR)

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
    args = parser.parse_args()

    check_deps()

    if args.verify_only:
        verify()
        return

    download_model(args.model, quantize=args.quantize)
    verify()


if __name__ == "__main__":
    main()
